//! SIB-1 fixtures (P5, docs/falsification/SIBLING-EVAL-SPEC.md r1), frozen
//! with the spec BEFORE any implementation. Each fixture drives the real
//! `invar dependents` in evaluation mode (no field query, `--repo`):
//! run in a producer repository, it evaluates every signed consumer lock
//! under `--locks` whose remote producer names this repository against the
//! producer files in the working tree. Advisory: a broken field never fails
//! the run (exit 0); anything not evaluated is a stated blind spot (exit 4).

use std::path::{Path, PathBuf};
use std::process::Command;

const TYPE: &str = "export interface Limits {\n  maxBytes: number;\n  maxLines: number;\n}\n";
const USE: &str = "import type { Limits } from \"../lib/limits\";\nexport function f(l: Limits) {\n  return [l.maxBytes, l.maxLines];\n}\n";
const TOML: &str = "[[invariant]]\nkind = \"schema\"\nid = \"lk\"\nproducer = \"lib/limits.ts\"\nconsumer = \"src/use.ts\"\n";
const REMOTE: &str = "github:acme/api@0123456789abcdef0123456789abcdef01234567:lib/limits.ts";
const NOTE: &str = "note: a lock records the fields a consumer depends on, not the lines that read them.";

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_p5_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn put(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

fn invar(dir: &Path, home: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(dir).env("HOME", home).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn keygen(dir: &Path, name: &str) -> PathBuf {
    let key = dir.join(name);
    assert!(Command::new("ssh-keygen").args(["-t", "ed25519", "-N", "", "-q", "-f"]).arg(&key).status().unwrap().success());
    key
}

/// One allowed-signers file for every consumer repository (P5-D3).
fn trust(root: &Path, entries: &[(&str, &Path)]) {
    let body: String = entries
        .iter()
        .map(|(who, key)| format!("{who} {}", std::fs::read_to_string(key.with_extension("pub")).unwrap()))
        .collect();
    put(root, "signers", &body);
}

fn sign(lock: &Path, key: &Path) {
    let _ = std::fs::remove_file(lock.with_extension("lock.sig"));
    let o = Command::new("ssh-keygen").args(["-Y", "sign", "-n", "invar-lock", "-f"]).arg(key).arg(lock).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}

/// A consumer lock binding Limits.maxBytes and Limits.maxLines, its
/// producer rewritten to `producer` (a remote ref), placed at
/// `<root>/locks/<name>/invar.lock`. Returns the lock path.
fn consumer_lock(root: &Path, name: &str, producer: &str) -> PathBuf {
    let c = root.join("work").join(name);
    put(&c, "lib/limits.ts", TYPE);
    put(&c, "src/use.ts", USE);
    put(&c, "invar.toml", TOML);
    let (code, out) = invar(&c, root, &["lock"]);
    assert_eq!(code, 0, "{out}");
    let text = std::fs::read_to_string(c.join("invar.lock")).unwrap();
    assert!(text.contains("producer = \"lib/limits.ts\""), "{text}");
    let lock = root.join("locks").join(name).join("invar.lock");
    put(root, &format!("locks/{name}/invar.lock"), &text.replace("producer = \"lib/limits.ts\"", &format!("producer = \"{producer}\"")));
    lock
}

/// The producer repository `api` with this version of lib/limits.ts.
fn producer(root: &Path, limits: &str) -> PathBuf {
    let p = root.join("api");
    put(&p, "lib/limits.ts", limits);
    p
}

const EVAL: &[&str] = &["dependents", "--locks", "../locks", "--repo", "acme/api", "--lock-signers", "../signers"];

/// A signed consumer-a lock against acme/api, trusted as consumer-a@acme.
fn signed_world(tag: &str, limits: &str) -> (PathBuf, PathBuf) {
    let root = ws(tag);
    let k = keygen(&root, "ka");
    trust(&root, &[("consumer-a@acme", &k)]);
    let lock = consumer_lock(&root, "consumer-a", REMOTE);
    sign(&lock, &k);
    (root.clone(), producer(&root, limits))
}

const REMOVED: &str = "export interface Limits {\n  maxBytes: number;\n}\n";
const RETYPED: &str = "export interface Limits {\n  maxBytes: number;\n  maxLines: string;\n}\n";

#[test]
fn p5_f01_a_removed_field_is_named_and_advisory() {
    let (root, api) = signed_world("f01", REMOVED);
    let (code, out) = invar(&api, &root, EVAL);
    assert_eq!(code, 0, "advisory: a broken field never fails the run: {out}");
    assert!(out.contains("consumer-a/invar.lock [verified as consumer-a@acme] lk:"), "{out}");
    assert!(out.contains("    Limits.maxLines absent (locked number)\n"), "{out}");
    assert!(!out.contains("Limits.maxBytes absent"), "{out}");
    assert!(out.contains(NOTE), "field-level, stated: {out}");
}

#[test]
fn p5_f02_a_retyped_field_is_named() {
    let (root, api) = signed_world("f02", RETYPED);
    let (code, out) = invar(&api, &root, EVAL);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("    Limits.maxLines retyped (locked number, now string)\n"), "{out}");
}

#[test]
fn p5_f03_all_provided_counts_each_field_once() {
    // A scoped binding absorbs the unscoped binding of the same field.
    let (root, api) = signed_world("f03", TYPE);
    let (code, out) = invar(&api, &root, EVAL);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("    2 binding(s) provided\n"), "{out}");
    assert!(!out.contains(" absent") && !out.contains(" retyped"), "{out}");
}

