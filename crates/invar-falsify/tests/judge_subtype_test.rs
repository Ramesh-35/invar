//! JUDGE-SUBTYPE r1 fixtures (docs/falsification/JUDGE-SUBTYPE-SPEC.md),
//! frozen with the spec BEFORE implementation. r1 reads a schema as a
//! validation contract: narrowing what it accepts is VIOLATED, widening
//! HOLDS. New kinds: narrowed (enum), tightened (bounds), escalated
//! (required), constraint_changed (pattern, format: changes are flagged,
//! never claimed as proven narrowings).
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - JS-M1 enum narrowing check dropped          -> js_f01
//! - JS-M2 bounds comparison inverted            -> js_f03, js_f04
//! - JS-M3 required diff dropped                 -> js_f05
//! - JS-M4 pattern/format change treated as HOLDS -> js_f06

use std::path::{Path, PathBuf};
use std::process::Command;

const B1_001_PRE: &str = include_str!("fixtures/judge/b1-001-pre.json");
const B1_001_POST: &str = include_str!("fixtures/judge/b1-001-post.json");

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_js_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(d: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(args)
        .current_dir(d)
        .env("GIT_AUTHOR_NAME", "js").env("GIT_AUTHOR_EMAIL", "js@test")
        .env("GIT_COMMITTER_NAME", "js").env("GIT_COMMITTER_EMAIL", "js@test")
        .output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

/// Base commit holds `pre` at schemas/contract.json; working tree holds `post`.
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

fn status(values: &str) -> String {
    format!(r#"{{"properties":{{"status":{{"type":"string","enum":{values}}}}}}}"#)
}

fn age(bounds: &str) -> String {
    format!(r#"{{"properties":{{"age":{{"type":"integer"{bounds}}}}}}}"#)
}

/// js_f01: an enum loses a value. Senders using it are now rejected.
/// VIOLATED narrowed, the lost value named. Kills JS-M1.
#[test]
fn js_f01_enum_value_removed_is_narrowed() {
    let d = repo("f01", &status(r#"["pending","active","deleted"]"#), &status(r#"["pending","active"]"#));
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 1, "out: {out}");
    assert!(out.contains("narrowed"), "out: {out}");
    assert!(out.contains("status no longer accepts"), "out: {out}");
    assert!(out.contains("deleted"), "out: {out}");
}

/// js_f02: an enum gains a value. Widening HOLDS.
#[test]
fn js_f02_enum_value_added_holds() {
    let d = repo("f02", &status(r#"["pending","active"]"#), &status(r#"["pending","active","deleted"]"#));
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 0, "out: {out}");
    assert!(out.contains("HOLDS"), "out: {out}");
}

/// js_f03: minimum raised 0 to 10. VIOLATED tightened, both values.
/// Kills JS-M2 (an inverted comparison would call this a loosening).
#[test]
fn js_f03_minimum_raised_is_tightened() {
    let d = repo("f03", &age(r#","minimum":0"#), &age(r#","minimum":10"#));
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 1, "out: {out}");
    assert!(out.contains("tightened"), "out: {out}");
    assert!(out.contains("age minimum raised from 0 to 10"), "out: {out}");
}

/// js_f04: maximum raised 100 to 200. Loosening HOLDS. Kills JS-M2 in
/// the other direction (an inverted comparison would VIOLATE here).
#[test]
fn js_f04_maximum_raised_holds() {
    let d = repo("f04", &age(r#","maximum":100"#), &age(r#","maximum":200"#));
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 0, "out: {out}");
    assert!(out.contains("HOLDS"), "out: {out}");
}

/// js_f05: an existing optional field promoted to required. Payloads
/// that omitted it, valid before, are now rejected. Kills JS-M3.
#[test]
fn js_f05_optional_promoted_to_required_is_escalated() {
    let pre = r#"{"properties":{"email":{"type":"string"},"name":{"type":"string"}},"required":["name"]}"#;
    let post = r#"{"properties":{"email":{"type":"string"},"name":{"type":"string"}},"required":["name","email"]}"#;
    let d = repo("f05", pre, post);
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 1, "out: {out}");
    assert!(out.contains("escalated"), "out: {out}");
    assert!(out.contains("email is now required"), "out: {out}");
}

/// js_f06: a pattern changed. Containment of ECMA-262 patterns is not
/// decided, so the change is VIOLATED as constraint_changed, and the
/// sentence says it is not decided: never claimed as a proven narrowing.
/// Kills JS-M4.
#[test]
fn js_f06_pattern_change_is_flagged_not_proven() {
    let pre = r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{3}$"}}}"#;
    let post = r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{2}$"}}}"#;
    let d = repo("f06", pre, post);
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 1, "out: {out}");
    assert!(out.contains("code pattern was"), "out: {out}");
    assert!(out.contains("not decided here"), "out: {out}");
    assert!(!out.contains("narrowed"), "a pattern change must never be called a narrowing: {out}");
}

/// js_f07: the real b1-001 bytes. r0 stated `maximum` as not judged on
/// this exact case; r1 judges it and finds the int32 ceilings Snowplow's
/// media_player 2-0-0 introduced.
#[test]
fn js_f07_b1_001_bounds_found_in_the_corpus() {
    let d = repo("f07", B1_001_PRE, B1_001_POST);
    let (code, out) = judge(&d, &[]);
    assert_eq!(code, 1, "out: {out}");
    assert!(out.contains("currentTime maximum lowered from 9007199254740991 to 2147483647"), "out: {out}");
    assert!(out.contains("duration maximum lowered from 9007199254740991 to 2147483647"), "out: {out}");
    assert!(out.contains("percentProgress"), "r0 witnesses still present: {out}");
}

/// js_f08: the JSON surface carries the new witness fields, and the
/// unjudged list is exactly r1's: judged classes leave it, $ref (an r0
/// gap never stated before) enters it.
#[test]
fn js_f08_json_witness_fields_and_r1_unjudged_list() {
    let d = repo("f08", &age(r#","minimum":0"#), &age(r#","minimum":10"#));
    let (code, out) = judge(&d, &["--format", "json"]);
    assert_eq!(code, 1, "out: {out}");
    let v: serde_json::Value = serde_json::from_str(&out).expect("valid json");
    let w = &v["witnesses"][0];
    assert_eq!(w["kind"], "tightened");
    assert_eq!(w["path"], "age");
    assert_eq!(w["keyword"], "minimum");
    assert_eq!(w["pre_value"], "0");
    assert_eq!(w["post_value"], "10");
    let uj: Vec<&str> = v["unjudged_constraints"].as_array().expect("list").iter().filter_map(|x| x.as_str()).collect();
    assert_eq!(uj, vec!["$ref", "allOf", "anyOf", "oneOf", "items"]);
}
