//! JUDGE-SARIF r0 fixtures (docs/falsification/SARIF-SPEC.md), frozen with
//! the spec BEFORE implementation. One voice, every encoding: each SARIF
//! result and each GitHub annotation carries the exact sentence the text
//! format prints for that witness. An empty log never reads as clean.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - SF-M1 results dropped                           -> sf_f02
//! - SF-M2 NOT EVALUATED emitted as a successful run  -> sf_f05
//! - SF-M3 base-file line used for the post-file line -> sf_f03

use std::path::{Path, PathBuf};
use std::process::Command;

const B1_001_PRE: &str = include_str!("fixtures/judge/b1-001-pre.json");
const B1_001_POST: &str = include_str!("fixtures/judge/b1-001-post.json");
const B1_002_PRE: &str = include_str!("fixtures/judge/b1-002-pre.json");
const B1_002_POST: &str = include_str!("fixtures/judge/b1-002-post.json");

const NESTED_PRE: &str = "{\n  \"properties\": {\n    \"payload\": {\n      \"type\": \"object\",\n      \"properties\": {\n        \"user\": {\n          \"type\": \"object\",\n          \"properties\": {\n            \"zipcode\": {\"type\": \"string\"},\n            \"name\": {\"type\": \"string\"}\n          }\n        }\n      }\n    }\n  }\n}\n";
const NESTED_POST: &str = "{\n  \"properties\": {\n    \"payload\": {\n      \"type\": \"object\",\n      \"properties\": {\n        \"user\": {\n          \"type\": \"object\",\n          \"properties\": {\n            \"name\": {\"type\": \"string\"}\n          }\n        }\n      }\n    }\n  }\n}\n";

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_sf_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(d: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(args)
        .current_dir(d)
        .env("GIT_AUTHOR_NAME", "sf").env("GIT_AUTHOR_EMAIL", "sf@test")
        .env("GIT_COMMITTER_NAME", "sf").env("GIT_COMMITTER_EMAIL", "sf@test")
        .output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

fn repo(tag: &str, pre: &str, post: &str) -> PathBuf {
    let d = ws(tag);
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    std::fs::write(d.join("schemas/contract.json"), pre).unwrap();
    git(&d, &["add", "schemas/contract.json"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    std::fs::write(d.join("schemas/contract.json"), post).unwrap();
    d
}

fn judge(d: &Path, format: &str) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["judge", "--contract", "schemas/contract.json", "--base", "main", "--format", format])
        .current_dir(d)
        .env("HOME", d)
        .output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned())
}

fn sarif(d: &Path) -> (i32, serde_json::Value) {
    let (code, out) = judge(d, "sarif");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}"));
    (code, v)
}

/// 1-based line of the first `"key"` occurrence in `text`.
fn line_of(text: &str, key: &str) -> u64 {
    let needle = format!("\"{key}\"");
    let pos = text.find(&needle).unwrap_or_else(|| panic!("{key} not in fixture"));
    text[..pos].bytes().filter(|&b| b == b'\n').count() as u64 + 1
}

fn result_for<'a>(v: &'a serde_json::Value, rule: &str, text_has: &str) -> &'a serde_json::Value {
    v["runs"][0]["results"].as_array().expect("results array").iter()
        .find(|r| r["ruleId"] == rule && r["message"]["text"].as_str().unwrap_or("").contains(text_has))
        .unwrap_or_else(|| panic!("no {rule} result mentioning {text_has}: {v}"))
}

fn start_line(r: &serde_json::Value) -> u64 {
    r["locations"][0]["physicalLocation"]["region"]["startLine"].as_u64().expect("startLine")
}

