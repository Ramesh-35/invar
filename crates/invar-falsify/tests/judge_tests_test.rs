//! JUDGE-TESTS r0 fixtures (docs/falsification/JUDGE-TESTS-SPEC.md), frozen
//! BEFORE implementation: `action/tests.py BASE HEAD DIR` reads a directory
//! of instance documents; one the base accepts and the head rejects is a
//! witness (VIOLATED, exit 1); otherwise NOT EVALUATED (exit 4), never
//! HOLDS. Needs python3 with jsonschema and PyYAML on the PATH, as CI has.
//! t_f14 and t_f15 are real bytes from SchemaStore (Apache-2.0).
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - TS-M1 a head rejection is never recorded            -> t_f01
//! - TS-M2 passing instances exit 0                      -> t_f02
//! - TS-M3 base validity is not checked                  -> t_f03
//! - TS-M4 an unreadable file is counted as passing      -> t_f06

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const SCRIPT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../action/tests.py");
const GA_BEFORE: &str = include_str!("fixtures/judge/schemastore-github-action-d9d98e698-before.json");
const GA_AFTER: &str = include_str!("fixtures/judge/schemastore-github-action-d9d98e698-after.json");
const GA_TEST: &str = include_str!("fixtures/judge/schemastore-github-action-d9d98e698-parallel-steps.json");
const RYL_BEFORE: &str = include_str!("fixtures/judge/schemastore-ryl-dd22b5c09-before.json");
const RYL_AFTER: &str = include_str!("fixtures/judge/schemastore-ryl-dd22b5c09-after.json");
const RYL_TEST: &str = include_str!("fixtures/judge/schemastore-ryl-dd22b5c09-01-valid.toml");

/// A temp dir holding base.json, head.json and an `instances` directory
/// of the named files.
fn setup(tag: &str, base: &str, head: &str, instances: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_ts_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join("instances")).unwrap();
    std::fs::write(d.join("base.json"), base).unwrap();
    std::fs::write(d.join("head.json"), head).unwrap();
    for (name, body) in instances {
        std::fs::write(d.join("instances").join(name), body).unwrap();
    }
    d
}

