//! JUDGE-REACH r0 fixtures (docs/falsification/JUDGE-REACH-SPEC.md), frozen
//! BEFORE implementation: HOLDS asserts every change was judged. A schema
//! that changed in a way the judge does not check (unreached by its walk,
//! or reached at a keyword it does not read) makes the verdict NOT
//! EVALUATED (unchecked_change), naming each such schema by its JSON
//! Pointer; under VIOLATED the same schemas are stated beside the breaks.
//! r_f12 is real bytes from asyncapi/spec-json-schemas at
//! eee8d2217d23b22c475090c0e12f20801f531f11 (Apache License 2.0).
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - RE-M1 coverage check dropped (unreached changes ignored)   -> r_f04
//! - RE-M2 annotation filter dropped (every byte counts)        -> r_f05
//! - RE-M3 annotation filter blind to position (names dropped)  -> r_f08
//! - RE-M4 a reached ancestor covers all its descendants        -> r_f01
//! - RE-M5 const counted as a keyword the judge reads           -> r_f13

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const ASYNC_200: &str = include_str!("fixtures/judge/asyncapi-2.0.0.json");
const ASYNC_210: &str = include_str!("fixtures/judge/asyncapi-2.1.0.json");

fn pair(tag: &str, base: &str, head: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_re_{tag}_{}_{nanos}", std::process::id()));
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

const HEADLINE: &str = "NOT EVALUATED (unchecked_change): b.json changed in a way this judge does not check. Nothing there was judged a break or a pass; review those schemas by hand.\n";

fn line(pointer: &str) -> String {
    format!("  not evaluated: the schema at {pointer} changed in a way this judge does not check.\n")
}

/// r_f01: the AsyncAPI shape. The changed schema is reached only through
/// patternProperties, then oneOf, then $ref: NOT EVALUATED, naming it.
/// Kills RE-M4 (the root is reached; a reached ancestor must not cover
/// everything beneath it).
#[test]
fn r_f01_change_behind_pattern_and_oneof_is_not_evaluated() {
    let pre = r##"{"properties":{"c":{"$ref":"#/definitions/C"}},"definitions":{"C":{"type":"object","patternProperties":{"^x":{"oneOf":[{"$ref":"#/definitions/T"}]}}},"T":{"type":"object","properties":{"summary":{"type":"string"}}}}}"##;
    let post = r##"{"properties":{"c":{"$ref":"#/definitions/C"}},"definitions":{"C":{"type":"object","patternProperties":{"^x":{"oneOf":[{"$ref":"#/definitions/T"}]}}},"T":{"type":"object","properties":{"summary":{"type":"string"}},"additionalProperties":false,"anyOf":[{"required":["payload"]}]}}}"##;
    let (code, text, v) = both("f01", pre, post);
    assert_eq!(code, 4, "{text}");
    assert!(text.starts_with(HEADLINE), "{text}");
    assert!(text.contains(&line("/definitions/T")), "{text}");
    assert_eq!(v["reason"], json!("unchecked_change"), "{v}");
    assert_eq!(v["unchecked"], json!(["/definitions/T"]), "{v}");
}

/// r_f02 guard: a plain local $ref is followed, judged and proven, as today.
#[test]
fn r_f02_plain_ref_still_judged_and_proven() {
    let pre = r##"{"properties":{"x":{"$ref":"#/definitions/X"}},"definitions":{"X":{"type":"string"}}}"##;
    let post = r##"{"properties":{"x":{"$ref":"#/definitions/X"}},"definitions":{"X":{"type":"integer"}}}"##;
    let (code, text, v) = both("f02", pre, post);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["counterexample"], json!({"x": ""}), "{v}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// r_f03 guard: a reached definition that changed and keeps validity holds.
#[test]
fn r_f03_reached_loosening_holds() {
    let pre = r##"{"properties":{"x":{"$ref":"#/definitions/X"}},"definitions":{"X":{"type":"string","maxLength":5}}}"##;
    let post = r##"{"properties":{"x":{"$ref":"#/definitions/X"}},"definitions":{"X":{"type":"string","maxLength":10}}}"##;
    let (code, text, v) = both("f03", pre, post);
    assert_eq!(code, 0, "{text}");
    assert!(text.starts_with("HOLDS: b.json keeps every promise it made at a.json.\n"), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// r_f04: a definition nothing references changed. Unused is itself a claim
/// r0 does not prove, so it is stated. Kills RE-M1.
#[test]
fn r_f04_unreferenced_definition_change_is_not_evaluated() {
    let pre = r#"{"properties":{"a":{"type":"string"}},"definitions":{"U":{"type":"string"}}}"#;
    let post = r#"{"properties":{"a":{"type":"string"}},"definitions":{"U":{"type":"integer"}}}"#;
    let (code, text, _) = both("f04", pre, post);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(&line("/definitions/U")), "{text}");
}

/// r_f05: an annotation-only change asserts nothing; it holds. Kills RE-M2.
#[test]
fn r_f05_annotation_only_change_holds() {
    let pre = r#"{"properties":{"a":{"type":"string"}},"definitions":{"U":{"type":"string","description":"old","title":"U","examples":["x"],"default":"x","$comment":"c"}}}"#;
    let post = r#"{"properties":{"a":{"type":"string"}},"definitions":{"U":{"type":"string","description":"new","title":"U2","examples":["y"],"default":"y","$comment":"d"}}}"#;
    let (code, text, v) = both("f05", pre, post);
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// r_f06: a keyword no JSON Schema vocabulary defines is ignored by
/// validators; changing it asserts nothing.
#[test]
fn r_f06_unknown_keyword_change_holds() {
    let pre = r#"{"properties":{"a":{"type":"string"}},"definitions":{"U":{"type":"string","markdownDescription":"old","x-vendor":1}}}"#;
    let post = r#"{"properties":{"a":{"type":"string"}},"definitions":{"U":{"type":"string","markdownDescription":"new","x-vendor":2}}}"#;
    let (code, text, _) = both("f06", pre, post);
    assert_eq!(code, 0, "{text}");
}

/// r_f07: additionalProperties as a schema is not judged (only open or
/// closed is): its change is stated, not passed.
#[test]
fn r_f07_additional_properties_schema_change_is_not_evaluated() {
    let pre = r#"{"properties":{"a":{"type":"string"}},"additionalProperties":{"type":"string"}}"#;
    let post = r#"{"properties":{"a":{"type":"string"}},"additionalProperties":{"type":"integer"}}"#;
    let (code, text, _) = both("f07", pre, post);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(&line("/additionalProperties")), "{text}");
}

/// r_f08: under `properties`, keys are names, not keywords: a property
/// named "title" is not an annotation. Kills RE-M3.
#[test]
fn r_f08_property_named_like_an_annotation_counts() {
    let pre = r#"{"properties":{"a":{"type":"string"}},"definitions":{"U":{"properties":{"title":{"type":"string"}}}}}"#;
    let post = r#"{"properties":{"a":{"type":"string"}},"definitions":{"U":{"properties":{"title":{"type":"integer"}}}}}"#;
    let (code, text, _) = both("f08", pre, post);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(&line("/definitions/U/properties/title")), "{text}");
}

/// r_f09: `$defs` (2019-09 and later) is read like `definitions`.
#[test]
fn r_f09_defs_container_is_read() {
    let pre = r#"{"properties":{"a":{"type":"string"}},"$defs":{"U":{"type":"string"}}}"#;
    let post = r#"{"properties":{"a":{"type":"string"}},"$defs":{"U":{"type":"integer"}}}"#;
    let (code, text, _) = both("f09", pre, post);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(&line("/$defs/U")), "{text}");
}

/// r_f10: the JSON surface lists every unreached schema, sorted, and an
/// ancestor stands for its descendants.
#[test]
fn r_f10_json_lists_unreached_sorted_and_coalesced() {
    let pre = r#"{"properties":{"a":{"type":"string"}},"definitions":{"B":{"type":"string"},"A":{"type":"object","properties":{"k":{"type":"string"}}}}}"#;
    let post = r#"{"properties":{"a":{"type":"string"}},"definitions":{"B":{"type":"integer"},"A":{"type":"array","properties":{"k":{"type":"integer"}}}}}"#;
    let (code, text, v) = both("f10", pre, post);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["unchecked"], json!(["/definitions/A", "/definitions/B"]), "{v}");
}

