//! COUNTEREXAMPLE r1 fixtures (docs/falsification/COUNTEREXAMPLE-R1-SPEC.md),
//! frozen BEFORE implementation: the checker reads minProperties,
//! maxProperties, uniqueItems, dependentRequired (and array dependencies)
//! and multipleOf, and the generator builds base instances that satisfy
//! them. The expected instances were checked with Python jsonschema at
//! freeze (COUNTEREXAMPLE-R1-SPEC.frozen).
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - K-M1 minProperties padding dropped          -> k_f02
//! - K-M2 minProperties check always valid       -> k_f03
//! - K-M3 uniqueItems distinct generation dropped -> k_f06
//! - K-M4 uniqueItems check always valid         -> k_f07 (not built for objects)
//! - K-M5 dependentRequired completion dropped   -> k_f08
//! - K-M6 dependentRequired check always valid   -> k_f09
//! - K-M7 multipleOf check always valid          -> k_f11
//! - K-M8 multipleOf generation dropped          -> k_f12

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo(tag: &str, pre: &str, post: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_k_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    let git = |args: &[&str]| {
        let o = Command::new("git").args(args).current_dir(&d)
            .env("GIT_AUTHOR_NAME", "k").env("GIT_AUTHOR_EMAIL", "k@test")
            .env("GIT_COMMITTER_NAME", "k").env("GIT_COMMITTER_EMAIL", "k@test")
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

fn judge(d: &Path) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["judge", "--contract", "schemas/contract.json", "--base", "main", "--format", "json"])
        .current_dir(d).env("HOME", d).output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned())
}

/// The single witness's counterexample and reason.
fn one(pre: &str, post: &str, tag: &str) -> (Value, Value) {
    let d = repo(tag, pre, post);
    let (code, out) = judge(&d);
    assert_eq!(code, 1, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}"));
    let w = v["witnesses"].as_array().expect("witnesses");
    assert_eq!(w.len(), 1, "{v}");
    (w[0]["counterexample"].clone(), w[0]["counterexample_reason"].clone())
}

const NO_CANDIDATE: &str = "no candidate value was valid at the base and rejected now";

