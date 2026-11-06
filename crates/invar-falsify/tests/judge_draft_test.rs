//! JUDGE-DRAFT r0 fixtures (docs/falsification/JUDGE-DRAFT-SPEC.md), frozen
//! BEFORE implementation: a `$schema` move among draft-04, -06 and -07,
//! with the document's identifier unchanged (`id` renamed `$id` with the
//! same value), is covered and stated, when neither document uses a keyword
//! whose meaning differs between those drafts. d_f11 is real bytes from
//! SchemaStore at 8cfea30d1c0a07914a6978ef9dde40d264ceec88 (Apache-2.0).
//!
//! Mutation operators (each named killer must fail under the mutant):
//! - DR-M1 draft migration never covered                   -> d_f01
//! - DR-M2 identifier equality not checked                  -> d_f02
//! - DR-M3 guard keywords not checked                       -> d_f03
//! - DR-M4 every boolean read as a draft-06 subschema       -> d_f07

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

const WEBJOBS_04: &str = include_str!("fixtures/judge/schemastore-webjobs-list-draft04.json");
const WEBJOBS_07: &str = include_str!("fixtures/judge/schemastore-webjobs-list-draft07.json");

fn pair(tag: &str, base: &str, head: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_dr_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("a.json"), base).unwrap();
    std::fs::write(d.join("b.json"), head).unwrap();
    d
}

fn judge(d: &Path, format: &str) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["judge", "a.json", "b.json", "--format", format])
        .current_dir(d).env("HOME", d).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn both(tag: &str, base: &str, head: &str) -> (i32, String, Value) {
    let d = pair(tag, base, head);
    let (code, text) = judge(&d, "text");
    let (_, j) = judge(&d, "json");
    (code, text, serde_json::from_str(&j).unwrap_or_else(|e| panic!("not JSON ({e}): {j}")))
}

