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
    broad_reads, changed_fields, code_fields, field_names, name_key, spec_fields, Fields,
};

pub type Sections = Vec<(String, Vec<(String, String)>)>;

#[derive(Debug, Clone)]
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
}

/// Languages INVAR does not read (alpha.4 scope: Go, Python, JS/TS, Rust).
fn unsupported_language(spec: &str) -> Option<&'static str> {
    let name = artifact_name(spec);
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    Some(match ext.as_str() {
        "c" | "h" => "C",
        "cc" | "cpp" | "cxx" | "hpp" | "hh" => "C++",
        "cs" => "C#",
        "php" => "PHP",
        "swift" => "Swift",
        _ => return None,
    })
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

fn is_code(spec: &str) -> bool {
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
fn load_object(
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
fn source_fields(id: &str, spec: &str, text: &str, pointer: Option<&str>) -> Result<Fields> {
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
) -> Result<(Vec<String>, Vec<Witness>)> {
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
        return Ok((r.clone(), read_witnesses(&r, &c, &cr, &missing)));
    }

    // Change-scoped: collect what changed in each producer source.
    let mut changed: BTreeMap<String, (Option<(String, String)>, String)> = BTreeMap::new();
    let mut add = |head_spec: &str, head: &Fields, base: &Fields, where_: String| {
        for (name, tc) in changed_fields(base, head) {
            changed.entry(name).or_insert((tc, where_.clone()));
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
    }
    if let (Some(cs), Some(cbs)) = (&code, &code_base) {
        let cr_head = fetch(resolver, cs, prov)?;
        let cr_base = fetch(resolver, cbs, prov)?;
        let head_f = source_fields(id, cs, &cr_head, None)?;
        let base_f = source_fields(id, cbs, &cr_base, None)?;
        add(cs, &head_f, &base_f, artifact_name(cs));
    }
    let cr = fetch(resolver, &c, prov)?;
    let reads = broad_reads(&cr);
    let cname = artifact_name(&c);
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
    Ok((residue, witnesses))
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

fn getk(pairs: &[(String, String)], k: &str) -> Option<String> {
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

fn fetch(resolver: &Resolver, spec: &str, prov: &mut Vec<Fetched>) -> Result<String> {
    let r = resolver.resolve(spec).map_err(|e| anyhow!("{e}"))?;
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

/// Markdown report for a PR comment or job summary. States plainly that a
/// contract violation is not a PROVEN_RESIDUE verdict.
pub fn render_report_md(outcomes: &[ContractOutcome]) -> String {
    let violated = outcomes.iter().filter(|o| !o.residue.is_empty()).count();
    let skipped = outcomes.iter().filter(|o| o.skipped.is_some()).count();
    let mut s = String::new();
    s.push_str(REPORT_MARKER);
    s.push('\n');
    if violated == 0 && skipped > 0 {
        s.push_str(&format!(
            "### INVAR contract check: NOT FULLY EVALUATED ({skipped} of {} skipped)\n\n",
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
        let (result, residue) = if let Some(why) = &o.skipped {
            ("SKIPPED", md_code(why))
        } else if o.residue.is_empty() {
            ("holds", String::new())
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

/// Evaluate every contract declared in the manifest.
pub fn evaluate(sections: &Sections, resolver: &Resolver) -> Result<Vec<ContractOutcome>> {
    let blocks = gather_blocks(sections)?;
    let mut out = Vec::with_capacity(blocks.len());
    for (idx, b) in blocks.iter().enumerate() {
        let kind = getk(b, "kind").unwrap_or_default();
        let id = getk(b, "id").unwrap_or_else(|| format!("inv-{idx}"));
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
                });
                continue;
            }
        }
        let witnesses: Vec<Witness>;
        let residue = match kind.as_str() {
            "schema" => {
                let (r, w) = schema_outcome(&id, b, resolver, &mut prov)?;
                witnesses = w;
                r
            }
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
            other => bail!("unknown invariant kind {other:?} (schema|event|lock_order)"),
        };
        out.push(ContractOutcome {
            id,
            kind,
            residue,
            provenance: prov,
            witnesses,
            skipped: None,
        });
    }
    Ok(out)
}
