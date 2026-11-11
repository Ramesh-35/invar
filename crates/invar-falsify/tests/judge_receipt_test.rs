//! JUDGE-RECEIPT r0 fixtures (docs/falsification/JUDGE-RECEIPT-SPEC.md),
//! frozen with the spec BEFORE implementation. A judge receipt names the
//! exact bytes it judged (base commit, both contract hashes) and
//! `verify-receipt` reproduces it from those bytes, so a forged verdict
//! fails even when its footer is recomputed.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - RM-1 re-derived body not compared          -> jr_f03
//! - RM-2 base read from base_ref, not commit   -> jr_f05
//! - RM-3 post contract hash not checked        -> jr_f06
//! - RM-4 base contract hash not checked        -> jr_f07

use std::path::{Path, PathBuf};
use std::process::Command;

// JUDGE-WIDEN r0 Amendment A (D7): b1-002 widens, so a receipt needs the
// narrowing direction: the same real bytes, pre and post swapped.
const PRE: &str = include_str!("fixtures/judge/b1-002-post.json");
const POST: &str = include_str!("fixtures/judge/b1-002-pre.json");
const C: &str = "schemas/contract.json";

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_jr_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(d: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .args(args)
        .current_dir(d)
        .env("GIT_AUTHOR_NAME", "jr").env("GIT_AUTHOR_EMAIL", "jr@test")
        .env("GIT_COMMITTER_NAME", "jr").env("GIT_COMMITTER_EMAIL", "jr@test")
        .output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

/// `main` holds PRE; branch `pr` is checked out with POST in the working tree.
fn repo(tag: &str) -> PathBuf {
    let d = ws(tag);
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    std::fs::write(d.join(C), PRE).unwrap();
    git(&d, &["add", C]);
    git(&d, &["commit", "-q", "-m", "base"]);
    git(&d, &["checkout", "-q", "-b", "pr"]);
    std::fs::write(d.join(C), POST).unwrap();
    d
}

fn invar(d: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(args)
        .current_dir(d)
        .env("HOME", d)
        .output().unwrap();
    let mut out = String::from_utf8_lossy(&o.stdout).into_owned();
    out.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), out)
}

fn make_receipt(d: &Path) -> String {
    let (code, out) = invar(d, &["judge", "--contract", C, "--base", "main", "--receipt", "r.json"]);
    assert_eq!(code, 1, "{out}");
    std::fs::read_to_string(d.join("r.json")).unwrap()
}

fn verify(d: &Path, extra: &[&str]) -> (i32, String) {
    let mut args = vec!["verify-receipt", "r.json"];
    args.extend_from_slice(extra);
    invar(d, &args)
}

fn body_of(receipt: &str) -> serde_json::Value {
    let idx = receipt.rfind("\n{\"receipt_blake3\":\"").expect("footer");
    serde_json::from_str(&receipt[..idx]).expect("body json")
}

/// Rewrite the body and recompute the footer, as a forger would.
fn reseal(d: &Path, body: &serde_json::Value) {
    let b = serde_json::to_string_pretty(body).unwrap();
    let h = blake3::hash(b.as_bytes()).to_hex().to_string();
    std::fs::write(d.join("r.json"), format!("{b}\n{{\"receipt_blake3\":\"{h}\"}}\n")).unwrap();
}

/// jr_f01: the receipt names what was judged.
#[test]
fn jr_f01_receipt_binds_inputs() {
    let d = repo("f01");
    let r = make_receipt(&d);
    let b = body_of(&r);
    assert_eq!(b["receipt_kind"], "judge", "{b}");
    assert_eq!(b["receipt_format"], 1, "{b}");
    assert_eq!(b["inputs"]["base_commit"], git(&d, &["rev-parse", "main"]).as_str(), "{b}");
    assert_eq!(b["inputs"]["base_ref"], "main");
    assert_eq!(b["inputs"]["contract"], C);
    assert_eq!(b["inputs"]["base_contract_blake3"], blake3::hash(PRE.as_bytes()).to_hex().as_str());
    assert_eq!(b["inputs"]["post_contract_blake3"], blake3::hash(POST.as_bytes()).to_hex().as_str());
    let (_, json) = invar(&d, &["judge", "--contract", C, "--base", "main", "--format", "json"]);
    let surface: serde_json::Value = serde_json::from_str(&json).expect("json surface");
    assert_eq!(b["result"], surface, "result must be the --format json document");
}

/// jr_f02: a fresh receipt reproduces.
#[test]
fn jr_f02_fresh_receipt_reproduces() {
    let d = repo("f02");
    make_receipt(&d);
    let (code, out) = verify(&d, &[]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("reproduced"), "{out}");
}

