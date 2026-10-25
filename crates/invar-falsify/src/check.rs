//! Manifest evaluation behind `invar check`.
//!
//! Reads the parsed sections of an `invar.toml`, gathers every `[[invariant]]`
//! block (or the legacy single `[producer]`/`[consumer]` schema form), resolves
//! each referenced artifact through a `Resolver` (local file, or remote fetched
//! into memory under `--ci`), and dispatches to `invariant::check_contract`.
//! Each outcome carries the provenance of any remote artifact it used.

use anyhow::{anyhow, bail, Context, Result};

use std::collections::BTreeMap;

use crate::invariant::{check_contract, payload_read_sites, ContractInput, Step};
use crate::resolver::{is_remote, Fetched, Resolver};
use crate::surface::{
    broad_reads, changed_fields, code_fields, field_names, mention_lines, moved_fields, name_key,
    relocated_keys, selector_switches, spec_fields, ts_omissions, Fields,
};

pub type Sections = Vec<(String, Vec<(String, String)>)>;

#[derive(Debug, Clone, Default)]
pub struct ContractOutcome {
    pub id: String,
    pub kind: String,
    pub residue: Vec<String>,
    pub provenance: Vec<Fetched>,
    /// Where each residue item shows up, so a finding points at a line.
    pub witnesses: Vec<Witness>,
    /// Set when the contract was not evaluated because an artifact is in a
    /// language INVAR does not read. A skip is never a pass.
    pub skipped: Option<String>,
    /// Residue covered by an active waiver: (field, owner, reason, days left).
    pub waived: Vec<(String, String, String, i64)>,
    /// Waivers past their `expires` date: (field, owner). Each fails the check.
    pub expired: Vec<(String, String)>,
    /// Facts about the evaluation that are not residue (lock state).
    pub notes: Vec<String>,
    /// A1: sites INVAR saw but did not evaluate (a changed field mentioned
    /// outside a recognised read, or a field moved between types). With no
    /// residue, these make the contract NOT FULLY EVALUATED, never HOLDS.
    pub unevaluated: Vec<String>,
    /// CHECK-INCR-1: set when this HOLDS was reused from a hold record
    /// instead of evaluated: "lock <at>" or "last run".
    pub reused: Option<String>,
}

impl ContractOutcome {
    /// Violated: unwaived residue, or an expired waiver.
    pub fn violated(&self) -> bool {
        !self.residue.is_empty() || !self.expired.is_empty()
    }
}

/// Languages INVAR does not read (scope: Go, Python, JS/TS, Rust); a contract
/// naming one is SKIPPED, never an ERROR or a pass.
pub(crate) fn unsupported_language(spec: &str) -> Option<&'static str> {
    let name = artifact_name(spec);
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    Some(match ext.as_str() {
        "c" | "h" => "C",
        "cc" | "cpp" | "cxx" | "hpp" | "hh" => "C++",
        "cs" => "C#",
        "php" => "PHP",
        "swift" => "Swift",
        "java" => "Java",
        "kt" | "kts" => "Kotlin",
        "rb" => "Ruby",
        _ => return None,
    })
}

/// First artifact in an unsupported language, as "language not supported (..)".
pub(crate) fn unsupported_language_of(specs: &[&str]) -> Option<String> {
    specs
        .iter()
        .filter(|s| !s.is_empty())
        .find_map(|s| unsupported_language(s).map(|l| format!("language not supported ({l}: {}) [not-evaluated:unsupported-language]", artifact_name(s))))
}

/// Locates one residue item. `sites` are `file:line` in the consumer source
/// (schema, event) or the trace (lock_order); `missing_from` names what the
/// producer side lacks. Witnesses explain a residue; they never change it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    pub residue: String,
    pub sites: Vec<String>,
    pub missing_from: Option<String>,
    /// For a retyped field: (old type, new type, where).
    pub type_change: Option<(String, String, String)>,
    /// Forms the consumer reads this field under when they differ from the
    /// producer's name only by case or `_`/`-` (e.g. `isFood` for `is_food`).
    pub read_as: Vec<String>,
}

/// Short display name for an artifact: the path of a remote reference, or the
/// local path as written in invar.toml.
pub fn artifact_name(spec: &str) -> String {
    if is_remote(spec) {
        spec.rsplit(':').next().unwrap_or(spec).to_string()
    } else {
        spec.to_string()
    }
}

fn read_witnesses(
    residue: &[String],
    consumer_spec: &str,
    consumer_src: &str,
    missing_from: &str,
) -> Vec<Witness> {
    let name = artifact_name(consumer_spec);
    let sites = payload_read_sites(consumer_src);
    residue
        .iter()
        .map(|r| Witness {
            residue: r.clone(),
            sites: sites
                .iter()
                .filter(|(f, _)| f == r)
                .map(|(_, l)| format!("{name}:{l}"))
                .collect(),
            missing_from: Some(missing_from.to_string()),
            type_change: None,
            read_as: Vec::new(),
        })
        .collect()
}

const CODE_EXTS: [&str; 9] = ["go", "py", "ts", "tsx", "rs", "js", "jsx", "mjs", "cjs"];

pub(crate) fn is_code(spec: &str) -> bool {
    let ext = artifact_name(spec)
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_string();
    CODE_EXTS.contains(&ext.as_str())
}

/// Parse a spec producer (JSON, or YAML by extension or as a fallback),
/// select `pointer`, and require a JSON object. Anything else is an
/// evaluation ERROR: reading it as "defines nothing" would report HOLDS or
/// invent residue (found by R1, 2026-10-08).
pub(crate) fn load_object(
    id: &str,
    spec: &str,
    text: &str,
    pointer: Option<&str>,
) -> Result<serde_json::Value> {
    let name = artifact_name(spec);
    let is_yaml = name.ends_with(".yaml") || name.ends_with(".yml");
    let parsed = if is_yaml {
        crate::yaml::parse_yaml(text).map_err(|e| anyhow!("{id}: producer {name}: {e}"))?
    } else {
        match serde_json::from_str::<serde_json::Value>(text) {
            Ok(v) => v,
            Err(_) => crate::yaml::parse_yaml(text).map_err(|_| {
                anyhow!("{id}: producer {name} is not a JSON object (and not readable as YAML)")
            })?,
        }
    };
    let v = match pointer {
        Some(ptr) => parsed
            .pointer(ptr)
            .cloned()
            .ok_or_else(|| anyhow!("{id}: JSON pointer {ptr:?} not found in {name}"))?,
        None => parsed,
    };
    if !v.is_object() {
        bail!("{id}: producer {name} is not a JSON object");
    }
    Ok(v)
}

