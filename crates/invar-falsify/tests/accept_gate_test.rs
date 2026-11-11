//! A3-GATE fixtures (docs/falsification/ACCEPT-GATE-SPEC.md), frozen with the spec
//! BEFORE implementation. `invar judge --contracts ... --acceptances DIR
//! --signers FILE` is the one CI command: exit 0 only for HOLDS or for a
//! change whose exact judged bytes a listed person signed for; otherwise
//! the judge's own exit code, with the way to accept stated.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - AG-M1 the judged identity not compared (any acceptance passes) -> ag_f03
//! - AG-M2 the signature not checked in the gate                    -> ag_f05
//! - AG-M3 the gate passes with no acceptance at all                -> ag_f01

use std::path::{Path, PathBuf};
use std::process::Command;
use serde_json::{json, Value};

const PRE: &str = r#"{"type":"object","properties":{"a":{"enum":["x","y"]},"b":{"type":"string"}}}"#;
const POST: &str = r#"{"type":"object","properties":{"a":{"enum":["x"]},"b":{"type":"string"}}}"#;
const FURTHER: &str = r#"{"type":"object","properties":{"a":{"enum":["x"]},"b":{"type":"string","minLength":1}}}"#;
const LOOSER: &str = r#"{"type":"object","properties":{"a":{"enum":["x","y","z"]},"b":{"type":"string"}}}"#;

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_ag_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(d: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(args)
        .current_dir(d)
        .env("GIT_AUTHOR_NAME", "ag").env("GIT_AUTHOR_EMAIL", "ag@test")
        .env("GIT_COMMITTER_NAME", "ag").env("GIT_COMMITTER_EMAIL", "ag@test")
        .output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

/// main holds `pre`; branch pr has `post` committed.
fn repo(tag: &str, pre: &str, post: &str) -> PathBuf {
    let d = ws(tag);
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    std::fs::write(d.join("schemas/contract.json"), pre).unwrap();
    git(&d, &["add", "schemas"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    git(&d, &["checkout", "-q", "-b", "pr"]);
    std::fs::write(d.join("schemas/contract.json"), post).unwrap();
    git(&d, &["add", "schemas"]);
    git(&d, &["commit", "-q", "-m", "change"]);
    d
}

fn invar(d: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(d).env("HOME", d).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn keypair(d: &Path, principal: &str) -> (PathBuf, PathBuf) {
    let key = d.join("id_ed25519");
    assert!(Command::new("ssh-keygen").args(["-t", "ed25519", "-N", "", "-q", "-f"]).arg(&key).status().unwrap().success());
    let pubkey = std::fs::read_to_string(key.with_extension("pub")).unwrap();
    let signers = d.join("signers");
    std::fs::write(&signers, format!("{principal} {pubkey}")).unwrap();
    (key, signers)
}

fn gate(d: &Path, signers: &Path) -> (i32, String) {
    invar(d, &["judge", "--contracts", "schemas/*.json", "--base", "main", "--acceptances", ".invar/acceptances", "--signers", signers.to_str().unwrap()])
}

/// Receipt + prepared, completed, signed acceptance, written under
/// .invar/acceptances/<name>.* and COMMITTED, the way a PR carries them.
fn accept_and_commit(d: &Path, key: &Path, name: &str) {
    let dir = d.join(".invar/acceptances");
    std::fs::create_dir_all(&dir).unwrap();
    let r = format!(".invar/acceptances/{name}.receipt.json");
    let a = format!(".invar/acceptances/{name}.acceptance.json");
    let _ = invar(d, &["judge", "--contracts", "schemas/*.json", "--base", "main", "--receipt", &r]);
    let (code, out) = invar(d, &["accept", &r, "--prepare", "--out", &a]);
    assert_eq!(code, 0, "{out}");
    let mut v: Value = serde_json::from_str(&std::fs::read_to_string(d.join(&a)).unwrap()).unwrap();
    v["reason"] = json!("intentional; consumers migrated");
    v["principal"] = json!("nikhil@test");
    v["signed_on"] = json!("2026-11-11");
    std::fs::write(d.join(&a), serde_json::to_string_pretty(&v).unwrap()).unwrap();
    let _ = std::fs::remove_file(d.join(format!("{a}.sig")));
    let o = Command::new("ssh-keygen").args(["-Y", "sign", "-n", "invar-acceptance", "-f"]).arg(key).arg(d.join(&a)).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    git(d, &["add", ".invar"]);
    git(d, &["commit", "-q", "-m", "accept the break"]);
}

/// ag_f01: a break nobody signed for blocks: the judge's own exit code, and
/// the gate says how to accept. Kills AG-M3.
#[test]
fn ag_f01_unaccepted_break_blocks_and_says_how_to_accept() {
    let d = repo("f01", PRE, POST);
    let (_, signers) = keypair(&d, "nikhil@test");
    let (code, out) = gate(&d, &signers);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("VIOLATED"), "{out}");
    assert!(out.contains("gate: no standing acceptance"), "{out}");
    assert!(out.contains("invar accept"), "{out}");
}

/// ag_f02: a signed acceptance committed on the PR branch opens the gate,
/// and committing it (which moves HEAD) does not void it: the gate matches
/// on the judged bytes and result, not on commit hashes.
#[test]
fn ag_f02_committed_signed_acceptance_opens_the_gate() {
    let d = repo("f02", PRE, POST);
    let (key, signers) = keypair(&d, "nikhil@test");
    accept_and_commit(&d, &key, "pr-1");
    let (code, out) = gate(&d, &signers);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("VIOLATED"), "the verdict is still stated: {out}");
    assert!(out.contains("gate: accepted by nikhil@test"), "{out}");
    assert!(out.contains("intentional; consumers migrated"), "{out}");
}

/// ag_f03: new bytes after the acceptance need a new acceptance; the old
/// one accepts nothing. Kills AG-M1.
#[test]
fn ag_f03_further_change_voids_the_acceptance() {
    let d = repo("f03", PRE, POST);
    let (key, signers) = keypair(&d, "nikhil@test");
    accept_and_commit(&d, &key, "pr-1");
    std::fs::write(d.join("schemas/contract.json"), FURTHER).unwrap();
    git(&d, &["commit", "-q", "-am", "one more narrowing"]);
    let (code, out) = gate(&d, &signers);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("gate: no standing acceptance"), "{out}");
}

/// ag_f04: HOLDS passes with no acceptance and no gate complaint.
#[test]
fn ag_f04_holds_needs_no_acceptance() {
    let d = repo("f04", PRE, LOOSER);
    let (_, signers) = keypair(&d, "nikhil@test");
    let (code, out) = gate(&d, &signers);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("gate: no standing acceptance"), "{out}");
}