/// jr_f03 FM-FORGERY-1: the audit's forgery. The verdict is edited and the
/// footer recomputed; reproduction exposes it. Kills RM-1.
#[test]
fn jr_f03_forged_verdict_with_recomputed_footer_fails() {
    let d = repo("f03");
    let mut b = body_of(&make_receipt(&d));
    b["result"]["verdict"] = serde_json::json!("HOLDS");
    b["result"]["witnesses"] = serde_json::json!([]);
    reseal(&d, &b);
    let (code, out) = verify(&d, &[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("NOT reproduced"), "{out}");
}

/// jr_f04: signed with the existing namespace; a forgery under the original
/// signature fails the signature check.
#[test]
fn jr_f04_signature_verifies_and_rejects_forgery() {
    let d = repo("f04");
    make_receipt(&d);
    let key = d.join("k");
    assert!(Command::new("ssh-keygen").args(["-t", "ed25519", "-N", "", "-q", "-f"]).arg(&key).status().unwrap().success());
    let pubkey = std::fs::read_to_string(d.join("k.pub")).unwrap();
    std::fs::write(d.join("allowed_signers"), format!("tester {}", pubkey.trim())).unwrap();
    let sign = Command::new("ssh-keygen")
        .args(["-Y", "sign", "-n", "invar-receipt", "-f"]).arg(&key).arg("r.json")
        .current_dir(&d).output().unwrap();
    assert!(sign.status.success(), "{}", String::from_utf8_lossy(&sign.stderr));
    let signed = ["--signers", "allowed_signers", "--principal", "tester"];
    let (code, out) = verify(&d, &signed);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("signature ok"), "{out}");

    let mut b = body_of(&std::fs::read_to_string(d.join("r.json")).unwrap());
    b["result"]["verdict"] = serde_json::json!("HOLDS");
    reseal(&d, &b);
    let (code, out) = verify(&d, &signed);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("NOT verified"), "{out}");
}

/// jr_f05: the base branch moves after the receipt; verification reads the
/// recorded commit, not the ref. Kills RM-2.
#[test]
fn jr_f05_base_is_pinned_to_the_commit_not_the_ref() {
    let d = repo("f05");
    make_receipt(&d);
    let wt = d.join("wt");
    git(&d, &["worktree", "add", "-q", wt.to_str().unwrap(), "main"]);
    std::fs::write(wt.join(C), "{\"properties\": {}}\n").unwrap();
    git(&wt, &["commit", "-q", "-am", "main moves"]);
    let (code, out) = verify(&d, &[]);
    assert_eq!(code, 0, "base must be read from base_commit: {out}");
}