/// Fields one producer source provides: spec `properties` or code fields.
pub(crate) fn source_fields(id: &str, spec: &str, text: &str, pointer: Option<&str>) -> Result<Fields> {
    if is_code(spec) {
        code_fields(&artifact_name(spec), text)
            .ok_or_else(|| anyhow!("{id}: cannot read fields from {}", artifact_name(spec)))
    } else {
        Ok(spec_fields(&load_object(id, spec, text, pointer)?))
    }
}

/// Schema contract. Without a base version: the strict check (consumer
/// `payload.<field>` reads the producer does not define). With a base version
/// (`producer_base`, and/or `producer_code` + `producer_code_base`): the
/// change-scoped check, residue = fields removed or retyped between base and
/// head that the consumer reads by any recognised pattern.
fn schema_outcome(
    id: &str,
    b: &[(String, String)],
    resolver: &Resolver,
    prov: &mut Vec<Fetched>,
) -> Result<(Vec<String>, Vec<Witness>, Vec<String>, Option<String>)> {
    let p = getk(b, "producer").context("schema invariant needs producer")?;
    let c = getk(b, "consumer").context("schema invariant needs consumer")?;
    let ptr = getk(b, "pointer");
    let base = getk(b, "producer_base");
    let code = getk(b, "producer_code");
    let code_base = getk(b, "producer_code_base");
    if code.is_some() != code_base.is_some() {
        bail!("{id}: producer_code and producer_code_base must be given together");
    }
    let pr = fetch(resolver, &p, prov)?;

    if base.is_none() && code.is_none() {
        let cr = fetch(resolver, &c, prov)?;
        // Directive 86428 pass 3: without a lock, the strict check below
        // compares only literal `payload.<field>` reads. A code producer
        // whose consumer has none would get a HOLDS in which NOTHING was
        // evaluated; that is reported as not evaluated, never as HOLDS.
        if is_code(&p) && payload_read_sites(&cr).is_empty() {
            return Ok((
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Some(format!(
                    "not evaluated: no invar.lock entry, and {} has no literal payload.<field> reads for the unlocked strict check; run `invar lock` first",
                    artifact_name(&c)
                )),
            ));
        }
        let defined: serde_json::Value = if is_code(&p) {
            let f = source_fields(id, &p, &pr, None)?;
            let props: serde_json::Map<String, serde_json::Value> = field_names(&f)
                .into_iter()
                .map(|k| (k, serde_json::json!({})))
                .collect();
            serde_json::json!({ "properties": props })
        } else {
            load_object(id, &p, &pr, ptr.as_deref())?
        };
        let r = check_contract(&ContractInput::Schema {
            producer_schema: &defined.to_string(),
            consumer_source: &cr,
        });
        let missing = format!(
            "{} {}/properties",
            artifact_name(&p),
            ptr.unwrap_or_default()
        );
        return Ok((r.clone(), read_witnesses(&r, &c, &cr, &missing), Vec::new(), None));
    }

    // Change-scoped: collect what changed in each producer source.
    let mut changed: BTreeMap<String, (Option<(String, String)>, String)> = BTreeMap::new();
    let mut moved: Vec<(String, String, Vec<String>, String)> = Vec::new();
    let mut omitted: Vec<(String, String)> = Vec::new();
    let mut relocs: Vec<(String, String, Vec<String>)> = Vec::new();
    let mut switches: Vec<(String, String, String, String, String)> = Vec::new();
    let mut add = |head_spec: &str, head: &Fields, base: &Fields, where_: String| {
        for (name, tc) in changed_fields(base, head) {
            changed.entry(name).or_insert((tc, where_.clone()));
        }
        for (name, from, to) in moved_fields(base, head) {
            moved.push((name, from, to, where_.clone()));
        }
        let _ = head_spec;
    };
    if let Some(bs) = &base {
        let br = fetch(resolver, bs, prov)?;
        let head_f = source_fields(id, &p, &pr, ptr.as_deref())?;
        let base_f = source_fields(id, bs, &br, ptr.as_deref())?;
        let where_ = format!("{} {}", artifact_name(&p), ptr.clone().unwrap_or_default())
            .trim()
            .to_string();
        add(&p, &head_f, &base_f, where_);
        omitted.extend(new_omissions(&p, &pr, &br, &base_f));
        relocs.extend(relocations(&p, &pr, &br));
        for (n, old, new, note) in selector_switches(&br, &pr) {
            switches.push((n, old, new, artifact_name(&p), note));
        }
    }
    if let (Some(cs), Some(cbs)) = (&code, &code_base) {
        let cr_head = fetch(resolver, cs, prov)?;
        let cr_base = fetch(resolver, cbs, prov)?;
        let head_f = source_fields(id, cs, &cr_head, None)?;
        let base_f = source_fields(id, cbs, &cr_base, None)?;
        add(cs, &head_f, &base_f, artifact_name(cs));
        omitted.extend(new_omissions(cs, &cr_head, &cr_base, &base_f));
        relocs.extend(relocations(cs, &cr_head, &cr_base));
        for (n, old, new, note) in selector_switches(&cr_base, &cr_head) {
            switches.push((n, old, new, artifact_name(cs), note));
        }
    }
    for (name, where_) in omitted {
        changed.entry(name).or_insert((None, where_));
    }
    let cr = fetch(resolver, &c, prov)?;
    let reads = broad_reads(&cr);
    let cname = artifact_name(&c);
    // E1 rule 3: a relocated key counts unless the consumer already mentions
    // a new path segment (it reads the new envelope).
    for (key, where_, new_segs) in relocs {
        let adapted = new_segs
            .iter()
            .any(|s| s != "*" && !mention_lines(&cr, s).is_empty());
        if !adapted {
            changed.entry(key).or_insert((None, where_));
        }
    }
    let mut residue = Vec::new();
    let mut witnesses = Vec::new();
    for (name, (tc, where_)) in &changed {
        let key = name_key(name);
        let matched: Vec<&(String, usize)> =
            reads.iter().filter(|(f, _)| name_key(f) == key).collect();
        if matched.is_empty() {
            continue;
        }
        let mut dedup: Vec<String> = matched
            .iter()
            .map(|(_, l)| format!("{cname}:{l}"))
            .collect();
        dedup.dedup();
        let mut read_as: Vec<String> = matched
            .iter()
            .map(|(f, _)| f.clone())
            .filter(|f| f != name)
            .collect();
        read_as.sort();
        read_as.dedup();
        residue.push(name.clone());
        witnesses.push(Witness {
            residue: name.clone(),
            sites: dedup,
            missing_from: if tc.is_none() {
                Some(where_.clone())
            } else {
                None
            },
            type_change: tc.clone().map(|(a, b2)| (a, b2, where_.clone())),
            read_as,
        });
    }
    // A1 rule 1: a changed field the consumer mentions but INVAR did not
    // count as a read. A1 rule 2: a field that moved between types and that
    // the consumer reads or mentions. Both are stated, never silent.
    let mut unevaluated = Vec::new();
    for name in changed.keys().filter(|n| !residue.contains(n)) {
        let lines = mention_lines(&cr, name);
        if let Some(l) = lines.first() {
            let more = if lines.len() > 1 { format!(" (+{} more)", lines.len() - 1) } else { String::new() };
            unevaluated.push(format!(
                "{name} changed; mentioned at {cname}:{l}{more}, not evaluated (pattern not read) [not-evaluated:pattern-not-read]"
            ));
        }
    }
    for (name, from, to, where_) in &moved {
        if residue.contains(name) || changed.contains_key(name) {
            continue;
        }
        let lines = mention_lines(&cr, name);
        if let Some(l) = lines.first() {
            unevaluated.push(format!(
                "{name} moved from {from} to {} in {where_}; consumer use at {cname}:{l} not evaluated [not-evaluated:field-moved]",
                to.join(", ")
            ));
        }
    }
    // F1: a default version or format selector changed while every field
    // literal survived. Textual analysis cannot tell which output path the
    // consumer now receives, so the honest verdict is "not evaluated",
    // never HOLDS and never VIOLATED. With residue, F1 adds nothing.
    if residue.is_empty() && !reads.is_empty() {
        for (name, old, new, where_, note) in &switches {
            unevaluated.push(format!(
                "default selector '{name}' changed {old} -> {new} in {where_}; emission of the fields read in {cname} not evaluated{note} [not-evaluated:default-switch]"
            ));
        }
    }
    Ok((residue, witnesses, unevaluated, None))
}