/// ag_f05: an acceptance signed by a key the signers file does not list
/// opens nothing. Kills AG-M2.
#[test]
fn ag_f05_unlisted_signature_opens_nothing() {
    let d = repo("f05", PRE, POST);
    let (key, _) = keypair(&d, "nikhil@test");
    accept_and_commit(&d, &key, "pr-1");
    let other = ws("f05other");
    let (_, other_signers) = keypair(&other, "nikhil@test");
    let (code, out) = gate(&d, &other_signers);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("gate: no standing acceptance"), "{out}");
    assert!(out.contains("signature"), "{out}");
}

/// ag_f06: NOT EVALUATED blocks with its own exit code 4 until someone
/// signs for it, then passes.
#[test]
fn ag_f06_not_evaluated_blocks_until_accepted() {
    let pre = r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{3}$"}}}"#;
    let post = r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{2}$"}}}"#;
    let d = repo("f06", pre, post);
    let (key, signers) = keypair(&d, "nikhil@test");
    let (code, out) = gate(&d, &signers);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("gate: no standing acceptance"), "{out}");
    accept_and_commit(&d, &key, "pr-1");
    let (code, out) = gate(&d, &signers);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("gate: accepted by nikhil@test"), "{out}");
}

/// ag_f07 guard: without --acceptances nothing changes: the judge's exit
/// code, and no gate lines.
#[test]
fn ag_f07_without_the_flags_judge_is_unchanged() {
    let d = repo("f07", PRE, POST);
    let (code, out) = invar(&d, &["judge", "--contracts", "schemas/*.json", "--base", "main"]);
    assert_eq!(code, 1, "{out}");
    assert!(!out.contains("gate:"), "{out}");
}
