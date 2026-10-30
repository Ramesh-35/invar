//! JUDGE-1 fixtures (docs/falsification/JUDGE-SPEC.md r1), frozen with the
//! spec BEFORE any implementation. `invar judge` is the declared-data-
//! contract verdict: the PRE contract is read from the base ref, the way
//! SIB-3 reads trust (never from the PR tree); the POST contract is the
//! working tree. A property removed or retyped is VIOLATED with a dotted
//! path witness; what r0 does not judge (enum, pattern, maximum, required)
//! is STATED in the receipt as unjudged_constraints, never silently passed.
//!
//! jf_f01 and jf_f02 are the REAL pinned bytes of B1 cases b1-001 and
//! b1-002 (fixtures/judge/, sha256-identical to the case manifests): the
//! kill gate financing the build gate.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - JM-M1 drop removal detection            -> jf_f01
//! - JM-M2 drop retype detection             -> jf_f02
//! - JM-M3 unparsable input treated as HOLDS -> jf_f05
//! - JM-M4 pre read from working tree, not base ref -> jf_f01 (the
//!   working tree holds ONLY post bytes, so a mutant comparing worktree
//!   to worktree sees no diff and answers HOLDS, failing the exit-1 assert)
//! - JM-M5 recursion removed, top level only -> jf_f07
//! - JM-M6 missing base contract treated as HOLDS -> jf_f06

use std::path::{Path, PathBuf};
use std::process::Command;

const B1_001_PRE: &str = include_str!("fixtures/judge/b1-001-pre.json");
const B1_001_POST: &str = include_str!("fixtures/judge/b1-001-post.json");
const B1_002_PRE: &str = include_str!("fixtures/judge/b1-002-pre.json");
const B1_002_POST: &str = include_str!("fixtures/judge/b1-002-post.json");

const NESTED_PRE: &str = r#"{"properties":{"payload":{"type":"object","properties":{"user":{"type":"object","properties":{"zipcode":{"type":"string"},"name":{"type":"string"}}}}}}}"#;
const NESTED_POST: &str = r#"{"properties":{"payload":{"type":"object","properties":{"user":{"type":"object","properties":{"name":{"type":"string"}}}}}}}"#;
const ADDED_POST: &str = r#"{"properties":{"payload":{"type":"object","properties":{"user":{"type":"object","properties":{"zipcode":{"type":"string"},"name":{"type":"string"},"email":{"type":"string"}}}}}}}"#;

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_jf_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(d: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(args)
        .current_dir(d)
        .env("GIT_AUTHOR_NAME", "jf").env("GIT_AUTHOR_EMAIL", "jf@test")
        .env("GIT_COMMITTER_NAME", "jf").env("GIT_COMMITTER_EMAIL", "jf@test")
        .output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

/// A repository whose base commit holds `pre` at schemas/contract.json and
/// whose working tree holds `post` there. With `pre` None the contract is
/// new in the working tree: nothing at base.
fn repo(tag: &str, pre: Option<&str>, post: &str) -> PathBuf {
    let d = ws(tag);
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    std::fs::write(d.join("schemas/contract.json"), pre.unwrap_or("{}")).unwrap();
    git(&d, &["add", "schemas/contract.json"]);
    if pre.is_none() {
        git(&d, &["rm", "-q", "--cached", "schemas/contract.json"]);
        std::fs::write(d.join("keep"), "x").unwrap();
        git(&d, &["add", "keep"]);
    }
    git(&d, &["commit", "-q", "-m", "base"]);
    std::fs::write(d.join("schemas/contract.json"), post).unwrap();
    d
}

fn judge(d: &Path, extra: &[&str]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["judge", "--contract", "schemas/contract.json", "--base", "main"])
        .args(extra)
        .current_dir(d)
        .env("HOME", d)
        .output().unwrap();
    (o.status.code().unwrap_or(-1),
     String::from_utf8_lossy(&o.stdout).into_owned(),
     String::from_utf8_lossy(&o.stderr).into_owned())
}

/// jf_f01: the real b1-001 bytes. media_player 2-0-0 removed
/// percentProgress and isLive that 1-0-0 promised. VIOLATED, exit 1, and
/// the text names each removed field in a sentence a human can act on.
/// Kills JM-M1 and JM-M4 (the working tree holds only post bytes).
#[test]
fn jf_f01_removal_is_violated_with_named_fields() {
    let d = repo("f01", Some(B1_001_PRE), B1_001_POST);
    let (code, out, _err) = judge(&d, &[]);
    assert_eq!(code, 1, "out: {out}");
    assert!(out.contains("VIOLATED"), "out: {out}");
    assert!(out.contains("percentProgress"), "out: {out}");
    assert!(out.contains("isLive"), "out: {out}");
    assert!(out.contains("removed"), "out: {out}");
}

