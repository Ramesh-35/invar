//! WITNESS r0 fixtures (docs/falsification/WITNESS-SPEC.md), frozen BEFORE
//! implementation: `invar witness` writes each counterexample the judge
//! carries as a SchemaStore test file.
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - W0-M1 the negative directory written under `src/test`      -> w_f01
//! - W0-M2 a witness without a counterexample written as `null`  -> w_f04
//! - W0-M3 the slug not applied (raw path in the file name)      -> w_f05

use std::path::{Path, PathBuf};
use std::process::Command;

fn dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_w_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn witness(d: &Path, base: &str, head: &str, extra: &[&str]) -> (i32, String) {
    std::fs::write(d.join("a.json"), base).unwrap();
    std::fs::write(d.join("b.json"), head).unwrap();
    let mut args = vec!["witness", "a.json", "b.json", "--name", "n", "--out", "out"];
    args.extend_from_slice(extra);
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(&args).current_dir(d).env("HOME", d).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn files(d: &Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(p: &Path, root: &Path, out: &mut Vec<String>) {
        if let Ok(rd) = std::fs::read_dir(p) {
            for e in rd.flatten() {
                let q = e.path();
                if q.is_dir() { walk(&q, root, out) } else { out.push(q.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/")) }
            }
        }
    }
    walk(&d.join("out"), &d.join("out"), &mut out);
    out.sort();
    out
}

fn doc(node: &str) -> String {
    format!(r#"{{"type":"object","properties":{{"a":{node}}}}}"#)
}

/// w_f01: one enum narrowing, one negative test file. Kills W0-M1.
#[test]
fn w_f01_one_break_one_negative_file() {
    let d = dir("f01");
    let (code, text) = witness(&d, &doc(r#"{"enum":["a","b"]}"#), &doc(r#"{"enum":["a"]}"#), &[]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(files(&d), vec!["src/negative_test/n/invar-01-a.json"], "{text}");
    assert_eq!(std::fs::read_to_string(d.join("out/src/negative_test/n/invar-01-a.json")).unwrap(), "{\n  \"a\": \"b\"\n}\n");
    assert!(text.contains("wrote src/negative_test/n/invar-01-a.json: narrowed a, valid at a.json, rejected at b.json\n"), "{text}");
    assert!(text.contains("verdict VIOLATED: 1 document(s) written, 0 witness(es) without one\n"), "{text}");
}

/// w_f02: --positive puts the same document under src/test.
#[test]
fn w_f02_positive_goes_under_src_test() {
    let d = dir("f02");
    let (code, text) = witness(&d, &doc(r#"{"enum":["a","b"]}"#), &doc(r#"{"enum":["a"]}"#), &["--positive"]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(files(&d), vec!["src/test/n/invar-01-a.json"], "{text}");
}

/// w_f03: HOLDS writes nothing and says so.
#[test]
fn w_f03_holds_writes_nothing() {
    let d = dir("f03");
    let (code, text) = witness(&d, &doc(r#"{"enum":["a"]}"#), &doc(r#"{"enum":["a","b"]}"#), &[]);
    assert_eq!(code, 0, "{text}");
    assert!(files(&d).is_empty(), "{text}");
    assert!(text.contains("verdict HOLDS: 0 document(s) written, 0 witness(es) without one\n"), "{text}");
}

/// w_f04: a witness without a counterexample is stated, never written. Kills W0-M2.
#[test]
fn w_f04_witness_without_document_is_stated() {
    let d = dir("f04");
    let (code, text) = witness(&d, &doc(r#"{"type":"string","pattern":"^a"}"#), &doc(r#"{"type":"string","pattern":"^b"}"#), &[]);
    assert_eq!(code, 0, "{text}");
    assert!(files(&d).is_empty(), "{text}");
    assert!(text.contains("no document for constraint_changed a: "), "{text}");
    assert!(text.contains("0 document(s) written, 1 witness(es) without one\n"), "{text}");
}

/// w_f05: two breaks, numbered in the judge's order; `[]` slugs to `items`. Kills W0-M3.
#[test]
fn w_f05_two_breaks_numbered_and_slugged() {
    let d = dir("f05");
    let pre = r#"{"type":"object","properties":{"a":{"enum":["x","y"]},"t":{"type":"array","items":{"enum":["p","q"]}}}}"#;
    let post = r#"{"type":"object","properties":{"a":{"enum":["x"]},"t":{"type":"array","items":{"enum":["p"]}}}}"#;
    let (code, text) = witness(&d, pre, post, &[]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(files(&d), vec!["src/negative_test/n/invar-01-a.json", "src/negative_test/n/invar-02-t-items.json"], "{text}");
    assert_eq!(std::fs::read_to_string(d.join("out/src/negative_test/n/invar-02-t-items.json")).unwrap(), "{\n  \"t\": [\n    \"q\"\n  ]\n}\n");
}

/// w_f06 guard: a name that is not one path component is refused before anything is written.
#[test]
fn w_f06_name_with_separator_is_refused() {
    let d = dir("f06");
    std::fs::write(d.join("a.json"), doc(r#"{"enum":["a","b"]}"#)).unwrap();
    std::fs::write(d.join("b.json"), doc(r#"{"enum":["a"]}"#)).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(["witness", "a.json", "b.json", "--name", "a/b", "--out", "out"]).current_dir(&d).env("HOME", &d).output().unwrap();
    assert_eq!(o.status.code(), Some(3), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(!d.join("out").exists());
}

/// w_f07: a second run leaves identical bytes.
#[test]
fn w_f07_idempotent() {
    let d = dir("f07");
    let (_, _) = witness(&d, &doc(r#"{"enum":["a","b"]}"#), &doc(r#"{"enum":["a"]}"#), &[]);
    let first = std::fs::read(d.join("out/src/negative_test/n/invar-01-a.json")).unwrap();
    let (code, text) = witness(&d, &doc(r#"{"enum":["a","b"]}"#), &doc(r#"{"enum":["a"]}"#), &[]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(std::fs::read(d.join("out/src/negative_test/n/invar-01-a.json")).unwrap(), first);
    assert_eq!(files(&d).len(), 1);
}