#[test]
fn p5_f04_an_unsigned_lock_is_never_evaluated() {
    let root = ws("f04");
    let k = keygen(&root, "ka");
    trust(&root, &[("consumer-a@acme", &k)]);
    consumer_lock(&root, "consumer-a", REMOTE);
    let api = producer(&root, REMOVED);
    let (code, out) = invar(&api, &root, EVAL);
    assert_eq!(code, 4, "a blind spot, stated: {out}");
    assert!(out.contains("consumer-a/invar.lock: not evaluated: unsigned (no invar.lock.sig)"), "{out}");
    assert!(!out.contains("maxLines"), "nothing read from an unverified lock reaches the output: {out}");
}

#[test]
fn p5_f05_a_signature_by_an_untrusted_key_is_refused() {
    let root = ws("f05");
    let trusted = keygen(&root, "ka");
    let other = keygen(&root, "kx");
    trust(&root, &[("consumer-a@acme", &trusted)]);
    let lock = consumer_lock(&root, "consumer-a", REMOTE);
    sign(&lock, &other);
    let api = producer(&root, REMOVED);
    let (code, out) = invar(&api, &root, EVAL);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("consumer-a/invar.lock: not evaluated: signature does not verify"), "{out}");
    assert!(!out.contains("maxLines"), "{out}");
}

#[test]
fn p5_f06_a_lock_edited_after_signing_is_refused() {
    let (root, api) = signed_world("f06", REMOVED);
    let lock = root.join("locks/consumer-a/invar.lock");
    let text = std::fs::read_to_string(&lock).unwrap();
    std::fs::write(&lock, format!("{text}\n")).unwrap();
    let (code, out) = invar(&api, &root, EVAL);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("consumer-a/invar.lock: not evaluated: signature does not verify"), "{out}");
}

#[test]
fn p5_f07_a_lock_on_another_repository_is_not_ours() {
    let root = ws("f07");
    let k = keygen(&root, "ka");
    trust(&root, &[("consumer-a@acme", &k)]);
    let lock = consumer_lock(&root, "consumer-a", &REMOTE.replace("acme/api", "other/api"));
    sign(&lock, &k);
    let api = producer(&root, REMOVED);
    let (code, out) = invar(&api, &root, EVAL);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("consumer-a"), "{out}");
    assert!(out.contains("summary: 0 broken | 0 provided | 0 lockfile(s) not evaluated"), "{out}");
}

#[test]
fn p5_f08_repository_names_match_without_case() {
    // GitHub owner and repository names are case-insensitive.
    let root = ws("f08");
    let k = keygen(&root, "ka");
    trust(&root, &[("consumer-a@acme", &k)]);
    let lock = consumer_lock(&root, "consumer-a", &REMOTE.replace("acme/api", "Acme/API"));
    sign(&lock, &k);
    let api = producer(&root, REMOVED);
    let (code, out) = invar(&api, &root, EVAL);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("    Limits.maxLines absent (locked number)\n"), "{out}");
}

#[test]
fn p5_f09_without_signers_nothing_is_evaluated() {
    let (root, api) = signed_world("f09", REMOVED);
    let (code, out) = invar(&api, &root, &["dependents", "--locks", "../locks", "--repo", "acme/api"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("consumer-a/invar.lock: not evaluated: no signers configured (--lock-signers)"), "{out}");
    assert!(!out.contains("maxLines"), "{out}");
}

#[test]
fn p5_f10_a_missing_producer_file_is_not_evaluated_not_absent() {
    let (root, api) = signed_world("f10", TYPE);
    std::fs::remove_file(api.join("lib/limits.ts")).unwrap();
    let (code, out) = invar(&api, &root, EVAL);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("consumer-a/invar.lock [verified as consumer-a@acme] lk: not evaluated: lib/limits.ts not found in this repository"), "{out}");
    assert!(!out.contains(" absent"), "{out}");
}

#[test]
fn p5_f11_no_query_and_no_repo_is_an_error() {
    let (root, api) = signed_world("f11", TYPE);
    let (code, out) = invar(&api, &root, &["dependents", "--locks", "../locks"]);
    assert_eq!(code, 3, "{out}");
    assert!(out.contains("give a field query, or --repo OWNER/NAME to evaluate"), "{out}");
}

#[test]
fn p5_f12_evaluation_is_text_only_in_this_slice() {
    let (root, api) = signed_world("f12", TYPE);
    let mut args = EVAL.to_vec();
    args.extend(["--format", "json"]);
    let (code, out) = invar(&api, &root, &args);
    assert_eq!(code, 3, "{out}");
    assert!(out.contains("evaluation (--repo) is text only"), "{out}");
}
