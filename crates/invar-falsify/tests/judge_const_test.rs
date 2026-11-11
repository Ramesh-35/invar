//! JUDGE-CONST r0 fixtures (docs/falsification/JUDGE-CONST-SPEC.md), frozen
//! BEFORE implementation: `const: v` reads as `enum: [v]` wherever the judge
//! reads enums, and is a keyword the judge reads at a reached node.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - K-R1 `const` not read as an enum by the narrowing rule   -> k_f01, k_f05
//! - K-R2 `const` not counted as read at a reached node        -> k_f04

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn pair(tag: &str, base: &str, head: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_k_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("a.json"), base).unwrap();
    std::fs::write(d.join("b.json"), head).unwrap();
    d
}

fn judge(d: &Path, format: &str) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["judge", "a.json", "b.json", "--format", format])
        .current_dir(d).env("HOME", d).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn both(tag: &str, base: &str, head: &str) -> (i32, String, Value) {
    let d = pair(tag, base, head);
    let (code, text) = judge(&d, "text");
    let (_, j) = judge(&d, "json");
    (code, text, serde_json::from_str(&j).unwrap_or_else(|e| panic!("not JSON ({e}): {j}")))
}

/// One property `k` whose schema is `node`.
fn doc(node: &str) -> String {
    format!(r#"{{"type":"object","properties":{{"k":{node}}}}}"#)
}

const HOLDS: &str = "HOLDS: b.json keeps every promise it made at a.json.\n";

/// k_f01: the same single value, spelled const then enum. Kills K-R1.
#[test]
fn k_f01_const_to_equal_enum_holds() {
    let (code, text, v) = both("f01", &doc(r#"{"const":"x"}"#), &doc(r#"{"enum":["x"]}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.starts_with(HOLDS), "{text}");
    assert!(text.contains("  noted: equivalent after rewriting literals (const, enum, boolean, null) at /properties/k\n"), "{text}");
    assert_eq!(v["witnesses"], json!([]), "{v}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// k_f02: a const widened to an enum that still contains it.
#[test]
fn k_f02_const_to_wider_enum_holds() {
    let (code, text, v) = both("f02", &doc(r#"{"const":"x"}"#), &doc(r#"{"enum":["x","y"]}"#));
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// k_f03: a const replaced by an enum without it.
#[test]
fn k_f03_const_to_disjoint_enum_is_violated() {
    let (code, text, v) = both("f03", &doc(r#"{"const":"x"}"#), &doc(r#"{"enum":["y"]}"#));
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["kind"], json!("narrowed"), "{v}");
    assert!(text.contains(r#"counterexample (valid at a.json, rejected now): {"k":"x"}"#), "{text}");
}

/// k_f04: an enum narrowed to a const. Kills K-R2.
#[test]
fn k_f04_enum_to_const_subset_is_violated() {
    let (code, text, v) = both("f04", &doc(r#"{"enum":["x","y"]}"#), &doc(r#"{"const":"x"}"#));
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["kind"], json!("narrowed"), "{v}");
    assert!(text.contains(r#"counterexample (valid at a.json, rejected now): {"k":"y"}"#), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// k_f05: a const changed. Kills K-R1.
#[test]
fn k_f05_const_changed_is_violated() {
    let (code, text, v) = both("f05", &doc(r#"{"const":"x"}"#), &doc(r#"{"const":"y"}"#));
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["kind"], json!("narrowed"), "{v}");
    assert!(text.contains(r#"counterexample (valid at a.json, rejected now): {"k":"x"}"#), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// k_f06 guard: a const unchanged beside an annotation.
#[test]
fn k_f06_unchanged_const_holds() {
    let (code, text, v) = both("f06", &doc(r#"{"const":"x"}"#), &doc(r#"{"const":"x","description":"d"}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.starts_with(HOLDS), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}
