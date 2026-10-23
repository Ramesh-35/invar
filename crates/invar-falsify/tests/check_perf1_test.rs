//! CHECK-PERF-1 fixtures (founder rulings P-D1 to P-D4 of 2026-10-22):
//! `check` memoizes its pure text analyses per process, keyed by the
//! artifact name and the text's BLAKE3, and `broad_reads` searches its keys
//! once per text. These are invariance fixtures: each passes on the
//! baseline (93e93ce) and must keep passing; the Gate 3 mutants of the memo
//! and of the scan must fail them.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_perf1_{tag}_{}_{nanos}", std::process::id()));
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
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(dir).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

/// A running `invar mcp`, initialized; `check()` returns the tool's text.
struct Mcp {
    child: std::process::Child,
    out: BufReader<std::process::ChildStdout>,
    id: u64,
}

impl Mcp {
    fn start(dir: &Path) -> Mcp {
        let mut child = Command::new(env!("CARGO_BIN_EXE_invar"))
            .arg("mcp")
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let out = BufReader::new(child.stdout.take().unwrap());
        let mut m = Mcp { child, out, id: 0 };
        m.call("initialize", r#"{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}"#);
        m
    }
    fn call(&mut self, method: &str, params: &str) -> String {
        self.id += 1;
        let msg = format!("{{\"jsonrpc\":\"2.0\",\"id\":{},\"method\":\"{method}\",\"params\":{params}}}\n", self.id);
        let stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(msg.as_bytes()).unwrap();
        stdin.flush().unwrap();
        let mut line = String::new();
        self.out.read_line(&mut line).unwrap();
        line
    }
    fn check(&mut self) -> String {
        self.call("tools/call", r#"{"name":"check","arguments":{}}"#)
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.wait();
    }
}

/// The answer a brand-new server gives: the reference for a warm one.
fn fresh(dir: &Path) -> String {
    Mcp::start(dir).check()
}

const PRODUCER: &str = "export interface A {\n  x_val: number;\n  y_val: number;\n  w_val: string;\n}\n";
const USE: &str = "export function f(o: any) {\n  return [o.x_val, o.y_val];\n}\n";
const TOML: &str = "[[invariant]]\nkind = \"schema\"\nid = \"code:a\"\nproducer = \"lib/a.ts\"\nconsumer = \"src/use.ts\"\n";

// ------------------------------------------------- the memo sees every edit

#[test]
fn p_f1_a_warm_server_sees_every_edit() {
    let d = ws("f1", &[("lib/a.ts", PRODUCER), ("src/use.ts", USE), ("invar.toml", TOML)]);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let mut warm = Mcp::start(&d);
    let steps: [(&str, &str, &str, &str); 4] = [
        ("baseline", PRODUCER, USE, "\"status\":\"holds\""),
        // the consumer reads one more producer field: the note names it
        ("consumer edited", PRODUCER, "export function f(o: any) {\n  return [o.x_val, o.y_val, o.w_val];\n}\n", "w_val"),
        // a bound field leaves the producer: violated
        ("producer edited", "export interface A {\n  x_val: number;\n  w_val: string;\n}\n", USE, "\"status\":\"violated\""),
        ("reverted", PRODUCER, USE, "\"status\":\"holds\""),
    ];
    for (label, producer, consumer, expect) in steps {
        put(&d, "lib/a.ts", producer);
        put(&d, "src/use.ts", consumer);
        let w = warm.check();
        let f = fresh(&d);
        assert!(w.replace("\\\"", "\"").replace(": ", ":").contains(expect), "{label}: {w}");
        assert_eq!(w.replace(&format!("\"id\":{}", warm.id), "\"id\":N"), f.replace("\"id\":2", "\"id\":N"), "{label}: warm differs from a fresh server");
    }
}

// --------------------------------- the artifact name is part of every key

#[test]
fn p_f2_the_same_bytes_under_two_artifact_names_are_read_twice() {
    // lib/b.rs holds the TypeScript text byte for byte: as Rust it has no
    // fields (0 bindings, NOT EVALUATED), as TypeScript it has three. The
    // .rs contract is evaluated first, so a memo keyed by the text alone
    // would hand its empty surface to the .ts contract.
    let toml = "[[invariant]]\nkind = \"schema\"\nid = \"code:b\"\nproducer = \"lib/b.rs\"\nconsumer = \"src/use.ts\"\n\n".to_string() + TOML;
    let d = ws("f2", &[("lib/a.ts", PRODUCER), ("lib/b.rs", PRODUCER), ("src/use.ts", USE), ("invar.toml", &toml)]);
    let (_, lock_out) = invar(&d, &["lock"]);
    assert!(lock_out.contains("+ contract code:b (0 bindings)"), "{lock_out}");
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("  [NOT EVALUATED] code:b (schema)"), "{out}");
    assert!(out.contains("  [HOLDS]    code:a (schema)\n"), "{out}");
    let mut warm = Mcp::start(&d);
    let (a, b) = (warm.check(), warm.check());
    assert_eq!(a.replace("\"id\":2", ""), b.replace("\"id\":3", ""), "a second warm check differs");
}

// ------------------------------------------------- the single-pass key scan

#[test]
fn p_f3_key_reads_across_lines_and_crlf() {
    // `.get("..")`, `json:".."`, `rename` and `alias` reads, several on one
    // line, on CRLF lines, and one key whose match ends its line.
    let producer = "export interface A {\n  x_val: number;\n  y_val: number;\n  w_val: string;\n  v_val: string;\n}\n";
    let consumer = "const a = m.get(\"x_val\"), b = m.get('y_val');\r\nconst t = { json:\"w_val\" };\r\nconst r = { rename: \"v_val\", alias: \"nothing_here\" };\r\nconst z = m.get(\r\n";
    let toml = TOML.to_string();
    let d = ws("f3", &[("lib/a.ts", producer), ("src/use.ts", consumer), ("invar.toml", &toml)]);
    let (_, lock_out) = invar(&d, &["lock"]);
    assert!(lock_out.contains("+ contract code:a (6 bindings)"), "{lock_out}");
    // Recorded at Gate 1 on the baseline (93e93ce): exactly these bindings.
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    let binds: Vec<String> = lock.lines().filter_map(|l| l.strip_prefix("bind = \"code ")).map(|l| l.splitn(2, ' ').nth(1).unwrap_or("").trim_end_matches('"').to_string()).collect();
    assert_eq!(binds, vec!["w_val ?", "x_val ?", "y_val ?", "A.w_val string", "A.x_val number", "A.y_val number"], "{lock}");
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "{out}");
}

