//! LOCK-1 fixtures (AGENT-1 P4, docs/falsification/LOCK-VERIFIED-SPEC.md
//! r0), frozen with the spec BEFORE any implementation. Each fixture drives
//! the real `invar mcp` server over stdio.
//!
//! The `check` and `contract` results state whether the invar.lock they
//! used was signature-verified in that call: `lock_verified` (bool) and
//! `lock_state` ("absent", "unsigned", "verified"). With
//! `--lock-signers`/`--lock-principal`, every call re-verifies
//! invar.lock.sig over the exact bytes it then reads; a missing or rejected
//! signature is an error (never a verdict), as `invar check --lock-signers`
//! already is. Stdout carries JSON-RPC only.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_p4_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (rel, body) in files {
        put(&dir, rel, body);
    }
    dir
}

fn put(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

fn invar(dir: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(dir).env("HOME", dir).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

const TOML: &str = "[[invariant]]\nkind = \"schema\"\nid = \"lk\"\nproducer = \"lib/limits.ts\"\nconsumer = \"src/use.ts\"\n";
const TYPE: &str = "export interface Limits {\n  maxBytes: number;\n  maxLines: number;\n}\n";
const USE: &str = "import type { Limits } from \"../lib/limits\";\nexport function f(l: Limits) {\n  return [l.maxBytes, l.maxLines];\n}\n";

fn project(tag: &str, lock: bool) -> PathBuf {
    let d = ws(tag, &[("lib/limits.ts", TYPE), ("src/use.ts", USE), ("invar.toml", TOML)]);
    if lock {
        let (c, out) = invar(&d, &["lock"]);
        assert_eq!(c, 0, "{out}");
    }
    d
}

/// A signing key and an allowed-signers file `signers` for principal
/// release@invar; returns the private key path.
fn keys(dir: &Path, name: &str) -> PathBuf {
    let key = dir.join(name);
    assert!(Command::new("ssh-keygen").args(["-t", "ed25519", "-N", "", "-q", "-f"]).arg(&key).status().unwrap().success());
    key
}

fn trust(dir: &Path, key: &Path) {
    let pubkey = std::fs::read_to_string(key.with_extension("pub")).unwrap();
    put(dir, "signers", &format!("release@invar {pubkey}"));
}

fn sign(dir: &Path, key: &Path) {
    let _ = std::fs::remove_file(dir.join("invar.lock.sig"));
    let o = Command::new("ssh-keygen").args(["-Y", "sign", "-n", "invar-lock", "-f"]).arg(key).arg(dir.join("invar.lock")).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}

/// A running `invar mcp` with `args`; `call` returns the tools/call result
/// and asserts every stdout line is JSON-RPC.
struct Mcp {
    child: std::process::Child,
    out: BufReader<std::process::ChildStdout>,
    id: u64,
}

impl Mcp {
    fn start(dir: &Path, args: &[&str]) -> Mcp {
        let mut child = Command::new(env!("CARGO_BIN_EXE_invar"))
            .arg("mcp")
            .args(args)
            .current_dir(dir)
            .env("HOME", dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let out = BufReader::new(child.stdout.take().unwrap());
        let mut m = Mcp { child, out, id: 0 };
        m.rpc("initialize", r#"{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}"#);
        m
    }
    fn rpc(&mut self, method: &str, params: &str) -> Value {
        self.id += 1;
        let msg = format!("{{\"jsonrpc\":\"2.0\",\"id\":{},\"method\":\"{method}\",\"params\":{params}}}\n", self.id);
        let stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(msg.as_bytes()).unwrap();
        stdin.flush().unwrap();
        let mut line = String::new();
        self.out.read_line(&mut line).unwrap();
        let v: Value = serde_json::from_str(&line).unwrap_or_else(|e| panic!("stdout carried a non-JSON line ({e}): {line:?}"));
        assert_eq!(v["id"], json!(self.id), "{v}");
        v
    }
    fn call(&mut self, tool: &str) -> Value {
        self.rpc("tools/call", &format!("{{\"name\":\"{tool}\",\"arguments\":{{}}}}"))["result"].clone()
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}

const SIGNED: &[&str] = &["--lock-signers", "signers", "--lock-principal", "release@invar"];

fn ok(r: &Value) -> &Value {
    assert_eq!(r["isError"], json!(false), "{r}");
    &r["structuredContent"]
}

fn refused(r: &Value) {
    assert_eq!(r["isError"], json!(true), "a lock that does not verify is an error, never a verdict: {r}");
    let text = r["content"][0]["text"].as_str().unwrap_or("");
    assert!(text.starts_with("ERROR (not a verdict): ") && text.contains("lock not confirmed"), "{r}");
    assert!(r.get("structuredContent").is_none(), "no verdict rides on a refusal: {r}");
}

// ------------------------------------------------------- without signers

#[test]
fn p4_f01_no_lock_is_absent() {
    let d = project("f01", false);
    let mut m = Mcp::start(&d, &[]);
    let v = m.call("check");
    let c = ok(&v);
    assert_eq!(c["lock_verified"], json!(false), "{c}");
    assert_eq!(c["lock_state"], json!("absent"), "{c}");
    // r1: the pre-P4 engine reports an unlocked contract as not evaluated.
    assert_eq!(c["result"], json!("not_fully_evaluated"), "existing keys unchanged: {c}");
    assert_eq!(c["exit_code"], json!(4), "existing keys unchanged: {c}");
}

#[test]
fn p4_f02_an_unsigned_lock_is_never_verified() {
    let d = project("f02", true);
    let mut m = Mcp::start(&d, &[]);
    for tool in ["check", "contract"] {
        let v = m.call(tool);
        let c = ok(&v);
        assert_eq!(c["lock_verified"], json!(false), "{tool}: {c}");
        assert_eq!(c["lock_state"], json!("unsigned"), "{tool}: {c}");
    }
    let v = m.call("contract");
    assert!(ok(&v)["contracts"][0]["bindings"].as_array().is_some_and(|b| !b.is_empty()), "existing keys unchanged: {v}");
}

#[test]
fn p4_f03_a_signature_on_an_unconfigured_server_is_still_unsigned() {
    // A signature file proves nothing until the operator says whose key
    // counts: without --lock-signers the lock is not authenticated.
    let d = project("f03", true);
    let k = keys(&d, "k");
    trust(&d, &k);
    sign(&d, &k);
    let mut m = Mcp::start(&d, &[]);
    let v = m.call("check");
    assert_eq!(ok(&v)["lock_state"], json!("unsigned"), "{v}");
    assert_eq!(ok(&v)["lock_verified"], json!(false), "{v}");
}

// ---------------------------------------------------------- with signers

#[test]
fn p4_f04_a_lock_signed_by_a_trusted_key_is_verified() {
    let d = project("f04", true);
    let k = keys(&d, "k");
    trust(&d, &k);
    sign(&d, &k);
    let mut m = Mcp::start(&d, SIGNED);
    for tool in ["check", "contract"] {
        let v = m.call(tool);
        let c = ok(&v);
        assert_eq!(c["lock_verified"], json!(true), "{tool}: {c}");
        assert_eq!(c["lock_state"], json!("verified"), "{tool}: {c}");
    }
}

#[test]
fn p4_f05_a_missing_signature_is_refused() {
    let d = project("f05", true);
    let k = keys(&d, "k");
    trust(&d, &k);
    let mut m = Mcp::start(&d, SIGNED);
    refused(&m.call("check"));
    refused(&m.call("contract"));
}

#[test]
fn p4_f06_a_signature_by_an_untrusted_key_is_refused() {
    let d = project("f06", true);
    let trusted = keys(&d, "k");
    trust(&d, &trusted);
    let other = keys(&d, "other");
    sign(&d, &other);
    let mut m = Mcp::start(&d, SIGNED);
    refused(&m.call("check"));
}

#[test]
fn p4_f07_a_lock_edited_after_signing_is_refused() {
    let d = project("f07", true);
    let k = keys(&d, "k");
    trust(&d, &k);
    sign(&d, &k);
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    put(&d, "invar.lock", &lock.replace("maxLines number", "maxLines string"));
    let mut m = Mcp::start(&d, SIGNED);
    refused(&m.call("check"));
}

#[test]
fn p4_f08_every_call_verifies_again() {
    // Verified at the first call; the lock is then edited while the server
    // runs; the next call must see it.
    let d = project("f08", true);
    let k = keys(&d, "k");
    trust(&d, &k);
    sign(&d, &k);
    let mut m = Mcp::start(&d, SIGNED);
    assert_eq!(ok(&m.call("check"))["lock_verified"], json!(true));
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    put(&d, "invar.lock", &format!("{lock}\n"));
    refused(&m.call("check"));
    refused(&m.call("contract"));
}

#[test]
fn p4_f09_signers_without_a_principal_fail_at_start() {
    let d = project("f09", true);
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["mcp", "--lock-signers", "signers"])
        .current_dir(&d)
        .env("HOME", &d)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(3), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stderr).contains("--lock-signers needs --lock-principal"), "{o:?}");
    assert!(o.stdout.is_empty(), "stdout carries JSON-RPC only: {o:?}");
}

#[test]
fn p4_f10_the_cli_is_unchanged() {
    // `invar check --lock-signers` keeps its exact behavior and text.
    let d = project("f10", true);
    let k = keys(&d, "k");
    trust(&d, &k);
    sign(&d, &k);
    let (code, out) = invar(&d, &["check", "--no-reuse", "--lock-signers", "signers", "--lock-principal", "release@invar"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.starts_with("lock signature ok (release@invar)\n"), "{out}");
    assert!(!out.contains("lock_verified") && !out.contains("lock_state"), "{out}");
}
