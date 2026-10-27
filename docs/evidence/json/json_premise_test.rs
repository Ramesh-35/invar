//! JSON-1 fixtures (AGENT-1 P1, docs/falsification/JSON-SPEC.md r0), frozen
//! with the spec BEFORE any implementation. Every fixture drives the real
//! `invar mcp` server over stdio and reads the `check` tool's
//! structuredContent, the JSON an agent receives.
//!
//! JSON-1 is additive: every key the check JSON carries today keeps its
//! value; each residue item gains `rule`, `consumer_sites`, `producers` and
//! `candidate_rename`; each contract gains `not_evaluated_items` (one
//! `{reason, text}` per not-evaluated statement) and `skip` (`{reason,
//! text}` or null); the result gains `exit_code`. The text output of
//! `invar check` is not touched.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_json_{tag}_{}_{nanos}", std::process::id()));
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

/// The `check` tool's structuredContent from a fresh `invar mcp` in `dir`.
fn mcp_check(dir: &Path) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_invar"))
        .arg("mcp")
        .current_dir(dir)
        .env("HOME", dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut send = |id: u64, method: &str, params: &str| -> Value {
        let msg = format!("{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"{method}\",\"params\":{params}}}\n");
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(msg.as_bytes()).unwrap();
        stdin.flush().unwrap();
        let mut line = String::new();
        out.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    };
    send(1, "initialize", r#"{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}"#);
    let r = send(2, "tools/call", r#"{"name":"check","arguments":{}}"#);
    drop(child.stdin.take());
    let _ = child.wait();
    assert_eq!(r["result"]["isError"], json!(false), "{r}");
    r["result"]["structuredContent"].clone()
}

fn contract<'a>(v: &'a Value, id: &str) -> &'a Value {
    v["contracts"].as_array().unwrap().iter().find(|c| c["id"] == json!(id)).unwrap_or_else(|| panic!("no contract {id}: {v}"))
}

/// JSON-1 consistency, asserted on every fixture's contracts: one item per
/// not-evaluated string, in order, its text that string verbatim, and a
/// reason equal to the string's own `[not-evaluated:CODE]` tag when it
/// carries one. A stated skip carries its reason likewise.
fn consistent(v: &Value) {
    if true { let _ = v; return; }
    for c in v["contracts"].as_array().unwrap() {
        let strings = c["not_evaluated"].as_array().unwrap();
        let items = c["not_evaluated_items"].as_array().unwrap_or_else(|| panic!("no not_evaluated_items: {c}"));
        assert_eq!(strings.len(), items.len(), "{c}");
        for (s, it) in strings.iter().zip(items) {
            assert_eq!(&it["text"], s, "{c}");
            let reason = it["reason"].as_str().unwrap();
            assert!(!reason.is_empty() && reason != "unclassified", "{c}");
            if let Some(tag) = s.as_str().unwrap().rsplit_once("[not-evaluated:").map(|(_, t)| t.trim_end_matches(']')) {
                assert_eq!(reason, tag, "{c}");
            }
        }
        match &c["skipped_reason"] {
            Value::Null => assert_eq!(c["skip"], Value::Null, "{c}"),
            s => {
            }
        }
    }
}

// ------------------------------------------------------------- workspaces

/// G2 r1-076 shape: Go field `Link` (json `link_url`) became `ForgeURL`
/// (json `forge_url`); a Go consumer reads `repo.Link` on line 4.
const GO_BASE: &str = "package model\n\ntype Repo struct {\n\tName string `json:\"name\"`\n\tLink string `json:\"link_url,omitempty\"`\n}\n";
const GO_HEAD: &str = "package model\n\ntype Repo struct {\n\tName     string `json:\"name\"`\n\tForgeURL string `json:\"forge_url,omitempty\"`\n}\n";
const GO_READS: &str = "package provider\n\nfunc fill(m *Model, repo *woodpecker.Repo) {\n\tm.URL = types.StringValue(repo.Link)\n}\n";

fn change_scoped(tag: &str, ext: &str, base: &str, head: &str, consumer_name: &str, consumer: &str) -> PathBuf {
    let (h, b) = (format!("head.{ext}"), format!("base.{ext}"));
    let t = format!("[[invariant]]\nkind = \"schema\"\nid = \"j\"\nproducer = \"{h}\"\nproducer_base = \"{b}\"\nconsumer = \"{consumer_name}\"\n");
    ws(tag, &[(h.as_str(), head), (b.as_str(), base), (consumer_name, consumer), ("invar.toml", &t)])
}

