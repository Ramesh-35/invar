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
//!
//! JUDGE-TESTS r1 (docs/falsification/JUDGE-TESTS-R1-SPEC.md):
//!
//! - TS-M5 Ajv's answer is ignored                       -> t_f16
//! - TS-M6 a split between validators is a witness       -> t_f16
//! - TS-M7 single-validator mode exits 1                 -> t_f17

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
const ES6_BEFORE: &str = include_str!("fixtures/judge/schemastore-es6importsorterrc-f9d455e8c-before.json");
const ES6_AFTER: &str = include_str!("fixtures/judge/schemastore-es6importsorterrc-f9d455e8c-after.json");
const ES6_TEST: &str = include_str!("fixtures/judge/schemastore-es6importsorterrc-f9d455e8c-test.json");

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

/// Like `run`, with extra arguments after DIR.
fn run_with(d: &Path, format: &str, extra: &[&str]) -> (i32, String) {
    let mut args = vec![SCRIPT, "base.json", "head.json", "instances", "--format", format];
    args.extend_from_slice(extra);
    let o = Command::new("python3").args(&args).current_dir(d).output().expect("python3 on the PATH");
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

/// t_f16 (r1): real bytes where the two validators disagree at the head.
/// Kills TS-M5 and TS-M6.
#[test]
fn t_f16_validators_disagree_is_disputed() {
    let d = setup("f16", ES6_BEFORE, ES6_AFTER, &[("es6importsorterrc-test.json", ES6_TEST)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("disputed"), "{v}");
    assert_eq!(v["witnesses"], json!([]), "{v}");
    assert_eq!(v["disputed"][0]["file"], json!("es6importsorterrc-test.json"), "{v}");
    assert_eq!(v["disputed"][0]["jsonschema"], json!("invalid"), "{v}");
    assert_eq!(v["disputed"][0]["ajv"], json!("valid"), "{v}");
    assert!(text.contains("  disputed: es6importsorterrc-test.json: jsonschema invalid, ajv valid"), "{text}");
    assert_eq!(v["validators"], json!(["jsonschema", "ajv"]), "{v}");
}

/// t_f17 (r1): one validator cannot say VIOLATED. Kills TS-M7.
#[test]
fn t_f17_single_validator_is_unconfirmed() {
    let d = setup("f17", STRING_A, STRING_A_MIN1, &[("empty.json", r#"{"a":""}"#)]);
    let (code, text) = run_with(&d, "text", &["--validators", "jsonschema"]);
    assert_eq!(code, 4, "{text}");
    assert!(text.starts_with("NOT EVALUATED (unconfirmed): 1 document(s) rejected by jsonschema alone; a break needs two validators.\n"), "{text}");
    let (_, j) = run_with(&d, "json", &["--validators", "jsonschema"]);
    let v: Value = serde_json::from_str(&j).unwrap();
    assert_eq!(v["validators"], json!(["jsonschema"]), "{v}");
    assert_eq!(v["witnesses"], json!([]), "{v}");
}

/// t_f18 (r1): both validators reject: a witness, with both messages.
#[test]
fn t_f18_both_reject_is_a_witness_with_both_messages() {
    let d = setup("f18", STRING_A, STRING_A_MIN1, &[("empty.json", r#"{"a":""}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["file"], json!("empty.json"), "{v}");
    assert!(v["witnesses"][0]["ajv_message"].as_str().is_some_and(|m| !m.is_empty()), "{v}");
    assert!(text.contains("  rejected: empty.json at /a: ") && text.contains(" | ajv: "), "{text}");
    assert_eq!(v["validators"], json!(["jsonschema", "ajv"]), "{v}");
    assert_eq!(v["instances"]["disputed"], json!(0), "{v}");
}

// JUDGE-TESTS r2 (docs/falsification/JUDGE-TESTS-R2-SPEC.md): the Ajv side
// reads each schema under its own draft: 2019-09 and 2020-12 through Ajv's
// own classes for them, draft-06 with its meta-schema added. Until r2 those
// drafts were `unverifiable` on the Ajv side and no document could be a
// witness. Mutation operators:
// - TS2-M1 every schema read as draft-07 (the draft switch dropped)  -> t2_f01
// - TS2-M2 2020-12 read with the 2019-09 class                        -> t2_f02

const D2019_A: &str = r#"{"$schema":"https://json-schema.org/draft/2019-09/schema","type":"object","properties":{"a":{"type":"string"}}}"#;
const D2019_A_MIN1: &str = r#"{"$schema":"https://json-schema.org/draft/2019-09/schema","type":"object","properties":{"a":{"type":"string","minLength":1}}}"#;
const D2020_ITEMS: &str = r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"t":{"type":"array","prefixItems":[{"type":"string"}],"items":true}}}"#;
const D2020_ITEMS_CLOSED: &str = r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":{"t":{"type":"array","prefixItems":[{"type":"string"}],"items":false}}}"#;
const D06_A: &str = r#"{"$schema":"http://json-schema.org/draft-06/schema#","type":"object","properties":{"a":{"type":"string"}}}"#;
const D06_A_MIN1: &str = r#"{"$schema":"http://json-schema.org/draft-06/schema#","type":"object","properties":{"a":{"type":"string","minLength":1}}}"#;

/// t2_f01: a 2019-09 schema pair; both validators run and both reject. Kills TS2-M1.
#[test]
fn t2_f01_draft_2019_09_is_checked_by_both_validators() {
    let d = setup("t2f01", D2019_A, D2019_A_MIN1, &[("empty.json", r#"{"a":""}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["validators"], json!(["jsonschema", "ajv"]), "{v}");
    assert_eq!(v["witnesses"][0]["file"], json!("empty.json"), "{v}");
    assert!(v["witnesses"][0]["ajv_message"].as_str().is_some_and(|m| !m.is_empty()), "{v}");
    assert_eq!(v["not_checked"], json!([]), "{v}");
}

/// t2_f02: a 2020-12 schema pair using `prefixItems` and `items: false`,
/// which only the 2020-12 class reads as the specification does. Kills TS2-M2.
#[test]
fn t2_f02_draft_2020_12_prefix_items_is_checked() {
    let d = setup("t2f02", D2020_ITEMS, D2020_ITEMS_CLOSED, &[("two.json", r#"{"t":["x",1]}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["file"], json!("two.json"), "{v}");
    assert!(v["witnesses"][0]["ajv_message"].as_str().is_some_and(|m| !m.is_empty()), "{v}");
    assert_eq!(v["not_checked"], json!([]), "{v}");
    assert_eq!(v["instances"]["disputed"], json!(0), "{v}");
}

/// t2_f03: a draft-06 schema pair; both validators run.
#[test]
fn t2_f03_draft_06_is_checked_by_both_validators() {
    let d = setup("t2f03", D06_A, D06_A_MIN1, &[("empty.json", r#"{"a":""}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["file"], json!("empty.json"), "{v}");
    assert!(v["witnesses"][0]["ajv_message"].as_str().is_some_and(|m| !m.is_empty()), "{v}");
    assert_eq!(v["not_checked"], json!([]), "{v}");
}

/// t2_f04 guard: an unknown `$schema` is still read as draft-07 by both, as before.
#[test]
fn t2_f04_unknown_schema_uri_reads_as_draft_07() {
    let pre = r#"{"$schema":"https://example.com/not-a-draft","type":"object","properties":{"a":{"type":"string"}}}"#;
    let post = r#"{"$schema":"https://example.com/not-a-draft","type":"object","properties":{"a":{"type":"string","minLength":1}}}"#;
    let d = setup("t2f04", pre, post, &[("empty.json", r#"{"a":""}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["file"], json!("empty.json"), "{v}");
}

// JUDGE-TESTS r3 (docs/falsification/JUDGE-TESTS-R3-SPEC.md): a `$ref` to a
// file is resolved from the schema's own directory, by the last segment of
// the URI (then with `.json` added), read-only, in both validators, never
// from the network. Frozen before implementation.
//
// - TS3-M1 the jsonschema side resolves nothing (offline registry back) -> t3_f01
// - TS3-M2 the Ajv side loads no sibling                                  -> t3_f01
// - TS3-M3 the `.json` fallback dropped                                   -> t3_f03
// - TS3-M4 head siblings read from the base directory                     -> t3_f06

const SIB_NAME: &str = r#"{"$schema":"http://json-schema.org/draft-07/schema#","definitions":{"name":{"type":"string"}}}"#;
const SIB_NAME_MIN2: &str = r#"{"$schema":"http://json-schema.org/draft-07/schema#","definitions":{"name":{"type":"string","minLength":2}}}"#;

fn with_siblings(d: &Path, files: &[(&str, &str)]) {
    for (name, body) in files {
        let p = d.join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
}

fn run_args(d: &Path, format: &str, extra: &[&str]) -> (i32, String) {
    let mut args = vec![SCRIPT, "base.json", "head.json", "instances", "--format", format];
    args.extend_from_slice(extra);
    let o = Command::new("python3").args(&args).current_dir(d).output().expect("python3 on the PATH");
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn ref_pair(r: &str) -> (String, String) {
    let pre = format!(r#"{{"$schema":"http://json-schema.org/draft-07/schema#","type":"object","properties":{{"n":{{"$ref":"{r}"}}}}}}"#);
    let post = format!(r#"{{"$schema":"http://json-schema.org/draft-07/schema#","type":"object","properties":{{"n":{{"allOf":[{{"$ref":"{r}"}},{{"minLength":2}}]}}}}}}"#);
    (pre, post)
}

/// t3_f01: a relative `$ref` to a sibling file is resolved by both validators;
/// the document goes through the reference on both sides. Kills TS3-M1, TS3-M2.
#[test]
fn t3_f01_relative_sibling_ref_is_resolved_by_both_validators() {
    let (pre, post) = ref_pair("common.json#/definitions/name");
    let d = setup("t3f01", &pre, &post, &[("one.json", r#"{"n":"x"}"#)]);
    with_siblings(&d, &[("common.json", SIB_NAME)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["validators"], json!(["jsonschema", "ajv"]), "{v}");
    assert_eq!(v["witnesses"][0]["file"], json!("one.json"), "{v}");
    assert!(v["witnesses"][0]["ajv_message"].as_str().is_some_and(|m| !m.is_empty()), "{v}");
    assert_eq!(v["not_checked"], json!([]), "{v}");
}

/// t3_f02: an absolute URL `$ref` whose last segment names a sibling is read
/// from that sibling, never fetched.
#[test]
fn t3_f02_absolute_url_ref_is_read_from_the_sibling_file() {
    let (pre, post) = ref_pair("https://www.schemastore.org/common.json#/definitions/name");
    let d = setup("t3f02", &pre, &post, &[("one.json", r#"{"n":"x"}"#)]);
    with_siblings(&d, &[("common.json", SIB_NAME)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["file"], json!("one.json"), "{v}");
    assert_eq!(v["not_checked"], json!([]), "{v}");
}

/// t3_f03: an extensionless `$ref` (`common#/...`) finds `common.json`. Kills TS3-M3.
#[test]
fn t3_f03_extensionless_ref_finds_the_json_sibling() {
    let (pre, post) = ref_pair("common#/definitions/name");
    let d = setup("t3f03", &pre, &post, &[("one.json", r#"{"n":"x"}"#)]);
    with_siblings(&d, &[("common.json", SIB_NAME)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["file"], json!("one.json"), "{v}");
    assert_eq!(v["not_checked"], json!([]), "{v}");
}

/// t3_f04 guard: a `$ref` to a file that is not there stays unverifiable in
/// both validators; nothing is fetched and nothing is a break.
#[test]
fn t3_f04_missing_sibling_stays_unverifiable() {
    let (pre, post) = ref_pair("absent.json#/definitions/name");
    let d = setup("t3f04", &pre, &post, &[("one.json", r#"{"n":"x"}"#)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("unverifiable"), "{v}");
    assert_eq!(v["witnesses"], json!([]), "{v}");
    assert_eq!(v["not_checked"].as_array().map(|a| a.len()), Some(1), "{v}");
}

/// t3_f05: a sibling that itself refers to another sibling is followed, both validators.
#[test]
fn t3_f05_transitive_sibling_refs_are_followed() {
    let a = r#"{"$schema":"http://json-schema.org/draft-07/schema#","definitions":{"wrapped":{"$ref":"b.json#/definitions/name"}}}"#;
    let (pre, post) = ref_pair("a.json#/definitions/wrapped");
    let d = setup("t3f05", &pre, &post, &[("one.json", r#"{"n":"x"}"#)]);
    with_siblings(&d, &[("a.json", a), ("b.json", SIB_NAME)]);
    let (code, text, v) = both(&d);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["file"], json!("one.json"), "{v}");
    assert_eq!(v["not_checked"], json!([]), "{v}");
}

/// t3_f06: each side resolves against its own siblings (`--base-siblings`,
/// `--head-siblings`): the contract is unchanged, the sibling it refers to
/// tightened, and the document that was a promise is rejected. Kills TS3-M4.
#[test]
fn t3_f06_each_side_resolves_against_its_own_siblings() {
    let (pre, _) = ref_pair("common.json#/definitions/name");
    let d = setup("t3f06", &pre, &pre, &[("one.json", r#"{"n":"x"}"#)]);
    with_siblings(&d, &[("sibs/base/common.json", SIB_NAME), ("sibs/head/common.json", SIB_NAME_MIN2)]);
    let extra = ["--base-siblings", "sibs/base", "--head-siblings", "sibs/head"];
    let (code, text) = run_args(&d, "text", &extra);
    let (_, j) = run_args(&d, "json", &extra);
    let v: Value = serde_json::from_str(&j).unwrap_or(Value::Null);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["file"], json!("one.json"), "{v}");
    assert!(v["witnesses"][0]["ajv_message"].as_str().is_some_and(|m| !m.is_empty()), "{v}");
    assert_eq!(v["not_checked"], json!([]), "{v}");
}
