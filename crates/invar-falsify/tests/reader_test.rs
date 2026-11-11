//! PROGRAM r0 fixtures (docs/falsification/PROGRAM-SPEC.md), frozen with
//! the spec BEFORE implementation. `invar reader` feeds the judge's own
//! counterexamples to the real program that reads the file and records
//! what it did: exit 0, exit N, or timed out. The reader line never
//! changes the schema verdict or the exit code; the environment is
//! cleared before the command runs; the report is recorded, not
//! re-derived.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - RD-M1 the environment not cleared (credentials leak)        -> rd_f04
//! - RD-M2 a nonzero exit recorded as exit 0                     -> rd_f02
//! - RD-M3 the control run skipped                               -> rd_f02
//! - RD-M4 the timeout not enforced                              -> rd_f05

use std::path::{Path, PathBuf};
use std::process::Command;
use serde_json::Value;

const NARROW_PRE: &str = r#"{"type":"object","properties":{"a":{"enum":["x","y"]}}}"#;
const NARROW_POST: &str = r#"{"type":"object","properties":{"a":{"enum":["x"]}}}"#;

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_rd_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn pair(tag: &str, pre: &str, post: &str) -> PathBuf {
    let d = ws(tag);
    std::fs::write(d.join("base.json"), pre).unwrap();
    std::fs::write(d.join("head.json"), post).unwrap();
    d
}

fn invar(d: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_invar"));
    c.args(args).current_dir(d).env("HOME", d);
    for (k, v) in env {
        c.env(k, v);
    }
    let o = c.output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn reader(d: &Path, cmd: &str, extra: &[&str], env: &[(&str, &str)]) -> (i32, String) {
    let mut args = vec!["reader", "base.json", "head.json", "--cmd", cmd];
    args.extend_from_slice(extra);
    invar(d, &args, env)
}

/// rd_f01: the counterexample runs through the real command; exit 0 is
/// recorded, the control is recorded, and the schema verdict stands.
#[test]
fn rd_f01_exit_zero_is_recorded_and_the_verdict_stands() {
    let d = pair("f01", NARROW_PRE, NARROW_POST);
    let (code, out) = reader(&d, r#"python3 -I -c "import json,sys; json.load(open(sys.argv[1]))" {}"#, &[], &[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("VIOLATED"), "{out}");
    assert!(out.contains("reader: control"), "{out}");
    assert!(out.contains("exit 0 on a"), "{out}");
}

/// rd_f02: a nonzero exit is recorded with its code, and the control run
/// shows the command loads what the head accepts. Kills RD-M2 and RD-M3.
#[test]
fn rd_f02_nonzero_exit_is_recorded_with_its_code() {
    let d = pair("f02", NARROW_PRE, NARROW_POST);
    let (code, out) = reader(&d, r#"grep -q '"y"' {} && exit 7 || exit 0"#, &[], &[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("reader: control"), "{out}");
    assert!(out.contains("control"), "{out}");
    assert!(out.contains("exit 0"), "{out}");
    assert!(out.contains("exit 7 on a"), "{out}");
    assert!(!out.contains("exit 7 on a: control"), "{out}");
}

/// rd_f03 guard: the reader never moves the verdict or the exit code; the
/// JSON surface is the judge's document plus a `reader` object.
#[test]
fn rd_f03_reader_never_moves_the_verdict() {
    let d = pair("f03", NARROW_PRE, NARROW_POST);
    let (jcode, jout) = invar(&d, &["judge", "base.json", "head.json", "--format", "json"], &[]);
    let (rcode, rout) = reader(&d, "exit 0", &["--format", "json"], &[]);
    assert_eq!(jcode, rcode, "judge: {jout}\nreader: {rout}");
    let jv: Value = serde_json::from_str(&jout).unwrap_or_else(|e| panic!("not JSON ({e}): {jout}"));
    let rv: Value = serde_json::from_str(&rout).unwrap_or_else(|e| panic!("not JSON ({e}): {rout}"));
    assert_eq!(jv["verdict"], rv["verdict"], "{rv}");
    assert_eq!(jv["witnesses"], rv["witnesses"], "{rv}");
    assert!(rv["reader"].is_object(), "{rv}");
    assert!(jv.get("reader").is_none(), "{jv}");
}

/// rd_f04: the command runs with a cleared environment: a variable set on
/// invar's own process must be absent, and HOME must be a fresh directory,
/// not the caller's. Kills RD-M1.
#[test]
fn rd_f04_environment_is_cleared() {
    let d = pair("f04", NARROW_PRE, NARROW_POST);
    let parent_home = d.display().to_string();
    let cmd = format!(r#"test -z "$LEAKY_CREDENTIAL" && test "$HOME" != "{parent_home}""#);
    let (code, out) = reader(&d, &cmd, &[], &[("LEAKY_CREDENTIAL", "sk-secret")]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("exit 0 on a"), "a cleared environment must make the probe exit 0: {out}");
}

/// rd_f05: a command that outlives --timeout is killed and recorded as
/// timed out, never as a verdict about the document. Kills RD-M4.
#[test]
fn rd_f05_timeout_is_enforced_and_stated() {
    let d = pair("f05", NARROW_PRE, NARROW_POST);
    let (code, out) = reader(&d, "sleep 3", &["--timeout", "1"], &[]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("timed out after 1s"), "{out}");
}

/// rd_f06: a pair with nothing to run (NOT EVALUATED, no counterexample)
/// runs nothing, states it, and keeps the judge's exit.
#[test]
fn rd_f06_nothing_to_run_is_stated() {
    let pre = r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{3}$"}}}"#;
    let post = r#"{"properties":{"code":{"type":"string","pattern":"^[A-Z]{2}$"}}}"#;
    let d = pair("f06", pre, post);
    let (code, out) = reader(&d, "exit 0", &[], &[]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("reader: no counterexamples to run"), "{out}");
}

/// rd_f07: the --out report carries the canonical footer and verifies;
/// verification says the program result is recorded, not re-derived.
#[test]
fn rd_f07_report_verifies_and_states_its_limit() {
    let d = pair("f07", NARROW_PRE, NARROW_POST);
    let (code, out) = reader(&d, "exit 0", &["--out", "r.json"], &[]);
    assert_eq!(code, 1, "{out}");
    let (vcode, vout) = invar(&d, &["verify-receipt", "r.json"], &[]);
    assert_eq!(vcode, 0, "{vout}");
    assert!(vout.contains("recorded, not re-derived"), "{vout}");
}

/// rd_f08: editing a schema file after the report is written is input
/// drift on verification.
#[test]
fn rd_f08_schema_drift_fails_verification() {
    let d = pair("f08", NARROW_PRE, NARROW_POST);
    let (code, _) = reader(&d, "exit 0", &["--out", "r.json"], &[]);
    assert_eq!(code, 1);
    std::fs::write(d.join("head.json"), NARROW_PRE).unwrap();
    let (vcode, vout) = invar(&d, &["verify-receipt", "r.json"], &[]);
    assert_eq!(vcode, 1, "{vout}");
    assert!(vout.contains("drift"), "{vout}");
}
