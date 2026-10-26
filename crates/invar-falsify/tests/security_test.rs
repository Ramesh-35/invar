//! SECURITY S1 fixtures (docs/falsification/SECURITY-SPEC.md r0), frozen
//! before the S1 implementation is gated. A reused HOLDS must never come
//! from a record an attacker can write: the local cache is accepted only
//! under a per-user key outside the repository, and invar.lock records are
//! reused only when the lock's signature verified in that run. Every
//! fixture runs with its own HOME, so no test touches a real home.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_s1_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join("repo")).unwrap();
    std::fs::create_dir_all(d.join("home")).unwrap();
    let r = d.join("repo");
    std::fs::write(r.join("base.yaml"), "properties:\n  id: {type: string}\n  legacy: {type: string}\n").unwrap();
    std::fs::write(r.join("head.yaml"), "properties:\n  id: {type: string}\n  legacy: {type: string}\n").unwrap();
    std::fs::write(r.join("client.py"), "def f(d):\n    return d[\"legacy\"]\n").unwrap();
    std::fs::write(r.join("invar.toml"), "[[invariant]]\nkind = \"schema\"\nid = \"s\"\nproducer = \"head.yaml\"\nproducer_base = \"base.yaml\"\nconsumer = \"client.py\"\n").unwrap();
    d
}

fn invar(d: &Path, home: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(d.join("repo")).env("HOME", home).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

fn h(d: &Path) -> PathBuf {
    d.join("home")
}

/// Break the contract: the producer drops the field the consumer reads.
fn break_it(d: &Path) {
    std::fs::write(d.join("repo/head.yaml"), "properties:\n  id: {type: string}\n").unwrap();
}

fn b3(p: &Path) -> String {
    blake3::hash(&std::fs::read(p).unwrap()).to_hex().to_string()
}

/// Rewrite every recorded input hash to the current bytes (the forger's
/// move: valid hashes, no key).
fn forge_cache(d: &Path) {
    let p = d.join("repo/.invar/cache/verdicts.json");
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    for (_, r) in v["records"].as_object_mut().unwrap().iter_mut() {
        let rec = if r.get("held").is_some() { &mut r["held"] } else { r };
        let inputs: Vec<serde_json::Value> = rec["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| {
                let f = i[1].as_str().unwrap().to_string();
                serde_json::json!([b3(&d.join("repo").join(&f)), f])
            })
            .collect();
        rec["inputs"] = serde_json::Value::Array(inputs);
    }
    std::fs::write(&p, v.to_string()).unwrap();
}

fn forge_lock(d: &Path) {
    let p = d.join("repo/invar.lock");
    let text = std::fs::read_to_string(&p).unwrap();
    let out: Vec<String> = text
        .lines()
        .map(|l| match l.strip_prefix("held_input = \"").and_then(|r| r.strip_suffix('"')).and_then(|r| r.split_once(' ')) {
            Some((_, f)) => format!("held_input = \"{} {f}\"", b3(&d.join("repo").join(f))),
            None => l.to_string(),
        })
        .collect();
    std::fs::write(&p, out.join("\n") + "\n").unwrap();
}

fn reused(out: &str) -> bool {
    out.lines().any(|l| l.contains("[HOLDS]") && l.contains("unchanged since"))
}

#[test]
fn s1_f01_a_forged_cache_record_is_never_reused() {
    let d = ws("f01");
    assert_eq!(invar(&d, &h(&d), &["check"]).0, 0);
    break_it(&d);
    forge_cache(&d);
    let (code, out) = invar(&d, &h(&d), &["check"]);
    assert_eq!(code, 1, "a forged record must not hide the violation:\n{out}");
    assert!(out.contains("[VIOLATED] s") && !reused(&out), "{out}");
}

#[test]
fn s1_f02_a_forged_unsigned_lock_record_is_never_reused() {
    let d = ws("f02");
    assert_eq!(invar(&d, &h(&d), &["lock"]).0, 0);
    break_it(&d);
    forge_lock(&d);
    let (code, out) = invar(&d, &h(&d), &["check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] s") && !reused(&out), "{out}");
}

