//! SIB-2 fixtures (P5 slice 2, docs/falsification/SIBLING-REPORT-SPEC.md
//! r0), frozen with the spec BEFORE any implementation. SIB-1's evaluation,
//! one code path, delivered where it is acted on: the MCP `dependents` tool
//! (an agent editing a producer sees what it breaks), a markdown report for
//! the PR (`dependents --report-md`), and the GitHub Action. Advisory
//! throughout: nothing here fails a run because a consumer field broke.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

const TYPE: &str = "export interface Limits {\n  maxBytes: number;\n  maxLines: number;\n}\n";
const USE: &str = "import type { Limits } from \"../lib/limits\";\nexport function f(l: Limits) {\n  return [l.maxBytes, l.maxLines];\n}\n";
const REMOTE: &str = "github:acme/api@0123456789abcdef0123456789abcdef01234567:lib/limits.ts";
const REMOVED: &str = "export interface Limits {\n  maxBytes: number;\n}\n";
const RETYPED: &str = "export interface Limits {\n  maxBytes: number;\n  maxLines: string;\n}\n";
const NOTE: &str = "a lock records the fields a consumer depends on, not the lines that read them.";

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s2_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn put(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

fn invar(dir: &Path, home: &Path, args: &[&str]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(dir).env("HOME", home).output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).to_string(), String::from_utf8_lossy(&o.stderr).to_string())
}

fn keygen(dir: &Path, name: &str) -> PathBuf {
    let key = dir.join(name);
    assert!(Command::new("ssh-keygen").args(["-t", "ed25519", "-N", "", "-q", "-f"]).arg(&key).status().unwrap().success());
    key
}

fn sign(lock: &Path, key: &Path) {
    let _ = std::fs::remove_file(lock.with_extension("lock.sig"));
    let o = Command::new("ssh-keygen").args(["-Y", "sign", "-n", "invar-lock", "-f"]).arg(key).arg(lock).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
}

/// A consumer lock (contract `id`) binding Limits.maxBytes and
/// Limits.maxLines of acme/api, at `api/index/<name>/invar.lock`.
fn consumer_lock(root: &Path, name: &str, id: &str) -> PathBuf {
    let c = root.join("work").join(name);
    put(&c, "lib/limits.ts", TYPE);
    put(&c, "src/use.ts", USE);
    put(&c, "invar.toml", &format!("[[invariant]]\nkind = \"schema\"\nid = \"{id}\"\nproducer = \"lib/limits.ts\"\nconsumer = \"src/use.ts\"\n"));
    let (code, out, err) = invar(&c, root, &["lock"]);
    assert_eq!(code, 0, "{out}{err}");
    let text = std::fs::read_to_string(c.join("invar.lock")).unwrap();
    let rel = format!("api/index/{name}/invar.lock");
    put(root, &rel, &text.replace("producer = \"lib/limits.ts\"", &format!("producer = \"{REMOTE}\"")));
    root.join(rel)
}

/// The producer repository `api` (this version of lib/limits.ts), with a
/// signed `web-app` lock and an unsigned `mobile` lock under api/index,
/// and `signers` trusting web-app@acme.
fn world(tag: &str, limits: &str, web_id: &str, with_mobile: bool) -> (PathBuf, PathBuf) {
    let root = ws(tag);
    let k = keygen(&root, "kw");
    put(&root, "api/signers", &format!("web-app@acme {}", std::fs::read_to_string(k.with_extension("pub")).unwrap()));
    let web = consumer_lock(&root, "web-app", web_id);
    sign(&web, &k);
    if with_mobile {
        consumer_lock(&root, "mobile", "limits");
    }
    put(&root, "api/lib/limits.ts", limits);
    (root.clone(), root.join("api"))
}

struct Mcp {
    child: std::process::Child,
    out: BufReader<std::process::ChildStdout>,
    id: u64,
}