/// r_f11: precedence. A property under a combinator that changed (D1) and
/// an unreached schema that changed: the existing reason leads, both stated.
#[test]
fn r_f11_unjudged_construct_reason_leads() {
    let pre = r#"{"properties":{"p":{"oneOf":[{"type":"string"}]}},"definitions":{"U":{"type":"string"}}}"#;
    let post = r#"{"properties":{"p":{"oneOf":[{"type":"integer"}]}},"definitions":{"U":{"type":"integer"}}}"#;
    let (code, text, v) = both("f11", pre, post);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("unjudged_construct"), "{v}");
    assert!(text.contains("  not evaluated: p uses anyOf, oneOf, allOf or $ref, which this judge does not follow.\n"), "{text}");
    assert!(text.contains(&line("/definitions/U")), "{text}");
}

/// r_f12: real AsyncAPI 2.0.0 to 2.1.0. The version tag narrows (VIOLATED,
/// exit 1, as today); the messageTrait change, which Python jsonschema
/// shows rejects a whole 2.0.0 document, is now stated beside it instead
/// of passing unseen.
#[test]
fn r_f12_real_asyncapi_states_the_unreached_message_trait() {
    let (code, text, v) = both("f12", ASYNC_200, ASYNC_210);
    assert_eq!(code, 1, "{text}");
    // Amendment B: the shallowest node whose own keywords changed is the
    // examples item schema, as the spec rule says, not messageTrait itself.
    assert!(text.contains(&line("/definitions/messageTrait/properties/examples/items")), "{text}");
    let u: Vec<&str> = v["unchecked"].as_array().expect("unchecked").iter().filter_map(Value::as_str).collect();
    assert!(u.contains(&"/definitions/messageTrait/properties/examples/items"), "{v}");
    let mut sorted = u.clone();
    sorted.sort();
    assert_eq!(u, sorted, "{v}");
}