fn run(d: &Path, format: &str) -> (i32, String) {
    let o = Command::new("python3")
        .args([SCRIPT, "base.json", "head.json", "instances", "--format", format])
        .current_dir(d).output().expect("python3 on the PATH");
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn both(d: &Path) -> (i32, String, Value) {
    let (code, text) = run(d, "text");
    let (_, j) = run(d, "json");
    (code, text, serde_json::from_str(&j).unwrap_or_else(|e| panic!("not JSON ({e}): {j}")))
}

const STRING_A: &str = r#"{"type":"object","properties":{"a":{"type":"string"}}}"#;
const STRING_A_MIN1: &str = r#"{"type":"object","properties":{"a":{"type":"string","minLength":1}}}"#;

/// t_f01: a document valid at the base and rejected by the head. Kills TS-M1.
#[test]
fn t_f01_rejected_instance_is_a_break() {
    let d = setup("f01", STRING_A, STRING_A_MIN1, &[("empty.json", r#"{"a":""}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert!(text.starts_with("VIOLATED: 1 document(s) valid at base.json are rejected by head.json.\n"), "{text}");
    assert!(text.contains("  rejected: empty.json at /a: "), "{text}");
    assert_eq!(v["verdict"], json!("VIOLATED"), "{v}");
    assert_eq!(v["witnesses"][0]["file"], json!("empty.json"), "{v}");
    assert_eq!(v["witnesses"][0]["pointer"], json!("/a"), "{v}");
    assert!(v["witnesses"][0]["message"].as_str().is_some_and(|m| !m.is_empty()), "{v}");
}

/// t_f02: passing instances are not a proof. Never exit 0. Kills TS-M2.
#[test]
fn t_f02_passing_instances_are_not_evaluated() {
    let d = setup("f02", STRING_A, STRING_A_MIN1, &[("one.json", r#"{"a":"x"}"#), ("two.json", r#"{"a":"yy"}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 4, "{text}");
    assert!(text.starts_with("NOT EVALUATED (instances_pass): "), "{text}");
    assert!(text.contains("  instances: 2 read, 0 not promises, 2 pass, 0 not checked\n"), "{text}");
    assert_eq!(v["reason"], json!("instances_pass"), "{v}");
    assert_eq!(v["instances"]["pass"], json!(2), "{v}");
}

/// t_f03: a document the base rejects was never a promise. Kills TS-M3.
#[test]
fn t_f03_base_invalid_instance_is_not_a_promise() {
    let d = setup("f03", STRING_A, STRING_A_MIN1, &[("bad.json", r#"{"a":1}"#), ("ok.json", r#"{"a":"x"}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("instances_pass"), "{v}");
    assert_eq!(v["instances"]["not_promises"], json!(1), "{v}");
    assert_eq!(v["witnesses"], json!([]), "{v}");
}

/// t_f04: nothing to read.
#[test]
fn t_f04_empty_directory() {
    let d = setup("f04", STRING_A, STRING_A_MIN1, &[]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 4, "{text}");
    assert!(text.starts_with("NOT EVALUATED (no_instances): "), "{text}");
    assert_eq!(v["instances"]["read"], json!(0), "{v}");
}

/// t_f05: only documents the base rejects.
#[test]
fn t_f05_only_base_invalid_instances() {
    let d = setup("f05", STRING_A, STRING_A_MIN1, &[("bad.json", r#"{"a":1}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("no_base_valid_instance"), "{v}");
}

/// t_f06: a file that cannot be read is named, and the verdict says so. Kills TS-M4.
#[test]
fn t_f06_unparsable_file_is_not_checked() {
    let d = setup("f06", STRING_A, STRING_A_MIN1, &[("broken.json", "{not json"), ("ok.json", r#"{"a":"x"}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("unverifiable"), "{v}");
    assert!(text.contains("  not checked: broken.json: "), "{text}");
    assert_eq!(v["not_checked"][0]["file"], json!("broken.json"), "{v}");
    assert_eq!(v["instances"]["not_checked"], json!(1), "{v}");
    assert_eq!(v["instances"]["pass"], json!(1), "{v}");
}

/// t_f07: TOML is an instance format.
#[test]
fn t_f07_toml_instance_is_read() {
    let d = setup("f07", STRING_A, STRING_A_MIN1, &[("c.toml", "a = \"\"\n")]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["file"], json!("c.toml"), "{v}");
}

/// t_f08: YAML is an instance format.
#[test]
fn t_f08_yaml_instance_is_read() {
    let d = setup("f08", STRING_A, STRING_A_MIN1, &[("c.yaml", "a: ''\n")]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["file"], json!("c.yaml"), "{v}");
}

/// t_f09: a reference outside the document is not resolved; stated.
#[test]
fn t_f09_sibling_ref_is_unverifiable() {
    let head = r##"{"type":"object","properties":{"a":{"$ref":"other.json#/definitions/x"}}}"##;
    let d = setup("f09", STRING_A, head, &[("ok.json", r#"{"a":"x"}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("unverifiable"), "{v}");
    assert!(v["not_checked"][0]["reason"].as_str().is_some_and(|r| r.contains("other.json")), "{v}");
}

/// t_f10: format is never asserted.
#[test]
fn t_f10_format_is_not_asserted() {
    let head = r#"{"type":"object","properties":{"a":{"type":"string","format":"email"}}}"#;
    let d = setup("f10", STRING_A, head, &[("ok.json", r#"{"a":"not an email"}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("instances_pass"), "{v}");
}

/// t_f11: every witness is listed, in file order.
#[test]
fn t_f11_every_witness_listed_in_order() {
    let d = setup("f11", STRING_A, STRING_A_MIN1, &[("b.json", r#"{"a":""}"#), ("a.json", r#"{"a":""}"#), ("c.json", r#"{"a":"x"}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert!(text.starts_with("VIOLATED: 2 document(s) valid at base.json are rejected by head.json.\n  rejected: a.json at /a: "), "{text}");
    assert_eq!(v["witnesses"].as_array().map(|w| w.iter().map(|x| x["file"].clone()).collect::<Vec<_>>()), Some(vec![json!("a.json"), json!("b.json")]), "{v}");
    assert!(text.contains("  instances: 3 read, 0 not promises, 1 pass, 0 not checked\n"), "{text}");
}

/// t_f12: GitHub annotations, one per witness.
#[test]
fn t_f12_github_format() {
    let d = setup("f12", STRING_A, STRING_A_MIN1, &[("empty.json", r#"{"a":""}"#)]);
    let (code, out) = run(&d, "github");
    assert_eq!(code, 1, "{out}");
    assert!(out.starts_with("::error file=head.json::"), "{out}");
    assert!(out.contains("empty.json"), "{out}");
}

/// t_f13: an unreadable schema is an error, not a verdict.
#[test]
fn t_f13_unparsable_head_is_an_error() {
    let d = setup("f13", STRING_A, "{", &[("ok.json", r#"{"a":"x"}"#)]);
    let (code, text) = run(&d, "text");
    assert_eq!(code, 3, "{text}");
}

/// t_f14: real bytes, "Removed github-action parallelization keywords".
#[test]
fn t_f14_real_github_action_break() {
    let d = setup("f14", GA_BEFORE, GA_AFTER, &[("parallel-steps.json", GA_TEST)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["pointer"], json!("/runs"), "{v}");
}

/// t_f15: real bytes, ryl.json v0.17.0 drops `yaml-files` from a closed object.
#[test]
fn t_f15_real_ryl_toml_break() {
    let d = setup("f15", RYL_BEFORE, RYL_AFTER, &[("01-valid.toml", RYL_TEST)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["pointer"], json!(""), "{v}");
    assert!(v["witnesses"][0]["message"].as_str().is_some_and(|m| m.contains("yaml-files")), "{v}");
}