/// E1: keys of runtime payloads that moved into a new envelope:
/// (key, "file: moved from A to B", new path segments).
fn relocations(spec: &str, head: &str, base: &str) -> Vec<(String, String, Vec<String>)> {
    let name = artifact_name(spec);
    let ext = name.rsplit('.').next().unwrap_or("");
    relocated_keys(base, head, ext)
        .into_iter()
        .map(|(k, bp, hp, new)| {
            let from = if bp.is_empty() { "the top level".to_string() } else { bp };
            (k, format!("{name}: moved from {from} to {hp}"), new)
        })
        .collect()
}

/// D1 rule 1: fields that a TypeScript type introduced at head leaves out
/// with `Omit`, and that the producer declared at base: (field, where).
fn new_omissions(spec: &str, head: &str, base: &str, base_f: &Fields) -> Vec<(String, String)> {
    let name = artifact_name(spec);
    if !(name.ends_with(".ts") || name.ends_with(".tsx")) {
        return Vec::new();
    }
    let declared = field_names(base_f);
    let mut out = Vec::new();
    for (t, keys) in ts_omissions(head) {
        let introduced = !base.lines().any(|l| {
            let l = l.trim().trim_start_matches("export ").trim_start_matches("declare ");
            l.strip_prefix("type ")
                .and_then(|r| r.strip_prefix(t.as_str()))
                .map(|r| !r.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(false)
        });
        if !introduced {
            continue;
        }
        for k in keys {
            if declared.contains(&k) && !out.iter().any(|(n, _)| n == &k) {
                out.push((k, format!("{name} (Omit in {t})")));
            }
        }
    }
    out
}

/// 1-based trace lines (comments and blanks counted, as in an editor) where
/// `step` occurs.
fn step_lines(trace: &str, step: &Step) -> Vec<usize> {
    trace
        .lines()
        .enumerate()
        .filter(|(_, l)| {
            let l = l.trim();
            !l.is_empty()
                && !l.starts_with('#')
                && parse_step(l).map(|s| &s == step).unwrap_or(false)
        })
        .map(|(i, _)| i + 1)
        .collect()
}

pub(crate) fn getk(pairs: &[(String, String)], k: &str) -> Option<String> {
    pairs.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone())
}

fn section_get(sections: &Sections, sec: &str, key: &str) -> Option<String> {
    sections
        .iter()
        .find(|(n, _)| n == sec)
        .and_then(|(_, p)| getk(p, key))
}

/// Parse `actor:action:resource`.
pub fn parse_step(s: &str) -> Result<Step> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        bail!("step {s:?} must be actor:action:resource");
    }
    Ok(Step::new(parts[0].trim(), parts[1].trim(), parts[2].trim()))
}

/// Parse a trace: one `actor:action:resource` per line, `#` comments allowed.
pub fn parse_trace(text: &str) -> Result<Vec<Step>> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(parse_step)
        .collect()
}

/// The `[[invariant]]` blocks, or the legacy single schema form.
pub fn gather_blocks(sections: &Sections) -> Result<Vec<Vec<(String, String)>>> {
    let mut blocks: Vec<Vec<(String, String)>> = sections
        .iter()
        .filter(|(n, _)| n == "invariant")
        .map(|(_, p)| p.clone())
        .collect();
    if blocks.is_empty() {
        if let (Some(ps), Some(cs)) = (
            section_get(sections, "producer", "schema"),
            section_get(sections, "consumer", "source"),
        ) {
            blocks.push(vec![
                ("kind".into(), "schema".into()),
                (
                    "id".into(),
                    section_get(sections, "case", "id").unwrap_or_else(|| "schema-0".into()),
                ),
                ("producer".into(), ps),
                ("consumer".into(), cs),
            ]);
        }
    }
    if blocks.is_empty() {
        bail!("invar.toml has no [[invariant]] blocks and no [producer]/[consumer] schema form");
    }
    Ok(blocks)
}

pub(crate) fn fetch_text(resolver: &Resolver, spec: &str, prov: &mut Vec<Fetched>) -> Result<String> {
    fetch(resolver, spec, prov)
}

