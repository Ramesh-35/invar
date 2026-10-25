//! G2 fixtures (RECALL-ATTACK-1 phase 4, docs/falsification/G2-SPEC.md r0),
//! frozen with the spec BEFORE any implementation. Change-scoped workspaces.
//!
//! G2-A convicts (VIOLATED) a removed Go struct field that a Go consumer
//! reads by its Go identifier (`repo.Link`) when the field's JSON key
//! differs (`link_url`): the read field is absent at head under both names.
//! G2-R annotates a conviction of absence with a CANDIDATE rename, stated
//! as not evaluated, only when one removed field and one added field pair
//! uniquely in the same scope with the same known type. A candidate never
//! changes a verdict.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_g2_{tag}_{}_{nanos}", std::process::id()));
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    dir
}

fn invar(dir: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(dir).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn run(tag: &str, ext: &str, base: &str, head: &str, consumer_name: &str, consumer: &str) -> (i32, String) {
    let (h, b) = (format!("head.{ext}"), format!("base.{ext}"));
    let t = format!("[[invariant]]\nkind = \"schema\"\nid = \"g2\"\nproducer = \"{h}\"\nproducer_base = \"{b}\"\nconsumer = \"{consumer_name}\"\n");
    let d = ws(tag, &[(h.as_str(), head), (b.as_str(), base), (consumer_name, consumer), ("invar.toml", &t)]);
    invar(&d, &["check", "--no-reuse"])
}

const CAND: &str = "candidate rename";

/// The r1-076 shape: Go field `Link` (json `link_url`) became `ForgeURL`
/// (json `forge_url`), same type, same struct.
const GO_BASE: &str = "package model\n\ntype Repo struct {\n\tName string `json:\"name\"`\n\tLink string `json:\"link_url,omitempty\"`\n}\n";
const GO_HEAD: &str = "package model\n\ntype Repo struct {\n\tName     string `json:\"name\"`\n\tForgeURL string `json:\"forge_url,omitempty\"`\n}\n";
/// A Go consumer reading the Go identifier on line 4.
const GO_READS: &str = "package provider\n\nfunc fill(m *Model, repo *woodpecker.Repo) {\n\tm.URL = types.StringValue(repo.Link)\n}\n";

// ------------------------------------------------------------------- G2-A

#[test]
fn g2_f01_a_removed_go_field_read_by_its_go_name_convicts() {
    let (code, out) = run("f01", "go", GO_BASE, GO_HEAD, "c.go", GO_READS);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] g2 (schema): [\"link_url\"]"), "{out}");
    assert!(
        out.contains("link_url read at c.go:4 · not in head.go · candidate rename: forge_url (same scope, same type string), not evaluated · naming differs, read as Link"),
        "{out}"
    );
}

#[test]
fn g2_f02_a_non_go_consumer_reading_the_go_name_is_not_a_read() {
    let consumer = "export const url = (repo) => repo.Link;\n";
    let (code, out) = run("f02", "go", GO_BASE, GO_HEAD, "c.ts", consumer);
    assert_eq!(code, 0, "a JSON consumer cannot see the Go identifier:\n{out}");
}

#[test]
fn g2_f03_an_unexported_struct_is_not_readable_by_another_module() {
    let base = GO_BASE.replace("type Repo struct", "type repo struct");
    let head = GO_HEAD.replace("type Repo struct", "type repo struct");
    let (code, out) = run("f03", "go", &base, &head, "c.go", GO_READS);
    assert_eq!(code, 0, "{out}");
}

#[test]
fn g2_f04_a_surviving_go_name_is_not_a_removal_for_go_readers() {
    // Only the JSON tag changed; the Go identifier `Link` still exists.
    let head = GO_BASE.replace("json:\"link_url,omitempty\"", "json:\"forge_url,omitempty\"");
    let (code, out) = run("f04", "go", GO_BASE, &head, "c.go", GO_READS);
    assert_eq!(code, 0, "{out}");
}

#[test]
fn g2_f05_go_identifiers_match_exactly() {
    // `repo.link` is not the Go field `Link` (Go identifiers are case
    // sensitive, and the exported field is capitalized).
    let consumer = GO_READS.replace("repo.Link", "repo.link");
    let (code, out) = run("f05", "go", GO_BASE, GO_HEAD, "c.go", &consumer);
    assert_eq!(code, 0, "{out}");
}

// ------------------------------------------------------------------- G2-R

/// TypeScript producer: `FILESIZE` became `TOTAL_FILESIZE_BYTES`, same type.
const TS_BASE: &str = "export interface Stats {\n  FILESIZE: number;\n  PLATFORMS: number;\n}\n";
const TS_HEAD: &str = "export interface Stats {\n  TOTAL_FILESIZE_BYTES: number;\n  PLATFORMS: number;\n}\n";
const TS_READS: &str = "export const size = (data) => data.FILESIZE;\n";

