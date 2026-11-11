//! A3-ACCEPT fixtures (docs/falsification/ACCEPT-SPEC.md), frozen with the
//! spec BEFORE implementation. An acceptance is a person's signed record
//! that a named receipt's verdict was seen and accepted: per receipt
//! hash, complete or nothing, never for HOLDS, prepared by anyone but
//! signed only by a key the repository lists.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - AC-M1 the verdict not compared to `accepts`            -> ac_f02
//! - AC-M2 witness completeness not checked                 -> ac_f03
//! - AC-M3 the receipt hash not compared                    -> ac_f04
//! - AC-M4 the signature result ignored                     -> ac_f05

use std::path::{Path, PathBuf};
use std::process::Command;
use serde_json::{json, Value};

const PRE: &str = r#"{"type":"object","properties":{"a":{"enum":["x","y"]},"b":{"type":"string"}}}"#;
const POST: &str = r#"{"type":"object","properties":{"a":{"enum":["x"]},"b":{"type":"string"}}}"#;

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_ac_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(d: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(args)
        .current_dir(d)
        .env("GIT_AUTHOR_NAME", "ac").env("GIT_AUTHOR_EMAIL", "ac@test")
        .env("GIT_COMMITTER_NAME", "ac").env("GIT_COMMITTER_EMAIL", "ac@test")
        .output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

/// A repo whose main holds PRE and whose working tree holds POST.
fn repo(tag: &str) -> PathBuf {
    let d = ws(tag);
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    std::fs::write(d.join("schemas/contract.json"), PRE).unwrap();
    git(&d, &["add", "schemas/contract.json"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    std::fs::write(d.join("schemas/contract.json"), POST).unwrap();
    d
}

fn invar(d: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(args).current_dir(d).env("HOME", d)
        .output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

/// An ed25519 key, its pubkey listed for `principal` in an allowed-signers file.
fn keypair(d: &Path, principal: &str) -> (PathBuf, PathBuf) {
    let key = d.join("id_ed25519");
    assert!(Command::new("ssh-keygen").args(["-t", "ed25519", "-N", "", "-q", "-f"]).arg(&key).status().unwrap().success());
    let pubkey = std::fs::read_to_string(key.with_extension("pub")).unwrap();
    let signers = d.join("signers");
    std::fs::write(&signers, format!("{principal} {pubkey}")).unwrap();
    (key, signers)
}

fn sign(key: &Path, file: &Path) {
    // Amendment A: ssh-keygen -Y sign refuses to overwrite an existing
    // .sig yet exits 0, silently keeping the stale signature; remove it
    // first so re-signing in a fixture signs the current bytes.
    let _ = std::fs::remove_file(format!("{}.sig", file.display()));
    let o = Command::new("ssh-keygen").args(["-Y", "sign", "-n", "invar-acceptance", "-f"]).arg(key).arg(file).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}

/// Write a receipt for the pair, prepare the acceptance, fill it in, sign
/// it, and return (receipt, acceptance) paths.
fn accepted(d: &Path, key: &Path) -> (String, String) {
    let (code, out) = invar(d, &["judge", "--contract", "schemas/contract.json", "--base", "main", "--receipt", "r.json"]);
    assert_eq!(code, 1, "{out}");
    let (code, out) = invar(d, &["accept", "r.json", "--prepare", "--out", "a.json"]);
    assert_eq!(code, 0, "{out}");
    let mut a: Value = serde_json::from_str(&std::fs::read_to_string(d.join("a.json")).unwrap()).unwrap();
    a["reason"] = json!("intentional narrowing; see PR #1");
    a["principal"] = json!("nikhil@test");
    a["signed_on"] = json!("2026-11-10");
    std::fs::write(d.join("a.json"), serde_json::to_string_pretty(&a).unwrap()).unwrap();
    sign(key, &d.join("a.json"));
    ("r.json".into(), "a.json".into())
}

/// ac_f01: the full round trip stands: prepare carries the receipt hash,
/// the verdict and every witness path; signed by a listed key, the check
/// exits 0 and names the principal.
#[test]
fn ac_f01_signed_complete_acceptance_stands() {
    let d = repo("f01");
    let (key, signers) = keypair(&d, "nikhil@test");
    let (r, a) = accepted(&d, &key);
    let (code, out) = invar(&d, &["accept", &r, "--acceptance", &a, "--signers", signers.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("standing"), "{out}");
    assert!(out.contains("nikhil@test"), "{out}");
    let a: Value = serde_json::from_str(&std::fs::read_to_string(d.join("a.json")).unwrap()).unwrap();
    assert_eq!(a["accepts"], json!(["VIOLATED"]), "{a}");
    assert_eq!(a["witness_paths"], json!(["a"]), "{a}");
    assert_eq!(a["acceptance_format"], json!(1), "{a}");
}

/// ac_f02: an acceptance whose `accepts` does not name the receipt's
/// verdict accepts nothing, and a HOLDS receipt has nothing to accept
/// (prepare refuses it). Kills AC-M1.
#[test]
fn ac_f02_verdict_mismatch_and_holds_are_refused() {
    let d = repo("f02");
    let (key, signers) = keypair(&d, "nikhil@test");
    let (r, a) = accepted(&d, &key);
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(d.join(&a)).unwrap()).unwrap();
    v["accepts"] = json!(["HOLDS"]);
    std::fs::write(d.join(&a), serde_json::to_string_pretty(&v).unwrap()).unwrap();
    sign(&key, &d.join(&a));
    let (code, out) = invar(&d, &["accept", &r, "--acceptance", &a, "--signers", signers.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("verdict"), "{out}");
    // HOLDS receipt: nothing to accept.
    std::fs::write(d.join("schemas/contract.json"), PRE).unwrap();
    let (_, out) = invar(&d, &["judge", "--contract", "schemas/contract.json", "--base", "main", "--receipt", "rh.json"]);
    assert!(out.contains("HOLDS"), "{out}");
    let (code, out) = invar(&d, &["accept", "rh.json", "--prepare", "--out", "ah.json"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("nothing to accept"), "{out}");
}

/// ac_f03: witness_paths must list every witness in the receipt; an
/// acceptance that omits one accepts nothing. Kills AC-M2.
#[test]
fn ac_f03_incomplete_witness_list_accepts_nothing() {
    let d = repo("f03");
    let (key, signers) = keypair(&d, "nikhil@test");
    let (r, a) = accepted(&d, &key);
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(d.join(&a)).unwrap()).unwrap();
    v["witness_paths"] = json!([]);
    std::fs::write(d.join(&a), serde_json::to_string_pretty(&v).unwrap()).unwrap();
    sign(&key, &d.join(&a));
    let (code, out) = invar(&d, &["accept", &r, "--acceptance", &a, "--signers", signers.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("every witness"), "{out}");
}

/// ac_f04: a new receipt for new bytes needs a new acceptance; the old one
/// no longer names the receipt's hash and accepts nothing. Kills AC-M3.
#[test]
fn ac_f04_new_receipt_needs_a_new_acceptance() {
    let d = repo("f04");
    let (key, signers) = keypair(&d, "nikhil@test");
    let (r, a) = accepted(&d, &key);
    // Amendment A: as frozen this removed a property from an open object,
    // which D2 states as a loosening (HOLDS), so no second receipt could
    // exist; a tightened bound restores the fixture's purpose.
    let further = r#"{"type":"object","properties":{"a":{"enum":["x"]},"b":{"type":"string","minLength":1}}}"#;
    std::fs::write(d.join("schemas/contract.json"), further).unwrap();
    let (_, out) = invar(&d, &["judge", "--contract", "schemas/contract.json", "--base", "main", "--receipt", &r]);
    assert!(out.contains("VIOLATED"), "{out}");
    let (code, out) = invar(&d, &["accept", &r, "--acceptance", &a, "--signers", signers.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("hash"), "{out}");
}

/// ac_f05: a signature by an unlisted key, or an acceptance edited after
/// signing, is refused; the agent may prepare, only a listed person
/// signs. Kills AC-M4.
#[test]
fn ac_f05_unlisted_key_or_tampered_acceptance_is_refused() {
    let d = repo("f05");
    let (key, _) = keypair(&d, "nikhil@test");
    let (r, a) = accepted(&d, &key);
    // signers file listing a DIFFERENT key for the same principal
    let other = ws("f05other");
    let (_, other_signers) = keypair(&other, "nikhil@test");
    let (code, out) = invar(&d, &["accept", &r, "--acceptance", &a, "--signers", other_signers.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("signature"), "{out}");
    // edited after signing
    let (_, signers) = keypair(&ws("f05b"), "nikhil@test");
    let _ = signers;
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(d.join(&a)).unwrap()).unwrap();
    v["reason"] = json!("edited after signing");
    std::fs::write(d.join(&a), serde_json::to_string_pretty(&v).unwrap()).unwrap();
    let (key_signers_dir, correct_signers) = (ws("f05c"), d.join("signers"));
    let _ = key_signers_dir;
    let (code, out) = invar(&d, &["accept", &r, "--acceptance", &a, "--signers", correct_signers.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("signature"), "{out}");
}

/// ac_f06: a change receipt (judge-change) is accepted as one unit, its
/// witness paths listed per contract.
#[test]
fn ac_f06_change_receipt_is_accepted_as_one_unit() {
    let d = ws("f06");
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    std::fs::write(d.join("schemas/one.json"), PRE).unwrap();
    std::fs::write(d.join("schemas/two.json"), PRE).unwrap();
    git(&d, &["add", "schemas"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    std::fs::write(d.join("schemas/one.json"), POST).unwrap();
    std::fs::write(d.join("schemas/two.json"), POST).unwrap();
    let (key, signers) = keypair(&d, "nikhil@test");
    let (code, out) = invar(&d, &["judge", "--contracts", "schemas/*.json", "--base", "main", "--receipt", "r.json"]);
    assert_eq!(code, 1, "{out}");
    let (code, out) = invar(&d, &["accept", "r.json", "--prepare", "--out", "a.json"]);
    assert_eq!(code, 0, "{out}");
    let mut a: Value = serde_json::from_str(&std::fs::read_to_string(d.join("a.json")).unwrap()).unwrap();
    assert_eq!(a["witness_paths"], json!(["schemas/one.json: a", "schemas/two.json: a"]), "{a}");
    a["reason"] = json!("both intentional");
    a["principal"] = json!("nikhil@test");
    a["signed_on"] = json!("2026-11-10");
    std::fs::write(d.join("a.json"), serde_json::to_string_pretty(&a).unwrap()).unwrap();
    sign(&key, &d.join("a.json"));
    let (code, out) = invar(&d, &["accept", "r.json", "--acceptance", "a.json", "--signers", signers.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("standing"), "{out}");
}

/// ac_f07: a NOT EVALUATED receipt is acceptable, and its stated paths
/// (what was NOT judged) must be listed the same way: omitting one
/// accepts nothing.
#[test]
fn ac_f07_not_evaluated_acceptance_lists_the_stated_paths() {
    let d = ws("f07");
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    std::fs::write(d.join("schemas/contract.json"), r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{3}$"}}}"#).unwrap();
    git(&d, &["add", "schemas/contract.json"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    std::fs::write(d.join("schemas/contract.json"), r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{2}$"}}}"#).unwrap();
    let (key, signers) = keypair(&d, "nikhil@test");
    let (code, out) = invar(&d, &["judge", "--contract", "schemas/contract.json", "--base", "main", "--receipt", "r.json"]);
    assert_eq!(code, 4, "{out}");
    let (code, out) = invar(&d, &["accept", "r.json", "--prepare", "--out", "a.json"]);
    assert_eq!(code, 0, "{out}");
    let mut a: Value = serde_json::from_str(&std::fs::read_to_string(d.join("a.json")).unwrap()).unwrap();
    assert_eq!(a["accepts"], json!(["NOT EVALUATED"]), "{a}");
    assert_eq!(a["witness_paths"], json!(["not evaluated: code"]), "{a}");
    a["reason"] = json!("undecided pattern change reviewed by hand; accepted");
    a["principal"] = json!("nikhil@test");
    a["signed_on"] = json!("2026-11-10");
    std::fs::write(d.join("a.json"), serde_json::to_string_pretty(&a).unwrap()).unwrap();
    sign(&key, &d.join("a.json"));
    let (code, out) = invar(&d, &["accept", "r.json", "--acceptance", "a.json", "--signers", signers.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(d.join("a.json")).unwrap()).unwrap();
    v["witness_paths"] = json!([]);
    std::fs::write(d.join("a.json"), serde_json::to_string_pretty(&v).unwrap()).unwrap();
    sign(&key, &d.join("a.json"));
    let (code, out2) = invar(&d, &["accept", "r.json", "--acceptance", "a.json", "--signers", signers.to_str().unwrap()]);
    assert_eq!(code, 1, "{out2}");
    assert!(!out.is_empty());
}