fn fetch(resolver: &Resolver, spec: &str, prov: &mut Vec<Fetched>) -> Result<String> {
    let r = resolver.resolve(spec).map_err(|e| anyhow!("{e}"))?;
    crate::reuse::note_read(spec, &r.text);
    if let Some(p) = r.provenance {
        prov.push(p);
    }
    Ok(r.text)
}

/// Marker used by the GitHub Action to find and update its own PR comment.
pub const REPORT_MARKER: &str = "<!-- invar-check -->";

/// Render a value from a scanned repo as GitHub-safe inline code. Contract ids
/// and residue are attacker-controllable in a fork PR, so they are always shown
/// as code (no mention pings, no links, no HTML), with table pipes escaped and
/// backticks and line breaks neutralized.
fn md_code(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| match c {
            '`' => '\'',
            '\n' | '\r' | '\t' => ' ',
            c => c,
        })
        .collect();
    format!("`{}`", cleaned.replace('|', "\\|"))
}

const MAX_RESIDUE_SHOWN: usize = 20;
const MAX_SITES_SHOWN: usize = 3;

/// One-line location suffix, e.g. " read at `client.js:6` · not in
/// `schema.json /properties`". Every repo-derived value is escaped code.
pub fn witness_text(w: &Witness, kind: &str, code: fn(&str) -> String) -> String {
    let mut s = String::new();
    if !w.sites.is_empty() {
        let verb = if kind == "lock_order" {
            "at"
        } else {
            "read at"
        };
        let mut shown: Vec<String> = w
            .sites
            .iter()
            .take(MAX_SITES_SHOWN)
            .map(|x| code(x))
            .collect();
        if w.sites.len() > MAX_SITES_SHOWN {
            shown.push(format!("+{} more", w.sites.len() - MAX_SITES_SHOWN));
        }
        s.push_str(&format!(" {verb} {}", shown.join(", ")));
    }
    if let Some(m) = &w.missing_from {
        s.push_str(&format!(" · not in {}", code(m)));
    }
    if !w.read_as.is_empty() {
        let forms: Vec<String> = w.read_as.iter().map(|x| code(x)).collect();
        s.push_str(&format!(" · naming differs, read as {}", forms.join(", ")));
    }
    if let Some((from, to, place)) = &w.type_change {
        s.push_str(&format!(
            " · type {} -> {} in {}",
            code(from),
            code(to),
            code(place)
        ));
    }
    s
}

fn witness_md(w: &Witness, kind: &str) -> String {
    witness_text(w, kind, md_code)
}

/// The per-contract block `invar check` prints, as text, plus the status tag
/// ("HOLDS" | "VIOLATED" | "SKIPPED" | "NOT_EVALUATED"). `invar demo` renders
/// through this same function, so what the demo shows IS what check prints,
/// by construction, not by imitation.
pub fn outcome_lines(o: &ContractOutcome) -> (&'static str, String) {
    use std::fmt::Write;
    let mut s = String::new();
    let tag = if let Some(why) = &o.skipped {
        let _ = writeln!(s, "  [SKIPPED]  {} ({}): {why}", o.id, o.kind);
        "SKIPPED"
    } else if !o.violated() && !o.unevaluated.is_empty() {
        let _ = writeln!(
            s,
            "  [NOT EVALUATED] {} ({}): {} site(s)",
            o.id,
            o.kind,
            o.unevaluated.len()
        );
        for u in &o.unevaluated {
            let _ = writeln!(s, "      {u}");
        }
        "NOT_EVALUATED"
    } else if !o.violated() {
        match &o.reused {
            Some(since) => {
                let _ = writeln!(s, "  [HOLDS]    {} ({}) unchanged since {since}", o.id, o.kind);
            }
            None => {
                let _ = writeln!(s, "  [HOLDS]    {} ({})", o.id, o.kind);
            }
        }
        "HOLDS"
    } else {
        let _ = writeln!(s, "  [VIOLATED] {} ({}): {:?}", o.id, o.kind, o.residue);
        for (f, owner) in &o.expired {
            let _ = writeln!(
                s,
                "      waiver EXPIRED: {f} (owner {owner}); renew or remove it in invar.lock"
            );
        }
        for w in &o.witnesses {
            let _ = writeln!(s, "      {}{}", w.residue, witness_text(w, &o.kind, |x| x.to_string()));
        }
        // Stated non-evaluation prints alongside a violation too: a verdict
        // never swallows a blind spot.
        for u in &o.unevaluated {
            let _ = writeln!(s, "      {u}");
        }
        "VIOLATED"
    };
    for (f, owner, reason, left) in &o.waived {
        let _ = writeln!(
            s,
            "      WAIVED {f} ({left} day(s) remaining, owner {owner}): {reason}"
        );
    }
    for n in &o.notes {
        let _ = writeln!(s, "      note: {n}");
    }
    for f in &o.provenance {
        let _ = writeln!(
            s,
            "      fetched {} ({} bytes, blake3 {})",
            f.reference, f.bytes, f.blake3_hex
        );
    }
    (tag, s)
}