#[test]
fn g2_f07_a_unique_same_type_replacement_is_a_stated_candidate() {
    let (code, out) = run("f07", "ts", TS_BASE, TS_HEAD, "c.js", TS_READS);
    assert_eq!(code, 1, "the verdict is the existing conviction:\n{out}");
    assert!(out.contains("[VIOLATED] g2 (schema): [\"FILESIZE\"]"), "{out}");
    assert!(
        out.contains("FILESIZE read at c.js:1 · not in head.ts · candidate rename: TOTAL_FILESIZE_BYTES (same scope, same type number), not evaluated"),
        "{out}"
    );
}

#[test]
fn g2_f08_two_removed_fields_for_one_addition_name_no_candidate() {
    let base = "export interface Stats {\n  FILESIZE: number;\n  FILECOUNT: number;\n  PLATFORMS: string;\n}\n";
    let head = "export interface Stats {\n  TOTAL_FILESIZE_BYTES: number;\n  PLATFORMS: string;\n}\n";
    let (code, out) = run("f08", "ts", base, head, "c.js", TS_READS);
    assert_eq!(code, 1, "{out}");
    assert!(!out.contains(CAND), "ambiguous: two removed fields claim one addition:\n{out}");
}

#[test]
fn g2_f09_one_removed_field_with_two_additions_names_no_candidate() {
    let head = "export interface Stats {\n  TOTAL_FILESIZE_BYTES: number;\n  ARCHIVE_BYTES: number;\n  PLATFORMS: number;\n}\n";
    let (code, out) = run("f09", "ts", TS_BASE, head, "c.js", TS_READS);
    assert_eq!(code, 1, "{out}");
    assert!(!out.contains(CAND), "ambiguous: two additions fit one removal:\n{out}");
}

#[test]
fn g2_f10_a_different_type_is_no_candidate() {
    let head = TS_HEAD.replace("TOTAL_FILESIZE_BYTES: number", "TOTAL_FILESIZE_BYTES: string");
    let (code, out) = run("f10", "ts", TS_BASE, &head, "c.js", TS_READS);
    assert_eq!(code, 1, "{out}");
    assert!(!out.contains(CAND), "{out}");
}

#[test]
fn g2_f11_an_unknown_type_is_no_candidate() {
    // Python dict keys carry no declared type.
    let base = "def info(meta):\n    out = {}\n    out['stream_name'] = meta.stream\n    out['viewers'] = meta.count\n    return out\n";
    let head = "def info(meta):\n    out = {}\n    out['channel_name'] = meta.channel\n    out['viewers'] = meta.count\n    return out\n";
    let consumer = "export const t = (s) => s.stream_name;\n";
    let (code, out) = run("f11", "py", base, head, "c.js", consumer);
    assert_eq!(code, 1, "{out}");
    assert!(!out.contains(CAND), "{out}");
}

#[test]
fn g2_f12_a_replacement_in_another_scope_is_no_candidate() {
    let head = "export interface Stats {\n  PLATFORMS: number;\n}\nexport interface Totals {\n  TOTAL_FILESIZE_BYTES: number;\n}\n";
    let (code, out) = run("f12", "ts", TS_BASE, head, "c.js", TS_READS);
    assert_eq!(code, 1, "{out}");
    assert!(!out.contains(CAND), "{out}");
}

#[test]
fn g2_f13_a_rename_nobody_reads_says_nothing() {
    let consumer = "export const p = (data) => data.PLATFORMS;\n";
    let (code, out) = run("f13", "ts", TS_BASE, TS_HEAD, "c.js", consumer);
    assert_eq!(code, 0, "a candidate never creates a finding:\n{out}");
    assert!(!out.contains(CAND), "{out}");
}

// ------------------------------------------- interface and determinism

#[test]
fn g2_f14_strict_mode_is_untouched() {
    let t = "[[invariant]]\nkind = \"schema\"\nid = \"g2\"\nproducer = \"head.go\"\nconsumer = \"c.go\"\n";
    let d = ws("f14", &[("head.go", GO_HEAD), ("c.go", GO_READS), ("invar.toml", t)]);
    let (_, out) = invar(&d, &["check", "--no-reuse"]);
    assert!(!out.contains(CAND), "{out}");
}

#[test]
fn g2_f15_identical_bytes_give_identical_output() {
    let t = "[[invariant]]\nkind = \"schema\"\nid = \"g2\"\nproducer = \"head.go\"\nproducer_base = \"base.go\"\nconsumer = \"c.go\"\n";
    let d = ws("f15", &[("head.go", GO_HEAD), ("base.go", GO_BASE), ("c.go", GO_READS), ("invar.toml", t)]);
    let a = invar(&d, &["check", "--no-reuse"]);
    let b = invar(&d, &["check", "--no-reuse"]);
    assert_eq!(a, b);
}
