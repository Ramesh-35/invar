//! REF-SIBLINGS r0 fixtures (docs/falsification/REF-SIBLINGS-SPEC.md), frozen
//! BEFORE implementation: under draft-07 and earlier a `$ref` with sibling
//! validation keywords is read two ways by the validators this judge
//! trusts, so the node is stated, not judged. rs_f08 is real bytes from
//! SchemaStore (Apache-2.0) at 449644d6d.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - RS-M1 two-way nodes judged anyway (the check dropped)   -> rs_f01
//! - RS-M2 2019-09 treated as two-way                          -> rs_f02
//! - RS-M3 annotations counted as siblings                     -> rs_f03
//! - RS-M4 the hop not checked                                 -> rs_f05
//! - RS-M5 the root not checked                                -> rs_f04

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const STALE_BEFORE: &str = include_str!("fixtures/judge/schemastore-stale-449644d6d-before.json");
const STALE_AFTER: &str = include_str!("fixtures/judge/schemastore-stale-449644d6d-after.json");

fn pair(tag: &str, base: &str, head: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_rs_{tag}_{}_{nanos}", std::process::id()));
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

const D07: &str = r##""$schema":"http://json-schema.org/draft-07/schema#","##;
const D2019: &str = r##""$schema":"https://json-schema.org/draft/2019-09/schema","##;

/// A document with definition `s` (a string) and property `a` = `node`.
fn doc(draft: &str, node: &str) -> String {
    format!(r##"{{{draft}"definitions":{{"s":{{"type":"string"}}}},"type":"object","properties":{{"a":{node}}}}}"##)
}

/// rs_f01: a sibling validation keyword changes beside `$ref`. Kills RS-M1.
#[test]
fn rs_f01_two_way_node_is_stated_not_judged() {
    let (code, text, v) = both("f01", &doc(D07, r##"{"$ref":"#/definitions/s","maxLength":2}"##), &doc(D07, r##"{"$ref":"#/definitions/s","maxLength":1}"##));
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("ref_siblings"), "{v}");
    assert_eq!(v["ref_siblings_paths"], json!(["a"]), "{v}");
    assert_eq!(v["witnesses"], json!([]), "{v}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
    assert!(text.contains("  not evaluated: a carries $ref beside maxLength; under draft-07 jsonschema ignores those keywords and Ajv applies them, so no single reading exists to judge.\n"), "{text}");
    assert!(text.starts_with("NOT EVALUATED (ref_siblings): b.json has a $ref with sibling keywords that validators read differently under its draft;"), "{text}");
}

/// rs_f02 guard: under 2019-09 siblings apply by the specification; judged as before. Kills RS-M2.
#[test]
fn rs_f02_under_2019_09_siblings_are_judged() {
    let d19 = |node: &str| format!(r##"{{{D2019}"$defs":{{"s":{{"type":"string"}}}},"type":"object","properties":{{"a":{node}}}}}"##);
    let (code, text, v) = both("f02", &d19(r##"{"$ref":"#/$defs/s","maxLength":2}"##), &d19(r##"{"$ref":"#/$defs/s","maxLength":1}"##));
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["kind"], json!("tightened"), "{v}");
    assert_eq!(v["ref_siblings_paths"], json!([]), "{v}");
}

/// rs_f03 guard: annotations beside `$ref` are not siblings that matter. Kills RS-M3.
#[test]
fn rs_f03_annotation_siblings_are_not_two_way() {
    let pre = format!(r##"{{{D07}"definitions":{{"s":{{"type":"string","maxLength":2}}}},"type":"object","properties":{{"a":{{"$ref":"#/definitions/s","description":"d","title":"t"}}}}}}"##);
    let post = format!(r##"{{{D07}"definitions":{{"s":{{"type":"string","maxLength":1}}}},"type":"object","properties":{{"a":{{"$ref":"#/definitions/s","description":"d","title":"t"}}}}}}"##);
    let (code, text, v) = both("f03", &pre, &post);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["kind"], json!("tightened"), "{v}");
    assert_eq!(v["ref_siblings_paths"], json!([]), "{v}");
}

/// rs_f04: the document root itself is two-way. Kills RS-M5.
#[test]
fn rs_f04_two_way_root_is_stated() {
    let pre = format!(r##"{{{D07}"definitions":{{"c":{{"type":"object"}}}},"$ref":"#/definitions/c","properties":{{"issues":{{}}}}}}"##);
    let post = format!(r##"{{{D07}"definitions":{{"c":{{"type":"object"}}}},"$ref":"#/definitions/c","properties":{{"issues":{{"type":"object"}}}}}}"##);
    let (code, text, v) = both("f04", &pre, &post);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("ref_siblings"), "{v}");
    assert_eq!(v["ref_siblings_paths"], json!([""]), "{v}");
    assert_eq!(v["witnesses"], json!([]), "{v}");
    assert_eq!(v["not_evaluated_paths"], json!([]), "{v}");
}

/// rs_f05: a two-way node reached through a hop. Kills RS-M4.
#[test]
fn rs_f05_two_way_node_through_a_hop_is_stated() {
    let pre = format!(r##"{{{D07}"definitions":{{"s":{{"type":"string","maxLength":2}},"d":{{"$ref":"#/definitions/s","type":"string"}}}},"type":"object","properties":{{"a":{{"$ref":"#/definitions/d"}}}}}}"##);
    let post = format!(r##"{{{D07}"definitions":{{"s":{{"type":"string","maxLength":1}},"d":{{"$ref":"#/definitions/s","type":"string"}}}},"type":"object","properties":{{"a":{{"$ref":"#/definitions/d"}}}}}}"##);
    let (code, text, v) = both("f05", &pre, &post);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("ref_siblings"), "{v}");
    assert_eq!(v["ref_siblings_paths"], json!(["a"]), "{v}");
}

/// rs_f06: a break away from the two-way node still decides the verdict.
#[test]
fn rs_f06_break_elsewhere_still_violated() {
    let pre = format!(r##"{{{D07}"definitions":{{"s":{{"type":"string"}}}},"type":"object","properties":{{"a":{{"$ref":"#/definitions/s","maxLength":2}},"b":{{"type":"string"}}}}}}"##);
    let post = format!(r##"{{{D07}"definitions":{{"s":{{"type":"string"}}}},"type":"object","properties":{{"a":{{"$ref":"#/definitions/s","maxLength":1}},"b":{{"type":"integer"}}}}}}"##);
    let (code, text, v) = both("f06", &pre, &post);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["path"], json!("b"), "{v}");
    assert_eq!(v["ref_siblings_paths"], json!(["a"]), "{v}");
}

/// rs_f07: no `$schema` reads as draft-07.
#[test]
fn rs_f07_no_schema_reads_as_draft_07() {
    let (code, text, v) = both("f07", &doc("", r##"{"$ref":"#/definitions/s","maxLength":2}"##), &doc("", r##"{"$ref":"#/definitions/s","maxLength":1}"##));
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("ref_siblings"), "{v}");
}

/// rs_f08: real bytes, stale.json: a root `$ref` beside `properties`.
#[test]
fn rs_f08_real_stale_root_is_stated() {
    let (code, text, v) = both("f08", STALE_BEFORE, STALE_AFTER);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("ref_siblings"), "{v}");
    assert_eq!(v["ref_siblings_paths"], json!([""]), "{v}");
    assert_eq!(v["witnesses"], json!([]), "{v}");
    assert!(text.contains("noted: draft migration: $schema moves from draft-04 to draft-07"), "{text}");
}
