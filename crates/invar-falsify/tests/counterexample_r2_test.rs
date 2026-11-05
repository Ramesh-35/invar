//! COUNTEREXAMPLE r2 fixtures (docs/falsification/COUNTEREXAMPLE-R2-SPEC.md),
//! frozen BEFORE implementation: the checker evaluates allOf, anyOf, oneOf
//! and not; the generator builds members' required names and picks the
//! first valid anyOf/oneOf member. The expected instances were checked
//! with Python jsonschema at freeze (COUNTEREXAMPLE-R2-SPEC.frozen).
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - C-M1 anyOf read as unknown                         -> c_f02
//! - C-M2 oneOf counted as anyOf                        -> c_f04
//! - C-M3 not not inverted                              -> c_f05
//! - C-M4 allOf required not built                      -> c_f01
//! - C-M5 unknown reason replaced by the combinator     -> c_f07

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo(tag: &str, pre: &str, post: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_c_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join("schemas")).unwrap();
    let git = |args: &[&str]| {
        let o = Command::new("git").args(args).current_dir(&d)
            .env("GIT_AUTHOR_NAME", "c").env("GIT_AUTHOR_EMAIL", "c@test")
            .env("GIT_COMMITTER_NAME", "c").env("GIT_COMMITTER_EMAIL", "c@test")
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

fn judge(d: &Path) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["judge", "--contract", "schemas/contract.json", "--base", "main", "--format", "json"])
        .current_dir(d).env("HOME", d).output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned())
}

fn one(pre: &str, post: &str, tag: &str) -> (Value, Value) {
    let d = repo(tag, pre, post);
    let (code, out) = judge(&d);
    assert_eq!(code, 1, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}"));
    let w = v["witnesses"].as_array().expect("witnesses");
    assert_eq!(w.len(), 1, "{v}");
    (w[0]["counterexample"].clone(), w[0]["counterexample_reason"].clone())
}

const NO_CANDIDATE: &str = "no candidate value was valid at the base and rejected now";

/// A closed root that declares `id` (required, with the schema given) and
/// `age`; the post removes `age`.
fn with_id(id: &str) -> (String, String) {
    (
        format!(r#"{{"additionalProperties":false,"required":["id"],"properties":{{"id":{id},"age":{{"type":"integer"}}}}}}"#),
        format!(r#"{{"additionalProperties":false,"required":["id"],"properties":{{"id":{id}}}}}"#),
    )
}

/// c_f01: a name required by an allOf member is built. Kills C-M4.
#[test]
fn c_f01_all_of_required_built() {
    let pre = r#"{"additionalProperties":false,"allOf":[{"required":["name"]}],"properties":{"name":{"type":"string"},"age":{"type":"integer"}}}"#;
    let post = r#"{"additionalProperties":false,"allOf":[{"required":["name"]}],"properties":{"name":{"type":"string"}}}"#;
    assert_eq!(one(pre, post, "f01").0, json!({"age": 0, "name": ""}));
}

/// c_f02: anyOf is read. Kills C-M1.
#[test]
fn c_f02_any_of_read() {
    let (pre, post) = with_id(r#"{"anyOf":[{"type":"string"},{"type":"integer"}]}"#);
    assert_eq!(one(&pre, &post, "f02").0, json!({"age": 0, "id": ""}));
}

/// c_f03: the first member that is valid gives the value.
#[test]
fn c_f03_any_of_first_valid_member() {
    let (pre, post) = with_id(r#"{"anyOf":[{"type":"string","minLength":2},{"type":"integer"}]}"#);
    assert_eq!(one(&pre, &post, "f03").0, json!({"age": 0, "id": "aa"}));
}

/// c_f04: "" is valid under both members, so oneOf rejects it. Kills C-M2.
#[test]
fn c_f04_one_of_counted() {
    let (pre, post) = with_id(r#"{"oneOf":[{"type":"string"},{"type":"string","maxLength":5}]}"#);
    let (cx, why) = one(&pre, &post, "f04");
    assert_eq!(cx, Value::Null);
    assert_eq!(why, NO_CANDIDATE);
}

/// c_f05: null is not a string, so `not` accepts it. Kills C-M3.
#[test]
fn c_f05_not_read() {
    let (pre, post) = with_id(r#"{"not":{"type":"string"}}"#);
    assert_eq!(one(&pre, &post, "f05").0, json!({"age": 0, "id": null}));
}

/// c_f06: the generator does not solve `not`; the checker rejects.
#[test]
fn c_f06_not_unsolved() {
    let (pre, post) = with_id(r#"{"type":"string","not":{"maxLength":5}}"#);
    let (cx, why) = one(&pre, &post, "f06");
    assert_eq!(cx, Value::Null);
    assert_eq!(why, NO_CANDIDATE);
}

/// c_f07: the reason names the keyword inside, not the combinator.
/// Kills C-M5.
#[test]
fn c_f07_unknown_names_the_inner_keyword() {
    let (pre, post) = with_id(r#"{"anyOf":[{"type":"string","pattern":"^a"},{"type":"string","pattern":"^b"}]}"#);
    let (cx, why) = one(&pre, &post, "f07");
    assert_eq!(cx, Value::Null);
    assert_eq!(why, "pattern is not checked by this judge");
}

/// c_f08: oneOf at both refs, read: the tightening is confirmed.
#[test]
fn c_f08_one_of_read_at_both_refs() {
    let pre = r#"{"oneOf":[{"required":["f"]},{"required":["g"]}],"properties":{"f":{"type":"integer","maximum":100}}}"#;
    let post = r#"{"oneOf":[{"required":["f"]},{"required":["g"]}],"properties":{"f":{"type":"integer","maximum":10}}}"#;
    assert_eq!(one(pre, post, "f08").0, json!({"f": 11}));
}

/// c_f09: if/then/else stays unchecked.
#[test]
fn c_f09_if_then_else_still_unchecked() {
    let pre = r#"{"additionalProperties":false,"if":{"required":["name"]},"then":{"required":["age"]},"properties":{"name":{"type":"string"},"age":{"type":"integer"}}}"#;
    let post = r#"{"additionalProperties":false,"if":{"required":["name"]},"then":{"required":["age"]},"properties":{"name":{"type":"string"}}}"#;
    let (cx, why) = one(pre, post, "f09");
    assert_eq!(cx, Value::Null);
    assert_eq!(why, "if is not checked by this judge");
}

/// c_f10: an allOf member given by a local $ref is resolved and read.
#[test]
fn c_f10_all_of_member_by_local_ref() {
    let pre = r##"{"$defs":{"req":{"required":["name"]}},"additionalProperties":false,"allOf":[{"$ref":"#/$defs/req"}],"properties":{"name":{"type":"string"},"age":{"type":"integer"}}}"##;
    let post = r##"{"$defs":{"req":{"required":["name"]}},"additionalProperties":false,"allOf":[{"$ref":"#/$defs/req"}],"properties":{"name":{"type":"string"}}}"##;
    assert_eq!(one(pre, post, "f10").0, json!({"age": 0, "name": ""}));
}
