//! JUDGE-CANON r1 fixtures (docs/falsification/JUDGE-CANON-R1-SPEC.md),
//! frozen BEFORE implementation: literals (`const`, `enum`, bare `boolean`
//! and `null`) compared in normal form, a type constraint folded into the
//! enum beside it, an `anyOf` (or disjoint `oneOf`) of enums as one enum,
//! and a difference only in annotations never a verdict. c1_f10 and c1_f11
//! are real bytes from SchemaStore (Apache-2.0) at the named commits.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - CR1-M1 `const` not rewritten to `enum`                -> c1_f01
//! - CR1-M2 `boolean` not a literal                        -> c1_f03
//! - CR1-M3 `oneOf` union without the disjointness check   -> c1_f07
//! - CR1-M4 the type constraint not folded into the enum   -> c1_f09
//! - CR1-M5 D1 keeps refusing annotation-only equality     -> c1_f08

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const HTMLHINT_BEFORE: &str = include_str!("fixtures/judge/schemastore-htmlhint-8a87ed47d-before.json");
const HTMLHINT_AFTER: &str = include_str!("fixtures/judge/schemastore-htmlhint-8a87ed47d-after.json");
const PRETTIERRC_BEFORE: &str = include_str!("fixtures/judge/schemastore-prettierrc-a3cf4d531-before.json");
const PRETTIERRC_AFTER: &str = include_str!("fixtures/judge/schemastore-prettierrc-a3cf4d531-after.json");

fn pair(tag: &str, base: &str, head: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_c1_{tag}_{}_{nanos}", std::process::id()));
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

const LIT_A: &str = "  noted: equivalent after rewriting literals (const, enum, boolean, null) at /properties/a\n";
const HOLDS: &str = "HOLDS: b.json keeps every promise it made at a.json.\n";

/// c1_f01: an enum of one value, written const. Kills CR1-M1.
#[test]
fn c1_f01_enum_of_one_value_to_const_holds() {
    let (code, text, v) = both("f01", &doc(r#"{"enum":["x"]}"#), &doc(r#"{"const":"x"}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.starts_with(HOLDS), "{text}");
    assert!(text.contains(LIT_A), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// c1_f02: a type the enum already implies, dropped; the order changed.
#[test]
fn c1_f02_redundant_type_and_order_hold() {
    let (code, text, _) = both("f02", &doc(r#"{"type":"string","enum":["a","b"]}"#), &doc(r#"{"enum":["b","a"]}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(LIT_A), "{text}");
}

/// c1_f03: the htmlhint shape. Kills CR1-M2.
#[test]
fn c1_f03_enum_to_anyof_of_boolean_and_typed_const_holds() {
    let (code, text, _) = both("f03", &doc(r#"{"enum":[false,true,"x"]}"#), &doc(r#"{"anyOf":[{"type":"boolean"},{"type":"string","const":"x"}]}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(LIT_A), "{text}");
}

/// c1_f04 guard: an enum narrowed is a break, as before.
#[test]
fn c1_f04_narrowed_enum_is_still_violated() {
    let (code, text, v) = both("f04", &doc(r#"{"enum":["a","b"]}"#), &doc(r#"{"enum":["a"]}"#));
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["kind"], json!("narrowed"), "{v}");
    assert!(text.contains(r#"counterexample (valid at a.json, rejected now): {"a":"b"}"#), "{text}");
    assert!(!text.contains("noted: equivalent"), "{text}");
}

/// c1_f05 guard: integer is not a literal.
#[test]
fn c1_f05_integer_branch_is_not_covered() {
    let (code, text, v) = both("f05", &doc(r#"{"enum":["a",1]}"#), &doc(r#"{"anyOf":[{"type":"string","const":"a"},{"type":"integer"}]}"#));
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("unjudged_construct"), "{v}");
    assert!(!text.contains("noted: equivalent"), "{text}");
}

/// c1_f06: a oneOf of disjoint enums is their union.
#[test]
fn c1_f06_disjoint_oneof_of_enums_holds() {
    let (code, text, _) = both("f06", &doc(r#"{"enum":["a","b","c"]}"#), &doc(r#"{"oneOf":[{"enum":["a"]},{"enum":["b","c"]}]}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(LIT_A), "{text}");
}

/// c1_f07 guard: overlapping oneOf enums reject the shared value. Kills CR1-M3.
#[test]
fn c1_f07_overlapping_oneof_is_not_the_union() {
    let (code, text, v) = both("f07", &doc(r#"{"enum":["a","b","c"]}"#), &doc(r#"{"oneOf":[{"enum":["a","b"]},{"enum":["b","c"]}]}"#));
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("unjudged_construct"), "{v}");
    assert!(!text.contains("noted: equivalent"), "{text}");
}

/// c1_f08: a combinator node differing only in annotations. Kills CR1-M5.
#[test]
fn c1_f08_annotation_only_difference_at_a_combinator_holds_silently() {
    let pre = doc(r#"{"description":"old","anyOf":[{"type":"string"},{"type":"integer"}]}"#);
    let post = doc(r#"{"description":"new","markdownDescription":"**new**","anyOf":[{"type":"string"},{"type":"integer"}]}"#);
    let (code, text, v) = both("f08", &pre, &post);
    assert_eq!(code, 0, "{text}");
    assert!(text.starts_with(HOLDS), "{text}");
    assert!(!text.contains("noted:"), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// c1_f09: a value its type never admitted. Kills CR1-M4.
#[test]
fn c1_f09_value_outside_the_type_is_dropped() {
    let (code, text, _) = both("f09", &doc(r#"{"type":"string","enum":["a",null]}"#), &doc(r#"{"const":"a"}"#));
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(LIT_A), "{text}");
}

/// c1_f10: real bytes; every stated node covered, the root withheld by JUDGE-DRAFT r0.
#[test]
fn c1_f10_real_htmlhint_nodes_covered_root_withheld() {
    let (code, text, v) = both("f10", HTMLHINT_BEFORE, HTMLHINT_AFTER);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["reason"], json!("unchecked_change"), "{v}");
    assert_eq!(v["unchecked"], json!([""]), "{v}");
    assert_eq!(v["not_evaluated_paths"], json!([]), "{v}");
    assert!(text.contains("noted: equivalent after rewriting literals"), "{text}");
}

/// c1_f11: real bytes; prettierrc's options written const.
#[test]
fn c1_f11_real_prettierrc_holds() {
    let (code, text, v) = both("f11", PRETTIERRC_BEFORE, PRETTIERRC_AFTER);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("  noted: equivalent after rewriting literals (const, enum, boolean, null) at /definitions/optionsDefinition/properties/arrowParens/oneOf/0\n"), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}