/// jr_f06: the judged contract is not the one in the tree. Kills RM-3.
#[test]
fn jr_f06_post_substitution_is_input_drift() {
    let d = repo("f06");
    make_receipt(&d);
    std::fs::write(d.join(C), PRE).unwrap();
    let (code, out) = verify(&d, &[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("input drift"), "{out}");
}

/// jr_f07: the receipt claims a different base contract. Kills RM-4.
#[test]
fn jr_f07_forged_base_hash_is_input_drift() {
    let d = repo("f07");
    let mut b = body_of(&make_receipt(&d));
    b["inputs"]["base_contract_blake3"] = serde_json::json!(blake3::hash(b"forged").to_hex().as_str());
    reseal(&d, &b);
    let (code, out) = verify(&d, &[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("input drift"), "{out}");
}

/// jr_f08: a base commit this repository does not have is an error, never
/// a verdict.
#[test]
fn jr_f08_missing_base_commit_is_an_error() {
    let d = repo("f08");
    let mut b = body_of(&make_receipt(&d));
    b["inputs"]["base_commit"] = serde_json::json!("0123456789abcdef0123456789abcdef01234567");
    reseal(&d, &b);
    let (code, out) = verify(&d, &[]);
    assert_eq!(code, 3, "{out}");
}

/// jr_f09: a v0.2.x judge receipt (no receipt_format) stays an error.
#[test]
fn jr_f09_legacy_receipt_is_an_error() {
    let d = repo("f09");
    let b = body_of(&make_receipt(&d));
    reseal(&d, &b["result"]);
    let (code, out) = verify(&d, &[]);
    assert_eq!(code, 3, "{out}");
}

/// jr_f10: byte-deterministic receipts.
#[test]
fn jr_f10_receipts_are_byte_identical() {
    let d = repo("f10");
    let a = make_receipt(&d);
    let b = make_receipt(&d);
    assert_eq!(a, b);
}

// JUDGE-RECEIPT r1 / A3-RECEIPT (docs/falsification/JUDGE-RECEIPT-R1-SPEC.md):
// one receipt for a `--contracts` run, every judged contract named by its
// bytes, the head commit named when the judged bytes are committed. Frozen
// before implementation.
//
// - RM1-1 head_commit written as HEAD even when a contract is dirty -> jr1_f04
// - RM1-2 one contract's post hash not checked on verify            -> jr1_f03
// - RM1-3 re-derived body not compared for change receipts          -> jr1_f06

const C2: &str = "schemas/second.json";
const SECOND_PRE: &str = r#"{"properties":{"name":{"type":"string"}}}"#;
const SECOND_POST: &str = r#"{"properties":{"name":{"type":"string","minLength":1}}}"#;

/// `main` holds PRE and SECOND_PRE; branch `pr` has POST and SECOND_POST in
/// the working tree, committed when `commit` is true.
fn repo2(tag: &str, commit: bool) -> PathBuf {
    let d = ws(tag);
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    std::fs::write(d.join(C), PRE).unwrap();
    std::fs::write(d.join(C2), SECOND_PRE).unwrap();
    git(&d, &["add", "schemas"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    git(&d, &["checkout", "-q", "-b", "pr"]);
    std::fs::write(d.join(C), POST).unwrap();
    std::fs::write(d.join(C2), SECOND_POST).unwrap();
    if commit {
        git(&d, &["add", "schemas"]);
        git(&d, &["commit", "-q", "-m", "change"]);
    }
    d
}

fn make_change_receipt(d: &Path) -> String {
    let (code, out) = invar(d, &["judge", "--contracts", "schemas/*.json", "--base", "main", "--receipt", "r.json"]);
    assert_eq!(code, 1, "{out}");
    std::fs::read_to_string(d.join("r.json")).unwrap()
}

/// jr1_f01: a `--contracts` receipt names every judged contract by its bytes.
#[test]
fn jr1_f01_change_receipt_names_every_judged_contract() {
    let d = repo2("r1f01", true);
    let b = body_of(&make_change_receipt(&d));
    assert_eq!(b["receipt_kind"], "judge-change", "{b}");
    assert_eq!(b["receipt_format"], 1, "{b}");
    assert_eq!(b["inputs"]["base_commit"], git(&d, &["rev-parse", "main"]).as_str(), "{b}");
    assert_eq!(b["inputs"]["base_ref"], "main", "{b}");
    let cs = b["inputs"]["contracts"].as_array().expect("contracts list");
    assert_eq!(cs.len(), 2, "{b}");
    assert_eq!(cs[0]["contract"], C, "{b}");
    assert_eq!(cs[0]["base_contract_blake3"], blake3::hash(PRE.as_bytes()).to_hex().as_str(), "{b}");
    assert_eq!(cs[0]["post_contract_blake3"], blake3::hash(POST.as_bytes()).to_hex().as_str(), "{b}");
    assert_eq!(cs[1]["contract"], C2, "{b}");
    assert_eq!(cs[1]["base_contract_blake3"], blake3::hash(SECOND_PRE.as_bytes()).to_hex().as_str(), "{b}");
    assert_eq!(cs[1]["post_contract_blake3"], blake3::hash(SECOND_POST.as_bytes()).to_hex().as_str(), "{b}");
    let (_, json) = invar(&d, &["judge", "--contracts", "schemas/*.json", "--base", "main", "--format", "json"]);
    let surface: serde_json::Value = serde_json::from_str(&json).expect("json surface");
    assert_eq!(b["result"], surface, "result must be the --contracts --format json document");
}

/// jr1_f02: a fresh change receipt reproduces.
#[test]
fn jr1_f02_fresh_change_receipt_reproduces() {
    let d = repo2("r1f02", true);
    make_change_receipt(&d);
    let (code, out) = verify(&d, &[]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("reproduced"), "{out}");
}

/// jr1_f03: editing one listed contract after writing is drift naming it. Kills RM1-2.
#[test]
fn jr1_f03_editing_one_contract_is_drift_naming_it() {
    let d = repo2("r1f03", true);
    make_change_receipt(&d);
    std::fs::write(d.join(C2), SECOND_PRE).unwrap();
    let (code, out) = verify(&d, &[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("drift") && out.contains(C2), "{out}");
}

/// jr1_f04: head_commit is HEAD when the judged bytes are committed, null
/// when one judged contract is dirty. Kills RM1-1.
#[test]
fn jr1_f04_head_commit_only_when_the_judged_bytes_are_committed() {
    let d = repo2("r1f04c", true);
    let b = body_of(&make_change_receipt(&d));
    assert_eq!(b["inputs"]["head_commit"], git(&d, &["rev-parse", "HEAD"]).as_str(), "{b}");
    let d = repo2("r1f04d", false);
    let b = body_of(&make_change_receipt(&d));
    assert_eq!(b["inputs"]["head_commit"], serde_json::Value::Null, "{b}");
}

/// jr1_f05 guard: a single-contract receipt is r0's, byte for byte in shape.
#[test]
fn jr1_f05_single_contract_receipt_is_unchanged() {
    let d = repo("r1f05");
    let b = body_of(&make_receipt(&d));
    assert_eq!(b["receipt_kind"], "judge", "{b}");
    assert!(b["inputs"].get("head_commit").is_none(), "{b}");
    assert!(b["inputs"].get("contracts").is_none(), "{b}");
    assert_eq!(b["inputs"]["contract"], C, "{b}");
}

/// jr1_f06: a forged verdict in a change receipt with a recomputed footer fails. Kills RM1-3.
#[test]
fn jr1_f06_forged_change_verdict_with_recomputed_footer_fails() {
    let d = repo2("r1f06", true);
    let mut b = body_of(&make_change_receipt(&d));
    b["result"]["summary"]["violated"] = serde_json::json!(0);
    for r in b["result"]["results"].as_array_mut().into_iter().flatten() {
        r["verdict"] = serde_json::json!("HOLDS");
        r["witnesses"] = serde_json::json!([]);
    }
    reseal(&d, &b);
    let (code, out) = verify(&d, &[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("NOT reproduced"), "{out}");
}
