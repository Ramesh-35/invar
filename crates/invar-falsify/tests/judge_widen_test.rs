//! JUDGE-WIDEN r0 fixtures (docs/falsification/JUDGE-WIDEN-SPEC.md),
//! frozen with the spec BEFORE implementation. D7: a type change that
//! keeps every value valid before is a stated note, not a break; a type
//! change that drops any type stays VIOLATED.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - W-M1 widening check dropped                 -> w_f07
//! - W-M2 integer not contained in number        -> w_f02
//! - W-M3 any shared type counts as widening     -> w_f10
//! - W-M4 widened note dropped (silent HOLDS)    -> w_f01

use std::path::{Path, PathBuf};
use std::process::Command;

const B1_001_PRE: &str = include_str!("fixtures/judge/b1-001-pre.json");
const B1_001_POST: &str = include_str!("fixtures/judge/b1-001-post.json");
const B1_002_PRE: &str = include_str!("fixtures/judge/b1-002-pre.json");
const B1_002_POST: &str = include_str!("fixtures/judge/b1-002-post.json");

fn repo(tag: &str, pre: &str, post: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_w_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    let git = |args: &[&str]| {
        let o = Command::new("git").args(args).current_dir(&d)
            .env("GIT_AUTHOR_NAME", "w").env("GIT_AUTHOR_EMAIL", "w@test")
            .env("GIT_COMMITTER_NAME", "w").env("GIT_COMMITTER_EMAIL", "w@test")
            .output().unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    };
    git(&["init", "-q", "-b", "main"]);
    std::fs::write(d.join("schemas/contract.json"), pre).unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "base"]);
    std::fs::write(d.join("schemas/contract.json"), post).unwrap();
    d
}

fn judge(d: &Path, format: &str) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["judge", "--contract", "schemas/contract.json", "--base", "main", "--format", format])
        .current_dir(d).env("HOME", d).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn field(t: &str) -> String {
    format!(r#"{{"properties":{{"f":{{"type":{t}}}}}}}"#)
}

/// w_f01: `boolean` to `boolean|null` keeps every value valid. Kills W-M4.
#[test]
fn w_f01_nullable_widening_is_a_note() {
    let d = repo("f01", &field(r#""boolean""#), &field(r#"["boolean","null"]"#));
    let (code, out) = judge(&d, "text");
    assert_eq!(code, 0, "{out}");
    assert!(out.starts_with("HOLDS: "), "{out}");
    assert!(out.contains("  noted: retyped (widened): f was boolean, is now boolean|null; every value valid before is still valid.\n"), "{out}");
}

/// w_f02: every integer is a number. Kills W-M2.
#[test]
fn w_f02_integer_to_number_is_a_note() {
    let d = repo("f02", &field(r#""integer""#), &field(r#""number""#));
    let (code, out) = judge(&d, "text");
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("retyped (widened): f was integer, is now number"), "{out}");
}

/// w_f03: `number` to `integer` rejects 1.5: a break, in the D7 sentence.
#[test]
fn w_f03_number_to_integer_is_violated() {
    let d = repo("f03", &field(r#""number""#), &field(r#""integer""#));
    let (code, out) = judge(&d, "text");
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("  retyped: f was number, is now integer. Values outside integer, valid before, are now rejected; restore the type, or version the change.\n"), "{out}");
}

/// w_f04: dropping `null` rejects null.
#[test]
fn w_f04_dropping_null_is_violated() {
    let d = repo("f04", &field(r#"["boolean","null"]"#), &field(r#""boolean""#));
    let (code, out) = judge(&d, "text");
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("retyped: f was boolean|null, is now boolean."), "{out}");
}

/// w_f05: disjoint types.
#[test]
fn w_f05_array_to_object_is_violated() {
    let d = repo("f05", &field(r#""array""#), &field(r#""object""#));
    let (code, _) = judge(&d, "text");
    assert_eq!(code, 1);
}

/// w_f06: dropping `type` (no combinator) accepts every value.
#[test]
fn w_f06_type_dropped_is_a_note() {
    let d = repo("f06", &field(r#""string""#), r#"{"properties":{"f":{"description":"any"}}}"#);
    let (code, out) = judge(&d, "text");
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("retyped (widened): f was string, is now unspecified"), "{out}");
}

/// w_f07: the real b1-002 change widens. Kills W-M1.
#[test]
fn w_f07_b1_002_holds_with_note() {
    let d = repo("f07", B1_002_PRE, B1_002_POST);
    let (code, out) = judge(&d, "text");
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("retyped (widened): deviceMemory was integer|null, is now number|null"), "{out}");
}

/// w_f08: the real b1-001 change breaks 6 promises; 4 retypes only widen.
#[test]
fn w_f08_b1_001_breaks_six() {
    let d = repo("f08", B1_001_PRE, B1_001_POST);
    let (code, out) = judge(&d, "text");
    assert_eq!(code, 1, "{out}");
    assert!(out.starts_with("VIOLATED: schemas/contract.json breaks 6 promises"), "{out}");
    for f in ["loop", "muted", "playbackRate", "volume"] {
        assert!(out.contains(&format!("retyped (widened): {f} was ")), "{f}: {out}");
    }
    assert!(!out.contains("  retyped: "), "{out}");
}

/// w_f09: no annotation for a widening.
#[test]
fn w_f09_github_quiet_on_widening() {
    let d = repo("f09", B1_002_PRE, B1_002_POST);
    let (code, out) = judge(&d, "github");
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("::error"), "{out}");
}

/// w_f10: sharing a type is not enough; `string` is dropped. Kills W-M3.
#[test]
fn w_f10_partial_overlap_is_violated() {
    let d = repo("f10", &field(r#"["integer","string"]"#), &field(r#""number""#));
    let (code, out) = judge(&d, "text");
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("retyped: f was integer|string, is now number."), "{out}");
}
