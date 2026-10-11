//! `invar demo` and `invar coverage` (docs/architecture/DEMO-ENGINE.md).
//!
//! `demo` replays real, caught cross-repository breaks from bytes shipped in
//! the repository's `demo/` directory: fully offline (local resolver, no
//! transport, by construction), through the stock engine path, with every
//! embedded file verified against the BLAKE3 the corpus pins. Its VIOLATED
//! verdicts ARE the demonstration; a demo case that does not come out
//! VIOLATED is a packaging error (exit 3), never a quiet pass. Wall time is
//! printed as measured on this machine, never promised in advance.
//!
//! `coverage` answers "what does INVAR actually evaluate here, and what does
//! it not": per contract, the static leg, the observed-reality leg, and the
//! fixed blind spots, with the stable reason codes each can state
//! (docs/reference/REASON-CODES.md). It renders disclosure, never a verdict:
//! exit 0 on success, 3 on error, never 1.

use crate::check::{evaluate_with, gather_blocks, outcome_lines, unsupported_language};
use crate::config::parse_sections;
use crate::lock::{self, SAMPLE_STALE_DAYS};
use crate::resolver::Resolver;
use anyhow::{bail, Context, Result};
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// The demo's closing line, frozen verbatim in DEMO-ENGINE.md. The "38 of 70"
/// figure is the same held-out record as the check summary's measured line;
/// when the record moves, both move in the same commit.
pub const DEMO_CODA: &str = "This is a real break INVAR catches. On held-out real breaks overall \
it currently catches 38 of 70; what it cannot evaluate it states instead of passing. \
invar init to try it on your own repositories.";

struct DemoCase {
    dir: String,
    title: String,
    story: String,
    source: String,
    /// (file name in the case dir, original pinned reference, BLAKE3 hex).
    files: Vec<(String, String, String)>,
}