// ------------------------------------------------------------ no regression

#[test]
fn p_n1_check_output_is_unchanged() {
    let toml = "[[invariant]]\nkind = \"schema\"\nid = \"code:b\"\nproducer = \"lib/b.rs\"\nconsumer = \"src/use.ts\"\n\n".to_string() + TOML;
    let d = ws("n1", &[("lib/a.ts", PRODUCER), ("lib/b.rs", PRODUCER), ("src/use.ts", USE), ("invar.toml", &toml)]);
    let _ = invar(&d, &["lock"]);
    let (_, out) = invar(&d, &["check"]);
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    let at = lock.lines().find(|l| l.starts_with("recorded_at = ")).unwrap().trim_start_matches("recorded_at = ").trim_matches('"').to_string();
    let day = &at[..10];
    assert_eq!(out.replace(&at, "T").replace(day, "D"), N1_CHECK);
}

// Recorded at Gate 1 on the baseline (93e93ce).
const N1_CHECK: &str = r#"INVAR check (2 contract(s))
  [NOT EVALUATED] code:b (schema): 1 site(s)
      the lock binds no field of this contract (0 bindings), so there is nothing to compare and a HOLDS would be vacuous; make the producer's fields readable and run `invar lock --update`
      note: producer unchanged since lock (T)
  [HOLDS]    code:a (schema)
      note: producer unchanged since lock (T)
  durability: held since D (0 days)
result: NOT FULLY EVALUATED, 1 of 2 contract(s) skipped or not evaluated, the rest hold
summary: held 1 | violated 0 | skipped or not evaluated 1
measured: 38 of 70 held-out real cross-repository breaks caught (batches 4-8; docs/epistemic/STATUS.md)
"#;

// ------------- added during Gate 2, before the code it pins (spec section K)

