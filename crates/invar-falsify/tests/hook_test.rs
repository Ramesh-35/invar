//! HOOK-1 fixtures (docs/falsification/HOOK-SPEC.md r0), frozen with the
//! spec BEFORE any implementation. `invar hook claude-code` IS the hook: a
//! Claude Code PostToolUse entry runs it, it reads the event JSON on stdin,
//! evaluates this repository's contracts with the engine check uses, and
//! speaks to the agent only when a contract is VIOLATED: the witness on
//! stderr, exit 2 (the one code PostToolUse feeds back to the model).
//! Everything else is exit 0 and silent, except a real error, which is
//! exit 3 with `ERROR (not a verdict)` (a non-blocking error to the agent,
//! never model feedback, so a broken setup cannot thrash the loop).

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const TYPE: &str = "export interface Limits {\n  maxBytes: number;\n  maxLines: number;\n}\n";
const REMOVED: &str = "export interface Limits {\n  maxBytes: number;\n}\n";
const USE: &str = "import type { Limits } from \"../lib/limits\";\nexport function f(l: Limits) {\n  return [l.maxBytes, l.maxLines];\n}\n";
const TOML: &str = "[[invariant]]\nkind = \"schema\"\nid = \"lk\"\nproducer = \"lib/limits.ts\"\nconsumer = \"src/use.ts\"\n";

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_hk_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn put(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

/// A repository whose contract lk binds Limits.maxBytes and Limits.maxLines;
/// locked while whole, then the producer is set to `limits`.
fn repo(tag: &str, limits: &str, lock: bool) -> PathBuf {
    let d = ws(tag);
    put(&d, "lib/limits.ts", TYPE);
    put(&d, "src/use.ts", USE);
    put(&d, "invar.toml", TOML);
    if lock {
        let o = Command::new(env!("CARGO_BIN_EXE_invar")).arg("lock").current_dir(&d).env("HOME", &d).output().unwrap();
        assert!(o.status.success(), "{o:?}");
    }
    put(&d, "lib/limits.ts", limits);
    d
}

/// Run `invar hook claude-code` with this stdin, from `from`, HOME=`home`.
fn hook(from: &Path, home: &Path, stdin: &str) -> (i32, String, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["hook", "claude-code"])
        .current_dir(from)
        .env("HOME", home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.as_mut().unwrap().write_all(stdin.as_bytes()).unwrap();
    let o = c.wait_with_output().unwrap();
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).to_string(),
        String::from_utf8_lossy(&o.stderr).to_string(),
    )
}

fn event(tool: &str, cwd: &Path) -> String {
    format!(
        "{{\"hook_event_name\":\"PostToolUse\",\"tool_name\":\"{tool}\",\"tool_input\":{{\"file_path\":\"lib/limits.ts\"}},\"cwd\":{}}}",
        serde_json::to_string(&cwd.display().to_string()).unwrap()
    )
}

#[test]
fn hk_f01_a_holding_repository_is_silent() {
    let d = repo("f01", TYPE, true);
    let (code, out, err) = hook(&d, &d, &event("Edit", &d));
    assert_eq!((code, out.as_str(), err.as_str()), (0, "", ""), "nothing to say: {out}{err}");
}

#[test]
fn hk_f02_a_violation_reaches_the_agent() {
    let d = repo("f02", REMOVED, true);
    let (code, out, err) = hook(&d, &d, &event("Edit", &d));
    assert_eq!(code, 2, "exit 2 is the one code the agent is fed: {out}{err}");
    assert!(err.contains("lk") && err.contains("maxLines"), "the witness names the contract and the field: {err}");
    assert!(err.contains("src/use.ts"), "and the consumer that reads it: {err}");
    assert_eq!(out, "", "stdout stays empty; the feedback channel is stderr: {out}");
}

#[test]
fn hk_f03_non_edit_tools_run_nothing() {
    // Even on a violated repository: a Read changed no producer.
    let d = repo("f03", REMOVED, true);
    let (code, out, err) = hook(&d, &d, &event("Read", &d));
    assert_eq!((code, out.as_str(), err.as_str()), (0, "", ""), "{out}{err}");
}

#[test]
fn hk_f04_a_repository_without_invar_is_silent() {
    let d = ws("f04");
    put(&d, "src/a.ts", "export const a = 1;\n");
    let (code, out, err) = hook(&d, &d, &event("Edit", &d));
    assert_eq!((code, out.as_str(), err.as_str()), (0, "", ""), "the hook must be installable everywhere: {out}{err}");
}

#[test]
fn hk_f05_malformed_stdin_is_an_error_never_feedback() {
    let d = repo("f05", REMOVED, true);
    let (code, _, err) = hook(&d, &d, "{not json");
    assert_eq!(code, 3, "{err}");
    assert!(err.starts_with("ERROR (not a verdict): "), "{err}");
}

#[test]
fn hk_f06_an_engine_error_is_an_error_never_feedback() {
    let d = repo("f06", REMOVED, true);
    put(&d, "invar.toml", "[[invariant]]\nkind = \"schema\"\n");
    let (code, _, err) = hook(&d, &d, &event("Edit", &d));
    assert_eq!(code, 3, "a broken setup must not thrash the agent loop: {err}");
    assert!(err.starts_with("ERROR (not a verdict): "), "{err}");
}

#[test]
fn hk_f11_an_unparsable_configuration_is_an_error_never_feedback() {
    // r1: hk_f06's invar.toml parses and fails only in evaluation, so the
    // parse path needs its own fixture (found by HK-M6 surviving in r0).
    let d = repo("f11", REMOVED, true);
    put(&d, "invar.toml", "[[[\n");
    let (code, _, err) = hook(&d, &d, &event("Edit", &d));
    assert_eq!(code, 3, "{err}");
    assert!(err.starts_with("ERROR (not a verdict): "), "{err}");
}

#[test]
fn hk_f07_settings_prints_the_posttooluse_entry() {
    let d = repo("f07", TYPE, true);
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["hook", "claude-code", "--settings"])
        .current_dir(&d)
        .env("HOME", &d)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("valid JSON");
    let h = &v["hooks"]["PostToolUse"][0];
    assert_eq!(h["matcher"], serde_json::json!("Write|Edit|MultiEdit|NotebookEdit"), "{v}");
    assert_eq!(h["hooks"][0]["command"], serde_json::json!("invar hook claude-code"), "{v}");
}

#[test]
fn hk_f08_not_evaluated_stays_out_of_the_loop() {
    // No lock: check would exit 4 (stated). The hook stays silent; coverage
    // disclosure belongs to the MCP coverage tool, not to every edit.
    let d = repo("f08", REMOVED, false);
    let (code, out, err) = hook(&d, &d, &event("Edit", &d));
    assert_eq!((code, out.as_str(), err.as_str()), (0, "", ""), "{out}{err}");
}

#[test]
fn hk_f09_the_events_cwd_wins_over_the_processes() {
    let d = repo("f09", REMOVED, true);
    let elsewhere = ws("f09_elsewhere");
    let (code, _, err) = hook(&elsewhere, &d, &event("Edit", &d));
    assert_eq!(code, 2, "the event says where the agent is working: {err}");
    assert!(err.contains("maxLines"), "{err}");
}

#[test]
fn hk_f10_an_unknown_agent_is_an_error() {
    let d = repo("f10", TYPE, true);
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["hook", "cursor"])
        .current_dir(&d)
        .env("HOME", &d)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(3), "{o:?}");
    assert!(String::from_utf8_lossy(&o.stderr).contains("unknown agent"), "{o:?}");
}
