//! INVAR-EXPLAIN-001 fixtures (docs/architecture/INVAR-EXPLAIN-001.md,
//! rulings D-E1 to D-E6 of 2026-10-18). GATE 1: written BEFORE src/explain.rs
//! exists; against the unchanged engine every fixture fails with clap's
//! "unrecognized subcommand" (exit 2), the declared Gate 1 reason.
//!
//! The registry at implementation time holds 33 entries: 19 engine abstention
//! codes, 8 proof codes (the spec's "24 lines" predates the proof landing,
//! recorded in the ledger), UNKNOWN-EXPLAIN-CODE itself, and the 5 exit
//! numerals. X-M1 (every existing output byte-identical) is the existing
//! suites, which must stay green unchanged.
//!
//! The drift test (R-X-6) is black-box: it compares the codes extracted from
//! the crate sources against the binary's own `invar explain` index, so no
//! Class 1 file is touched and no library symbol is linked.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_xpl_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    dir
}

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_invar")
}

fn explain_in(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, String) {
    let mut c = Command::new(bin());
    c.arg("explain").args(args).current_dir(dir);
    for (k, v) in env {
        c.env(k, v);
    }
    let o = c.output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn explain(args: &[&str]) -> (i32, String) {
    let d = ws("plain", &[]);
    let (c1, o1) = explain_in(&d, args, &[]);
    let (c2, o2) = explain_in(&d, args, &[]);
    assert_eq!((c1, &o1), (c2, &o2), "explain is not deterministic");
    (c1, o1)
}

/// Parse the index: one `CODE (kind): condition` line per entry.
fn index() -> Vec<(String, String)> {
    let (c, out) = explain(&[]);
    assert_eq!(c, 0, "{out}");
    out.lines()
        .filter_map(|l| {
            let (code, rest) = l.split_once(" (")?;
            let (kind, _) = rest.split_once("): ")?;
            Some((code.trim().to_string(), kind.to_string()))
        })
        .collect()
}

/// Every code the engine can emit, extracted from the crate sources: the
/// `code: "X"` struct-literal sites and proof.rs's `abstain("X"` sites.
fn emitted_codes() -> (BTreeSet<String>, BTreeSet<String>) {
    let src_dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
    let mut files = Vec::new();
    collect_rs(&src_dir, &mut files);
    assert!(files.len() >= 5, "the drift scan must sweep the whole crate, found {} files", files.len());
    let mut codes = BTreeSet::new();
    let mut provenance = BTreeSet::new();
    for f in &files {
        let text = std::fs::read_to_string(f).unwrap();
        let mut found = false;
        for pat in ["code: \"", "abstain(\""] {
            for (i, _) in text.match_indices(pat) {
                let rest = &text[i + pat.len()..];
                if let Some(end) = rest.find('"') {
                    let code = &rest[..end];
                    if !code.is_empty() && code.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-') {
                        codes.insert(code.to_string());
                        found = true;
                    }
                }
            }
        }
        if found {
            provenance.insert(f.file_name().unwrap().to_string_lossy().to_string());
        }
    }
    (codes, provenance)
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()) {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

// ------------------------------------------------------------- fixtures

#[test]
fn x_f1_entry_is_five_lines_and_tsc_governed_names_both_emitters() {
    let (c, out) = explain(&["TSC-GOVERNED"]);
    assert_eq!(c, 0, "{out}");
    let lines: Vec<&str> = out.trim_end().lines().collect();
    assert_eq!(lines.len(), 5, "an entry is exactly five lines: {out}");
    assert!(lines[0].starts_with("TSC-GOVERNED (abstention)"), "{out}");
    assert!(lines[1].starts_with("scope: "), "{out}");
    assert!(lines[2].starts_with("condition: "), "{out}");
    assert!(lines[3].starts_with("action: "), "{out}");
    assert!(lines[4].starts_with("defined in: "), "{out}");
    // The condition names both emitters: K2 type-governed consumption and
    // K3 keyof-constrained lists (REV1).
    assert!(out.contains("type") && out.contains("keyof"), "{out}");
}

#[test]
fn x_f2_index_is_sorted_complete_and_exactly_once() {
    let idx = index();
    // INIT-DISCOVERY-SLICE5 (ruling S5-D1) adds the engine code RECV. Line
    // before SLICE5, kept verbatim:
    //     assert_eq!(idx.len(), 33, "19 engine + 8 proof + UNKNOWN-EXPLAIN-CODE + 5 exit numerals: {idx:?}");
    assert_eq!(idx.len(), 34, "20 engine + 8 proof + UNKNOWN-EXPLAIN-CODE + 5 exit numerals: {idx:?}");
    let codes: Vec<&String> = idx.iter().map(|(c, _)| c).collect();
    let mut sorted = codes.clone();
    sorted.sort();
    assert_eq!(codes, sorted, "the index is sorted by code");
    let set: BTreeSet<&&String> = codes.iter().collect();
    assert_eq!(set.len(), codes.len(), "every code appears exactly once");
    for numeral in ["0", "1", "2", "3", "4"] {
        assert!(idx.iter().any(|(c, k)| c == numeral && k == "exit"), "exit numeral {numeral} missing");
    }
}

#[test]
fn x_f3_unknown_code_is_exit_3_with_the_known_list() {
    let (c, out) = explain(&["NOT-A-CODE"]);
    assert_eq!(c, 3, "{out}");
    assert!(out.contains("UNKNOWN-EXPLAIN-CODE") && out.contains("NOT-A-CODE"), "{out}");
    assert!(out.contains("TSC-GOVERNED") && out.contains("PROOF-FORK"), "the known codes are listed: {out}");
    // No fuzzy suggestion machinery (ruling D-E3).
    assert!(!out.to_lowercase().contains("did you mean"), "{out}");
}

#[test]
fn x_f4_exit_numerals_are_explained_and_five_is_unknown() {
    for n in ["0", "1", "2", "3", "4"] {
        let (c, out) = explain(&[n]);
        assert_eq!(c, 0, "explain {n}: {out}");
        assert!(out.starts_with(&format!("{n} (exit)")), "{out}");
    }
    let (_, two) = explain(&["2"]);
    assert!(two.contains("ledger") && two.contains("never returned by proof verification"), "CM-X-7: {two}");
    let (c, out) = explain(&["5"]);
    assert_eq!(c, 3, "{out}");
}

#[test]
fn x_f5_explain_reads_no_repository_state() {
    let empty = ws("f5a", &[]);
    let malformed = ws("f5b", &[("invar.toml", "this is [ not toml\n= broken\n")]);
    let busy = ws("f5c", &[("invar.toml", "[[invariant]]\nkind = \"egress\"\nid = \"x\"\n"), ("invar.lock", "garbage\n"), ("invar.proof", "junk\n")]);
    for args in [vec![], vec!["TSC-GOVERNED"], vec!["NOT-A-CODE"]] {
        let a = explain_in(&empty, &args, &[]);
        let b = explain_in(&malformed, &args, &[]);
        let c = explain_in(&busy, &args, &[]);
        assert_eq!(a, b, "output varies with repository state: {args:?}");
        assert_eq!(a, c, "output varies with repository state: {args:?}");
    }
}

#[test]
fn x_f6_registry_and_emission_sites_cannot_drift() {
    let (emitted, provenance) = emitted_codes();
    assert!(provenance.len() >= 3, "codes must come from at least three source files, found {provenance:?}");
    assert!(provenance.contains("proof.rs") && provenance.contains("strings.rs"), "{provenance:?}");
    let idx = index();
    let registry: BTreeSet<String> = idx
        .iter()
        .filter(|(_, k)| k == "abstention" || k == "error")
        .map(|(c, _)| c.clone())
        .collect();
    // Direction a: every emitted code has a registry entry.
    for code in &emitted {
        assert!(registry.contains(code), "emitted code {code} has no registry entry (drift)");
    }
    // Direction b: every registry entry is emitted somewhere, except explain's
    // own error marker, which lives in explain.rs itself.
    for code in &registry {
        if code == "UNKNOWN-EXPLAIN-CODE" {
            let xp = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/explain.rs")).unwrap();
            assert!(xp.contains("UNKNOWN-EXPLAIN-CODE"), "explain's own marker is gone");
            continue;
        }
        assert!(emitted.contains(code), "registry entry {code} is emitted nowhere (drift)");
    }
}

#[test]
fn x_f7_locale_and_timezone_change_nothing() {
    let d = ws("f7", &[]);
    let base = explain_in(&d, &["PROOF-FORK"], &[]);
    assert_eq!(base.0, 0, "{}", base.1);
    for env in [vec![("LC_ALL", "C")], vec![("LC_ALL", "en_US.UTF-8")], vec![("TZ", "UTC")], vec![("TZ", "Asia/Ho_Chi_Minh")]] {
        let v = explain_in(&d, &["PROOF-FORK"], &env);
        assert_eq!(base, v, "output varies under {env:?}");
    }
    let bi = explain_in(&d, &[], &[("LC_ALL", "C")]);
    let bj = explain_in(&d, &[], &[("TZ", "UTC")]);
    assert_eq!(bi, bj);
}