impl Mcp {
    fn start(dir: &Path, home: &Path, args: &[&str]) -> Mcp {
        let mut child = Command::new(env!("CARGO_BIN_EXE_invar"))
            .arg("mcp")
            .args(args)
            .current_dir(dir)
            .env("HOME", home)
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
    /// Every stdout line is JSON-RPC with the request's id (f08).
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
    fn dependents(&mut self, args: Value) -> Value {
        self.rpc("tools/call", &json!({"name": "dependents", "arguments": args}).to_string())["result"].clone()
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}

const SERVER: &[&str] = &["--repo", "acme/api", "--dependents-signers", "signers"];

fn ok(r: &Value) -> &Value {
    assert_eq!(r["isError"], json!(false), "{r}");
    &r["structuredContent"]
}

fn consumer<'a>(v: &'a Value, lockfile: &str) -> &'a Value {
    v["consumers"].as_array().unwrap().iter().find(|c| c["lockfile"] == json!(lockfile)).unwrap_or_else(|| panic!("{lockfile}: {v}"))
}

// ------------------------------------------------------------------- MCP

#[test]
fn s2_f01_the_agent_sees_a_removed_field() {
    let (root, api) = world("f01", REMOVED, "limits", false);
    let mut m = Mcp::start(&api, &root, SERVER);
    let r = m.dependents(json!({"locks": "index"}));
    let v = ok(&r);
    assert_eq!(v["repo"], json!("acme/api"), "{v}");
    let c = consumer(v, "web-app/invar.lock");
    assert_eq!(c["state"], json!("verified"), "{v}");
    assert_eq!(c["principal"], json!("web-app@acme"), "{v}");
    assert_eq!(c["contracts"][0]["id"], json!("limits"), "{v}");
    assert_eq!(c["contracts"][0]["producer"], json!("lib/limits.ts"), "{v}");
    assert_eq!(c["contracts"][0]["broken"], json!([{"field": "Limits.maxLines", "rule": "absent", "locked_type": "number"}]), "{v}");
    assert_eq!(c["contracts"][0]["provided"], json!(1), "{v}");
    assert_eq!(v["summary"], json!({"broken": 1, "provided": 1, "lockfiles_not_evaluated": 0, "contracts_not_evaluated": 0}), "{v}");
    assert_eq!(v["exit_code"], json!(0), "advisory: {v}");
    assert_eq!(v["note"], json!(NOTE), "{v}");
}

#[test]
fn s2_f02_the_agent_sees_a_retyped_field() {
    let (root, api) = world("f02", RETYPED, "limits", false);
    let mut m = Mcp::start(&api, &root, SERVER);
    let r = m.dependents(json!({"locks": "index"}));
    let c = consumer(ok(&r), "web-app/invar.lock");
    assert_eq!(c["contracts"][0]["broken"], json!([{"field": "Limits.maxLines", "rule": "retyped", "locked_type": "number", "now_type": "string"}]), "{r}");
}

#[test]
fn s2_f03_an_unsigned_lock_reaches_the_agent_as_a_reason_only() {
    let (root, api) = world("f03", REMOVED, "limits", true);
    let mut m = Mcp::start(&api, &root, SERVER);
    let r = m.dependents(json!({"locks": "index"}));
    let v = ok(&r);
    let c = consumer(v, "mobile/invar.lock");
    assert_eq!(c, &json!({"lockfile": "mobile/invar.lock", "state": "not_evaluated", "reason": "unsigned", "text": "unsigned (no invar.lock.sig)"}), "{v}");
    assert_eq!(v["exit_code"], json!(4), "{v}");
    assert_eq!(r.to_string().matches("maxLines").count(), 2, "only web-app's binding, in structuredContent and its text copy: {r}");
}

