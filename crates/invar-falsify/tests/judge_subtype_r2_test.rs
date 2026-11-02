//! JUDGE-SUBTYPE r2 fixtures (docs/falsification/JUDGE-SUBTYPE-R2-SPEC.md),
//! frozen with the spec BEFORE implementation. Found by replaying the judge
//! over six months of SchemaStore: what was not judged must never be
//! reported as a break (D1), a removal from an object that admits other
//! keys is not a narrowing (D2), and an enum replaced by bounds is judged
//! on the values it used to accept (D3).
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - R2-M1 combinator check dropped                 -> r2_f01
//! - R2-M2 closed-object check ignored              -> r2_f04
//! - R2-M3 old enum values not checked              -> r2_f07
//! - R2-M4 unjudged_construct reported as HOLDS     -> r2_f01
//! - R2-M5 local reference resolution disabled     -> r2_f11, r2_f12
//! - R2-M6 implied-type check dropped               -> r2_f15

use std::path::{Path, PathBuf};
use std::process::Command;

const B1_001_PRE: &str = include_str!("fixtures/judge/b1-001-pre.json");
const B1_001_POST: &str = include_str!("fixtures/judge/b1-001-post.json");
const B1_002_PRE: &str = include_str!("fixtures/judge/b1-002-pre.json");
const B1_002_POST: &str = include_str!("fixtures/judge/b1-002-post.json");
const C: &str = "schemas/contract.json";

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_r2_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(d: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(args)
        .current_dir(d)
        .env("GIT_AUTHOR_NAME", "r2").env("GIT_AUTHOR_EMAIL", "r2@test")
        .env("GIT_COMMITTER_NAME", "r2").env("GIT_COMMITTER_EMAIL", "r2@test")
        .output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

fn repo(tag: &str, pre: &str, post: &str) -> PathBuf {
    let d = ws(tag);
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    std::fs::write(d.join(C), pre).unwrap();
    git(&d, &["add", C]);
    git(&d, &["commit", "-q", "-m", "base"]);
    std::fs::write(d.join(C), post).unwrap();
    d
}

fn judge(d: &Path, format: &str) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["judge", "--contract", C, "--base", "main", "--format", format])
        .current_dir(d)
        .env("HOME", d)
        .output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned())
}

fn json(d: &Path) -> (i32, serde_json::Value) {
    let (code, out) = judge(d, "json");
    (code, serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}")))
}

fn strs(v: &serde_json::Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().map(|x| x.as_str().unwrap_or(&x.to_string()).to_string()).collect()).unwrap_or_default()
}

/// r2_f01 D1: a field moved behind anyOf and nothing else changed. Not
/// judged, so not accused and not passed: NOT EVALUATED, the path named.
/// Kills R2-M1 and R2-M4.
#[test]
fn r2_f01_field_moved_into_anyof_is_not_evaluated() {
    let pre = r#"{"properties":{"deps":{"type":"array","items":{"type":"string"}}}}"#;
    let post = r#"{"properties":{"deps":{"anyOf":[{"type":"array"},{"type":"string"}]}}}"#;
    let d = repo("f01", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 4, "{v}");
    assert_eq!(v["verdict"], "NOT EVALUATED", "{v}");
    assert_eq!(v["reason"], "unjudged_construct", "{v}");
    assert_eq!(strs(&v["not_evaluated_paths"]), vec!["deps"], "{v}");
    assert_eq!(v["witnesses"].as_array().map(Vec::len), Some(0), "{v}");
    let (_, text) = judge(&d, "text");
    assert!(!text.contains("retyped"), "{text}");
    assert!(text.contains("deps"), "{text}");
}

/// r2_f02 D1: a $ref node identical at both refs is not listed; a real
/// closed removal beside it is still VIOLATED.
#[test]
fn r2_f02_unchanged_ref_is_not_listed() {
    let pre = r##"{"additionalProperties":false,"properties":{"a":{"$ref":"#/definitions/x"},"b":{"type":"string"}}}"##;
    let post = r##"{"additionalProperties":false,"properties":{"a":{"$ref":"#/definitions/x"}}}"##;
    let d = repo("f02", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 1, "{v}");
    assert_eq!(v["witnesses"][0]["kind"], "removed", "{v}");
    assert_eq!(v["witnesses"][0]["path"], "b", "{v}");
    assert!(strs(&v["not_evaluated_paths"]).is_empty(), "{v}");
}

/// r2_f03 D1: a changed allOf field plus a closed removal: VIOLATED, and
/// the unjudged field is still stated.
#[test]
fn r2_f03_violation_and_unjudged_field_both_stated() {
    let pre = r#"{"additionalProperties":false,"properties":{"a":{"allOf":[{"type":"string"}]},"b":{"type":"string"}}}"#;
    let post = r#"{"additionalProperties":false,"properties":{"a":{"allOf":[{"type":"string"},{"minLength":2}]}}}"#;
    let d = repo("f03", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 1, "{v}");
    assert_eq!(v["witnesses"].as_array().map(Vec::len), Some(1), "{v}");
    assert_eq!(strs(&v["not_evaluated_paths"]), vec!["a"], "{v}");
}