/// Walk upward from `start` to the nearest directory containing
/// `demo/demo.toml` (a fresh clone or an unpacked source tarball).
fn find_demo_dir(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        let cand = dir.join("demo");
        if cand.join("demo.toml").is_file() {
            return Some(cand);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn parse_manifest(path: &Path) -> Result<Vec<DemoCase>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let sections = parse_sections(&path.display().to_string(), &text)?;
    let mut cases = Vec::new();
    for (name, kv) in &sections {
        if name != "case" {
            bail!("demo.toml: unknown section {name:?} (expected [[case]])");
        }
        let get = |k: &str| -> Result<String> {
            kv.iter()
                .find(|(kk, _)| kk == k)
                .map(|(_, v)| v.clone())
                .with_context(|| format!("demo.toml: [[case]] is missing {k:?}"))
        };
        let mut files = Vec::new();
        for (k, v) in kv {
            if k == "file" {
                let mut it = v.split_whitespace();
                match (it.next(), it.next(), it.next()) {
                    (Some(f), Some(r), Some(h)) => {
                        files.push((f.to_string(), r.to_string(), h.to_string()))
                    }
                    _ => bail!("demo.toml: file needs `name reference blake3`, got {v:?}"),
                }
            }
        }
        if files.is_empty() {
            bail!("demo.toml: [[case]] {} has no file pins", get("id")?);
        }
        cases.push(DemoCase {
            dir: get("dir")?,
            title: get("title")?,
            story: get("story")?,
            source: get("source")?,
            files,
        });
    }
    if cases.is_empty() {
        bail!("demo.toml: no [[case]] sections");
    }
    Ok(cases)
}

/// Run the demo, writing its output into `out`. Returns the exit code:
/// 0 when every shipped case replays to VIOLATED, 3 on any error (missing
/// files, a BLAKE3 mismatch, or a case that no longer comes out VIOLATED).
pub fn run_demo(start: &Path, out: &mut String) -> Result<i32> {
    let Some(root) = find_demo_dir(start) else {
        bail!(
            "demo/demo.toml not found here or in any parent directory; the demo ships inside \
             the INVAR repository and its source tarballs, so run `invar demo` from a clone \
             or an unpacked tarball"
        );
    };
    let cases = parse_manifest(&root.join("demo.toml"))?;
    let _ = writeln!(
        out,
        "INVAR demo: {} real cross-repository breaks, replayed offline from bytes pinned in demo/.",
        cases.len()
    );
    for c in &cases {
        let cdir = root.join(&c.dir);
        let _ = writeln!(out);
        let _ = writeln!(out, "== {} ==", c.title);
        for (fname, reference, pin) in &c.files {
            let p = cdir.join(fname);
            let bytes =
                std::fs::read(&p).with_context(|| format!("reading {}", p.display()))?;
            let got = blake3::hash(&bytes).to_hex().to_string();
            if got != *pin {
                bail!(
                    "{}/{fname} does not match its pinned BLAKE3 (expected {pin}, got {got}); \
                     the shipped demo bytes were modified",
                    c.dir
                );
            }
            let _ = writeln!(out, "   pinned: {fname} = {reference} (blake3 verified)");
        }
        let cfg = cdir.join("invar.toml");
        let t0 = std::time::Instant::now();
        let raw = std::fs::read_to_string(&cfg)
            .with_context(|| format!("reading {}", cfg.display()))?;
        let sections = parse_sections(&cfg.display().to_string(), &raw)?;
        // Local resolver, no transport: the demo cannot reach the network
        // even by accident.
        let resolver = Resolver::local(&cdir);
        let outcomes = evaluate_with(&sections, &resolver, None, lock::today_utc())?;
        let elapsed = t0.elapsed();
        for o in &outcomes {
            let (tag, text) = outcome_lines(o);
            out.push_str(&text);
            if tag != "VIOLATED" {
                bail!(
                    "demo case {} came out {tag}, expected VIOLATED; the shipped demo is \
                     broken, please report this",
                    o.id
                );
            }
        }
        let _ = writeln!(out, "   what happened: {}", c.story);
        let _ = writeln!(out, "   source: {}", c.source);
        let _ = writeln!(
            out,
            "   measured: {:.1} ms this run (config parse + evaluation, this machine)",
            elapsed.as_secs_f64() * 1000.0
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "{DEMO_CODA}");
    Ok(0)
}

/// Run coverage disclosure, writing output into `out`. Exit 0 on success,
/// errors bubble to exit 3; never a verdict.
pub fn run_coverage(
    config_path: &str,
    only_contract: Option<&str>,
    format: &str,
    measured_record: &str,
    out: &mut String,
) -> Result<i32> {
    if format != "text" && format != "json" {
        bail!("--format must be \"text\" or \"json\", got {format:?}");
    }
    let raw = std::fs::read_to_string(config_path)
        .with_context(|| format!("reading {config_path}"))?;
    let sections = parse_sections(config_path, &raw)?;
    let blocks = gather_blocks(&sections)?;
    let base_dir = Path::new(config_path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let lock_path = base_dir.join("invar.lock");
    let lockf = if lock_path.exists() {
        let text = std::fs::read_to_string(&lock_path)
            .with_context(|| format!("reading {}", lock_path.display()))?;
        Some(lock::parse(&lock_path.display().to_string(), &text)?)
    } else {
        None
    };
    let today = lock::today_utc();
    let getk = |b: &[(String, String)], k: &str| -> Option<String> {
        b.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone())
    };

    let mut rows = Vec::new();
    for (idx, b) in blocks.iter().enumerate() {
        let id = getk(b, "id").unwrap_or_else(|| format!("inv-{idx}"));
        if let Some(want) = only_contract {
            if id != want {
                continue;
            }
        }
        let kind = getk(b, "kind").unwrap_or_default();
        let producer = getk(b, "producer").unwrap_or_default();
        let producer_base = getk(b, "producer_base");
        let producer_code = getk(b, "producer_code");
        let consumer = getk(b, "consumer").unwrap_or_default();
        let pointer = getk(b, "pointer");

        // Static leg.
        let unsupported: Vec<String> = [
            Some(&producer),
            producer_base.as_ref(),
            producer_code.as_ref(),
            Some(&consumer),
        ]
        .iter()
        .flatten()
        .filter(|s| !s.is_empty())
        .filter_map(|s| {
            unsupported_language(s).map(|l| format!("{l}: {}", crate::check::artifact_name(s)))
        })
        .collect();
        let locked = lockf
            .as_ref()
            .and_then(|l| l.contracts.iter().find(|c| c.id == id));
        let static_mode = if (kind == "schema" || kind == "event") && !unsupported.is_empty() {
            format!(
                "SKIPPED, language not supported ({}) [not-evaluated:unsupported-language]",
                unsupported.join("; ")
            )
        } else {
            match kind.as_str() {
                "schema" => match (&locked, &producer_base, &producer_code) {
                    (Some(e), _, _) => format!(
                        "locked slice from invar.lock (recorded {}): consumer reads vs the locked fields",
                        e.recorded_at
                    ),
                    (None, Some(_), _) | (None, None, Some(_)) => {
                        "change-scoped diff: producer_base vs producer, filtered to what the consumer reads".to_string()
                    }
                    (None, None, None) => {
                        "full surface: producer fields vs consumer reads".to_string()
                    }
                },
                "event" => "top-level keys of the producer payload vs consumer reads".to_string(),
                "lock_order" => "ordering of the two steps in the trace".to_string(),
                other => format!("unknown kind {other:?}"),
            }
        };
        let accuses = match kind.as_str() {
            "schema" => "a field the consumer reads that the producer removed or retyped",
            "event" => "a top-level key the consumer reads that the payload lacks",
            "lock_order" => "the two steps observed in the wrong order",
            _ => "nothing (unknown kind)",
        };
        let states = match kind.as_str() {
            "schema" => vec![
                "field moved between producer types [not-evaluated:field-moved]",
                "consumer read form not parsed [not-evaluated:pattern-not-read]",
                "default selector switched at the producer [not-evaluated:default-switch]",
            ],
            _ => Vec::new(),
        };

        // Observed-reality leg.
        let sample = lockf
            .as_ref()
            .and_then(|l| l.samples.iter().find(|s| s.contract == id));
        let observed = match sample {
            Some(s) => {
                let wild = s.witnesses.iter().filter(|w| w.path.contains('*')).count();
                let age = lock::parse_date(&s.taken).map(|d| today - d).unwrap_or(0);
                let staleness = if age > SAMPLE_STALE_DAYS {
                    format!(
                        "STALE: last observation {} is older than {SAMPLE_STALE_DAYS} days; drift is stated, not accused [not-evaluated:stale-sample]",
                        s.taken
                    )
                } else {
                    format!("fresh (last observation {})", s.taken)
                };
                format!(
                    "{} observation(s), window {}..{}, {} path witness(es) ({} wildcarded); {}; accuses only a path present in every observation with 3+ observations; optional absences stated [not-evaluated:optional-field-absent]; under 3 observations stated [not-evaluated:witness-count-low]",
                    s.observations,
                    s.window_start,
                    s.taken,
                    s.witnesses.len(),
                    wild,
                    staleness
                )
            }
            None => {
                if kind == "schema" {
                    "no locked sample: runtime payload drift is NOT evaluated (invar lock --sample <file> adds witnesses)".to_string()
                } else {
                    "not applicable to this kind".to_string()
                }
            }
        };
        let never = "field values and semantics; runtime behavior; payloads that are not JSON (TOML, YAML, binary)";

        rows.push((
            id, kind, producer, producer_base, consumer, pointer, static_mode, accuses, states,
            observed, never,
        ));
    }
    if rows.is_empty() {
        match only_contract {
            Some(c) => bail!("{config_path}: no contract with id {c:?}"),
            None => bail!("{config_path}: no [[invariant]] blocks"),
        }
    }

    if format == "json" {
        let arr: Vec<serde_json::Value> = rows
            .iter()
            .map(
                |(id, kind, producer, base, consumer, pointer, mode, accuses, states, observed, never)| {
                    serde_json::json!({
                        "id": id,
                        "kind": kind,
                        "producer": producer,
                        "producer_base": base,
                        "consumer": consumer,
                        "pointer": pointer,
                        "static": mode,
                        "accuses": accuses,
                        "states_never_accuses": states,
                        "observed_reality": observed,
                        "never_evaluated": never,
                    })
                },
            )
            .collect();
        let doc = serde_json::json!({
            "config": config_path,
            "contracts": arr,
            "measured": measured_record,
        });
        let _ = writeln!(out, "{}", serde_json::to_string_pretty(&doc).unwrap_or_default());
        return Ok(0);
    }

    let _ = writeln!(out, "INVAR coverage ({config_path}, {} contract(s))", rows.len());
    for (id, kind, producer, base, consumer, pointer, mode, accuses, states, observed, never) in
        &rows
    {
        let _ = writeln!(out, "  {id} ({kind})");
        let _ = writeln!(out, "    producer: {producer}");
        if let Some(b) = base {
            let _ = writeln!(out, "    producer_base: {b}");
        }
        let _ = writeln!(out, "    consumer: {consumer}");
        if let Some(p) = pointer {
            let _ = writeln!(out, "    pointer: {p}");
        }
        let _ = writeln!(out, "    static: {mode}");
        let _ = writeln!(out, "    accuses: {accuses}");
        for st in states {
            let _ = writeln!(out, "    states, never accuses: {st}");
        }
        let _ = writeln!(out, "    observed reality: {observed}");
        let _ = writeln!(out, "    never evaluated: {never}");
    }
    let _ = writeln!(out, "measured: {measured_record}");
    Ok(0)
}