#[test]
fn s2_f04_a_server_without_dependents_signers_evaluates_nothing() {
    let (root, api) = world("f04", REMOVED, "limits", false);
    let mut m = Mcp::start(&api, &root, &["--repo", "acme/api"]);
    let r = m.dependents(json!({"locks": "index"}));
    let v = ok(&r);
    assert_eq!(consumer(v, "web-app/invar.lock")["reason"], json!("no-signers"), "{v}");
    assert_eq!(v["exit_code"], json!(4), "{v}");
    assert!(!r.to_string().contains("maxLines"), "{r}");
}

#[test]
fn s2_f05_no_query_on_a_server_without_repo_is_an_error() {
    let (root, api) = world("f05", REMOVED, "limits", false);
    let mut m = Mcp::start(&api, &root, &[]);
    let r = m.dependents(json!({"locks": "index"}));
    assert_eq!(r["isError"], json!(true), "{r}");
    assert!(r["content"][0]["text"].as_str().unwrap().contains("give a field query, or start the server with --repo OWNER/NAME"), "{r}");
}

#[test]
fn s2_f06_locks_outside_the_configuration_directory_stay_refused() {
    // SECURITY-SPEC S2-5 holds in evaluation mode too.
    let (root, api) = world("f06", REMOVED, "limits", false);
    let mut m = Mcp::start(&api, &root, SERVER);
    let r = m.dependents(json!({"locks": root.join("api/index").parent().unwrap().parent().unwrap().display().to_string()}));
    assert_eq!(r["isError"], json!(true), "{r}");
    assert!(r["content"][0]["text"].as_str().unwrap().contains("outside the configuration directory"), "{r}");
}

#[test]
fn s2_f07_text_and_json_are_one_evaluation() {
    let (root, api) = world("f07", RETYPED, "limits", true);
    let (code, text, _) = invar(&api, &root, &["dependents", "--locks", "index", "--repo", "acme/api", "--lock-signers", "signers"]);
    let mut m = Mcp::start(&api, &root, SERVER);
    let r = m.dependents(json!({"locks": "index"}));
    let s = &ok(&r)["summary"];
    let line = format!(
        "summary: {} broken | {} provided | {} lockfile(s) not evaluated\n",
        s["broken"], s["provided"], s["lockfiles_not_evaluated"]
    );
    assert!(text.contains(&line), "{text} vs {s}");
    assert_eq!(ok(&r)["exit_code"], json!(code), "{r}");
}

#[test]
fn s2_f08_the_query_mode_is_unchanged() {
    let (root, api) = world("f08", TYPE, "limits", false);
    let mut m = Mcp::start(&api, &root, SERVER);
    let r = m.dependents(json!({"query": "Limits.maxLines", "locks": "index"}));
    let v = ok(&r);
    assert_eq!(v["dependents"][0]["consumer"], json!("web-app/invar.lock"), "{v}");
    assert!(v.get("consumers").is_none(), "{v}");
}

#[test]
fn s2_f09_tools_list_declares_evaluation() {
    let (root, api) = world("f09", TYPE, "limits", false);
    let mut m = Mcp::start(&api, &root, SERVER);
    let l = m.rpc("tools/list", "{}");
    let t = l["result"]["tools"].as_array().unwrap().iter().find(|t| t["name"] == json!("dependents")).unwrap().clone();
    assert_eq!(t["inputSchema"]["required"], json!(["locks"]), "{t}");
    assert!(t["description"].as_str().unwrap().contains("Without a query"), "{t}");
}

// -------------------------------------------------------------- PR report