/// jf_f02: the real b1-002 bytes. browser_context deviceMemory retyped
/// integer|null -> number|null. VIOLATED, exit 1, both types stated.
/// Kills JM-M2.
#[test]
fn jf_f02_retype_is_violated_with_both_types() {
    let d = repo("f02", Some(B1_002_PRE), B1_002_POST);
    let (code, out, _err) = judge(&d, &[]);
    assert_eq!(code, 1, "out: {out}");
    assert!(out.contains("deviceMemory"), "out: {out}");
    assert!(out.contains("retyped"), "out: {out}");
    assert!(out.contains("integer") && out.contains("number"), "out: {out}");
}

/// jf_f03: identical pre and post. HOLDS, exit 0.
#[test]
fn jf_f03_identical_holds() {
    let d = repo("f03", Some(B1_001_PRE), B1_001_PRE);
    let (code, out, _err) = judge(&d, &[]);
    assert_eq!(code, 0, "out: {out}");
    assert!(out.contains("HOLDS"), "out: {out}");
}

/// jf_f04: post only ADDS a property. Additions are HOLDS, exit 0.
#[test]
fn jf_f04_addition_holds() {
    let d = repo("f04", Some(NESTED_PRE), ADDED_POST);
    let (code, out, _err) = judge(&d, &[]);
    assert_eq!(code, 0, "out: {out}");
    assert!(out.contains("HOLDS"), "out: {out}");
}

/// jf_f05: post bytes are not JSON. NOT EVALUATED unparsable, exit 4,
/// with the reason stated in plain words. Kills JM-M3.
#[test]
fn jf_f05_unparsable_is_not_evaluated() {
    let d = repo("f05", Some(B1_001_PRE), "[[[");
    let (code, out, _err) = judge(&d, &[]);
    assert_eq!(code, 4, "out: {out}");
    assert!(out.contains("NOT EVALUATED"), "out: {out}");
    assert!(out.contains("unparsable"), "out: {out}");
}

/// jf_f06: no contract at base: a new contract is an unchecked baseline,
/// not a verified promise. NOT EVALUATED new_contract, exit 4, and the
/// text says plainly that nothing was promised before. Kills JM-M6.
#[test]
fn jf_f06_new_contract_is_not_evaluated() {
    let d = repo("f06", None, NESTED_PRE);
    let (code, out, _err) = judge(&d, &[]);
    assert_eq!(code, 4, "out: {out}");
    assert!(out.contains("NOT EVALUATED"), "out: {out}");
    assert!(out.contains("new_contract"), "out: {out}");
    assert!(out.contains("nothing was promised before"), "out: {out}");
}

/// jf_f07: a NESTED removal only (payload.user.zipcode). The witness
/// carries the dotted path, so depth can never hide a break. Kills JM-M5.
#[test]
fn jf_f07_nested_removal_has_dotted_path() {
    let d = repo("f07", Some(NESTED_PRE), NESTED_POST);
    let (code, out, _err) = judge(&d, &[]);
    assert_eq!(code, 1, "out: {out}");
    assert!(out.contains("payload.user.zipcode"), "out: {out}");
}

/// The JSON format is the machine surface: witnesses with path, kind,
/// pre_type, post_type, pre_line; and the receipt's honesty block,
/// unjudged_constraints, naming what r0 does not judge.
#[test]
fn json_format_carries_witnesses_and_unjudged_constraints() {
    let d = repo("json", Some(B1_002_PRE), B1_002_POST);
    let (code, out, _err) = judge(&d, &["--format", "json"]);
    assert_eq!(code, 1, "out: {out}");
    let v: serde_json::Value = serde_json::from_str(&out).expect("valid json");
    assert_eq!(v["verdict"], "VIOLATED");
    let w = &v["witnesses"][0];
    assert_eq!(w["path"], "deviceMemory");
    assert_eq!(w["kind"], "retyped");
    assert!(w["pre_type"].as_str().unwrap().contains("integer"));
    assert!(w["post_type"].as_str().unwrap().contains("number"));
    assert!(w["pre_line"].as_u64().unwrap() > 0);
    // JUDGE-SPEC r2 (2026-10-30): r1 of JUDGE-SUBTYPE judges enum, bounds,
    // required and pattern/format, so they leave the list; the r1 list is
    // asserted exactly in judge_subtype_test.rs js_f08. Here: the list
    // exists, is non-empty, and states the r0 gap $ref.
    let uj = v["unjudged_constraints"].as_array().expect("unjudged_constraints[]");
    assert!(!uj.is_empty(), "the unjudged list is stated, never empty: {uj:?}");
    assert!(uj.iter().any(|x| x == "$ref"), "missing $ref in {uj:?}");
}