/// Markdown report for a PR comment or job summary. States plainly that a
/// contract violation is not a PROVEN_RESIDUE verdict.
pub fn render_report_md(outcomes: &[ContractOutcome]) -> String {
    let violated = outcomes.iter().filter(|o| o.violated()).count();
    let skipped = outcomes
        .iter()
        .filter(|o| o.skipped.is_some() || (!o.violated() && !o.unevaluated.is_empty()))
        .count();
    let mut s = String::new();
    s.push_str(REPORT_MARKER);
    s.push('\n');
    if violated == 0 && skipped > 0 {
        s.push_str(&format!(
            "### INVAR contract check: NOT FULLY EVALUATED ({skipped} of {} not evaluated)\n\n",
            outcomes.len()
        ));
    } else if violated == 0 {
        s.push_str(&format!(
            "### INVAR contract check: ALL HOLD ({} contract(s))\n\n",
            outcomes.len()
        ));
    } else {
        s.push_str(&format!(
            "### INVAR contract check: VIOLATED ({violated} of {})\n\n",
            outcomes.len()
        ));
    }
    s.push_str("| Contract | Kind | Result | Residue |\n|---|---|---|---|\n");
    for o in outcomes {
        let mut extra: Vec<String> = o
            .expired
            .iter()
            .map(|(f, owner)| format!("**WAIVER EXPIRED** {} (owner {})", md_code(f), md_code(owner)))
            .collect();
        extra.extend(o.waived.iter().map(|(f, owner, _, left)| {
            format!(
                "WAIVED {} ({left} day{} remaining, owner {})",
                md_code(f),
                if *left == 1 { "" } else { "s" },
                md_code(owner)
            )
        }));
        let (result, residue) = if let Some(why) = &o.skipped {
            ("SKIPPED", md_code(why))
        } else if !o.violated() && !o.unevaluated.is_empty() {
            let mut cells: Vec<String> = o.unevaluated.iter().map(|u| md_code(u)).collect();
            cells.extend(extra);
            ("NOT EVALUATED", cells.join(", "))
        } else if !o.violated() {
            (if o.waived.is_empty() { "holds" } else { "holds, WAIVED" }, extra.join(", "))
        } else if o.residue.is_empty() {
            ("**VIOLATED**", extra.join(", "))
        } else {
            let mut cells: Vec<String> = o
                .residue
                .iter()
                .take(MAX_RESIDUE_SHOWN)
                .map(|r| match o.witnesses.iter().find(|w| &w.residue == r) {
                    Some(w) => format!("{}{}", md_code(r), witness_md(w, &o.kind)),
                    None => md_code(r),
                })
                .collect();
            if o.residue.len() > MAX_RESIDUE_SHOWN {
                cells.push(format!("+{} more", o.residue.len() - MAX_RESIDUE_SHOWN));
            }
            cells.extend(extra);
            ("**VIOLATED**", cells.join(", "))
        };
        s.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            md_code(&o.id),
            md_code(&o.kind),
            result,
            residue
        ));
    }
    let fetched: Vec<&Fetched> = outcomes.iter().flat_map(|o| o.provenance.iter()).collect();
    if !fetched.is_empty() {
        s.push_str("\n**Remote artifacts fetched in-process**\n\n");
        for f in fetched {
            s.push_str(&format!(
                "- {} ({} bytes, BLAKE3 {})\n",
                md_code(&f.reference),
                f.bytes,
                md_code(&f.blake3_hex)
            ));
        }
    }
    s.push_str(
        "\n<sub>A violation is contract residue found by INVAR's differential probe. \
         It is not a PROVEN_RESIDUE verdict, which additionally requires the reality \
         gate and an executed composite baseline.</sub>\n",
    );
    s
}

/// Evaluate every contract declared in the manifest (no lock, no waivers).
pub fn evaluate(sections: &Sections, resolver: &Resolver) -> Result<Vec<ContractOutcome>> {
    evaluate_with(sections, resolver, None, 0)
}

/// Evaluate with an `invar.lock`: a locked schema contract is checked against
/// its locked slice (not `producer_base`), and waivers apply as of `today`
/// (a day number, see `lock::today_utc`).
pub fn evaluate_with(
    sections: &Sections,
    resolver: &Resolver,
    lock: Option<&crate::lock::Lock>,
    today: i64,
) -> Result<Vec<ContractOutcome>> {
    Ok(evaluate_reusing(sections, resolver, lock, today, None)?.0)
}