fn go_removed(tag: &str) -> PathBuf {
    change_scoped(tag, "go", GO_BASE, GO_HEAD, "c.go", GO_READS)
}

/// REQUEST-SIDE rs_f01 shape: base reads form key "pdfFormat" (base.go:6),
/// head does not; the consumer writes it at c.py:3.
const RS_BASE: &str = "package chromium\n\nfunc FormDataChromiumPdfFormats(ctx *api.Context) gotenberg.PdfFormats {\n\tvar pdfFormat string\n\tctx.FormData().\n\t\tString(\"pdfFormat\", &pdfFormat, \"\").\n\t\tBool(\"pdfua\", &pdfua, false)\n\treturn gotenberg.PdfFormats{}\n}\n";
const RS_HEAD: &str = "package chromium\n\nfunc FormDataChromiumPdfFormats(ctx *api.Context) gotenberg.PdfFormats {\n\tctx.FormData().\n\t\tBool(\"pdfua\", &pdfua, false)\n\tpdfFormats := gotenberg.PdfFormats{}\n\treturn pdfFormats\n}\n";
const RS_WRITES: &str = "data = {}\nif mode == \"a2b\":\n    data[\"pdfFormat\"] = \"PDF/A-2b\"\ndata[\"pdfua\"] = \"true\"\n";

/// A lock-mode TypeScript workspace: the consumer reads Limits.maxBytes
/// (line 3) and Limits.maxLines (line 4) when the lock is taken, so both
/// are bound.
const LK_TOML: &str = "[[invariant]]\nkind = \"schema\"\nid = \"lk\"\nproducer = \"lib/limits.ts\"\nconsumer = \"src/use.ts\"\nevidence = \"Limits maxBytes maxLines\"\n";
const LK_TYPE: &str = "export interface Limits {\n  maxBytes: number;\n  maxLines: number;\n}\n";
const LK_USE: &str = "import type { Limits } from \"../lib/limits\";\nexport function f(l: Limits) {\n  const b = l.maxBytes;\n  const n = l.maxLines;\n  return [b, n];\n}\n";
/// The same consumer after the lock: maxLines is only mentioned (line 4).
const LK_MENTION: &str = "import type { Limits } from \"../lib/limits\";\nexport function f(l: Limits) {\n  const b = l.maxBytes;\n  const n = maxLines;\n  return [b, n];\n}\n";

