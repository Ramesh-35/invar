//! JUDGE-ITEMS r0 fixtures (docs/falsification/JUDGE-ITEMS-SPEC.md), frozen
//! BEFORE implementation: at a judged property node, an object (or absent)
//! `items` is judged as a child named `[]` by the property-node rules; a
//! counterexample places the element in a one-element array. i_f12 and
//! i_f13 are real bytes from SchemaStore (Apache-2.0) at the named commits.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - IT-M1 the element is never judged                      -> i_f03
//! - IT-M2 an absent `items` is not read as `{}`            -> i_f04
//! - IT-M3 a tuple `items` judged as one schema             -> i_f08
//! - IT-M4 the counterexample drops the element's array     -> i_f06

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const CARGO_BEFORE: &str = include_str!("fixtures/judge/schemastore-cargo-13e89998e-before.json");
const CARGO_AFTER: &str = include_str!("fixtures/judge/schemastore-cargo-13e89998e-after.json");
const VSEXT_BEFORE: &str = include_str!("fixtures/judge/schemastore-vsext-3227450c3-before.json");
const VSEXT_AFTER: &str = include_str!("fixtures/judge/schemastore-vsext-3227450c3-after.json");

fn pair(tag: &str, base: &str, head: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_it_{tag}_{}_{nanos}", std::process::id()));
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

/// One array property `r` whose element schema is `el` (None: no `items`).
fn arr(el: Option<&str>) -> String {
    match el {
        Some(e) => format!(r#"{{"type":"object","properties":{{"r":{{"type":"array","items":{e}}}}}}}"#),
        None => r#"{"type":"object","properties":{"r":{"type":"array"}}}"#.to_string(),
    }
}

/// The witness at `path`, or a panic naming what was found.
fn witness<'a>(v: &'a Value, path: &str) -> &'a Value {
    v["witnesses"].as_array().and_then(|ws| ws.iter().find(|w| w["path"] == path)).unwrap_or_else(|| panic!("no witness at {path}: {v}"))
}

