//! COUNTEREXAMPLE r0 fixtures (docs/falsification/COUNTEREXAMPLE-SPEC.md),
//! frozen with the spec BEFORE implementation. Each VIOLATED witness
//! carries one instance the base accepts and the post rejects, checked by
//! the judge itself before it is printed; otherwise the output says why
//! none was built. Never a guess. The expected instances were checked with
//! Python jsonschema at freeze (COUNTEREXAMPLE-SPEC.frozen).
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - CX-M1 base check skipped                          -> cx_f11
//! - CX-M2 post check skipped                          -> cx_f12
//! - CX-M3 kept enum value used instead of the lost one -> cx_f03
//! - CX-M4 tightened value on the bound, not past it   -> cx_f04
//! - CX-M5 required properties not built               -> cx_f06

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const B1_001_PRE: &str = include_str!("fixtures/judge/b1-001-pre.json");
const B1_001_POST: &str = include_str!("fixtures/judge/b1-001-post.json");

fn repo(tag: &str, pre: &str, post: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_cx_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    let git = |args: &[&str]| {
        let o = Command::new("git").args(args).current_dir(&d)
            .env("GIT_AUTHOR_NAME", "cx").env("GIT_AUTHOR_EMAIL", "cx@test")
            .env("GIT_COMMITTER_NAME", "cx").env("GIT_COMMITTER_EMAIL", "cx@test")
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

/// The witnesses of one judgment, as JSON.
fn witnesses(pre: &str, post: &str, tag: &str) -> Vec<Value> {
    let d = repo(tag, pre, post);
    let (code, out) = judge(&d, "json");
    assert_eq!(code, 1, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}"));
    v["witnesses"].as_array().expect("witnesses").clone()
}

/// The single witness's counterexample and reason.
fn one(pre: &str, post: &str, tag: &str) -> (Value, Value) {
    let w = witnesses(pre, post, tag);
    assert_eq!(w.len(), 1, "{w:?}");
    (w[0]["counterexample"].clone(), w[0]["counterexample_reason"].clone())
}

fn field(f: &str) -> String {
    format!(r#"{{"properties":{{"f":{f}}}}}"#)
}

const CLOSED: &str = r#"{"additionalProperties":false,"properties":{"name":{"type":"string"},"age":{"type":"integer"}}}"#;
const CLOSED_NARROWER: &str = r#"{"additionalProperties":false,"properties":{"name":{"type":"string"}}}"#;

/// cx_f01: a removed property from a closed object, in every surface's words.
#[test]
fn cx_f01_removed_from_closed_object() {
    let (cx, why) = one(CLOSED, CLOSED_NARROWER, "f01");
    assert_eq!(cx, json!({"age": 0}));
    assert_eq!(why, Value::Null);
    let d = repo("f01t", CLOSED, CLOSED_NARROWER);
    let (_, out) = judge(&d, "text");
    assert!(out.contains("\n    counterexample (valid at main, rejected now): {\"age\":0}\n"), "{out}");
}

/// cx_f02: a value of a dropped type.
#[test]
fn cx_f02_retyped_carries_a_dropped_type_value() {
    let (cx, _) = one(&field(r#"{"type":"number"}"#), &field(r#"{"type":"integer"}"#), "f02a");
    assert_eq!(cx, json!({"f": 0.5}));
    let (cx, _) = one(&field(r#"{"type":["boolean","null"]}"#), &field(r#"{"type":"boolean"}"#), "f02b");
    assert_eq!(cx, json!({"f": null}));
}

/// cx_f03: the first lost enum value. Kills CX-M3.
#[test]
fn cx_f03_narrowed_carries_the_lost_value() {
    let (cx, _) = one(&field(r#"{"enum":["a","b","c"]}"#), &field(r#"{"enum":["a","b"]}"#), "f03");
    assert_eq!(cx, json!({"f": "c"}));
}

/// cx_f04: one step past the new bound. Kills CX-M4.
#[test]
fn cx_f04_tightened_steps_past_the_new_bound() {
    let (cx, _) = one(&field(r#"{"type":"integer","maximum":100}"#), &field(r#"{"type":"integer","maximum":10}"#), "f04a");
    assert_eq!(cx, json!({"f": 11}));
    let (cx, _) = one(&field(r#"{"type":"integer","minimum":0}"#), &field(r#"{"type":"integer","minimum":5}"#), "f04b");
    assert_eq!(cx, json!({"f": 4}));
    let (cx, _) = one(&field(r#"{"type":"integer","minimum":0}"#), &field(r#"{"type":"integer","minimum":0.5}"#), "f04c");
    assert_eq!(cx, json!({"f": 0}));
    let (cx, _) = one(&field(r#"{"type":"string","maxLength":10}"#), &field(r#"{"type":"string","maxLength":3}"#), "f04d");
    assert_eq!(cx, json!({"f": "aaaa"}));
    let (cx, _) = one(&field(r#"{"type":"array","items":{"type":"string"}}"#), &field(r#"{"type":"array","items":{"type":"string"},"minItems":2}"#), "f04e");
    assert_eq!(cx, json!({"f": [""]}));
}

/// cx_f05: the instance that omits the newly required property.
#[test]
fn cx_f05_escalated_omits_the_field() {
    let (cx, _) = one(r#"{"properties":{"id":{"type":"string"}}}"#, r#"{"required":["id"],"properties":{"id":{"type":"string"}}}"#, "f05");
    assert_eq!(cx, json!({}));
}

/// cx_f06: required parents are built along the path. Kills CX-M5.
#[test]
fn cx_f06_nested_path_builds_required_parents() {
    let pre = r#"{"required":["payload"],"properties":{"payload":{"type":"object","required":["user"],"properties":{"user":{"type":"object","additionalProperties":false,"properties":{"zipcode":{"type":"string"},"name":{"type":"string"}}}}}}}"#;
    let post = r#"{"required":["payload"],"properties":{"payload":{"type":"object","required":["user"],"properties":{"user":{"type":"object","additionalProperties":false,"properties":{"name":{"type":"string"}}}}}}}"#;
    let (cx, _) = one(pre, post, "f06");
    assert_eq!(cx, json!({"payload": {"user": {"zipcode": ""}}}));
}

/// cx_f07 (amended by JUDGE-CC r0): a pattern change produces no witness
/// at all, so there is nothing for the builder to answer: NOT EVALUATED.
#[test]
fn cx_f07_pattern_change_has_no_counterexample() {
    let d = repo("f07", &field(r#"{"type":"string","pattern":"^a"}"#), &field(r#"{"type":"string","pattern":"^b"}"#));
    let (code, out) = judge(&d, "json");
    assert_eq!(code, 4, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}"));
    assert_eq!(v["witnesses"], serde_json::json!([]), "{v}");
    assert_eq!(v["reason"], serde_json::json!("undecided_constraint"), "{v}");
}

/// cx_f08: a required sibling the checker cannot judge: no guess.
#[test]
fn cx_f08_unchecked_required_sibling_is_stated() {
    let pre = r#"{"additionalProperties":false,"required":["name"],"properties":{"name":{"type":"string","pattern":"^[A-Z]"},"age":{"type":"integer"}}}"#;
    let post = r#"{"additionalProperties":false,"required":["name"],"properties":{"name":{"type":"string","pattern":"^[A-Z]"}}}"#;
    let (cx, why) = one(pre, post, "f08");
    assert_eq!(cx, Value::Null);
    assert_eq!(why, "pattern is not checked by this judge");
}

/// cx_f09: the real b1-001 change: every one of its 6 breaks is shown.
#[test]
fn cx_f09_b1_001_every_break_has_a_counterexample() {
    let w = witnesses(B1_001_PRE, B1_001_POST, "f09");
    assert_eq!(w.len(), 6, "{w:?}");
    for x in &w {
        assert!(x["counterexample"].is_object(), "{x}");
        assert_eq!(x["counterexample_reason"], Value::Null, "{x}");
    }
    let d = repo("f09b", B1_001_PRE, B1_001_POST);
    assert_eq!(judge(&d, "json"), judge(&d, "json"));
}

/// cx_f10 (amended by JUDGE-CC r0): the JSON keys exist on every witness
/// (a decided one; pattern changes no longer carry witnesses); GitHub
/// carries the line.
#[test]
fn cx_f10_surfaces() {
    let w = witnesses(&field(r#"{"enum":["x","y"]}"#), &field(r#"{"enum":["x"]}"#), "f10");
    let keys: Vec<&String> = w[0].as_object().expect("object").keys().collect();
    assert!(keys.iter().any(|k| *k == "counterexample") && keys.iter().any(|k| *k == "counterexample_reason"), "{keys:?}");
    let d = repo("f10g", CLOSED, CLOSED_NARROWER);
    let (_, out) = judge(&d, "github");
    let line = out.lines().find(|l| l.starts_with("::error ")).expect("annotation");
    assert!(line.ends_with("%0Acounterexample (valid at main, rejected now): {\"age\":0}"), "{line}");
}

/// cx_f11: the base check refuses what it cannot check. Kills CX-M1.
/// COUNTEREXAMPLE r1 amendment: r1 reads `minProperties`, so the root
/// keyword the checker cannot read is `propertyNames` instead.
#[test]
fn cx_f11_unchecked_keyword_at_base_is_stated() {
    let pre = r#"{"propertyNames":{"maxLength":10},"additionalProperties":false,"properties":{"name":{"type":"string"},"age":{"type":"integer"}}}"#;
    let post = r#"{"propertyNames":{"maxLength":10},"additionalProperties":false,"properties":{"name":{"type":"string"}}}"#;
    let (cx, why) = one(pre, post, "f11");
    assert_eq!(cx, Value::Null);
    assert_eq!(why, "propertyNames is not checked by this judge");
}

/// cx_f12: the judge claims a break the post check cannot confirm, so
/// the output says so. Kills CX-M2. JUDGE-R3 r0 amendment: the original
/// case (patternProperties admitting the removed key) was a false claim,
/// which J1 now refuses to make. Here the post bound sits beside a `$ref`:
/// the judge merges the two (r2 D4) and claims `maxLength` tightened, but
/// drafts disagree on whether a keyword beside `$ref` applies (draft-07
/// ignores it), so the checker cannot confirm the rejection.
/// Amended by REF-SIBLINGS r0: under draft-07 the bound beside `$ref` is no
/// longer judged at all. Under 2019-09 siblings apply and the node is
/// judged, so the same guard moves there: the judge claims `maxLength`
/// tightened, the checker does not follow `$ref` and cannot confirm the
/// rejection, and no counterexample is claimed. Kills CX-M2.
#[test]
fn cx_f12_unconfirmed_rejection_is_stated() {
    let pre = r#"{"$schema":"https://json-schema.org/draft/2019-09/schema","properties":{"f":{"type":"string","maxLength":10}}}"#;
    let post = r##"{"$schema":"https://json-schema.org/draft/2019-09/schema","$defs":{"s":{"type":"string"}},"properties":{"f":{"$ref":"#/$defs/s","maxLength":3}}}"##;
    let (cx, why) = one(pre, post, "f12");
    assert_eq!(cx, Value::Null);
    assert_eq!(why, "$ref is not checked by this judge");
}

/// cx_f14 (added with JUDGE-CC r0's corpus run, 2026-11-09): a bound at the
/// i64 edge must not crash the builder. Found on Iglu
/// com.snowplowanalytics.mobile/remote_config 1-0-0 -> 1-0-1, where the head
/// adds `maximum: 9223372036854775807` and the step above it overflowed
/// (panic, exit 101) in every debug build since the step was written. The
/// verdict is VIOLATED (tightened); whether a counterexample can be built at
/// the edge is stated, never a crash.
#[test]
fn cx_f14_i64_edge_bound_never_panics() {
    let pre = r#"{"properties":{"n":{"type":"integer"}}}"#;
    let post = r#"{"properties":{"n":{"type":"integer","maximum":9223372036854775807}}}"#;
    let d = repo("f14", pre, post);
    let (code, out) = judge(&d, "json");
    assert_eq!(code, 1, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}"));
    assert_eq!(v["witnesses"][0]["kind"], serde_json::json!("tightened"), "{v}");
    // The down direction: a minimum added at the i64 edge; the step below
    // it saturates. (A one-step raise at the edge is not used here: the
    // bound comparison is f64, where MIN and MIN+1 are equal — the parked
    // numeric-precision class, stated, not decided by this fixture.)
    let pre2 = r#"{"properties":{"n":{"type":"integer"}}}"#;
    let post2 = r#"{"properties":{"n":{"type":"integer","minimum":-9223372036854775808}}}"#;
    let d2 = repo("f14b", pre2, post2);
    let (code2, out2) = judge(&d2, "json");
    assert_eq!(code2, 1, "{out2}");
}