/// `evaluate_with`, plus CHECK-INCR-1. With a plan, a contract whose hold
/// record matches (engine, block, lock entry, the bytes of every input) is
/// reported HOLDS without evaluation, and every reusable HOLDS is returned as
/// a record. Without a plan this is exactly `evaluate_with`.
pub fn evaluate_reusing(
    sections: &Sections,
    resolver: &Resolver,
    lock: Option<&crate::lock::Lock>,
    today: i64,
    plan: Option<&crate::reuse::Plan>,
) -> Result<(Vec<ContractOutcome>, BTreeMap<String, crate::reuse::Held>)> {
    use crate::reuse::{block_hash, entry_hash, inputs_unchanged, Capture, Held, ENGINE};
    let mut records: BTreeMap<String, Held> = BTreeMap::new();
    let now = crate::lock::now_rfc3339();
    let blocks = gather_blocks(sections)?;
    if let Some(l) = lock {
        let ids: Vec<String> = blocks.iter().filter_map(|b| getk(b, "id")).collect();
        for w in &l.waivers {
            if !ids.contains(&w.contract) {
                bail!("invar.lock: waiver for unknown contract {:?}", w.contract);
            }
        }
        crate::lock::validate_waivers(&l.waivers, today)?;
    }
    let mut out = Vec::with_capacity(blocks.len());
    for (idx, b) in blocks.iter().enumerate() {
        let kind = getk(b, "kind").unwrap_or_default();
        let id = getk(b, "id").unwrap_or_else(|| format!("inv-{idx}"));
        // I-D2a: a waiver or a locked sample makes the outcome move with
        // the date, so such a contract is never recorded or reused.
        let dated = lock.is_some_and(|l| l.waivers.iter().any(|w| w.contract == id) || l.samples.iter().any(|s| s.contract == id));
        let key = match plan {
            Some(_) if !dated => {
                let entry = lock.and_then(|l| l.contracts.iter().find(|c| c.id == id));
                Some((block_hash(b), entry_hash(entry), entry.and_then(|c| c.held.as_ref())))
            }
            _ => None,
        };
        if let (Some(p), Some((block, entry, from_lock))) = (plan, &key) {
            let candidates = [
                from_lock.filter(|_| p.from_lock).map(|r| (r, format!("lock {}", r.at))),
                p.cache.get(&id).map(|r| (r, "last run".to_string())),
            ];
            let hit = candidates.into_iter().flatten().find_map(|(r, since)| {
                if r.engine != ENGINE || &r.block != block || &r.entry != entry {
                    return None;
                }
                inputs_unchanged(r, resolver).map(|prov| (r.clone(), since, prov))
            });
            if let Some((r, since, provenance)) = hit {
                out.push(ContractOutcome { id: id.clone(), kind, provenance, reused: Some(since), ..Default::default() });
                records.insert(id, r);
                continue;
            }
        }
        let capture = key.as_ref().map(|_| Capture::start());
        let mut prov = Vec::new();
        if kind == "schema" || kind == "event" {
            let unsupported = [
                "producer",
                "producer_base",
                "producer_code",
                "producer_code_base",
                "consumer",
            ]
            .iter()
            .filter_map(|k| getk(b, k))
            .find_map(|spec| unsupported_language(&spec).map(|lang| (lang, artifact_name(&spec))));
            if let Some((lang, name)) = unsupported {
                out.push(ContractOutcome {
                    id,
                    kind,
                    residue: Vec::new(),
                    provenance: Vec::new(),
                    witnesses: Vec::new(),
                    skipped: Some(format!("language not supported ({lang}: {name})")),
                    ..Default::default()
                });
                continue;
            }
        }
        let witnesses: Vec<Witness>;
        let mut notes: Vec<String> = Vec::new();
        let mut unevaluated: Vec<String> = Vec::new();
        let mut skipped: Option<String> = None;
        let residue = match kind.as_str() {
            "schema" => match lock.and_then(|l| l.contracts.iter().find(|c| c.id == id)) {
                Some(entry) => {
                    let r = crate::lock::check_contract(
                        entry,
                        &getk(b, "producer").context("schema invariant needs producer")?,
                        getk(b, "pointer").as_deref(),
                        getk(b, "producer_code").as_deref(),
                        &getk(b, "consumer").context("schema invariant needs consumer")?,
                        resolver,
                        &mut prov,
                    )?;
                    notes = r.notes;
                    unevaluated = r.unevaluated;
                    witnesses = r.witnesses;
                    r.residue
                }
                None => {
                    if lock.is_some() {
                        notes.push("not in invar.lock; run `invar lock --update` to lock it".into());
                    }
                    let (r, w, u, sk) = schema_outcome(&id, b, resolver, &mut prov)?;
                    witnesses = w;
                    unevaluated = u;
                    skipped = sk;
                    r
                }
            },
            "event" => {
                let p = getk(b, "producer").context("event invariant needs producer")?;
                let c = getk(b, "consumer").context("event invariant needs consumer")?;
                let pr = load_object(&id, &p, &fetch(resolver, &p, &mut prov)?, None)?.to_string();
                let cr = fetch(resolver, &c, &mut prov)?;
                let r = check_contract(&ContractInput::Event {
                    producer_payload: &pr,
                    consumer_handler: &cr,
                });
                let missing = format!("{} (top-level keys)", artifact_name(&p));
                witnesses = read_witnesses(&r, &c, &cr, &missing);
                r
            }
            "lock_order" => {
                let before = parse_step(&getk(b, "before").context("lock_order needs before")?)?;
                let after = parse_step(&getk(b, "after").context("lock_order needs after")?)?;
                let trace = getk(b, "trace").context("lock_order needs trace")?;
                let text = fetch(resolver, &trace, &mut prov)?;
                let observed = parse_trace(&text)?;
                let name = artifact_name(&trace);
                let mut lines: Vec<usize> = step_lines(&text, &after);
                lines.extend(step_lines(&text, &before));
                lines.sort_unstable();
                lines.dedup();
                let r = check_contract(&ContractInput::LockOrder {
                    before,
                    after,
                    observed,
                });
                witnesses = r
                    .iter()
                    .map(|x| Witness {
                        residue: x.clone(),
                        sites: lines.iter().map(|l| format!("{name}:{l}")).collect(),
                        missing_from: None,
                        type_change: None,
                        read_as: Vec::new(),
                    })
                    .collect();
                r
            }
            "egress" => {
                let p = getk(b, "producer").context("egress invariant needs producer")?;
                let shape = getk(b, "shape").context("egress invariant needs shape")?;
                match lock.and_then(|l| l.contracts.iter().find(|c| c.id == id)) {
                    Some(entry) => {
                        let (r, w, n, sk) = egress_outcome(entry, &p, &shape, resolver, &mut prov)?;
                        witnesses = w;
                        notes = n;
                        skipped = sk;
                        r
                    }
                    None => {
                        // R-K1-5: no lock, no verdict. An egress contract has
                        // no unlocked comparison to fall back to.
                        if lock.is_some() {
                            notes.push("not in invar.lock; run `invar lock --update` to lock it".into());
                        }
                        skipped = Some(format!(
                            "not evaluated: no invar.lock entry for this egress contract ({shape} in {}); run `invar lock` first, an unlocked egress contract cannot hold",
                            artifact_name(&p)
                        ));
                        witnesses = Vec::new();
                        Vec::new()
                    }
                }
            }
            "strings" => {
                let p = getk(b, "producer").context("strings invariant needs producer")?;
                let shape = getk(b, "shape").context("strings invariant needs shape")?;
                let list = getk(b, "list").context("strings invariant needs list")?;
                let listat = getk(b, "listat").context("strings invariant needs listat")?;
                match lock.and_then(|l| l.contracts.iter().find(|c| c.id == id)) {
                    Some(entry) => {
                        let (r, w, n, sk) = strings_outcome(entry, &p, &shape, &list, &listat, resolver, &mut prov)?;
                        witnesses = w;
                        notes = n;
                        skipped = sk;
                        r
                    }
                    None => {
                        // R-K3: no lock, no verdict.
                        if lock.is_some() {
                            notes.push("not in invar.lock; run `invar lock --update` to lock it".into());
                        }
                        skipped = Some(format!(
                            "not evaluated: no invar.lock entry for this strings contract ({listat} in {} against {shape} in {}); run `invar lock` first, an unlocked strings contract cannot hold",
                            artifact_name(&list),
                            artifact_name(&p)
                        ));
                        witnesses = Vec::new();
                        Vec::new()
                    }
                }
            }
            other => bail!("unknown invariant kind {other:?} (schema|event|lock_order|egress|strings)"),
        };
        let mut o = ContractOutcome {
            id,
            kind,
            residue,
            provenance: prov,
            witnesses,
            notes,
            unevaluated,
            skipped,
            ..Default::default()
        };
        if let Some(l) = lock {
            // LOCK-SLICE1 (L1): a contract whose lock binds no field has
            // nothing to compare; a HOLDS would be vacuous, so it is not
            // evaluated (exit 4), never passed.
            let unbound = l.contracts.iter().any(|c| c.id == o.id && c.bindings.is_empty());
            if unbound && o.residue.is_empty() && o.skipped.is_none() && o.unevaluated.is_empty() {
                o.unevaluated.push(
                    "the lock binds no field of this contract (0 bindings), so there is nothing to compare and a HOLDS would be vacuous; make the producer's fields readable and run `invar lock --update`".into(),
                );
            }
            apply_waivers(&mut o, &l.waivers, today);
        }
        if let (Some(c), Some((block, entry, _))) = (capture, key) {
            let inputs = c.take();
            let holds = o.skipped.is_none() && !o.violated() && o.unevaluated.is_empty();
            if holds && !inputs.is_empty() {
                records.insert(o.id.clone(), Held { at: now.clone(), engine: ENGINE.to_string(), block, entry, inputs });
            }
        }
        out.push(o);
    }
    Ok((out, records))
}