/// r_f13: a reached node, at a keyword the judge does not read: `const`
/// is not judged, so its change is stated, never covered. Kills RE-M5.
#[test]
fn r_f13_reached_node_unread_keyword_is_not_evaluated() {
    let pre = r#"{"properties":{"a":{"type":"string","const":"x"}}}"#;
    let post = r#"{"properties":{"a":{"type":"string","const":"y"}}}"#;
    let (code, text, v) = both("f13", pre, post);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(&line("/properties/a")), "{text}");
    assert_eq!(v["unchecked"], json!(["/properties/a"]), "{v}");
}

// Amendment A (docs/falsification/JUDGE-REACH-SPEC.md): the strict model.
// - RE-M6 an added property always covered                      -> r_f14
// - RE-M7 any additionalProperties change covered               -> r_f15
// - RE-M8 the root judged like a property node                  -> r_f19

const ROOT_LINE: &str = "  not evaluated: the document root changed in a way this judge does not check.\n";

/// r_f14: an open object gains a typed property; {"k":1} was valid before
/// and is rejected now. Kills RE-M6.
#[test]
fn r_f14_open_object_gains_constrained_property() {
    let (code, text, v) = both("f14", r#"{"properties":{"a":{"type":"string"}}}"#, r#"{"properties":{"a":{"type":"string"},"k":{"type":"string"}}}"#);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(&line("/properties/k")), "{text}");
    assert_eq!(v["unchecked"], json!(["/properties/k"]), "{v}");
}

/// r_f15: the root object is closed. Kills RE-M7.
#[test]
fn r_f15_object_closed_is_not_evaluated() {
    let (code, text, v) = both("f15", r#"{"properties":{"a":{"type":"string"}},"additionalProperties":true}"#, r#"{"properties":{"a":{"type":"string"}},"additionalProperties":false}"#);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(ROOT_LINE), "{text}");
    assert_eq!(v["unchecked"], json!([""]), "{v}");
}

/// r_f16 guard: a closed object gains a property; no base document could
/// carry it, so nothing valid before is rejected.
#[test]
fn r_f16_closed_object_gains_property_holds() {
    let (code, text, _) = both("f16", r#"{"properties":{"a":{"type":"string"}},"additionalProperties":false}"#, r#"{"properties":{"a":{"type":"string"},"k":{"type":"string"}},"additionalProperties":false}"#);
    assert_eq!(code, 0, "{text}");
}

/// r_f17: a closed object drops a patternProperties entry.
#[test]
fn r_f17_closed_object_drops_pattern_is_not_evaluated() {
    let (code, text, _) = both("f17", r#"{"type":"object","additionalProperties":false,"patternProperties":{"^x":{}},"properties":{"a":{}}}"#, r#"{"type":"object","additionalProperties":false,"properties":{"a":{}}}"#);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(&line("/patternProperties/^x")), "{text}");
}

/// r_f18 guard: an object opened (false to true) rejects nothing.
#[test]
fn r_f18_object_opened_holds() {
    let (code, text, _) = both("f18", r#"{"properties":{"a":{"type":"string"}},"additionalProperties":false}"#, r#"{"properties":{"a":{"type":"string"}},"additionalProperties":true}"#);
    assert_eq!(code, 0, "{text}");
}

/// r_f19: the root's own type changed; the root is not judged like a
/// property node. Kills RE-M8.
#[test]
fn r_f19_root_type_change_is_not_evaluated() {
    let (code, text, _) = both("f19", r#"{"type":"object","properties":{"f":{}}}"#, r#"{"type":"array","properties":{"f":{}}}"#);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(ROOT_LINE), "{text}");
}