#[test]
fn s2_f10_the_pr_report_names_the_break_and_changes_nothing_else() {
    let (root, api) = world("f10", REMOVED, "limits", true);
    let base = ["dependents", "--locks", "index", "--repo", "acme/api", "--lock-signers", "signers"];
    let (code0, out0, _) = invar(&api, &root, &base);
    let mut args = base.to_vec();
    args.extend(["--report-md", "report.md"]);
    let (code, out, _) = invar(&api, &root, &args);
    assert_eq!((code, &out), (code0, &out0), "the report adds a file, nothing else");
    let md = std::fs::read_to_string(api.join("report.md")).unwrap();
    assert!(md.starts_with("<!-- invar-dependents -->\n"), "{md}");
    assert!(md.contains("| `web-app/invar.lock` | `web-app@acme` | `limits` | `Limits.maxLines` | absent (locked number) |\n"), "{md}");
    assert!(md.contains("- `mobile/invar.lock`: not evaluated: unsigned (no invar.lock.sig)\n"), "{md}");
    assert!(md.contains("Advisory: this report never fails the check."), "{md}");
    assert!(md.contains(NOTE), "{md}");
}

#[test]
fn s2_f11_repository_values_are_inert_in_the_pr_report() {
    let (root, api) = world("f11", REMOVED, "x|y @someone`z", false);
    let (_, _, _) = invar(&api, &root, &["dependents", "--locks", "index", "--repo", "acme/api", "--lock-signers", "signers", "--report-md", "report.md"]);
    let md = std::fs::read_to_string(api.join("report.md")).unwrap();
    assert!(md.contains("| `x\\|y @someone'z` |"), "pipes escaped, mention inside code, backtick neutralized: {md}");
    let row = md.lines().find(|l| l.contains("Limits.maxLines")).unwrap();
    assert_eq!(row.replace("\\|", "").matches('|').count(), 6, "five cells: {row}");
}

// ---------------------------------------------------------- GitHub Action

#[test]
fn s2_f12_the_action_wires_evaluation_as_advisory() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let yml = std::fs::read_to_string(root.join("action.yml")).unwrap();
    for (input, default) in [("check", "'true'"), ("dependents_repo", "''"), ("dependents_locks", "''"), ("dependents_signers", "''")] {
        assert!(yml.contains(&format!("  {input}:")), "input {input}: {yml}");
        let block = yml.split(&format!("  {input}:")).nth(1).unwrap();
        let d = block.lines().find(|l| l.trim_start().starts_with("default:")).unwrap();
        assert_eq!(d.trim_start().trim_start_matches("default:").trim(), default, "{input}");
    }
    assert!(yml.contains("action/dependents.sh"), "{yml}");
    assert!(yml.contains("if: inputs.check == 'true'"), "a pure producer repository can skip check: {yml}");
    assert!(yml.contains("if: inputs.dependents_repo != ''"), "{yml}");
    assert!(root.join("action/dependents.sh").is_file());
    let comment = std::fs::read_to_string(root.join("action/comment.sh")).unwrap();
    assert!(comment.contains("marker=\"${INVAR_MARKER:-<!-- invar-check -->}\""), "{comment}");
}

#[test]
fn s2_f13_the_action_step_never_fails_the_job() {
    let (root, api) = world("f13", REMOVED, "limits", true);
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../action/dependents.sh");
    let out_file = root.join("github_output");
    let o = Command::new("bash")
        .arg(&script)
        .current_dir(&api)
        .env("HOME", &root)
        .env("INVAR_BIN", env!("CARGO_BIN_EXE_invar"))
        .env("INVAR_DEPENDENTS_REPO", "acme/api")
        .env("INVAR_DEPENDENTS_LOCKS", "index")
        .env("INVAR_DEPENDENTS_SIGNERS", "signers")
        .env("RUNNER_TEMP", &root)
        .env("GITHUB_OUTPUT", &out_file)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "advisory: {o:?}");
    let outputs = std::fs::read_to_string(&out_file).unwrap();
    assert!(outputs.contains("dependents_exit=4\n"), "{outputs}");
    assert!(outputs.contains("dependents_broken=1\n"), "{outputs}");
    let report = outputs.lines().find_map(|l| l.strip_prefix("dependents_report=")).unwrap();
    assert!(std::fs::read_to_string(report).unwrap().starts_with("<!-- invar-dependents -->\n"));
}
