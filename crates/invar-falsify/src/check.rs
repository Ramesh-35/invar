//! Manifest evaluation behind `invar check`.
//!
//! Reads the parsed sections of an `invar.toml`, gathers every `[[invariant]]`
//! block (or the legacy single `[producer]`/`[consumer]` schema form), resolves
//! each referenced artifact through a `Resolver` (local file, or remote fetched
//! into memory under `--ci`), and dispatches to `invariant::check_contract`.
//! Each outcome carries the provenance of any remote artifact it used.

use anyhow::{anyhow, bail, Context, Result};

use crate::invariant::{check_contract, select_pointer, ContractInput, Step};
use crate::resolver::{Fetched, Resolver};

pub type Sections = Vec<(String, Vec<(String, String)>)>;

#[derive(Debug, Clone)]
pub struct ContractOutcome {
    pub id: String,
    pub kind: String,
    pub residue: Vec<String>,
    pub provenance: Vec<Fetched>,
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

/// Markdown report for a PR comment or job summary. States plainly that a
/// contract violation is not a PROVEN_RESIDUE verdict.
pub fn render_report_md(outcomes: &[ContractOutcome]) -> String {
    let violated = outcomes.iter().filter(|o| !o.residue.is_empty()).count();
    let mut s = String::new();
    s.push_str(REPORT_MARKER);
    s.push('\n');
    if violated == 0 {
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
        let (result, residue) = if o.residue.is_empty() {
            ("holds", String::new())
        } else {
            let mut cells: Vec<String> =
                o.residue.iter().take(MAX_RESIDUE_SHOWN).map(|r| md_code(r)).collect();
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
        let residue = match kind.as_str() {
            "schema" => {
                let p = getk(b, "producer").context("schema invariant needs producer")?;
                let c = getk(b, "consumer").context("schema invariant needs consumer")?;
                let mut pr = fetch(resolver, &p, &mut prov)?;
                if let Some(ptr) = getk(b, "pointer") {
                    pr = select_pointer(&pr, &ptr).map_err(|e| anyhow!("{id}: {e}"))?;
                }
                let cr = fetch(resolver, &c, &mut prov)?;
                check_contract(&ContractInput::Schema { producer_schema: &pr, consumer_source: &cr })
            }
            "event" => {
                let p = getk(b, "producer").context("event invariant needs producer")?;
                let c = getk(b, "consumer").context("event invariant needs consumer")?;
                let pr = fetch(resolver, &p, &mut prov)?;
                let cr = fetch(resolver, &c, &mut prov)?;
                check_contract(&ContractInput::Event { producer_payload: &pr, consumer_handler: &cr })
            }
            "lock_order" => {
                let before = parse_step(&getk(b, "before").context("lock_order needs before")?)?;
                let after = parse_step(&getk(b, "after").context("lock_order needs after")?)?;
                let trace = getk(b, "trace").context("lock_order needs trace")?;
                let observed = parse_trace(&fetch(resolver, &trace, &mut prov)?)?;
                check_contract(&ContractInput::LockOrder { before, after, observed })
            }
            other => bail!("unknown invariant kind {other:?} (schema|event|lock_order)"),
        };
        out.push(ContractOutcome { id, kind, residue, provenance: prov });
    }
    Ok(out)
}
