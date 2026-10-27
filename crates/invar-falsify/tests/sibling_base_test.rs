//! SIB-3 fixtures (docs/falsification/SIBLING-BASE-SPEC.md r0), frozen with
//! the spec BEFORE any implementation. On a pull request the Action's
//! dependents step reads the consumer index and the trust root from the
//! BASE branch, never from the pull request: a PR can change the producer
//! files under review, but it cannot forge, add or remove what it is
//! judged against. Each fixture runs the real action/dependents.sh in a
//! real git checkout whose `origin` is a local bare repository.

use std::path::{Path, PathBuf};
use std::process::Command;

const TYPE: &str = "export interface Limits {\n  maxBytes: number;\n  maxLines: number;\n}\n";
const REMOVED: &str = "export interface Limits {\n  maxBytes: number;\n}\n";
const USE: &str = "import type { Limits } from \"@demo/api\";\nexport function f(l: Limits) {\n  return [l.maxBytes, l.maxLines];\n}\n";
const REMOTE: &str = "github:acme/api@0123456789abcdef0123456789abcdef01234567:src/limits.ts";

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_b3_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn put(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

fn keygen(dir: &Path, name: &str) -> PathBuf {
    let key = dir.join(name);
    assert!(Command::new("ssh-keygen").args(["-t", "ed25519", "-N", "", "-q", "-f"]).arg(&key).status().unwrap().success());
    key
}

fn signers_line(who: &str, key: &Path) -> String {
    format!("{who} {}", std::fs::read_to_string(key.with_extension("pub")).unwrap())
}

fn sign(lock: &Path, key: &Path) {
    let _ = std::fs::remove_file(lock.with_extension("lock.sig"));
    let o = Command::new("ssh-keygen").args(["-Y", "sign", "-n", "invar-lock", "-f"]).arg(key).arg(lock).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}

/// A consumer lock binding Limits.maxBytes and Limits.maxLines of acme/api,
/// written to `dest` (absolute).
fn consumer_lock(root: &Path, name: &str, dest: &Path) {
    let c = root.join("work").join(name);
    put(&c, "lib/limits.ts", TYPE);
    put(&c, "src/use.ts", USE);
    put(&c, "invar.toml", "[[invariant]]\nkind = \"schema\"\nid = \"limits\"\nproducer = \"lib/limits.ts\"\nconsumer = \"src/use.ts\"\n");
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).arg("lock").current_dir(&c).env("HOME", root).output().unwrap();
    assert!(o.status.success(), "{o:?}");
    let text = std::fs::read_to_string(c.join("invar.lock")).unwrap();
    std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
    std::fs::write(dest, text.replace("producer = \"lib/limits.ts\"", &format!("producer = \"{REMOTE}\""))).unwrap();
}

/// origin (bare) whose main holds the producer, a signed web lock in
/// .invar/index and the trust root .invar/consumer-signers; `ws` is a clone
/// on branch `pr` (the pull request), returned with the root and web's key.
fn world(tag: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = ws(tag);
    let web = keygen(&root, "kweb");
    let seed = root.join("seed");
    put(&seed, "src/limits.ts", TYPE);
    put(&seed, ".invar/consumer-signers", &signers_line("web@demo", &web));
    let lock = seed.join(".invar/index/web/invar.lock");
    consumer_lock(&root, "web", &lock);
    sign(&lock, &web);
    git(&seed, &["init", "-q", "-b", "main"]);
    git(&seed, &["add", "."]);
    git(&seed, &["commit", "-qm", "base"]);
    git(&root, &["clone", "-q", "--bare", "seed", "origin.git"]);
    git(&root, &["clone", "-q", "origin.git", "ws"]);
    let w = root.join("ws");
    git(&w, &["checkout", "-qb", "pr"]);
    put(&w, "src/limits.ts", REMOVED);
    (root, w, web)
}

/// Run action/dependents.sh as the Action does; returns (exit, stdout,
/// GITHUB_OUTPUT contents).
fn step(root: &Path, w: &Path, base: &str, locks: &str, signers: &str) -> (i32, String, String) {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../action/dependents.sh");
    let out_file = root.join(format!("github_output_{}", std::process::id()));
    let _ = std::fs::remove_file(&out_file);
    let runner = root.join("runner");
    std::fs::create_dir_all(&runner).unwrap();
    let o = Command::new("bash")
        .arg(&script)
        .current_dir(w)
        .env("HOME", root)
        .env("INVAR_BIN", env!("CARGO_BIN_EXE_invar"))
        .env("INVAR_DEPENDENTS_REPO", "acme/api")
        .env("INVAR_DEPENDENTS_LOCKS", locks)
        .env("INVAR_DEPENDENTS_SIGNERS", signers)
        .env("INVAR_BASE_REF", base)
        .env("RUNNER_TEMP", &runner)
        .env("GITHUB_OUTPUT", &out_file)
        .output()
        .unwrap();
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).to_string(),
        std::fs::read_to_string(&out_file).unwrap_or_default(),
    )
}