/// sf_f01: a SARIF 2.1.0 log with the invar driver and the six rules.
#[test]
fn sf_f01_sarif_shape() {
    let d = repo("f01", B1_002_PRE, B1_002_POST);
    let (code, v) = sarif(&d);
    assert_eq!(code, 1);
    assert_eq!(v["version"], "2.1.0");
    assert!(v["$schema"].as_str().unwrap_or("").contains("sarif-2.1.0"), "{v}");
    let runs = v["runs"].as_array().expect("runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["tool"]["driver"]["name"], "invar");
    let ids: Vec<&str> = runs[0]["tool"]["driver"]["rules"].as_array().expect("rules")
        .iter().filter_map(|r| r["id"].as_str()).collect();
    for k in ["removed", "retyped", "narrowed", "tightened", "escalated", "constraint_changed"] {
        assert!(ids.contains(&format!("invar.judge.{k}").as_str()), "missing rule {k}: {ids:?}");
    }
}

/// sf_f02: the real b1-002 retype becomes one error result whose message
/// is exactly the text format's sentence. Kills SF-M1.
#[test]
fn sf_f02_result_message_is_the_text_sentence() {
    let d = repo("f02", B1_002_PRE, B1_002_POST);
    let (_, text) = judge(&d, "text");
    let sentence = text.lines().find(|l| l.trim_start().starts_with("retyped:")).expect("text sentence").trim().to_string();
    let (_, v) = sarif(&d);
    let results = v["runs"][0]["results"].as_array().expect("results");
    assert_eq!(results.len(), 1, "{v}");
    assert_eq!(results[0]["ruleId"], "invar.judge.retyped");
    assert_eq!(results[0]["level"], "error");
    assert_eq!(results[0]["message"]["text"], sentence.as_str());
}

/// sf_f03: b1-001's `volume` retype sits on line 62 at base and line 115
/// in the post file; the result is located at the POST line. Kills SF-M3.
#[test]
fn sf_f03_retype_located_at_post_line() {
    let d = repo("f03", B1_001_PRE, B1_001_POST);
    let (_, v) = sarif(&d);
    let r = result_for(&v, "invar.judge.retyped", "volume");
    let post = line_of(B1_001_POST, "volume");
    assert_ne!(post, line_of(B1_001_PRE, "volume"), "fixture must separate the two lines");
    assert_eq!(start_line(r), post);
}

/// sf_f04: a removed field anchors where it used to live: the post root
/// `properties` line for top-level `isLive`; the parent `user` key line
/// for the nested `payload.user.zipcode`.
#[test]
fn sf_f04_removed_field_anchors_to_parent() {
    let d = repo("f04a", B1_001_PRE, B1_001_POST);
    let (_, v) = sarif(&d);
    let r = result_for(&v, "invar.judge.removed", "isLive");
    assert_eq!(start_line(r), line_of(B1_001_POST, "properties"));

    let d = repo("f04b", NESTED_PRE, NESTED_POST);
    let (_, v) = sarif(&d);
    let r = result_for(&v, "invar.judge.removed", "payload.user.zipcode");
    assert_eq!(start_line(r), line_of(NESTED_POST, "user"));
}

/// sf_f05: an unparsable post never reads as clean: zero results, a
/// failed execution, a warning notification naming the reason, exit 4.
/// Kills SF-M2.
#[test]
fn sf_f05_not_evaluated_is_a_failed_execution() {
    let d = repo("f05", B1_002_PRE, "[[[");
    let (code, v) = sarif(&d);
    assert_eq!(code, 4);
    let run = &v["runs"][0];
    assert_eq!(run["results"].as_array().map(Vec::len), Some(0), "{v}");
    assert_eq!(run["invocations"][0]["executionSuccessful"], false, "{v}");
    let n = &run["invocations"][0]["toolExecutionNotifications"][0];
    assert_eq!(n["descriptor"]["id"], "unparsable", "{v}");
    assert_eq!(n["level"], "warning", "{v}");
    assert!(n["message"]["text"].as_str().unwrap_or("").contains("NOT EVALUATED"), "{v}");
}

/// sf_f06: byte-stable: two runs on the same inputs are identical.
#[test]
fn sf_f06_byte_identical_runs() {
    let d = repo("f06", B1_001_PRE, B1_001_POST);
    let (_, a) = judge(&d, "sarif");
    let (_, b) = judge(&d, "sarif");
    assert_eq!(a, b);
}

/// sf_f07: the stated unjudged list travels with the log.
#[test]
fn sf_f07_unjudged_constraints_in_run_properties() {
    let d = repo("f07", B1_002_PRE, B1_002_POST);
    let (_, v) = sarif(&d);
    let uj = v["runs"][0]["properties"]["unjudged_constraints"].as_array().expect("unjudged list");
    assert!(uj.iter().any(|x| x == "$ref"), "{v}");
}

/// sf_f08: GitHub annotations: one ::error line per witness with file,
/// post line, rule title and the sentence; NOT EVALUATED is one ::warning.
#[test]
fn sf_f08_github_annotations() {
    let d = repo("f08a", B1_002_PRE, B1_002_POST);
    let (code, out) = judge(&d, "github");
    assert_eq!(code, 1);
    let line = line_of(B1_002_POST, "deviceMemory");
    let want = format!("::error file=schemas/contract.json,line={line},title=invar.judge.retyped::retyped: deviceMemory");
    assert!(out.lines().any(|l| l.starts_with(&want)), "want prefix {want}\ngot:\n{out}");

    let d = repo("f08b", B1_002_PRE, "[[[");
    let (code, out) = judge(&d, "github");
    assert_eq!(code, 4);
    assert!(out.lines().any(|l| l.starts_with("::warning file=schemas/contract.json,title=invar.judge.not_evaluated::NOT EVALUATED (unparsable)")), "{out}");
}
