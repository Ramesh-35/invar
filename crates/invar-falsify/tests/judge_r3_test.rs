//! JUDGE-R3 r0 fixtures (docs/falsification/JUDGE-R3-SPEC.md), frozen
//! BEFORE implementation: the three rules that falsely accused three
//! SchemaStore commits (J1 patternProperties, J2 enum added over a finite
//! domain, J3 required inside allOf), corrected.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - R3-M1 patternProperties check dropped     -> r3_f01
//! - R3-M2 finite domain not compared          -> r3_f03
//! - R3-M3 allOf members ignored at base       -> r3_f07
//! - R3-M4 allOf members ignored at post       -> r3_f08

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo(tag: &str, pre: &str, post: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_r3_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    let git = |args: &[&str]| {
        let o = Command::new("git").args(args).current_dir(&d)
            .env("GIT_AUTHOR_NAME", "r3").env("GIT_AUTHOR_EMAIL", "r3@test")
            .env("GIT_COMMITTER_NAME", "r3").env("GIT_COMMITTER_EMAIL", "r3@test")
            .output().unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    };
    git(&["init", "-q", "-b", "main"]);
    std::fs::write(d.join("schemas/contract.json"), pre).unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "base"]);
    std::fs::write(d.join("schemas/contract.json"), post).unwrap();
    d
}

fn judge(d: &Path, format: &str) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["judge", "--contract", "schemas/contract.json", "--base", "main", "--format", format])
        .current_dir(d).env("HOME", d).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn both(pre: &str, post: &str, tag: &str) -> (i32, String, Value) {
    let d = repo(tag, pre, post);
    let (code, text) = judge(&d, "text");
    let (_, j) = judge(&d, "json");
    let v: Value = serde_json::from_str(&j).unwrap_or_else(|e| panic!("not JSON ({e}): {j}"));
    (code, text, v)
}

fn field(f: &str) -> String {
    format!(r#"{{"properties":{{"f":{f}}}}}"#)
}

const CLOSED: &str = r#"{"additionalProperties":false,"properties":{"name":{"type":"string"},"age":{"type":"integer"}}}"#;

/// r3_f01 J1: a pattern may still admit the removed key: not accused.
/// Kills R3-M1.
#[test]
fn r3_f01_removal_under_pattern_properties_is_not_evaluated() {
    let post = r#"{"additionalProperties":false,"patternProperties":{"^a":{}},"properties":{"name":{"type":"string"}}}"#;
    let (code, text, v) = both(CLOSED, post, "f01");
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], "unjudged_construct", "{v}");
    assert!(v["not_evaluated_paths"].as_array().is_some_and(|a| a.iter().any(|p| p == "age")), "{v}");
    assert!(!text.contains("removed:"), "{text}");
}

/// r3_f02 J1 guard: no patternProperties, the removal is a break.
#[test]
fn r3_f02_removal_without_pattern_properties_is_violated() {
    let post = r#"{"additionalProperties":false,"properties":{"name":{"type":"string"}}}"#;
    let (code, text, _) = both(CLOSED, post, "f02");
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("removed: age"), "{text}");
}

/// r3_f03 J2: every value valid before (-1, 0, 1) is in the new enum.
/// Kills R3-M2.
#[test]
fn r3_f03_enum_over_finite_domain_that_keeps_every_value_holds() {
    let (code, text, _) = both(&field(r#"{"type":"integer","minimum":-1,"maximum":1}"#), &field(r#"{"type":"integer","enum":[-2,-1,0,1,2]}"#), "f03");
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("  noted: enum added: f now lists every value valid before (-1, 0, 1).\n"), "{text}");
}

/// r3_f04 J2: the lost values are named, and shown.
#[test]
fn r3_f04_enum_over_finite_domain_that_loses_values_is_narrowed() {
    let (code, text, v) = both(&field(r#"{"type":"integer","minimum":0,"maximum":5}"#), &field(r#"{"type":"integer","enum":[0,1,2]}"#), "f04");
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("narrowed: f no longer accepts 3, 4, 5."), "{text}");
    assert_eq!(v["witnesses"][0]["counterexample"], json!({"f": 3}), "{v}");
}

/// r3_f05 J2: a boolean domain is finite.
#[test]
fn r3_f05_boolean_to_single_value_enum_is_narrowed() {
    let (code, text, _) = both(&field(r#"{"type":"boolean"}"#), &field(r#"{"type":"boolean","enum":[true]}"#), "f05");
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("narrowed: f no longer accepts false."), "{text}");
}

/// r3_f06 J2 guard: an unbounded domain keeps r2 behaviour.
#[test]
fn r3_f06_enum_over_unbounded_domain_keeps_r2_sentence() {
    let (code, text, _) = both(&field(r#"{"type":"string"}"#), &field(r#"{"type":"string","enum":["a"]}"#), "f06");
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("narrowed: f now accepts only [\"a\"]."), "{text}");
}

/// r3_f07 J3: required inside allOf at base is required. Kills R3-M3.
#[test]
fn r3_f07_required_moved_out_of_all_of_holds() {
    let pre = r#"{"type":"object","allOf":[{"required":["site_name"]}],"properties":{"site_name":{"type":"string"}}}"#;
    let post = r#"{"type":"object","required":["site_name"],"properties":{"site_name":{"type":"string"}}}"#;
    let (code, text, _) = both(pre, post, "f07");
    assert_eq!(code, 0, "{text}");
    assert!(!text.contains("escalated"), "{text}");
}

/// r3_f08 J3: newly required inside allOf at post is escalated. Kills R3-M4.
#[test]
fn r3_f08_newly_required_inside_all_of_is_escalated() {
    let pre = r#"{"type":"object","properties":{"site_name":{"type":"string"}}}"#;
    let post = r#"{"type":"object","allOf":[{"required":["site_name"]}],"properties":{"site_name":{"type":"string"}}}"#;
    let (code, text, _) = both(pre, post, "f08");
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("escalated: site_name is now required."), "{text}");
}

/// r3_f09 J3 guard: required nowhere before, required now.
#[test]
fn r3_f09_newly_required_at_root_is_escalated() {
    let pre = r#"{"type":"object","properties":{"site_name":{"type":"string"}}}"#;
    let post = r#"{"type":"object","required":["site_name"],"properties":{"site_name":{"type":"string"}}}"#;
    let (code, text, _) = both(pre, post, "f09");
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("escalated: site_name is now required."), "{text}");
}