/// Check one locked egress contract (INIT-DISCOVERY-SLICE2, K1) against the
/// producer's current shape. Additive tolerance (R-K1-3): a locked field that
/// is no longer a direct key of the shape is residue; fields added since the
/// lock are noted and tolerated. A shape that is no longer statically
/// extractable is NOT EVALUATED (R-K1-4), never a pass.
fn egress_outcome(
    entry: &crate::lock::LockedContract,
    producer: &str,
    shape: &str,
    resolver: &Resolver,
    prov: &mut Vec<Fetched>,
) -> Result<(Vec<String>, Vec<Witness>, Vec<String>, Option<String>)> {
    if entry.producer != producer {
        bail!(
            "{}: invar.toml names a different producer than invar.lock; run `invar lock --update`",
            entry.id
        );
    }
    let text = fetch(resolver, producer, prov)?;
    let name = artifact_name(producer);
    let mut notes = vec![
        format!(
            "emitted-shape contract on the schema value surface of {shape}; additions are tolerated; runtime serialization is not verified"
        ),
        if blake3::hash(text.as_bytes()).to_hex().to_string() == entry.blake3 {
            format!("producer unchanged since lock ({})", entry.recorded_at)
        } else {
            format!("producer changed since lock ({})", entry.recorded_at)
        },
    ];
    let live = match crate::valuepairs::read_shape(&text, shape) {
        crate::valuepairs::ShapeRead::Dynamic(why) => {
            return Ok((
                Vec::new(),
                Vec::new(),
                notes,
                Some(format!(
                    "not evaluated: {shape} in {name} is no longer statically extractable ({why}); the locked emitted shape cannot be compared"
                )),
            ));
        }
        crate::valuepairs::ShapeRead::NotFound => None,
        crate::valuepairs::ShapeRead::Found(f) => Some(f),
    };
    let mut residue = Vec::new();
    let mut witnesses = Vec::new();
    for b in entry.bindings.iter().filter(|b| b.scope == shape) {
        let present = live.as_ref().is_some_and(|f| f.iter().any(|x| *x == b.field));
        if present {
            continue;
        }
        let shown = b.display();
        residue.push(shown.clone());
        witnesses.push(Witness {
            residue: shown,
            sites: Vec::new(),
            missing_from: Some(match &live {
                Some(_) => format!("{name} {shape}"),
                None => format!("{name} (no `export const {shape} = z.object(...)` found)"),
            }),
            type_change: None,
            read_as: Vec::new(),
        });
    }
    if let Some(f) = &live {
        let added: Vec<&String> = f
            .iter()
            .filter(|x| !entry.bindings.iter().any(|b| b.scope == shape && b.field == **x))
            .collect();
        if !added.is_empty() {
            notes.push(format!(
                "{} field(s) added since lock, tolerated ({}); `invar lock --update` records them",
                added.len(),
                added.iter().take(5).map(|x| x.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }
    }
    Ok((residue, witnesses, notes, None))
}

/// Check one locked strings contract (INIT-DISCOVERY-SLICE2B, R-K3-7) with
/// "no stale reference" semantics. For each locked name n, using the freshly
/// extracted list L and shape S: n in L and n not in S is residue (VIOLATED);
/// n not in L holds (removed or renamed on the consumer); n in S and not in L
/// holds (a list may narrow); n in both holds. If either side is no longer
/// statically extractable, or an inline list can no longer be located by its
/// locked names, the contract is NOT EVALUATED (exit 4), never a pass.
fn strings_outcome(
    entry: &crate::lock::LockedContract,
    producer: &str,
    shape: &str,
    list: &str,
    listat: &str,
    resolver: &Resolver,
    prov: &mut Vec<Fetched>,
) -> Result<(Vec<String>, Vec<Witness>, Vec<String>, Option<String>)> {
    use crate::surface::name_key;
    use crate::valuepairs::strings::{read_any_shape, read_list, ListRead};
    use crate::valuepairs::ShapeRead;
    if entry.producer != producer {
        bail!(
            "{}: invar.toml names a different producer than invar.lock; run `invar lock --update`",
            entry.id
        );
    }
    let text = fetch(resolver, producer, prov)?;
    let list_text = fetch(resolver, list, prov)?;
    let pname = artifact_name(producer);
    let lname = artifact_name(list);
    let mut notes = vec![
        format!(
            "string-coupling contract: list {listat} in {lname} against the key set of {shape} in {pname}; a name still in the list but gone from the shape is a stale reference; that the list reaches the store is not verified"
        ),
        if blake3::hash(text.as_bytes()).to_hex().to_string() == entry.blake3 {
            format!("shape file unchanged since lock ({})", entry.recorded_at)
        } else {
            format!("shape file changed since lock ({})", entry.recorded_at)
        },
    ];
    let live = match read_any_shape(&text, shape) {
        ShapeRead::Found(f) => f,
        ShapeRead::Dynamic(why) => {
            return Ok((Vec::new(), Vec::new(), notes, Some(format!("not evaluated: {shape} in {pname} is no longer statically extractable ({why}); the locked shape cannot be compared"))));
        }
        ShapeRead::NotFound => {
            return Ok((Vec::new(), Vec::new(), notes, Some(format!("not evaluated: no shape {shape:?} found in {pname}; the locked shape cannot be compared"))));
        }
    };
    let elems = match read_list(&list_text, listat) {
        ListRead::Found(e) => e,
        ListRead::Dynamic(why) => {
            return Ok((Vec::new(), Vec::new(), notes, Some(format!("not evaluated: list {listat} in {lname} is no longer statically extractable ({why}); the locked list cannot be compared"))));
        }
        ListRead::NotFound => {
            return Ok((Vec::new(), Vec::new(), notes, Some(format!("not evaluated: list {listat} no longer found in {lname}; the locked list cannot be compared"))));
        }
    };
    let locked: Vec<&crate::lock::Binding> = entry.bindings.iter().filter(|b| b.scope == shape).collect();
    if listat.starts_with("call:") && !locked.iter().any(|b| elems.iter().any(|(s, _)| name_key(s) == name_key(&b.field))) {
        return Ok((
            Vec::new(),
            Vec::new(),
            notes,
            Some(format!("not evaluated: inline list {listat} in {lname} no longer contains any locked name, so it cannot be told apart from an unrelated list")),
        ));
    }
    let mut residue = Vec::new();
    let mut witnesses = Vec::new();
    for b in &locked {
        let in_list: Vec<usize> = elems.iter().filter(|(s, _)| name_key(s) == name_key(&b.field)).map(|(_, l)| *l).collect();
        let in_shape = live.iter().any(|k| name_key(k) == name_key(&b.field));
        if !in_list.is_empty() && !in_shape {
            let shown = b.display();
            residue.push(shown.clone());
            witnesses.push(Witness {
                residue: shown,
                sites: in_list.iter().map(|l| format!("{lname}:{l}")).collect(),
                missing_from: Some(format!("{pname} {shape}")),
                type_change: None,
                read_as: Vec::new(),
            });
        }
    }
    let added: Vec<&String> = live.iter().filter(|k| !locked.iter().any(|b| name_key(&b.field) == name_key(k))).collect();
    if !added.is_empty() {
        notes.push(format!(
            "{} key(s) of the shape not in the lock, tolerated ({}); `invar lock --update` records them",
            added.len(),
            added.iter().take(5).map(|x| x.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }
    Ok((residue, witnesses, notes, None))
}

/// Move waived residue out of `residue`; record expired waivers (which fail
/// the contract) and active waivers that match nothing (which can go).
pub fn apply_waivers(o: &mut ContractOutcome, waivers: &[crate::lock::Waiver], today: i64) {
    for w in waivers.iter().filter(|w| w.contract == o.id) {
        let left = w.days_left(today);
        if left <= 0 {
            o.expired.push((w.field.clone(), w.owner.clone()));
            continue;
        }
        let before = o.residue.len();
        let mut kept = Vec::new();
        for r in std::mem::take(&mut o.residue) {
            if w.covers(&r) {
                o.waived.push((r, w.owner.clone(), w.reason.clone(), left));
            } else {
                kept.push(r);
            }
        }
        o.residue = kept;
        if o.residue.len() == before {
            o.notes.push(format!("waiver for {} matches no residue; it can be removed", w.field));
        }
    }
    o.witnesses.retain(|x| !o.waived.iter().any(|(f, ..)| f == &x.residue));
}

/// SARIF 2.1.0 export of check outcomes (strategy/DECLARATION-GRAPH.md:
/// become the format platforms ingest). Honest mapping, stated:
/// VIOLATED residue -> level "error" (rule "violated"), one result per
/// residue item, located at the consumer read sites; stated
/// not-evaluated lines -> level "note" under their stable reason code as
/// the rule id (a blind spot rides into the platform UI instead of
/// disappearing); SKIPPED contracts -> level "note" (rule
/// "skipped-unsupported-language"); HOLDS emits no result, because SARIF
/// results are findings and a hold is the absence of one; the fixed
/// summary and the measured record travel in run properties so the
/// export can never read as an effectiveness claim.
pub fn render_sarif(outcomes: &[ContractOutcome]) -> serde_json::Value {
    use serde_json::json;
    let mut rules: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut results = Vec::new();
    let reason_code = |u: &str| -> String {
        u.rfind("[not-evaluated:")
            .and_then(|i| u[i..].strip_prefix('['))
            .and_then(|r| r.strip_suffix(']'))
            .unwrap_or("not-evaluated")
            .to_string()
    };
    let loc = |site: &str| -> serde_json::Value {
        let (file, line) = match site.rsplit_once(':') {
            Some((f, l)) if l.parse::<u64>().is_ok() => (f, l.parse::<u64>().unwrap()),
            _ => (site, 1),
        };
        json!({"physicalLocation": {"artifactLocation": {"uri": file},
               "region": {"startLine": line}}})
    };
    for o in outcomes {
        if let Some(why) = &o.skipped {
            rules.insert(
                "skipped-unsupported-language".into(),
                "The contract names an artifact INVAR does not read; stated, never a pass.".into(),
            );
            results.push(json!({
                "ruleId": "skipped-unsupported-language",
                "level": "note",
                "message": {"text": format!("{} ({}): SKIPPED: {why}", o.id, o.kind)},
            }));
            continue;
        }
        if o.violated() {
            rules.insert(
                "violated".into(),
                "A field the consumer reads changed at the producer.".into(),
            );
            for w in &o.witnesses {
                let text = format!(
                    "{} ({}): {} {}",
                    o.id,
                    o.kind,
                    w.residue,
                    witness_text(w, &o.kind, |x| x.to_string()).trim()
                );
                let locations: Vec<_> = w.sites.iter().map(|s| loc(s)).collect();
                let mut r = json!({"ruleId": "violated", "level": "error",
                                   "message": {"text": text}});
                if !locations.is_empty() {
                    r["locations"] = json!(locations);
                }
                results.push(r);
            }
        }
        for u in &o.unevaluated {
            let code = reason_code(u);
            rules.insert(
                code.clone(),
                "INVAR could not evaluate this and says so instead of passing.".into(),
            );
            results.push(json!({
                "ruleId": code,
                "level": "note",
                "message": {"text": format!("{} ({}): {u}", o.id, o.kind)},
            }));
        }
    }
    let rule_objs: Vec<_> = rules
        .iter()
        .map(|(id, desc)| json!({"id": id, "shortDescription": {"text": desc}}))
        .collect();
    let violations = outcomes.iter().filter(|o| o.violated()).count();
    let skipped = outcomes
        .iter()
        .filter(|o| o.skipped.is_some() || (!o.violated() && !o.unevaluated.is_empty()))
        .count();
    json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {"driver": {
                "name": "invar",
                "version": env!("CARGO_PKG_VERSION"),
                "informationUri": "https://github.com/Ramesh-35/invar",
                "rules": rule_objs
            }},
            "properties": {
                "summary": format!("held {} | violated {violations} | skipped or not evaluated {skipped}",
                                    outcomes.len() - violations - skipped),
                "measured": "38 of 70 held-out real cross-repository breaks caught (batches 4-8; docs/epistemic/STATUS.md)"
            },
            "results": results
        }]
    })
}