/// i_f01: an element that gains minLength 1 rejects the empty string.
#[test]
fn i_f01_element_min_length_is_a_break() {
    let (code, text, v) = both("f01", &arr(Some(r#"{"type":"string"}"#)), &arr(Some(r#"{"type":"string","minLength":1}"#)));
    assert_eq!(code, 1, "{text}");
    let w = witness(&v, "r[]");
    assert_eq!(w["kind"], json!("tightened"), "{v}");
    assert_eq!(w["counterexample"], json!({"r": [""]}), "{v}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// i_f02: an element enum that gains a value rejects nothing.
#[test]
fn i_f02_element_enum_widened_holds() {
    let (code, text, v) = both("f02", &arr(Some(r#"{"enum":["a","b"]}"#)), &arr(Some(r#"{"enum":["a","b","c"]}"#)));
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// i_f03: an element limited to an enum it did not have. Kills IT-M1.
#[test]
fn i_f03_element_enum_added_is_a_break() {
    let (code, text, v) = both("f03", &arr(Some(r#"{"type":"string"}"#)), &arr(Some(r#"{"type":"string","enum":["bin","lib"]}"#)));
    assert_eq!(code, 1, "{text}");
    let cx = &witness(&v, "r[]")["counterexample"];
    let el = &cx["r"][0];
    assert!(el.is_string() && el != "bin" && el != "lib", "{v}");
    assert_eq!(cx["r"].as_array().map(Vec::len), Some(1), "{v}");
}

/// i_f04: `items` added where elements were unconstrained. Kills IT-M2.
#[test]
fn i_f04_items_added_is_a_break() {
    let (code, text, v) = both("f04", &arr(None), &arr(Some(r#"{"type":"string"}"#)));
    assert_eq!(code, 1, "{text}");
    let w = witness(&v, "r[]");
    assert_eq!(w["kind"], json!("retyped"), "{v}");
    assert!(w["counterexample"]["r"][0].is_null() || !w["counterexample"]["r"][0].is_string(), "{v}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// i_f05: `items` removed: every element valid before still is.
#[test]
fn i_f05_items_removed_holds_and_is_stated() {
    let (code, text, v) = both("f05", &arr(Some(r#"{"type":"string"}"#)), &arr(None));
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("noted: retyped (widened): r[] was string"), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// i_f06: a property of the element retyped. Kills IT-M4.
#[test]
fn i_f06_element_property_retyped() {
    let el = |t: &str| format!(r#"{{"type":"object","properties":{{"a":{{"type":"{t}"}}}}}}"#);
    let (code, text, v) = both("f06", &arr(Some(&el("string"))), &arr(Some(&el("integer"))));
    assert_eq!(code, 1, "{text}");
    let w = witness(&v, "r[].a");
    assert_eq!(w["kind"], json!("retyped"), "{v}");
    let cx = &w["counterexample"];
    assert!(cx["r"].is_array(), "the element sits in its array: {v}");
    assert!(cx["r"][0]["a"].is_string(), "{v}");
}

/// i_f07: the element newly requires a field.
#[test]
fn i_f07_element_newly_requires_a_field() {
    let el = |req: &str| format!(r#"{{"type":"object","properties":{{"a":{{"type":"string"}},"b":{{"type":"string"}}}}{req}}}"#);
    let (code, text, v) = both("f07", &arr(Some(&el(""))), &arr(Some(&el(r#","required":["b"]"#))));
    assert_eq!(code, 1, "{text}");
    let w = witness(&v, "r[].b");
    assert_eq!(w["kind"], json!("escalated"), "{v}");
    assert!(w["counterexample"]["r"][0].is_object() && w["counterexample"]["r"][0].get("b").is_none(), "{v}");
}

/// i_f08: a tuple is not one element schema. Kills IT-M3.
#[test]
fn i_f08_tuple_items_stay_unchecked() {
    let (code, text, v) = both("f08", &arr(Some(r#"[{"type":"string"}]"#)), &arr(Some(r#"[{"type":"string","minLength":1}]"#)));
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["unchecked"], json!(["/properties/r/items/0"]), "{v}");
    assert!(v["witnesses"].as_array().is_none_or(Vec::is_empty), "{v}");
}

/// i_f09 guard: a node under anyOf is compared whole (D1), as in v0.2.14.
#[test]
fn i_f09_items_under_a_combinator_node_is_d1() {
    let node = |el: &str| format!(r#"{{"properties":{{"r":{{"anyOf":[{{"type":"array","items":{el}}},{{"type":"null"}}]}}}}}}"#);
    let (code, text, v) = both("f09", &node(r#"{"type":"string"}"#), &node(r#"{"type":"string","minLength":1}"#));
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("unjudged_construct"), "{v}");
}

/// i_f10: an element given through a local $ref is judged at its target.
#[test]
fn i_f10_element_through_a_local_ref() {
    let doc = |max: u32| format!(r##"{{"definitions":{{"N":{{"type":"integer","maximum":{max}}}}},"properties":{{"r":{{"type":"array","items":{{"$ref":"#/definitions/N"}}}}}}}}"##);
    let (code, text, v) = both("f10", &doc(10), &doc(5));
    assert_eq!(code, 1, "{text}");
    let w = witness(&v, "r[]");
    assert_eq!(w["keyword"], json!("maximum"), "{v}");
    assert_eq!(w["counterexample"], json!({"r": [6]}), "{v}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// i_f11 guard: a property literally named `r[]` is that property.
#[test]
fn i_f11_literal_bracket_name_is_a_property() {
    let doc = |t: &str| format!(r#"{{"properties":{{"r[]":{{"type":"{t}"}},"r":{{"type":"array","items":{{"type":"string"}}}}}}}}"#);
    let (code, text, v) = both("f11", &doc("string"), &doc("integer"));
    assert_eq!(code, 1, "{text}");
    let w = witness(&v, "r[]");
    assert!(w["counterexample"]["r[]"].is_string(), "{v}");
}

/// i_f12: real SchemaStore bytes, "Add missing enum for crate-type".
#[test]
fn i_f12_real_cargo_crate_type_enum_is_a_break() {
    let (code, text, v) = both("f12", CARGO_BEFORE, CARGO_AFTER);
    assert_eq!(code, 1, "{text}");
    let w = witness(&v, "lib.crate-type[]");
    let el = &w["counterexample"]["lib"]["crate-type"][0];
    assert!(el.is_string(), "{v}");
    assert!(!["bin", "lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"].iter().any(|k| el == k), "{v}");
}

/// i_f13: real SchemaStore bytes, "Migrated vsext.json to draft-07": the
/// migration and the element's implied object type, both stated.
#[test]
fn i_f13_real_vsext_holds_with_both_notes() {
    let (code, text, v) = both("f13", VSEXT_BEFORE, VSEXT_AFTER);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("noted: draft migration: $schema moves from draft-04 to draft-07"), "{text}");
    assert!(text.contains("extensions[]"), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}