const LOCKS: &str = ".invar/index";
const SIGNERS: &str = ".invar/consumer-signers";

#[test]
fn b3_f01_a_forged_trust_root_in_the_pr_is_ignored() {
    // The PR replaces the trust root with an attacker key; the base one
    // still verifies web's lock, and the change under review is the PR's.
    let (root, w, _) = world("f01");
    let attacker = keygen(&root, "kx");
    put(&w, SIGNERS, &signers_line("web@demo", &attacker));
    let (code, out, outputs) = step(&root, &w, "main", LOCKS, SIGNERS);
    assert_eq!(code, 0, "advisory: {out}");
    assert!(out.contains("web/invar.lock [verified as web@demo] limits:"), "{out}");
    assert!(out.contains("    Limits.maxLines absent (locked number)\n"), "the PR's producer is what is judged: {out}");
    assert!(outputs.contains("dependents_broken=1\n"), "{outputs}");
}

#[test]
fn b3_f02_a_consumer_lock_deleted_in_the_pr_is_still_evaluated() {
    let (root, w, _) = world("f02");
    std::fs::remove_dir_all(w.join(".invar/index/web")).unwrap();
    let (_, out, outputs) = step(&root, &w, "main", LOCKS, SIGNERS);
    assert!(out.contains("    Limits.maxLines absent (locked number)\n"), "{out}");
    assert!(outputs.contains("dependents_broken=1\n"), "{outputs}");
}

#[test]
fn b3_f03_a_lock_and_key_added_in_the_pr_are_not_trusted() {
    // The PR adds its own signed lock and its own key: neither is on base.
    let (root, w, web) = world("f03");
    let attacker = keygen(&root, "kx");
    let forged = w.join(".invar/index/forged/invar.lock");
    consumer_lock(&root, "forged", &forged);
    sign(&forged, &attacker);
    put(&w, SIGNERS, &format!("{}{}", signers_line("web@demo", &web), signers_line("forged@x", &attacker)));
    let (_, out, _) = step(&root, &w, "main", LOCKS, SIGNERS);
    assert!(!out.contains("forged"), "{out}");
    assert!(out.contains("(locks: "), "{out}");
    assert!(out.contains("1 lockfile(s) scanned"), "{out}");
}

#[test]
fn b3_f04_an_unreadable_base_fails_the_job() {
    // B-D2 as revised 2026-10-27: a broken contract is advisory, but a
    // trust root the step cannot establish is an infrastructure failure.
    // Exiting 0 there would be a downgrade attack: break the base read,
    // get a green job.
    let (root, w, _) = world("f04");
    let (code, out, outputs) = step(&root, &w, "no-such-branch", LOCKS, SIGNERS);
    assert_eq!(code, 1, "an unreadable trust root fails the job: {out}");
    assert!(outputs.contains("dependents_exit=3\n"), "{outputs}");
    assert!(out.contains("not evaluated: base ref no-such-branch could not be read; the pull request's own index is never used"), "{out}");
    assert!(!out.contains("verified as"), "no fallback to the PR tree: {out}");
}

#[test]
fn b3_f05_without_a_base_ref_the_workspace_is_used() {
    // A push to main: the workspace is the branch itself.
    let (root, w, _) = world("f05");
    let (_, out, _) = step(&root, &w, "", LOCKS, SIGNERS);
    assert!(out.contains("    Limits.maxLines absent (locked number)\n"), "{out}");
}

#[test]
fn b3_f06_absolute_paths_are_the_operators_and_used_as_given() {
    // An index checked out by the operator outside the workspace is not
    // PR-controlled: it is read where it is.
    let (root, w, web) = world("f06");
    let outside = root.join("index-checkout");
    consumer_lock(&root, "web2", &outside.join("web/invar.lock"));
    sign(&outside.join("web/invar.lock"), &web);
    put(&outside, "signers", &signers_line("web@demo", &web));
    let (_, out, _) = step(
        &root,
        &w,
        "main",
        &outside.display().to_string(),
        &outside.join("signers").display().to_string(),
    );
    assert!(out.contains("web/invar.lock [verified as web@demo] limits:"), "{out}");
    assert!(out.contains("    Limits.maxLines absent (locked number)\n"), "{out}");
}

#[test]
fn b3_f07_the_action_passes_the_base_ref_to_the_step() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let yml = std::fs::read_to_string(root.join("action.yml")).unwrap();
    let step = yml.split("    - name: Evaluate consumer locks").nth(1).unwrap();
    let step = step.split("    - name:").next().unwrap();
    assert!(step.contains("        INVAR_BASE_REF: ${{ inputs.base_ref }}\n"), "{step}");
}