/// r2_f04 D2: removal from an open root object is a note, not a break.
/// Kills R2-M2.
#[test]
fn r2_f04_open_object_removal_is_a_note() {
    let pre = r#"{"properties":{"keep":{"type":"string"},"gone":{"type":"boolean"}}}"#;
    let post = r#"{"properties":{"keep":{"type":"string"}}}"#;
    let d = repo("f04", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["verdict"], "HOLDS", "{v}");
    let notes = strs(&v["notes"]);
    assert_eq!(notes.len(), 1, "{v}");
    assert!(notes[0].contains("gone"), "{v}");
    let (_, text) = judge(&d, "text");
    assert!(text.contains("still validate"), "{text}");
}

/// r2_f05 D2: removal from a closed object is VIOLATED.
#[test]
fn r2_f05_closed_object_removal_is_violated() {
    let pre = r#"{"additionalProperties":false,"properties":{"keep":{"type":"string"},"gone":{"type":"boolean"}}}"#;
    let post = r#"{"additionalProperties":false,"properties":{"keep":{"type":"string"}}}"#;
    let d = repo("f05", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 1, "{v}");
    assert_eq!(v["witnesses"][0]["kind"], "removed", "{v}");
    assert!(strs(&v["notes"]).is_empty(), "{v}");
}

/// r2_f06 D2: closedness is judged at the object the field left.
#[test]
fn r2_f06_nested_open_and_closed_objects() {
    let pre = r#"{"properties":{"open":{"type":"object","properties":{"x":{"type":"string"}}},"shut":{"type":"object","unevaluatedProperties":false,"properties":{"y":{"type":"string"}}}}}"#;
    let post = r#"{"properties":{"open":{"type":"object","properties":{}},"shut":{"type":"object","unevaluatedProperties":false,"properties":{}}}}"#;
    let d = repo("f06", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 1, "{v}");
    assert_eq!(v["witnesses"].as_array().map(Vec::len), Some(1), "{v}");
    assert_eq!(v["witnesses"][0]["path"], "shut.y", "{v}");
    assert!(strs(&v["notes"]).iter().any(|n| n.contains("open.x")), "{v}");
}

/// r2_f07 D3: the real bun.lock change (SchemaStore 1ace7f834): enum [0, 1]
/// replaced by minimum 0 accepts more. Kills R2-M3.
#[test]
fn r2_f07_enum_replaced_by_wider_bound_holds() {
    let pre = r#"{"properties":{"lockfileVersion":{"type":"integer","enum":[0,1],"description":"Specifies the version of the lockfile format. Currently only sypporting 0 or 1."}}}"#;
    let post = r#"{"properties":{"lockfileVersion":{"type":"integer","minimum":0,"description":"Specifies the version of the lockfile format."}}}"#;
    let d = repo("f07", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["verdict"], "HOLDS", "{v}");
}

/// r2_f08 D3: a bound that rejects an old enum value is still a break.
#[test]
fn r2_f08_bound_rejecting_an_old_enum_value_is_violated() {
    let pre = r#"{"properties":{"n":{"type":"integer","enum":[0,1,5]}}}"#;
    let post = r#"{"properties":{"n":{"type":"integer","maximum":3}}}"#;
    let d = repo("f08", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 1, "{v}");
    assert_eq!(v["witnesses"][0]["kind"], "tightened", "{v}");
}

/// r2_f09: the B1 corpus is unchanged by r2.
#[test]
fn r2_f09_b1_verdicts_unchanged() {
    let d = repo("f09a", B1_001_PRE, B1_001_POST);
    let (code, text) = judge(&d, "text");
    assert_eq!(code, 1);
    assert!(text.starts_with("VIOLATED: schemas/contract.json breaks 10 promises"), "{text}");
    let d = repo("f09b", B1_002_PRE, B1_002_POST);
    let (code, text) = judge(&d, "text");
    assert_eq!(code, 1);
    assert!(text.starts_with("VIOLATED: schemas/contract.json breaks 1 promise "), "{text}");
}

/// r2_f10: on GitHub, an unjudged construct is one warning naming the reason.
#[test]
fn r2_f10_github_warning_for_unjudged_construct() {
    let pre = r#"{"properties":{"deps":{"type":"string"}}}"#;
    let post = r#"{"properties":{"deps":{"oneOf":[{"type":"string"},{"type":"number"}]}}}"#;
    let d = repo("f10", pre, post);
    let (code, out) = judge(&d, "github");
    assert_eq!(code, 4, "{out}");
    assert!(out.lines().any(|l| l.starts_with("::warning file=schemas/contract.json,title=invar.judge.not_evaluated::NOT EVALUATED (unjudged_construct)")), "{out}");
}