#[test]
fn p_f4_each_contract_keeps_its_own_zod_shapes() {
    // One producer, two contracts: the first binds the interface A, the
    // second the zod object B, whose keys enter the surface only because
    // that contract names B. A memo keyed by the producer alone would hand
    // the first contract's surface (without B) to the second.
    let producer = "import { z } from \"zod\";\nexport interface A {\n  a_x: number;\n  a_y: number;\n}\nexport const B = z.object({\n  b_x: z.string(),\n  b_y: z.number(),\n});\n";
    let toml = "[[invariant]]\nkind = \"schema\"\nid = \"code:a\"\nproducer = \"lib/p.ts\"\nconsumer = \"src/a.ts\"\nevidence = \"A a_x a_y\"\n\n[[invariant]]\nkind = \"schema\"\nid = \"code:b\"\nproducer = \"lib/p.ts\"\nconsumer = \"src/b.ts\"\nevidence = \"B b_x b_y\"\n";
    let d = ws(
        "f4",
        &[
            ("lib/p.ts", producer),
            ("src/a.ts", "export function f(o: any) {\n  return [o.a_x, o.a_y];\n}\n"),
            ("src/b.ts", "export function g(q: any) {\n  return [q.b_x, q.b_y];\n}\n"),
            ("invar.toml", toml),
        ],
    );
    let (_, lock_out) = invar(&d, &["lock"]);
    assert!(lock_out.contains("+ contract code:b (2 bindings)"), "{lock_out}");
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("  [HOLDS]    code:a (schema)\n") && out.contains("  [HOLDS]    code:b (schema)\n"), "{out}");
    let mut warm = Mcp::start(&d);
    let (a, b) = (warm.check(), warm.check());
    assert_eq!(a.replace("\"id\":2", ""), b.replace("\"id\":3", ""), "a second warm check differs");
}

// ------------- added at Gate 3, for the one key part no fixture reached (K)

#[test]
fn p_f5_a_warm_server_sees_a_code_companion_edit() {
    // A spec producer with a code companion: only the companion changes,
    // so a surface keyed without the companion's bytes would go stale.
    let api = r#"{"components":{"schemas":{"Order":{"properties":{"amount_cents":{"type":"integer"},"currency":{"type":"string"}}}}}}"#;
    let go = "type Order struct {\n\tAmountCents int64 `json:\"amount_cents\"`\n}\n";
    let toml = "[[invariant]]\nkind = \"schema\"\nid = \"orders\"\nproducer = \"api.json\"\npointer = \"/components/schemas/Order\"\nproducer_code = \"order.go\"\nconsumer = \"client.js\"\n";
    let d = ws("f5", &[("api.json", api), ("order.go", go), ("client.js", "const a = order.amount_cents;\nconst c = order.currency;\n"), ("invar.toml", toml)]);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let mut warm = Mcp::start(&d);
    for (label, body, expect) in [
        ("baseline", go, "\"status\":\"holds\""),
        ("companion retyped", "type Order struct {\n\tAmountCents string `json:\"amount_cents\"`\n}\n", "\"status\":\"violated\""),
        ("reverted", go, "\"status\":\"holds\""),
    ] {
        put(&d, "order.go", body);
        let w = warm.check();
        assert!(w.replace("\\\"", "\"").replace(": ", ":").contains(expect), "{label}: {w}");
        assert_eq!(w.replace(&format!("\"id\":{}", warm.id), "\"id\":N"), fresh(&d).replace("\"id\":2", "\"id\":N"), "{label}: warm differs from a fresh server");
    }
}

#[test]
fn p_f6_each_contract_keeps_its_own_pointer() {
    // One spec file, two contracts at two pointers whose last segment (and
    // so the binding scope) is the same: the second must not be handed the
    // first one's surface.
    let api = r#"{"v1":{"Order":{"properties":{"amount_cents":{"type":"integer"},"currency":{"type":"string"}}}},"v2":{"Order":{"properties":{"refund_ref":{"type":"string"},"reason_code":{"type":"string"}}}}}"#;
    let toml = "[[invariant]]\nkind = \"schema\"\nid = \"orders\"\nproducer = \"api.json\"\npointer = \"/v1/Order\"\nconsumer = \"orders.js\"\n\n[[invariant]]\nkind = \"schema\"\nid = \"refunds\"\nproducer = \"api.json\"\npointer = \"/v2/Order\"\nconsumer = \"refunds.js\"\n";
    let d = ws(
        "f6",
        &[
            ("api.json", api),
            ("orders.js", "const a = order.amount_cents;\nconst c = order.currency;\n"),
            ("refunds.js", "const a = r.refund_ref;\nconst c = r.reason_code;\n"),
            ("invar.toml", toml),
        ],
    );
    let (_, lock_out) = invar(&d, &["lock"]);
    assert!(lock_out.contains("+ contract refunds (2 bindings)"), "{lock_out}");
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("  [HOLDS]    orders (schema)\n") && out.contains("  [HOLDS]    refunds (schema)\n"), "{out}");
}
