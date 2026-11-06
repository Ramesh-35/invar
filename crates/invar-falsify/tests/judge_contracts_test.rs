//! JUDGE-CONTRACTS r0 fixtures (docs/falsification/JUDGE-CONTRACTS-SPEC.md),
//! frozen with the spec BEFORE implementation: one pass judges every
//! contract a change touches, an unchanged contract costs nothing, a glob
//! that matches nothing never reads as a pass. D6: sentences state the
//! structural change, never a guess about who reads the contract.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - JC-M1 unchanged-file filter dropped            -> jc_f01
//! - JC-M2 aggregate takes the last verdict         -> jc_f08
//! - JC-M3 a glob matching nothing passes           -> jc_f05
//! - JC-M4 untracked files ignored                  -> jc_f07

use std::path::{Path, PathBuf};
use std::process::Command;

const B1_002_PRE: &str = include_str!("fixtures/judge/b1-002-pre.json");
const B1_002_POST: &str = include_str!("fixtures/judge/b1-002-post.json");

const OPEN: &str = r#"{"properties":{"name":{"type":"string"}}}"#;
// JUDGE-REACH r0 Amendment A: this fixture was `name` plus an added
// `"nick":{"type":"string"}` in an open object, called a widening. Under the
// strict model it is not one: {"nick":1} was valid before and is rejected
// after, and v0.2.12 passed it unseen. A true widening (the type dropped)
// keeps the test's intent: one schema widened, one narrowed, one untouched.
const OPEN_WIDER: &str = r#"{"properties":{"name":{}}}"#;
const CLOSED: &str = r#"{"additionalProperties":false,"properties":{"name":{"type":"string"},"age":{"type":"integer"}}}"#;
const CLOSED_NARROWER: &str = r#"{"additionalProperties":false,"properties":{"name":{"type":"string"}}}"#;
const STILL: &str = r#"{"properties":{"id":{"type":"integer"}}}"#;

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_jc_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(d: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(args)
        .current_dir(d)
        .env("GIT_AUTHOR_NAME", "jc").env("GIT_AUTHOR_EMAIL", "jc@test")
        .env("GIT_COMMITTER_NAME", "jc").env("GIT_COMMITTER_EMAIL", "jc@test")
        .output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

/// Base commit holds `base`; the working tree then gets `post` (None
/// deletes the file). Files are committed with `git add -A`, so untracked
/// files exist only when a test writes them after this returns.
fn repo(tag: &str, base: &[(&str, &str)], post: &[(&str, Option<&str>)]) -> PathBuf {
    let d = ws(tag);
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    for (p, c) in base {
        std::fs::write(d.join(p), c).unwrap();
    }
    git(&d, &["add", "-A"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    for (p, c) in post {
        match c {
            Some(c) => std::fs::write(d.join(p), c).unwrap(),
            None => std::fs::remove_file(d.join(p)).unwrap(),
        }
    }
    // A pull request's files are committed: stage tracked changes the way
    // a checkout of the PR head presents them, leaving nothing untracked.
    git(&d, &["add", "-A"]);
    d
}

fn judge(d: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .arg("judge")
        .args(args)
        .current_dir(d)
        .env("HOME", d)
        .output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn json(d: &Path, glob: &str) -> (i32, serde_json::Value) {
    let (code, out) = judge(d, &["--contracts", glob, "--base", "main", "--format", "json"]);
    let v = serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}"));
    (code, v)
}

fn verdicts(v: &serde_json::Value) -> Vec<(String, String)> {
    v["contracts"].as_array().expect("contracts").iter()
        .map(|c| (c["contract"].as_str().unwrap_or("").to_string(), c["verdict"].as_str().unwrap_or("").to_string()))
        .collect()
}

fn three() -> PathBuf {
    repo("three",
        &[("schemas/a.json", OPEN), ("schemas/b.json", CLOSED), ("schemas/c.json", STILL)],
        &[("schemas/a.json", Some(OPEN_WIDER)), ("schemas/b.json", Some(CLOSED_NARROWER))])
}

/// jc_f01: widened, narrowed, untouched: two judged, one stated as
/// unchanged. Kills JC-M1.
#[test]
fn jc_f01_three_schemas_only_changed_ones_judged() {
    let d = three();
    let (code, v) = json(&d, "schemas/*.json");
    assert_eq!(code, 1, "{v}");
    assert_eq!(verdicts(&v), vec![("schemas/a.json".into(), "HOLDS".into()), ("schemas/b.json".into(), "VIOLATED".into())], "{v}");
    assert_eq!(v["summary"]["unchanged"], 1, "{v}");
    assert_eq!(v["summary"]["violated"], 1, "{v}");
    assert_eq!(v["summary"]["holds"], 1, "{v}");
}

/// jc_f02: annotations land only on the narrowed file.
#[test]
fn jc_f02_github_annotations_name_their_own_file() {
    let d = three();
    let (code, out) = judge(&d, &["--contracts", "schemas/*.json", "--base", "main", "--format", "github"]);
    assert_eq!(code, 1, "{out}");
    let errors: Vec<&str> = out.lines().filter(|l| l.starts_with("::error ")).collect();
    assert_eq!(errors.len(), 1, "{out}");
    assert!(errors[0].starts_with("::error file=schemas/b.json,"), "{out}");
    assert!(!out.contains("file=schemas/a.json") && !out.contains("file=schemas/c.json"), "{out}");
}

/// jc_f03: an added schema promised nothing before.
#[test]
fn jc_f03_added_schema_is_new_contract() {
    let d = repo("f03", &[("schemas/a.json", OPEN)], &[("schemas/d.json", Some(STILL))]);
    let (code, v) = json(&d, "schemas/*.json");
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["contracts"][0]["contract"], "schemas/d.json", "{v}");
    assert_eq!(v["contracts"][0]["reason"], "new_contract", "{v}");
}

/// jc_f04: a deleted schema withdraws everything it promised: stated.
#[test]
fn jc_f04_deleted_schema_is_not_evaluated() {
    let d = repo("f04", &[("schemas/a.json", OPEN), ("schemas/e.json", STILL)], &[("schemas/e.json", None)]);
    let (code, v) = json(&d, "schemas/*.json");
    assert_eq!(code, 4, "{v}");
    assert_eq!(v["contracts"][0]["contract"], "schemas/e.json", "{v}");
    assert_eq!(v["contracts"][0]["reason"], "contract_deleted", "{v}");
}

/// jc_f05: a glob matching nothing is a misconfiguration, never a pass.
/// Kills JC-M3.
#[test]
fn jc_f05_glob_matching_nothing_is_an_error() {
    let d = three();
    let (code, out) = judge(&d, &["--contracts", "nowhere/*.json", "--base", "main"]);
    assert_eq!(code, 3, "{out}");
}

/// jc_f06: matching schemas, none changed: nothing judged, and it says so.
#[test]
fn jc_f06_nothing_changed_is_stated() {
    let d = repo("f06", &[("schemas/a.json", OPEN), ("schemas/c.json", STILL)], &[]);
    let (code, out) = judge(&d, &["--contracts", "schemas/*.json", "--base", "main"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("judged 0 changed contracts"), "{out}");
    assert!(out.contains("2 unchanged"), "{out}");
}

/// jc_f07: an untracked new schema (local use, before commit) is judged.
/// Kills JC-M4.
#[test]
fn jc_f07_untracked_schema_is_judged() {
    let d = repo("f07", &[("schemas/a.json", OPEN)], &[]);
    std::fs::write(d.join("schemas/z.json"), STILL).unwrap();
    let (code, v) = json(&d, "schemas/*.json");
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["contracts"][0]["contract"], "schemas/z.json", "{v}");
    assert_eq!(v["contracts"][0]["reason"], "new_contract", "{v}");
}

/// jc_f08: the worst verdict decides the exit; every verdict is stated.
/// Kills JC-M2 (b sorts before u, so "last verdict" would give 4).
#[test]
fn jc_f08_violation_dominates_and_both_are_stated() {
    let d = repo("f08", &[("schemas/b.json", CLOSED), ("schemas/u.json", OPEN)],
        &[("schemas/b.json", Some(CLOSED_NARROWER)), ("schemas/u.json", Some("{ not json"))]);
    let (code, v) = json(&d, "schemas/*.json");
    assert_eq!(code, 1, "{v}");
    assert_eq!(verdicts(&v), vec![("schemas/b.json".into(), "VIOLATED".into()), ("schemas/u.json".into(), "NOT EVALUATED".into())], "{v}");
}

/// jc_f09: one SARIF run, results located in their own files.
#[test]
fn jc_f09_sarif_one_run_many_files() {
    let d = repo("f09", &[("schemas/b.json", CLOSED), ("schemas/k.json", CLOSED)],
        &[("schemas/b.json", Some(CLOSED_NARROWER)), ("schemas/k.json", Some(CLOSED_NARROWER))]);
    let (code, out) = judge(&d, &["--contracts", "schemas/*.json", "--base", "main", "--format", "sarif"]);
    assert_eq!(code, 1, "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).expect("sarif json");
    assert_eq!(v["runs"].as_array().map(Vec::len), Some(1), "{v}");
    let mut uris: Vec<&str> = v["runs"][0]["results"].as_array().expect("results").iter()
        .filter_map(|r| r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"].as_str()).collect();
    uris.sort();
    assert_eq!(uris, vec!["schemas/b.json", "schemas/k.json"], "{v}");
}

/// jc_f10 D6: the sentences state the structural change, not a reader.
#[test]
fn jc_f10_voice_states_structure_not_readers() {
    let d = three();
    let (_, out) = judge(&d, &["--contracts", "schemas/*.json", "--base", "main"]);
    assert!(out.contains("  removed: age was integer (line 1 at main), from an object that admits no other keys. Instances that carry it, valid before, are now rejected; keep the field, or version the change.\n"), "{out}");
    // JUDGE-WIDEN r0 amendment (D7): b1-002 swapped, the narrowing direction.
    let d2 = repo("f10b", &[("schemas/x.json", B1_002_POST)], &[("schemas/x.json", Some(B1_002_PRE))]);
    let (_, out2) = judge(&d2, &["--contract", "schemas/x.json", "--base", "main"]);
    assert!(out2.contains("  retyped: deviceMemory was number|null, is now integer|null. Values outside integer|null, valid before, are now rejected; restore the type, or version the change.\n"), "{out2}");
    for o in [&out, &out2] {
        for word in ["Consumers", "Senders", "Payloads", "misread"] {
            assert!(!o.contains(word), "{word} in: {o}");
        }
    }
}

/// jc_f11: one contract or many, never both.
#[test]
fn jc_f11_contract_and_contracts_together_is_an_error() {
    let d = three();
    let (code, out) = judge(&d, &["--contract", "schemas/a.json", "--contracts", "schemas/*.json", "--base", "main"]);
    assert_eq!(code, 3, "{out}");
}