/// r2_f11 D4: moving a field inline to a local $ref with the same content
/// changes nothing a validator accepts (the helmfile case). Kills R2-M5.
#[test]
fn r2_f11_inline_to_local_ref_same_content_holds() {
    let pre = r#"{"properties":{"mode":{"type":"string","enum":["Error","Warn"]}}}"#;
    let post = r##"{"definitions":{"Mode":{"type":"string","enum":["Error","Warn"]}},"properties":{"mode":{"$ref":"#/definitions/Mode"}}}"##;
    let d = repo("f11", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["verdict"], "HOLDS", "{v}");
    assert!(strs(&v["not_evaluated_paths"]).is_empty(), "{v}");
}

/// r2_f12 D4: a $defs target tightened is a break at the field that uses
/// it. Kills R2-M5.
#[test]
fn r2_f12_tightened_defs_target_is_violated() {
    let pre = r##"{"$defs":{"Name":{"type":"string"}},"properties":{"name":{"$ref":"#/$defs/Name"}}}"##;
    let post = r##"{"$defs":{"Name":{"type":"string","minLength":1}},"properties":{"name":{"$ref":"#/$defs/Name"}}}"##;
    let d = repo("f12", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 1, "{v}");
    assert_eq!(v["witnesses"][0]["kind"], "tightened", "{v}");
    assert_eq!(v["witnesses"][0]["path"], "name", "{v}");
}

/// r2_f13 D4: a reference cycle is cut, stated, and the judge returns.
#[test]
fn r2_f13_reference_cycle_is_not_evaluated() {
    let pre = r##"{"definitions":{"Node":{"$ref":"#/definitions/Node"}},"properties":{"n":{"$ref":"#/definitions/Node"}}}"##;
    let post = r##"{"definitions":{"Node":{"$ref":"#/definitions/Node"}},"properties":{"n":{"$ref":"#/definitions/Node","description":"x"}}}"##;
    let d = repo("f13", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 4, "{v}");
    assert_eq!(v["reason"], "unjudged_construct", "{v}");
    assert_eq!(strs(&v["not_evaluated_paths"]), vec!["n"], "{v}");
}

/// r2_f14 D4: an external reference is not followed; it is stated.
#[test]
fn r2_f14_external_reference_is_not_evaluated() {
    let pre = r#"{"properties":{"x":{"type":"string"}}}"#;
    let post = r##"{"properties":{"x":{"$ref":"other.json#/x"}}}"##;
    let d = repo("f14", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 4, "{v}");
    assert_eq!(strs(&v["not_evaluated_paths"]), vec!["x"], "{v}");
}

/// r2_f15 D5: a type made explicit that the field's own keywords already
/// implied is a stated note, not a break. Kills R2-M6.
#[test]
fn r2_f15_implied_type_made_explicit_is_a_note() {
    let pre = r#"{"properties":{"header":{"properties":{"title":{"type":"string"}}}}}"#;
    let post = r#"{"properties":{"header":{"type":"object","properties":{"title":{"type":"string"}}}}}"#;
    let d = repo("f15", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["verdict"], "HOLDS", "{v}");
    assert!(strs(&v["notes"]).iter().any(|n| n.contains("header") && n.contains("type made explicit")), "{v}");
}

/// r2_f16 D5: a new type that contradicts the implied one is a break.
#[test]
fn r2_f16_contradicting_type_is_violated() {
    let pre = r#"{"properties":{"header":{"properties":{"title":{"type":"string"}}}}}"#;
    let post = r#"{"properties":{"header":{"type":"string","properties":{"title":{"type":"string"}}}}}"#;
    let d = repo("f16", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 1, "{v}");
    assert_eq!(v["witnesses"][0]["kind"], "retyped", "{v}");
}

/// r2_f17: a reference cycle through `properties` (found by the replay on
/// SchemaStore's jsone.json, which overflowed the stack) terminates, and
/// a real change beside it is still judged.
#[test]
fn r2_f17_reference_cycle_through_properties_terminates() {
    let pre = r##"{"additionalProperties":false,"definitions":{"Tree":{"type":"object","properties":{"child":{"$ref":"#/definitions/Tree"},"name":{"type":"string"}}}},"properties":{"root":{"$ref":"#/definitions/Tree"},"gone":{"type":"string"}}}"##;
    let post = r##"{"additionalProperties":false,"definitions":{"Tree":{"type":"object","properties":{"child":{"$ref":"#/definitions/Tree"},"name":{"type":"string"}}}},"properties":{"root":{"$ref":"#/definitions/Tree"}}}"##;
    let d = repo("f17", pre, post);
    let (code, v) = json(&d);
    assert_eq!(code, 1, "{v}");
    assert_eq!(v["witnesses"][0]["path"], "gone", "{v}");
}