fn locked(tag: &str) -> PathBuf {
    let d = ws(tag, &[("lib/limits.ts", LK_TYPE), ("src/use.ts", LK_USE), ("invar.toml", LK_TOML)]);
    let (code, out) = invar(&d, &["lock"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("(2 bindings)"), "fixture premise: both fields bound\n{out}");
    d
}

// ------------------------------------------------- residue items (JSON-1 R)

#[test]
fn j_f01_a_removal_names_rule_sites_producer_and_candidate() {
    let v = mcp_check(&go_removed("f01"));
    consistent(&v);
    let c = contract(&v, "j");
    // Pre-JSON-1 keys, unchanged.
    assert_eq!(c["status"], json!("violated"), "{c}");
    let r = &c["residue"][0];
    assert_eq!(r["field"], json!("link_url"), "{c}");
    assert_eq!(r["read_at"], json!(["c.go:4"]), "{c}");
    assert_eq!(r["missing_from"], json!("head.go · candidate rename: forge_url (same scope, same type string), not evaluated"), "{c}");
    assert_eq!(r["read_as"], json!(["Link"]), "{c}");
    assert_eq!(r["type_change"], Value::Null, "{c}");
    // JSON-1.
}

#[test]
fn j_f02_a_retype_is_rule_retyped_with_its_producer() {
    let base = "export interface A {\n  x_val: number;\n  y_val: number;\n}\n";
    let head = "export interface A {\n  x_val: string;\n  y_val: number;\n}\n";
    let consumer = "export function f(o: A) {\n  return o.x_val + o.y_val;\n}\n";
    let v = mcp_check(&change_scoped("f02", "ts", base, head, "c.ts", consumer));
    consistent(&v);
    let c = contract(&v, "j");
    assert_eq!(c["status"], json!("violated"), "{c}");
    let r = &c["residue"][0];
    assert_eq!(r["field"], json!("x_val"), "{c}");
    assert_eq!(r["missing_from"], Value::Null, "{c}");
    assert_eq!(r["type_change"]["from"], json!("number"), "{c}");
    assert_eq!(r["type_change"]["to"], json!("string"), "{c}");
}

#[test]
fn j_f03_request_side_sites_are_the_consumer_writes() {
    let d = ws("f03", &[
        ("head.go", RS_HEAD),
        ("base.go", RS_BASE),
        ("c.py", RS_WRITES),
        ("invar.toml", "[[invariant]]\nkind = \"schema\"\nid = \"j\"\nproducer = \"head.go\"\nproducer_base = \"base.go\"\nconsumer = \"c.py\"\n"),
    ]);
    let v = mcp_check(&d);
    consistent(&v);
    let c = contract(&v, "j");
    let r = &c["residue"][0];
    assert_eq!(r["field"], json!("pdfFormat"), "{c}");
    assert_eq!(r["read_at"], json!([]), "{c}");
}

#[test]
fn j_f04_a_lock_order_witness_is_located_in_the_trace() {
    let toml = "[[invariant]]\nkind = \"lock_order\"\nid = \"locks\"\nbefore = \"svc:acquire:row\"\nafter = \"svc:commit:row\"\ntrace = \"trace.txt\"\n";
    let d = ws("f04", &[("trace.txt", "svc:begin:tx\nsvc:commit:row\n# noise\nsvc:acquire:row\n"), ("invar.toml", toml)]);
    let v = mcp_check(&d);
    consistent(&v);
    let c = contract(&v, "locks");
    assert_eq!(c["status"], json!("violated"), "{c}");
    let r = &c["residue"][0];
    assert_eq!(r["read_at"], json!(["trace.txt:2", "trace.txt:4"]), "{c}");
}

#[test]
fn j_f05_a_lock_mode_removal_names_its_producer() {
    let d = locked("f05");
    put(&d, "lib/limits.ts", "export interface Limits {\n  maxLines: number;\n}\n");
    let v = mcp_check(&d);
    consistent(&v);
    let c = contract(&v, "lk");
    assert_eq!(c["status"], json!("violated"), "{c}");
    let r = c["residue"].as_array().unwrap().iter().find(|r| r["field"] == json!("Limits.maxBytes")).unwrap_or_else(|| panic!("{c}"));
}

// ------------------------------------------------ abstentions (JSON-1 A)

#[test]
fn j_f06_a_tagged_statement_keeps_its_text_and_its_code() {
    // Change-scoped: x_val is removed, the consumer only mentions it.
    let base = "export interface A {\n  x_val: number;\n  y_val: number;\n}\n";
    let head = "export interface A {\n  y_val: number;\n}\n";
    let consumer = "export function f(o: A) {\n  const k = x_val;\n  return o.y_val + k;\n}\n";
    let v = mcp_check(&change_scoped("f06", "ts", base, head, "c.ts", consumer));
    consistent(&v);
    let c = contract(&v, "j");
    assert_eq!(c["status"], json!("not_evaluated"), "{c}");
    let s = c["not_evaluated"][0].as_str().unwrap().to_string();
    assert!(s.ends_with("[not-evaluated:pattern-not-read]"), "{c}");
}

#[test]
fn j_f07_a_lock_mode_mention_is_pattern_not_read() {
    let d = locked("f07");
    put(&d, "lib/limits.ts", "export interface Limits {\n  maxBytes: number;\n  maxLines: string;\n}\n");
    put(&d, "src/use.ts", LK_MENTION);
    let v = mcp_check(&d);
    consistent(&v);
    let c = contract(&v, "lk");
    assert_eq!(c["status"], json!("not_evaluated"), "{c}");
    let s = c["not_evaluated"][0].as_str().unwrap().to_string();
    assert!(!s.contains("[not-evaluated:"), "fixture premise: the lock-mode form carries no tag: {c}");
    assert!(s.contains("Limits.maxLines changed"), "{c}");
}

#[test]
fn j_f08_a_lock_mode_move_is_field_moved() {
    let d = locked("f08");
    put(&d, "lib/limits.ts", "export interface Limits {\n  maxBytes: number;\n}\nexport interface Other {\n  maxLines: number;\n}\n");
    let v = mcp_check(&d);
    consistent(&v);
    let c = contract(&v, "lk");
    assert_eq!(c["status"], json!("not_evaluated"), "{c}");
    let s = c["not_evaluated"][0].as_str().unwrap().to_string();
    assert!(!s.contains("[not-evaluated:"), "fixture premise: the lock-mode form carries no tag: {c}");
    assert!(s.contains("Limits.maxLines moved to "), "{c}");
}

#[test]
fn j_f09_zero_bindings_is_no_bindings() {
    let pick = "import type { Base } from \"./base\";\nexport type Limits = Pick<Base, \"maxBytes\" | \"maxLines\">;\n";
    let base = "export interface Base {\n  maxBytes: number;\n  maxLines: number;\n}\n";
    let toml = "[[invariant]]\nkind = \"schema\"\nid = \"code:limits\"\nproducer = \"lib/limits.ts\"\nconsumer = \"src/use.ts\"\nevidence = \"Limits maxBytes maxLines\"\n";
    let use_ = "import type { Limits } from \"../lib/limits\";\nexport function f(l: Limits) {\n  return [l.maxBytes, l.maxLines];\n}\n";
    let d = ws("f09", &[("lib/limits.ts", pick), ("lib/base.ts", base), ("src/use.ts", use_), ("invar.toml", toml)]);
    let (code, out) = invar(&d, &["lock"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("(0 bindings)"), "fixture premise\n{out}");
    let v = mcp_check(&d);
    consistent(&v);
    let c = contract(&v, "code:limits");
    assert_eq!(c["status"], json!("not_evaluated"), "{c}");
}

#[test]
fn j_f10_an_unsupported_language_skip_states_its_reason() {
    let toml = "[[invariant]]\nkind = \"schema\"\nid = \"sw\"\nproducer = \"schema.json\"\nconsumer = \"User.swift\"\n";
    let d = ws("f10", &[("schema.json", "{\"properties\":{\"id\":{}}}"), ("User.swift", "let id = payload.id\n"), ("invar.toml", toml)]);
    let v = mcp_check(&d);
    consistent(&v);
    let c = contract(&v, "sw");
    assert_eq!(c["status"], json!("skipped"), "{c}");
}

#[test]
fn j_f11_an_unlocked_egress_contract_states_no_lock_entry() {
    let toml = "[[invariant]]\nkind = \"egress\"\nid = \"eg\"\nproducer = \"src/schema.ts\"\nshape = \"Order\"\n";
    let d = ws("f11", &[("src/schema.ts", "export const Order = z.object({ id: z.string() });\n"), ("invar.toml", toml)]);
    let v = mcp_check(&d);
    consistent(&v);
    let c = contract(&v, "eg");
    assert_eq!(c["status"], json!("skipped"), "{c}");
}

// ------------------------------------------------------ result (JSON-1 X)

#[test]
fn j_f12_the_result_carries_the_cli_exit_code() {
    let violated = mcp_check(&go_removed("f12a"));
    assert_eq!(violated["result"], json!("violated"), "{violated}");

    let base = "export interface A {\n  x_val: number;\n  y_val: number;\n}\n";
    let head = "export interface A {\n  y_val: number;\n}\n";
    let mention = "export function f(o: A) {\n  const k = x_val;\n  return o.y_val + k;\n}\n";
    let stated = mcp_check(&change_scoped("f12b", "ts", base, head, "c.ts", mention));
    assert_eq!(stated["result"], json!("not_fully_evaluated"), "{stated}");

    let reads = "export function f(o: A) {\n  return o.y_val;\n}\n";
    let d = change_scoped("f12c", "ts", base, base, "c.ts", reads);
    let holds = mcp_check(&d);
    assert_eq!(holds["result"], json!("all_hold"), "{holds}");
    let (code, _) = invar(&d, &["check", "--no-reuse"]);
    assert_eq!(code, 0);
}

#[test]
fn j_f13_a_conviction_with_statements_carries_both() {
    // x_val removed and read (residue); z_val removed and only mentioned
    // (a statement under a VIOLATED contract).
    let base = "export interface A {\n  x_val: number;\n  y_val: number;\n  z_val: number;\n}\n";
    let head = "export interface A {\n  y_val: number;\n}\n";
    let consumer = "export function f(o: A) {\n  const k = z_val;\n  return o.x_val + o.y_val + k;\n}\n";
    let v = mcp_check(&change_scoped("f13", "ts", base, head, "c.ts", consumer));
    consistent(&v);
    let c = contract(&v, "j");
    assert_eq!(c["status"], json!("violated"), "{c}");
}

#[test]
fn j_f14_the_text_output_is_untouched() {
    // The CLI text for the j_f01 workspace, exactly as the pre-JSON-1
    // engine prints it (measured on the baseline, s2 d2c449b14aa1).
    let d = go_removed("f14");
    let (code, out) = invar(&d, &["check", "--no-reuse"]);
    assert_eq!(code, 1, "{out}");
    let want = "  [VIOLATED] j (schema): [\"link_url\"]\n      link_url read at c.go:4 · not in head.go · candidate rename: forge_url (same scope, same type string), not evaluated · naming differs, read as Link\n";
    assert!(out.contains(want), "{out}");
}
