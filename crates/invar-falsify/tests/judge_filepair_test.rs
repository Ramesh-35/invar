//! JUDGE-FILEPAIR r0 fixtures (docs/falsification/JUDGE-FILEPAIR-SPEC.md),
//! frozen BEFORE implementation: `invar judge <base.json> <head.json>`
//! judges two files directly, with no git anywhere, through the same
//! engine and with the same verdicts as the repository mode. The first
//! path is the base (the promise), the second is the head (the change),
//! and the paths as given are the labels in every output.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - FP-M1 file order swapped (head judged as the base) -> fp_f01
//! - FP-M2 base label falls back to "main"              -> fp_f01
//! - FP-M3 the base read from the head path             -> fp_f10

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const PRE_CLOSED: &str =
    r#"{"additionalProperties":false,"required":["id"],"properties":{"id":{"type":"string"},"age":{"type":"integer"}}}"#;
const POST_DROPPED: &str =
    r#"{"additionalProperties":false,"required":["id"],"properties":{"id":{"type":"string"}}}"#;

fn dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_fp_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Runs `invar judge` with the given arguments in `d`, which is never a
/// git repository: the two-file mode must not need one.
fn judge(d: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .arg("judge").args(args)
        .current_dir(d).env("HOME", d).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn pair(tag: &str, base: &str, head: &str) -> PathBuf {
    let d = dir(tag);
    std::fs::write(d.join("a.json"), base).unwrap();
    std::fs::write(d.join("b.json"), head).unwrap();
    d
}

/// fp_f01: a removed field in a closed object is a break, the labels are
/// the file paths, and the counterexample is printed. Kills FP-M1 (the
/// swapped order would judge an ADDED field: HOLDS) and FP-M2 (the base
/// label would read "main").
#[test]
fn fp_f01_removed_field_violated_with_file_labels() {
    let d = pair("f01", PRE_CLOSED, POST_DROPPED);
    let (code, text) = judge(&d, &["a.json", "b.json"]);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("VIOLATED: b.json breaks 1 promise made at a.json.\n"), "{text}");
    assert!(text.contains("  removed: age was integer (line 1 at a.json), from an object that admits no other keys. Instances that carry it, valid before, are now rejected; keep the field, or version the change.\n"), "{text}");
    assert!(text.contains("    counterexample (valid at a.json, rejected now): {\"age\":0,\"id\":\"\"}\n"), "{text}");
}

/// fp_f02: identical files hold, worded with the file labels.
#[test]
fn fp_f02_identical_files_hold() {
    let d = pair("f02", PRE_CLOSED, PRE_CLOSED);
    let (code, text) = judge(&d, &["a.json", "b.json"]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("HOLDS: b.json keeps every promise it made at a.json.\n"), "{text}");
}

/// fp_f03: D7 parity, a widening is a stated note, not a break.
#[test]
fn fp_f03_widened_is_a_note() {
    let post = r#"{"additionalProperties":false,"required":["id"],"properties":{"id":{"type":"string"},"age":{"type":["integer","null"]}}}"#;
    let d = pair("f03", PRE_CLOSED, post);
    let (code, text) = judge(&d, &["a.json", "b.json"]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("  noted: retyped (widened): age was integer, is now integer|null; every value valid before is still valid.\n"), "{text}");
}

/// fp_f04: D9 parity, a type the enum already implies is a stated note.
#[test]
fn fp_f04_enum_covered_type_is_a_note() {
    let pre = r#"{"properties":{"f":{"enum":["a","b"]}}}"#;
    let post = r#"{"properties":{"f":{"type":"string","enum":["a","b"]}}}"#;
    let d = pair("f04", pre, post);
    let (code, text) = judge(&d, &["a.json", "b.json"]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("  noted: type made explicit: f is now string, which every value of its enum already is; every value valid before is still valid.\n"), "{text}");
}

/// fp_f05: a missing base file is an error, never a verdict.
#[test]
fn fp_f05_missing_base_is_an_error() {
    let d = dir("f05");
    std::fs::write(d.join("b.json"), POST_DROPPED).unwrap();
    let (code, text) = judge(&d, &["a.json", "b.json"]);
    assert_eq!(code, 3, "{text}");
    assert!(text.contains("ERROR (not a verdict): reading a.json:"), "{text}");
}

/// fp_f06: a head that is not JSON is an error, never a verdict.
#[test]
fn fp_f06_non_json_head_is_an_error() {
    let d = pair("f06", PRE_CLOSED, "not json [[[");
    let (code, text) = judge(&d, &["a.json", "b.json"]);
    assert_eq!(code, 3, "{text}");
    assert!(text.contains("ERROR (not a verdict): b.json is not JSON:"), "{text}");
}