fn field(f: &str) -> String {
    format!(r#"{{"properties":{{"f":{f}}}}}"#)
}

/// k_f01: minProperties is read now.
#[test]
fn k_f01_min_properties_is_read() {
    let pre = r#"{"minProperties":1,"additionalProperties":false,"properties":{"name":{"type":"string"},"age":{"type":"integer"}}}"#;
    let post = r#"{"minProperties":1,"additionalProperties":false,"properties":{"name":{"type":"string"}}}"#;
    assert_eq!(one(pre, post, "f01").0, json!({"age": 0}));
}

/// k_f02: the instance is padded to minProperties. Kills K-M1.
#[test]
fn k_f02_min_properties_padding() {
    let pre = r#"{"type":"object","minProperties":2,"properties":{"a":{"type":"string"},"b":{"type":"string"},"id":{"type":"string"}}}"#;
    let post = r#"{"type":"object","minProperties":2,"required":["id"],"properties":{"a":{"type":"string"},"b":{"type":"string"},"id":{"type":"string"}}}"#;
    assert_eq!(one(pre, post, "f02").0, json!({"a": "", "b": ""}));
}

/// k_f03: no base instance has three keys: nothing printed. Kills K-M2.
#[test]
fn k_f03_min_properties_impossible() {
    let pre = r#"{"minProperties":3,"additionalProperties":false,"properties":{"name":{"type":"string"},"age":{"type":"integer"}}}"#;
    let post = r#"{"minProperties":3,"additionalProperties":false,"properties":{"name":{"type":"string"}}}"#;
    let (cx, why) = one(pre, post, "f03");
    assert_eq!(cx, Value::Null);
    assert_eq!(why, NO_CANDIDATE);
}

/// k_f04: maxProperties is read now.
#[test]
fn k_f04_max_properties_is_read() {
    let pre = r#"{"maxProperties":1,"additionalProperties":false,"properties":{"name":{"type":"string"},"age":{"type":"integer"}}}"#;
    let post = r#"{"maxProperties":1,"additionalProperties":false,"properties":{"name":{"type":"string"}}}"#;
    assert_eq!(one(pre, post, "f04").0, json!({"age": 0}));
}

/// k_f05: uniqueItems is read now (the pnpm shape).
#[test]
fn k_f05_unique_items_is_read() {
    let (cx, _) = one(&field(r#"{"type":"array","uniqueItems":true,"items":{"type":"string"}}"#), &field(r#"{"type":"object"}"#), "f05");
    assert_eq!(cx, json!({"f": []}));
}

/// k_f06: two items, distinct. Kills K-M3.
#[test]
fn k_f06_unique_items_distinct_generation() {
    let (cx, _) = one(
        &field(r#"{"type":"array","uniqueItems":true,"maxItems":3,"items":{"type":"string"}}"#),
        &field(r#"{"type":"array","uniqueItems":true,"maxItems":1,"items":{"type":"string"}}"#),
        "f06",
    );
    assert_eq!(cx, json!({"f": ["", "a"]}));
}

/// k_f07: r1 does not build distinct objects (two copies of {} are
/// equal), so the checker rejects and nothing is printed, though such an
/// instance exists. Kills K-M4.
#[test]
fn k_f07_unique_items_not_built_for_objects() {
    let (cx, why) = one(
        &field(r#"{"type":"array","uniqueItems":true,"maxItems":3,"items":{"type":"object"}}"#),
        &field(r#"{"type":"array","uniqueItems":true,"maxItems":1,"items":{"type":"object"}}"#),
        "f07",
    );
    assert_eq!(cx, Value::Null);
    assert_eq!(why, NO_CANDIDATE);
}

/// k_f08: a dependency of the placed key is built. Kills K-M5.
#[test]
fn k_f08_dependent_required_completion() {
    let pre = r#"{"additionalProperties":false,"dependentRequired":{"a":["b"]},"properties":{"a":{"type":"string"},"b":{"type":"string"}}}"#;
    let post = r#"{"additionalProperties":false,"properties":{"b":{"type":"string"}}}"#;
    assert_eq!(one(pre, post, "f08").0, json!({"a": "", "b": ""}));
}

/// k_f09: an undeclared dependency is not built; the checker rejects.
/// Kills K-M6.
#[test]
fn k_f09_dependent_required_unbuildable() {
    let pre = r#"{"additionalProperties":false,"dependentRequired":{"a":["zzz"]},"properties":{"a":{"type":"string"},"b":{"type":"string"}}}"#;
    let post = r#"{"additionalProperties":false,"properties":{"b":{"type":"string"}}}"#;
    let (cx, why) = one(pre, post, "f09");
    assert_eq!(cx, Value::Null);
    assert_eq!(why, NO_CANDIDATE);
}

/// k_f10: the draft-07 array form of dependencies, as k_f08.
#[test]
fn k_f10_array_dependencies_completion() {
    let pre = r#"{"additionalProperties":false,"dependencies":{"a":["b"]},"properties":{"a":{"type":"string"},"b":{"type":"string"}}}"#;
    let post = r#"{"additionalProperties":false,"properties":{"b":{"type":"string"}}}"#;
    assert_eq!(one(pre, post, "f10").0, json!({"a": "", "b": ""}));
}

/// k_f11: 11 is not a multiple of 5, so the old bound is shown. Kills K-M7.
#[test]
fn k_f11_multiple_of_is_read() {
    let (cx, _) = one(&field(r#"{"type":"integer","multipleOf":5,"maximum":100}"#), &field(r#"{"type":"integer","multipleOf":5,"maximum":10}"#), "f11");
    assert_eq!(cx, json!({"f": 100}));
}

/// k_f12: the smallest multiple of 5 at or above 7. Kills K-M8.
#[test]
fn k_f12_multiple_of_generation() {
    let pre = r#"{"additionalProperties":false,"properties":{"n":{"type":"integer","minimum":7,"multipleOf":5},"name":{"type":"string"}}}"#;
    let post = r#"{"additionalProperties":false,"properties":{"name":{"type":"string"}}}"#;
    assert_eq!(one(pre, post, "f12").0, json!({"n": 10}));
}

/// k_f13: dependencies with a schema value is still not read.
#[test]
fn k_f13_schema_dependencies_still_unchecked() {
    let pre = r#"{"additionalProperties":false,"dependencies":{"a":{"required":["b"]}},"properties":{"a":{"type":"string"},"b":{"type":"string"},"age":{"type":"integer"}}}"#;
    let post = r#"{"additionalProperties":false,"dependencies":{"a":{"required":["b"]}},"properties":{"a":{"type":"string"},"b":{"type":"string"}}}"#;
    let (cx, why) = one(pre, post, "f13");
    assert_eq!(cx, Value::Null);
    assert_eq!(why, "dependencies is not checked by this judge");
}
