//! JUDGE-CANON r0 fixtures (docs/falsification/JUDGE-CANON-SPEC.md), frozen
//! BEFORE implementation: a node that differs from its counterpart only by
//! spelling a type list as the `anyOf` of its types is covered and stated,
//! never unchecked or unjudged. c_f09 and c_f10 are real bytes from
//! SchemaStore (Apache-2.0) at the named commits.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - CN-M1 the normal form is never applied                       -> c_f01
//! - CN-M2 a typed keyword shared instead of following its type   -> c_f02
//! - CN-M3 annotations compared instead of set aside              -> c_f06
//! - CN-M4 property names filtered as if they were keywords       -> c_f11
//!   (c_f11 was added after the freeze, when the prediction gate caught
//!   the first build doing exactly that; see the spec's Measured outcome)

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const PLAGIARIZE_BEFORE: &str = include_str!("fixtures/judge/schemastore-plagiarize-7de30f637-before.json");
const PLAGIARIZE_AFTER: &str = include_str!("fixtures/judge/schemastore-plagiarize-7de30f637-after.json");
const ESQUIO_BEFORE: &str = include_str!("fixtures/judge/schemastore-esquio-a0b8327b0-before.json");
const ESQUIO_AFTER: &str = include_str!("fixtures/judge/schemastore-esquio-a0b8327b0-after.json");

fn pair(tag: &str, base: &str, head: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_cn_{tag}_{}_{nanos}", std::process::id()));
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

/// One property `a` whose schema is `node`.
fn doc(node: &str) -> String {
    format!(r#"{{"type":"object","properties":{{"a":{node}}}}}"#)
}

const NOTE_A: &str = "  noted: equivalent after rewriting type lists as anyOf at /properties/a\n";

/// c_f01: the plain rewrite at a property. Kills CN-M1.
#[test]
fn c_f01_type_list_as_anyof_holds_and_is_stated() {
    let (code, text, v) = both("f01", &doc(r#"{"type":["string","null"]}"#), &doc(r#"{"anyOf":[{"type":"string"},{"type":"null"}]}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.starts_with("HOLDS: b.json keeps every promise it made at a.json.\n"), "{text}");
    assert!(text.contains(NOTE_A), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// c_f02: a typed keyword follows its type. Kills CN-M2.
#[test]
fn c_f02_typed_keyword_follows_its_type() {
    let (code, text, _) = both("f02", &doc(r#"{"type":["array","string"],"items":{"type":"string"}}"#), &doc(r#"{"anyOf":[{"type":"array","items":{"type":"string"}},{"type":"string"}]}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(NOTE_A), "{text}");
}

/// c_f03: a shared keyword stays beside the anyOf.
#[test]
fn c_f03_shared_keyword_beside_anyof_holds() {
    let (code, text, _) = both("f03", &doc(r#"{"type":["string","null"],"enum":["x",null]}"#), &doc(r#"{"enum":["x",null],"anyOf":[{"type":"string"},{"type":"null"}]}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(NOTE_A), "{text}");
}

/// c_f04 guard: the shared keyword moved inside one branch is not the rewrite.
/// Amended by JUDGE-CANON r1: `null` became `integer`, because with `null`
/// the two spellings admit the same documents and r1 proves it.
#[test]
fn c_f04_keyword_inside_a_branch_is_not_covered() {
    let (code, text, v) = both("f04", &doc(r#"{"type":["string","integer"],"enum":["x",1]}"#), &doc(r#"{"anyOf":[{"type":"string","enum":["x",1]},{"type":"integer"}]}"#));
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("unjudged_construct"), "{v}");
    assert!(!text.contains("noted: equivalent"), "{text}");
}

/// c_f05 guard: a branch that also narrows is not the rewrite.
#[test]
fn c_f05_narrowed_branch_is_not_covered() {
    let (code, text, v) = both("f05", &doc(r#"{"type":["string","null"]}"#), &doc(r#"{"anyOf":[{"type":"string","minLength":1},{"type":"null"}]}"#));
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("unjudged_construct"), "{v}");
    assert!(!text.contains("noted: equivalent"), "{text}");
}

/// c_f06: annotations are set aside on both sides. Kills CN-M3.
#[test]
fn c_f06_annotations_are_set_aside() {
    let (code, text, _) = both("f06", &doc(r#"{"type":["string","null"],"description":"old","title":"T"}"#), &doc(r#"{"description":"new","anyOf":[{"type":"string","title":"S"},{"type":"null","description":"nothing"}]}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(NOTE_A), "{text}");
}

/// c_f07: a node the walk never reaches, covered by `finish`.
#[test]
fn c_f07_under_definitions_is_covered() {
    let pre = r#"{"definitions":{"D":{"type":["integer","null"],"minimum":0}},"properties":{"a":{"type":"string"}}}"#;
    let post = r#"{"definitions":{"D":{"anyOf":[{"type":"integer","minimum":0},{"type":"null"}]}},"properties":{"a":{"type":"string"}}}"#;
    let (code, text, v) = both("f07", pre, post);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("  noted: equivalent after rewriting type lists as anyOf at /definitions/D\n"), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// c_f08 guard: oneOf is not anyOf.
#[test]
fn c_f08_oneof_is_not_the_rewrite() {
    let (code, text, _) = both("f08", &doc(r#"{"type":["string","null"]}"#), &doc(r#"{"oneOf":[{"type":"string"},{"type":"null"}]}"#));
    assert_eq!(code, 4, "{text}");
    assert!(!text.contains("noted: equivalent"), "{text}");
}

/// c_f09: real bytes, the draft migration plus two rewritten nodes.
#[test]
fn c_f09_real_plagiarize_holds() {
    let (code, text, v) = both("f09", PLAGIARIZE_BEFORE, PLAGIARIZE_AFTER);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("noted: draft migration: $schema moves from draft-04 to draft-07"), "{text}");
    assert!(text.contains("  noted: equivalent after rewriting type lists as anyOf at /properties/strings/additionalProperties\n"), "{text}");
    assert!(text.contains("  noted: equivalent after rewriting type lists as anyOf at /properties/vars/additionalProperties\n"), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// c_f10: real bytes, nested type lists under a node D1 states.
#[test]
fn c_f10_real_esquio_holds() {
    let (code, text, v) = both("f10", ESQUIO_BEFORE, ESQUIO_AFTER);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("noted: equivalent after rewriting type lists as anyOf at "), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// c_f11 guard (Amendment A): a property named like no keyword changes
/// inside the rewritten node; the rewrite does not hide it. Kills CN-M4.
#[test]
fn c_f11_change_under_a_property_name_is_not_hidden() {
    let pre = doc(r#"{"type":["object","null"],"properties":{"x":{"type":"string"}}}"#);
    let post = doc(r#"{"anyOf":[{"type":"object","properties":{"x":{"type":"integer"}}},{"type":"null"}]}"#);
    let (code, text, v) = both("f11", &pre, &post);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("unjudged_construct"), "{v}");
    assert!(!text.contains("noted: equivalent"), "{text}");
}
