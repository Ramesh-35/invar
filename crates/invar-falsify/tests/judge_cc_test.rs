//! JUDGE-CC r0 fixtures (docs/falsification/JUDGE-CC-SPEC.md), frozen with
//! the spec BEFORE implementation. A pattern, format or non-numeric bound
//! change is a claim this judge cannot decide, so it is NOT EVALUATED
//! (`undecided_constraint`), stated with both values, exit 4: never a
//! VIOLATED without a decidable witness, never a silent pass.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - CC-M1 the undecided branch dropped from the verdict (falls to HOLDS) -> cc_f01
//! - CC-M2 undecided changes left in the witness list (stays VIOLATED)    -> cc_f01
//! - CC-M3 enum narrowing also demoted to undecided                       -> cc_f03

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_cc_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(d: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(args)
        .current_dir(d)
        .env("GIT_AUTHOR_NAME", "cc").env("GIT_AUTHOR_EMAIL", "cc@test")
        .env("GIT_COMMITTER_NAME", "cc").env("GIT_COMMITTER_EMAIL", "cc@test")
        .output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

fn repo(tag: &str, pre: &str, post: &str) -> PathBuf {
    let d = ws(tag);
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    std::fs::write(d.join("schemas/contract.json"), pre).unwrap();
    git(&d, &["add", "schemas/contract.json"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    std::fs::write(d.join("schemas/contract.json"), post).unwrap();
    d
}

fn judge(d: &Path, extra: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["judge", "--contract", "schemas/contract.json", "--base", "main"])
        .args(extra)
        .current_dir(d)
        .env("HOME", d)
        .output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned())
}

fn json(d: &Path) -> (i32, serde_json::Value) {
    let (code, out) = judge(d, &["--format", "json"]);
    (code, serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}")))
}

/// cc_f01: a pattern changed and nothing else. NOT EVALUATED
/// (undecided_constraint), exit 4, both patterns stated, no witness, no
/// counterexample claimed. Kills CC-M1 and CC-M2.
#[test]
fn cc_f01_pattern_change_alone_is_not_evaluated() {
    let pre = r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{3}$"}}}"#;
    let post = r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{2}$"}}}"#;
    let d = repo("f01", pre, post);
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 4, "out: {out}");
    assert!(out.contains("NOT EVALUATED (undecided_constraint)"), "out: {out}");
    assert!(out.contains("code pattern was"), "out: {out}");
    assert!(out.contains("not decided"), "out: {out}");
    assert!(!out.contains("VIOLATED"), "out: {out}");
    let (_, v) = json(&d);
    assert_eq!(v["verdict"], serde_json::json!("NOT EVALUATED"), "{v}");
    assert_eq!(v["reason"], serde_json::json!("undecided_constraint"), "{v}");
    assert_eq!(v["witnesses"], serde_json::json!([]), "{v}");
    assert_eq!(v["undecided_paths"], serde_json::json!(["code"]), "{v}");
}

/// cc_f02: format uri -> uri-reference (the SARIF class: the new format
/// accepts more). Still NOT EVALUATED, never VIOLATED: the judge does not
/// decide format containment in either direction.
#[test]
fn cc_f02_format_change_is_not_evaluated() {
    let pre = r#"{"properties":{"u":{"type":"string","format":"uri"}}}"#;
    let post = r#"{"properties":{"u":{"type":"string","format":"uri-reference"}}}"#;
    let d = repo("f02", pre, post);
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 4, "out: {out}");
    assert!(out.contains("undecided_constraint"), "out: {out}");
    assert!(out.contains("u format was"), "out: {out}");
    assert!(!out.contains("VIOLATED"), "out: {out}");
}

/// cc_f03 guard: an enum narrowing is decided and stays VIOLATED with its
/// lost value; demotion must not leak onto decidable witnesses. Kills CC-M3.
#[test]
fn cc_f03_enum_narrowing_stays_violated() {
    let pre = r#"{"properties":{"status":{"type":"string","enum":["a","b","c"]}}}"#;
    let post = r#"{"properties":{"status":{"type":"string","enum":["a","b"]}}}"#;
    let d = repo("f03", pre, post);
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 1, "out: {out}");
    assert!(out.contains("VIOLATED"), "out: {out}");
    assert!(out.contains("narrowed"), "out: {out}");
}

/// cc_f04: a removal and a pattern change in one diff. The removal is a
/// decided break, so VIOLATED stands, and the pattern change is stated as
/// undecided beside it: neither silenced by the other.
#[test]
fn cc_f04_mixed_change_keeps_violated_and_states_undecided() {
    let pre = r#"{"properties":{"name":{"type":"string"},"code":{"type":"string","pattern":"^[A-Z]{3}$"}}}"#;
    let post = r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{2}$"}}}"#;
    let d = repo("f04", pre, post);
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 1, "out: {out}");
    assert!(out.contains("VIOLATED"), "out: {out}");
    assert!(out.contains("removed"), "out: {out}");
    assert!(out.contains("not evaluated: code pattern was"), "out: {out}");
    let (_, v) = json(&d);
    assert_eq!(v["verdict"], serde_json::json!("VIOLATED"), "{v}");
    assert_eq!(v["undecided_paths"], serde_json::json!(["code"]), "{v}");
    assert_eq!(v["witnesses"].as_array().map(|a| a.len()), Some(1), "{v}");
}

/// cc_f05 guard: a pattern removed is a loosening and stays HOLDS; the
/// demotion must not turn loosenings into NOT EVALUATED.
#[test]
fn cc_f05_pattern_removed_stays_holds() {
    let pre = r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{3}$"}}}"#;
    let post = r#"{"properties":{"code":{"type":"string"}}}"#;
    let d = repo("f05", pre, post);
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 0, "out: {out}");
    assert!(out.contains("HOLDS"), "out: {out}");
}

/// cc_f06: a pattern added where none existed. Rejecting strings is what
/// patterns do, but whether this one rejects anything is regex analysis
/// this judge does not perform: NOT EVALUATED, stated as added.
#[test]
fn cc_f06_pattern_added_is_not_evaluated() {
    let pre = r#"{"properties":{"f":{"type":"string"}}}"#;
    let post = r#"{"properties":{"f":{"type":"string","pattern":"^a"}}}"#;
    let d = repo("f06", pre, post);
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 4, "out: {out}");
    assert!(out.contains("undecided_constraint"), "out: {out}");
    assert!(out.contains("f pattern was absent, is now"), "out: {out}");
}