#[test]
fn s1_f03_an_honest_unsigned_lock_is_evaluated_then_cached() {
    let d = ws("f03");
    assert_eq!(invar(&d, &h(&d), &["lock"]).0, 0);
    let (code, out) = invar(&d, &h(&d), &["check"]);
    assert_eq!(code, 0, "{out}");
    // Only the HOLDS line says whether a verdict was reused; the evaluated
    // lock-mode note "producer unchanged since lock (...)" is not a reuse.
    assert!(!reused(&out), "an unsigned lock is not evidence:\n{out}");
    let (code, out) = invar(&d, &h(&d), &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.lines().any(|l| l.contains("[HOLDS]") && l.contains("unchanged since last run")), "the keyed cache is:\n{out}");
}

#[test]
fn s1_f04_a_signed_lock_is_reused_locally() {
    let d = ws("f04");
    assert_eq!(invar(&d, &h(&d), &["lock"]).0, 0);
    let r = d.join("repo");
    assert!(Command::new("ssh-keygen").args(["-t", "ed25519", "-N", "", "-q", "-f"]).arg(d.join("k")).status().unwrap().success());
    assert!(Command::new("ssh-keygen").args(["-Y", "sign", "-n", "invar-lock", "-f"]).arg(d.join("k")).arg(r.join("invar.lock")).status().unwrap().success());
    std::fs::write(r.join("signers"), format!("release@invar {}", std::fs::read_to_string(d.join("k.pub")).unwrap())).unwrap();
    let (code, out) = invar(&d, &h(&d), &["check", "--lock-signers", "signers", "--lock-principal", "release@invar"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.lines().any(|l| l.contains("[HOLDS]") && l.contains("unchanged since lock")), "a verified lock record is reused:\n{out}");
}

#[test]
fn s1_f05_an_unkeyed_format_1_cache_is_never_read() {
    // The pre-S1 cache format: the same record fields with no key. The
    // forger copies the real engine, block and entry values from an honest
    // record and sets valid hashes of the broken inputs.
    let d = ws("f05");
    assert_eq!(invar(&d, &h(&d), &["check"]).0, 0);
    let r = d.join("repo");
    let p = r.join(".invar/cache/verdicts.json");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    let held = v["records"]["s"].get("held").cloned().unwrap_or_else(|| v["records"]["s"].clone());
    break_it(&d);
    let mut forged = held.clone();
    forged["inputs"] = serde_json::json!([[b3(&r.join("head.yaml")), "head.yaml"], [b3(&r.join("base.yaml")), "base.yaml"], [b3(&r.join("client.py")), "client.py"]]);
    std::fs::write(&p, serde_json::json!({"format": "1", "records": {"s": forged}}).to_string()).unwrap();
    let (code, out) = invar(&d, &h(&d), &["check"]);
    assert_eq!(code, 1, "{out}");
    assert!(!reused(&out), "{out}");
}

#[test]
fn s1_f06_the_key_lives_in_home_never_in_the_repository() {
    let d = ws("f06");
    assert_eq!(invar(&d, &h(&d), &["check"]).0, 0);
    let key = std::fs::read(h(&d).join(".invar/key")).expect("the per-user key is created in HOME");
    assert_eq!(key.len(), 32);
    assert!(!d.join("repo/.invar/key").exists(), "never inside the repository");
}

#[cfg(unix)]
#[test]
fn s1_f07_the_key_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let d = ws("f07");
    assert_eq!(invar(&d, &h(&d), &["check"]).0, 0);
    let mode = std::fs::metadata(h(&d).join(".invar/key")).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "mode {mode:o}");
}

#[test]
fn s1_f08_without_a_key_nothing_is_reused() {
    let d = ws("f08");
    assert_eq!(invar(&d, &h(&d), &["check"]).0, 0);
    assert!(reused(&invar(&d, &h(&d), &["check"]).1), "control: reused with the key");
    let (code, out) = invar(&d, Path::new(""), &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(!reused(&out), "no usable key: the record cannot be verified, so it is not used:\n{out}");
}

#[test]
fn s1_f09_another_users_key_does_not_verify() {
    let d = ws("f09");
    assert_eq!(invar(&d, &h(&d), &["check"]).0, 0);
    let other = d.join("other-home");
    std::fs::create_dir_all(&other).unwrap();
    let (code, out) = invar(&d, &other, &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(!reused(&out), "a record keyed by another user is not this user's evidence:\n{out}");
}