/// fp_f07: the two modes do not mix; saying both is an error that names
/// the conflict, never a silent choice.
#[test]
fn fp_f07_mixing_modes_is_an_error() {
    let d = pair("f07", PRE_CLOSED, POST_DROPPED);
    let (code, text) = judge(&d, &["a.json", "b.json", "--contract", "a.json"]);
    assert_eq!(code, 3, "{text}");
    assert!(text.contains("ERROR (not a verdict): two files and --contract cannot be combined: judge the two files, or judge a contract in this repository."), "{text}");
    let (code, text) = judge(&d, &["a.json", "b.json", "--base", "main"]);
    assert_eq!(code, 3, "{text}");
    assert!(text.contains("ERROR (not a verdict): two files and --base cannot be combined: the first file is the base."), "{text}");
}

/// fp_f08: epistemic parity. The same bytes judged in a git repository
/// and as a file pair give the same verdict and the same witnesses,
/// counterexamples included; only the labels differ.
#[test]
fn fp_f08_parity_with_repository_mode() {
    let pre = r#"{"oneOf":[{"required":["f"]},{"required":["g"]}],"properties":{"f":{"type":"integer","maximum":100}}}"#;
    let post = r#"{"oneOf":[{"required":["f"]},{"required":["g"]}],"properties":{"f":{"type":"integer","maximum":10}}}"#;
    // Repository half.
    let d = dir("f08_repo");
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    let git = |args: &[&str]| {
        let o = Command::new("git").args(args).current_dir(&d)
            .env("GIT_AUTHOR_NAME", "fp").env("GIT_AUTHOR_EMAIL", "fp@test")
            .env("GIT_COMMITTER_NAME", "fp").env("GIT_COMMITTER_EMAIL", "fp@test")
            .output().unwrap();
        assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    };
    git(&["init", "-q", "-b", "main"]);
    std::fs::write(d.join("schemas/contract.json"), pre).unwrap();
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "base"]);
    std::fs::write(d.join("schemas/contract.json"), post).unwrap();
    let (rc, rout) = judge(&d, &["--contract", "schemas/contract.json", "--base", "main", "--format", "json"]);
    // File-pair half, in a directory with no git anywhere.
    let p = pair("f08_pair", pre, post);
    let (fc, fout) = judge(&p, &["a.json", "b.json", "--format", "json"]);
    assert_eq!(rc, 1, "{rout}");
    assert_eq!(fc, 1, "{fout}");
    let strip = |s: &str| -> Value {
        let mut v: Value = serde_json::from_str(s).unwrap_or_else(|e| panic!("not JSON ({e}): {s}"));
        let o = v.as_object_mut().unwrap();
        o.remove("base");
        o.remove("contract");
        for w in o.get_mut("witnesses").unwrap().as_array_mut().unwrap() {
            w.as_object_mut().unwrap().remove("contract");
        }
        v
    };
    assert_eq!(strip(&rout), strip(&fout), "repo:\n{rout}\npair:\n{fout}");
}

/// fp_f09: the JSON surface carries the file labels, and a pattern change
/// stays undecided, with the same reason as the repository mode.
#[test]
fn fp_f09_json_labels_and_undecided_pattern() {
    let pre = r#"{"properties":{"f":{"type":"string"}}}"#;
    let post = r#"{"properties":{"f":{"type":"string","pattern":"^a"}}}"#;
    let d = pair("f09", pre, post);
    let (code, out) = judge(&d, &["a.json", "b.json", "--format", "json"]);
    assert_eq!(code, 1, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}"));
    assert_eq!(v["base"], json!("a.json"), "{v}");
    assert_eq!(v["contract"], json!("b.json"), "{v}");
    assert_eq!(v["witnesses"][0]["kind"], json!("constraint_changed"), "{v}");
    assert_eq!(v["witnesses"][0]["counterexample"], Value::Null, "{v}");
    assert_eq!(v["witnesses"][0]["counterexample_reason"], json!("pattern and format changes are not decided"), "{v}");
}

/// fp_f10: exit 4 parity, a change only where the judge does not look is
/// NOT EVALUATED, never a pass. Kills FP-M3 (a base read from the head
/// path would see no change at all and hold).
#[test]
fn fp_f10_unjudged_construct_is_not_evaluated() {
    let pre = r#"{"additionalProperties":false,"properties":{"id":{"type":"string"},"age":{"type":"integer"}}}"#;
    let post = r#"{"additionalProperties":false,"patternProperties":{"^x":{"type":"string"}},"properties":{"id":{"type":"string"}}}"#;
    let d = pair("f10", pre, post);
    let (code, text) = judge(&d, &["a.json", "b.json"]);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains("NOT EVALUATED (unjudged_construct): b.json changed only where this judge does not look"), "{text}");
}

/// fp_f11: receipts stay repository-bound in r0; asking is an error.
#[test]
fn fp_f11_receipt_is_refused() {
    let d = pair("f11", PRE_CLOSED, POST_DROPPED);
    let (code, text) = judge(&d, &["a.json", "b.json", "--receipt", "r.json"]);
    assert_eq!(code, 3, "{text}");
    assert!(text.contains("ERROR (not a verdict): --receipt needs a repository; judge two files without it."), "{text}");
    assert!(!d.join("r.json").exists());
}
