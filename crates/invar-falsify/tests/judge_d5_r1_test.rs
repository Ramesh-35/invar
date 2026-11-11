//! JUDGE-D5 r1 fixtures (docs/falsification/JUDGE-D5-R1-SPEC.md), frozen
//! BEFORE implementation: a type written where none was is a narrowing,
//! judged by the type rule like any other retype. d5_f05 and d5_f06 are real
//! bytes from SchemaStore (Apache-2.0): schema-catalog v12/v13 (the pair the
//! WG dataset labelled a break and v0.2.22 called HOLDS) and devinit 2.0 at
//! cc3c8b16c.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - D5R1-M1 the implied-type note reintroduced   -> d5_f01 (and r2_f15 as amended)
//! - D5R1-M2 unspecified-to-object not retyped     -> d5_f01

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const CATALOG_V12: &str = include_str!("fixtures/judge/schemastore-schema-catalog-v12.json");
const CATALOG_V13: &str = include_str!("fixtures/judge/schemastore-schema-catalog-v13.json");
const DEVINIT_BEFORE: &str = include_str!("fixtures/judge/schemastore-devinit-2.0-cc3c8b16c-before.json");
const DEVINIT_AFTER: &str = include_str!("fixtures/judge/schemastore-devinit-2.0-cc3c8b16c-after.json");

fn pair(tag: &str, base: &str, head: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_d5_{tag}_{}_{nanos}", std::process::id()));
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

fn doc(node: &str) -> String {
    format!(r#"{{"type":"object","properties":{{"h":{node}}}}}"#)
}

/// d5_f01: object keywords, no type, then `type: object`. Kills D5R1-M1 and D5R1-M2.
#[test]
fn d5_f01_object_type_written_where_none_was_is_violated() {
    let (code, text, v) = both("f01", &doc(r#"{"properties":{"t":{"type":"string"}}}"#), &doc(r#"{"type":"object","properties":{"t":{"type":"string"}}}"#));
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["kind"], json!("retyped"), "{v}");
    assert_eq!(v["witnesses"][0]["pre_type"], json!("unspecified"), "{v}");
    assert_eq!(v["witnesses"][0]["post_type"], json!("object"), "{v}");
    assert!(text.contains(r#"counterexample (valid at a.json, rejected now): {"h":null}"#), "{text}");
    assert!(!text.contains("type made explicit"), "{text}");
}

/// d5_f02: array keywords, no type, then `type: array`.
#[test]
fn d5_f02_array_type_written_where_none_was_is_violated() {
    let (code, text, v) = both("f02", &doc(r#"{"items":{"type":"string"}}"#), &doc(r#"{"type":"array","items":{"type":"string"}}"#));
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["kind"], json!("retyped"), "{v}");
    assert!(!text.contains("type made explicit"), "{text}");
}

/// d5_f03 guard (D9): an enum already limited the values to objects.
#[test]
fn d5_f03_enum_already_limited_the_type_holds() {
    let (code, text, _) = both("f03", &doc(r#"{"enum":[{"a":1}]}"#), &doc(r#"{"type":"object","enum":[{"a":1}]}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("noted:"), "{text}");
}

/// d5_f04 guard (D7): a widened type is still a note.
#[test]
fn d5_f04_widened_type_holds() {
    let (code, text, _) = both("f04", &doc(r#"{"type":"string"}"#), &doc(r#"{"type":["string","null"]}"#));
    assert_eq!(code, 0, "{text}");
}

/// d5_f05: real bytes, the pair the WG dataset labelled a break.
#[test]
fn d5_f05_real_schema_catalog_v12_v13_is_violated() {
    let (code, text, v) = both("f05", CATALOG_V12, CATALOG_V13);
    assert_eq!(code, 1, "{text}");
    let w = &v["witnesses"][0];
    assert_eq!(w["kind"], json!("retyped"), "{v}");
    assert_eq!(w["path"], json!("schemas[].versions"), "{v}");
    assert!(w["counterexample"].is_object(), "{v}");
}

/// d5_f06: real bytes, devinit 2.0, `run` gained `type: array`.
#[test]
fn d5_f06_real_devinit_run_is_violated() {
    let (code, text, v) = both("f06", DEVINIT_BEFORE, DEVINIT_AFTER);
    assert_eq!(code, 1, "{text}");
    assert!(v["witnesses"].as_array().unwrap().iter().any(|w| w["kind"] == "retyped" && w["path"] == "run"), "{v}");
}
