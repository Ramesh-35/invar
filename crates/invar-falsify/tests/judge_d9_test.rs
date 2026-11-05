//! JUDGE-D9 r0 fixtures (docs/falsification/JUDGE-D9-SPEC.md), frozen
//! BEFORE implementation: a field with an enum (or const) and no type that
//! gains a type every one of its values already has is not a break.
//! d9_f07 and d9_f08 are real bytes from snowplow/iglu-central at
//! 7771e92d6e8a43f3bc9074393869ae9bcb08af1e (Apache License 2.0).
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - D9-M1 enum coverage check dropped          -> d9_f04
//! - D9-M2 integer not an instance of number    -> d9_f06
//! - D9-M3 enum values not offered as candidates -> d9_f04

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const CAMPAIGN_PRE: &str = include_str!("fixtures/judge/iglu-campaign-attribution-1-0-0.json");
const CAMPAIGN_POST: &str = include_str!("fixtures/judge/iglu-campaign-attribution-1-0-1.json");
const YAUAA_PRE: &str = include_str!("fixtures/judge/iglu-yauaa-context-1-0-3.json");
const YAUAA_POST: &str = include_str!("fixtures/judge/iglu-yauaa-context-1-0-4.json");

fn repo(tag: &str, pre: &str, post: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_d9_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    let git = |args: &[&str]| {
        let o = Command::new("git").args(args).current_dir(&d)
            .env("GIT_AUTHOR_NAME", "d9").env("GIT_AUTHOR_EMAIL", "d9@test")
            .env("GIT_COMMITTER_NAME", "d9").env("GIT_COMMITTER_EMAIL", "d9@test")
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

fn both(pre: &str, post: &str, tag: &str) -> (i32, String, Value) {
    let d = repo(tag, pre, post);
    let (code, text) = judge(&d, "text");
    let (_, j) = judge(&d, "json");
    (code, text, serde_json::from_str(&j).unwrap_or_else(|e| panic!("not JSON ({e}): {j}")))
}

fn field(f: &str) -> String {
    format!(r#"{{"properties":{{"f":{f}}}}}"#)
}

/// d9_f01: every enum value is a string. Kills D9-M1.
#[test]
fn d9_f01_enum_of_strings_gains_string() {
    let (code, text, _) = both(&field(r#"{"enum":["a","b"]}"#), &field(r#"{"type":"string","enum":["a","b"]}"#), "f01");
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("  noted: type made explicit: f is now string, which every value of its enum already is; every value valid before is still valid.\n"), "{text}");
}

/// d9_f02: a wider type list still covers every value.
#[test]
fn d9_f02_enum_of_strings_gains_string_or_null() {
    let (code, text, _) = both(&field(r#"{"enum":["a","b"]}"#), &field(r#"{"type":["string","null"],"enum":["a","b"]}"#), "f02");
    assert_eq!(code, 0, "{text}");
}

/// d9_f03: const counts as a one-value enum.
#[test]
fn d9_f03_const_gains_its_type() {
    let (code, text, _) = both(&field(r#"{"const":"x"}"#), &field(r#"{"type":"string","const":"x"}"#), "f03");
    assert_eq!(code, 0, "{text}");
}

/// d9_f04: 2 is not a string: a break, and 2 is the instance shown.
/// Kills D9-M1 and D9-M3.
#[test]
fn d9_f04_type_excluding_an_enum_value_is_violated() {
    let (code, text, v) = both(&field(r#"{"enum":["1",2]}"#), &field(r#"{"type":"string","enum":["1",2]}"#), "f04");
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["kind"], "retyped", "{v}");
    assert_eq!(v["witnesses"][0]["counterexample"], json!({"f": 2}), "{v}");
}

/// d9_f05: 1.5 is not an integer.
#[test]
fn d9_f05_fraction_is_not_an_integer() {
    let (code, text, v) = both(&field(r#"{"enum":[1.5]}"#), &field(r#"{"type":"integer","enum":[1.5]}"#), "f05");
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["counterexample"], json!({"f": 1.5}), "{v}");
}

/// d9_f06: integers are numbers. Kills D9-M2.
#[test]
fn d9_f06_integers_are_numbers() {
    let (code, text, _) = both(&field(r#"{"enum":[1,2]}"#), &field(r#"{"type":"number","enum":[1,2]}"#), "f06");
    assert_eq!(code, 0, "{text}");
}

/// d9_f07: the real campaign_attribution 1-0-0 to 1-0-1 change.
#[test]
fn d9_f07_real_campaign_attribution_holds() {
    let (code, text, _) = both(CAMPAIGN_PRE, CAMPAIGN_POST, "f07");
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("type made explicit: parameters.mapping is now string|null"), "{text}");
}

/// d9_f08: the real yauaa_context 1-0-3 to 1-0-4 change.
#[test]
fn d9_f08_real_yauaa_context_holds() {
    let (code, text, _) = both(YAUAA_PRE, YAUAA_POST, "f08");
    assert_eq!(code, 0, "{text}");
    for f in ["agentClass", "layoutEngineClass", "operatingSystemClass"] {
        assert!(text.contains(&format!("type made explicit: {f} is now string, which every value of its enum already is")), "{f}: {text}");
    }
}

/// d9_f09 guard: no enum, no type: everything was valid, now only strings.
#[test]
fn d9_f09_no_enum_keeps_the_retype_rule() {
    let (code, text, _) = both(&field(r#"{"description":"any"}"#), &field(r#"{"type":"string"}"#), "f09");
    assert_eq!(code, 1, "{text}");
}

/// d9_f10 guard: the enum rule still judges the enum change.
#[test]
fn d9_f10_enum_change_is_still_judged() {
    let (code, text, v) = both(&field(r#"{"enum":["a","b"]}"#), &field(r#"{"type":"string","enum":["a"]}"#), "f10");
    assert_eq!(code, 1, "{text}");
    let kinds: Vec<&str> = v["witnesses"].as_array().expect("witnesses").iter().filter_map(|w| w["kind"].as_str()).collect();
    assert_eq!(kinds, vec!["narrowed"], "{v}");
}