/// A draft-04 document and its draft-07 migration; `extra` is spliced into
/// both roots' properties so each fixture varies one thing.
fn migration(extra: &str, base_id: &str, head_id: &str, head_draft: &str) -> (String, String) {
    (
        format!(r#"{{"$schema":"http://json-schema.org/draft-04/schema#","id":"{base_id}","type":"object","properties":{{"a":{{"type":"string"}}{extra}}}}}"#),
        format!(r#"{{"$schema":"{head_draft}","$id":"{head_id}","type":"object","properties":{{"a":{{"type":"string"}}{extra}}}}}"#),
    )
}

const D07: &str = "http://json-schema.org/draft-07/schema#";
const ID: &str = "https://example.com/s.json";
const ROOT_LINE: &str = "  not evaluated: the document root changed in a way this judge does not check.\n";
const NOTE_04_07: &str = "  noted: draft migration: $schema moves from draft-04 to draft-07, with id renamed $id (same value); no keyword either version uses means something different between those drafts.\n";

/// d_f01: the plain migration holds, and says why. Kills DR-M1.
#[test]
fn d_f01_plain_migration_holds_and_is_stated() {
    let (pre, post) = migration("", ID, ID, D07);
    let (code, text, v) = both("f01", &pre, &post);
    assert_eq!(code, 0, "{text}");
    assert!(text.starts_with("HOLDS: b.json keeps every promise it made at a.json.\n"), "{text}");
    assert!(text.contains(NOTE_04_07), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// d_f02: the identifier changed, so the document is not the same one
/// moved to a new draft. Kills DR-M2.
#[test]
fn d_f02_identifier_changed_is_not_covered() {
    let (pre, post) = migration("", ID, "https://example.com/other.json", D07);
    let (code, text, _) = both("f02", &pre, &post);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(ROOT_LINE), "{text}");
}

/// d_f03: `const` is ignored by draft-04 and asserted by draft-07. Kills DR-M3.
#[test]
fn d_f03_const_blocks_the_migration() {
    let (pre, post) = migration(r#","k":{"const":"x"}"#, ID, ID, D07);
    let (code, text, _) = both("f03", &pre, &post);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(ROOT_LINE), "{text}");
}

/// d_f04: exclusiveMinimum is a boolean in draft-04 and a number after.
#[test]
fn d_f04_exclusive_bound_blocks_the_migration() {
    let (pre, post) = migration(r#","n":{"type":"number","minimum":0,"exclusiveMinimum":true}"#, ID, ID, D07);
    let (code, text, _) = both("f04", &pre, &post);
    assert_eq!(code, 4, "{text}");
}

/// d_f05: a boolean subschema is a draft-06 construct.
#[test]
fn d_f05_boolean_subschema_blocks_the_migration() {
    let (pre, post) = migration(r#","t":true"#, ID, ID, D07);
    let (code, text, _) = both("f05", &pre, &post);
    assert_eq!(code, 4, "{text}");
}

/// d_f06: a non-local $ref resolves against the identifier's base URI.
#[test]
fn d_f06_non_local_ref_blocks_the_migration() {
    let (pre, post) = migration(r#","r":{"$ref":"other.json#/x"}"#, ID, ID, D07);
    let (code, text, _) = both("f06", &pre, &post);
    assert_eq!(code, 4, "{text}");
}

/// d_f07 guard: additionalProperties false is legal draft-04; it does not
/// block. Kills DR-M4.
#[test]
fn d_f07_draft04_booleans_do_not_block() {
    let pre = format!(r#"{{"$schema":"http://json-schema.org/draft-04/schema#","id":"{ID}","type":"object","additionalProperties":false,"properties":{{"a":{{"type":"string"}}}}}}"#);
    let post = format!(r#"{{"$schema":"{D07}","$id":"{ID}","type":"object","additionalProperties":false,"properties":{{"a":{{"type":"string"}}}}}}"#);
    let (code, text, _) = both("f07", &pre, &post);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(NOTE_04_07), "{text}");
}

/// d_f08: 2020-12 is out of scope in r0.
#[test]
fn d_f08_draft_2020_12_is_not_covered() {
    let (pre, post) = migration("", ID, ID, "https://json-schema.org/draft/2020-12/schema");
    let (code, text, _) = both("f08", &pre, &post);
    assert_eq!(code, 4, "{text}");
    assert!(text.contains(ROOT_LINE), "{text}");
}

/// d_f09: a migration alongside a real break: the break is judged, the
/// migration stated, nothing unchecked.
#[test]
fn d_f09_migration_beside_a_break() {
    let pre = format!(r#"{{"$schema":"http://json-schema.org/draft-04/schema#","id":"{ID}","properties":{{"a":{{"type":"string"}}}}}}"#);
    let post = format!(r#"{{"$schema":"{D07}","$id":"{ID}","properties":{{"a":{{"type":"integer"}}}}}}"#);
    let (code, text, v) = both("f09", &pre, &post);
    assert_eq!(code, 1, "{text}");
    assert_eq!(v["witnesses"][0]["kind"], json!("retyped"), "{v}");
    assert!(text.contains(NOTE_04_07), "{text}");
    assert_eq!(v["unchecked"], json!([]), "{v}");
}

/// d_f10: covering the root's migration hides nothing beneath it: an
/// unchecked change elsewhere is still named, by its own pointer.
#[test]
fn d_f10_migration_does_not_hide_unchecked_changes() {
    let pre = format!(r#"{{"$schema":"http://json-schema.org/draft-04/schema#","id":"{ID}","properties":{{"a":{{"type":"string"}}}},"definitions":{{"U":{{"type":"string"}}}}}}"#);
    let post = format!(r#"{{"$schema":"{D07}","$id":"{ID}","properties":{{"a":{{"type":"string"}}}},"definitions":{{"U":{{"type":"integer"}}}}}}"#);
    let (code, text, v) = both("f10", &pre, &post);
    assert_eq!(code, 4, "{text}");
    assert_eq!(v["unchecked"], json!(["/definitions/U"]), "{v}");
}

/// d_f11: real SchemaStore bytes, "Migrated Azure Webjobs to draft-07".
#[test]
fn d_f11_real_schemastore_migration_holds() {
    let (code, text, _) = both("f11", WEBJOBS_04, WEBJOBS_07);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(NOTE_04_07), "{text}");
}

/// d_f12: the `$schema` URI's trailing `#` and http(s) scheme do not
/// change the draft it names.
#[test]
fn d_f12_uri_spelling_does_not_matter() {
    let (pre, post) = migration("", ID, ID, "https://json-schema.org/draft-07/schema");
    let (code, text, _) = both("f12", &pre, &post);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains(NOTE_04_07), "{text}");
}
