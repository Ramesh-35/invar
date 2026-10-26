//! INVAR-GATE-001 fixtures (docs/architecture/INVAR-GATE-001.md, rulings
//! G-D1 to G-D7 of 2026-10-18). GATE 1: written BEFORE src/gate.rs exists.
//!
//! Declared Gate 1 outcomes against the unchanged engine: every G-* fixture
//! fails via clap's unrecognized subcommand (exit 2), G-M2 fails because
//! src/gate.rs is absent, L-F1, L-F2 and L-F4 fail because the pointer line
//! does not exist, and L-F3 (init byte-determinism) passes as an invariance
//! control. L-N1 is the standing suites, which must stay green unchanged.
//!
//! Interpretations carried to the Gate 5 ledger:
//! - G-INT-1: `invar gate` with arguments but no `--` is rejected by clap
//!   with its CLI-wide usage error (exit 2, nothing run); the gate's own
//!   exit 3 covers the empty command after `--`. Both refuse without
//!   running anything, which is the spec sentence's substance.
//! - G-INT-2: a wrapped command killed by a signal has no exit code; the
//!   gate states it and exits 3 (an error, never a verdict).
//! - G-INT-3: in init's TOML output the pointer line carries the `# `
//!   comment prefix like every other annotation, so a written invar.toml
//!   stays valid; in check --proof console output it is bare.
//! - G-F4 carries two sub-cases: a clean command that breaks a contract
//!   inherits the post-check code (CM-G-3), and a command that both breaks
//!   a contract and exits nonzero keeps its own code (CM-G-2), which is
//!   what kills mutation 5.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_gate_{tag}_{}_{nanos}", std::process::id()));
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

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_invar")
}

/// Run the gate in `dir`; capture (exit, stdout, stderr) separately.
fn gate(dir: &Path, pre_args: &[&str], cmd: &[&str]) -> (i32, String, String) {
    let mut c = Command::new(bin());
    c.arg("gate").args(pre_args).arg("--").args(cmd).current_dir(dir).stdout(Stdio::piped()).stderr(Stdio::piped());
    let o = c.output().unwrap();
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).to_string(),
        String::from_utf8_lossy(&o.stderr).to_string(),
    )
}

fn invar(dir: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(bin()).args(no_reuse(args)).current_dir(dir).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn git(dir: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(["-c", "user.email=t@t", "-c", "user.name=t", "-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

const SCHEMAS: &str = "import { z } from \"zod\";\nexport const OrderResponse = z.object({\n  id: z.string(),\n  total_minor: z.number(),\n  currency_code: z.string(),\n  nested: z.object({ inner_ref: z.string() }),\n});\n";

const SCHEMAS_RENAMED: &str = "import { z } from \"zod\";\nexport const OrderResponse = z.object({\n  id: z.string(),\n  total_minor: z.number(),\n  currency_kode: z.string(),\n  nested: z.object({ inner_ref: z.string() }),\n});\n";

const ROUTE: &str = "import { OrderResponse } from \"../../../../lib/schemas\";\nexport async function GET() {\n  const order = load();\n  return Response.json(OrderResponse.parse(order));\n}\n";

const PRODUCER_TOML: &str = "[[invariant]]\nkind = \"egress\"\nid = \"egress:OrderResponse\"\nproducer = \"lib/schemas.ts\"\nshape = \"OrderResponse\"\nboundary = \"app/api/v1/orders/route.ts\"\neligibility = \"versioned-path\"\nanchor = \"GET /v1/orders\"\n";

/// A locked producer repository whose one contract HOLDS.
fn holding(tag: &str) -> PathBuf {
    let d = ws(tag, &[("lib/schemas.ts", SCHEMAS), ("lib/schemas_v2.ts", SCHEMAS_RENAMED), ("app/api/v1/orders/route.ts", ROUTE), ("invar.toml", PRODUCER_TOML)]);
    git(&d, &["init", "-q"]);
    git(&d, &["add", "-A"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    let (c, out) = invar(&d, &["lock"]);
    assert_eq!(c, 0, "{out}");
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "pre-condition: the repo must hold: {out}");
    d
}

/// The same repository after the producer renamed a locked field: VIOLATED.
fn violated(tag: &str) -> PathBuf {
    let d = holding(tag);
    put(&d, "lib/schemas.ts", SCHEMAS_RENAMED);
    let (c, _) = invar(&d, &["check"]);
    assert_eq!(c, 1, "pre-condition: the repo must be violated");
    d
}

// ------------------------------------------------------------- G: the gate

#[test]
fn g_f1_holding_repo_clean_command_exits_0() {
    let d = holding("gf1");
    let (c, _out, err) = gate(&d, &[], &[bin(), "explain", "0"]);
    assert_eq!(c, 0, "{err}");
    assert!(err.contains("GATE"), "{err}");
    assert!(!err.contains("GATE REFUSED"), "{err}");
}

#[test]
fn g_f2_violated_repo_refuses_and_runs_nothing() {
    let d = violated("gf2");
    let marker = d.join("marker.txt");
    let (c, _out, err) = gate(&d, &[], &["sh", "-c", "echo ran > marker.txt"]);
    assert_eq!(c, 1, "the refusal carries the pre-check's own code: {err}");
    assert!(err.contains("GATE REFUSED") && err.contains("command not run"), "{err}");
    assert!(!marker.exists(), "the command must never have run");
}

#[test]
fn g_f3_failing_command_code_is_forwarded_untouched() {
    let d = holding("gf3");
    let (c, out, err) = gate(&d, &[], &[bin(), "explain", "NOT-A-CODE"]);
    assert_eq!(c, 3, "the command's own exit is the primary signal: {err}");
    assert!(out.contains("UNKNOWN-EXPLAIN-CODE"), "the wrapped command's stdout passes through: {out}");
    assert!(!err.contains("GATE REFUSED"), "{err}");
}

#[test]
fn g_f4_clean_command_that_breaks_a_contract_inherits_the_post_check() {
    // CM-G-3: the command exits 0 but renames a locked field.
    let d = holding("gf4a");
    let (c, _out, err) = gate(&d, &[], &["sh", "-c", "cp lib/schemas_v2.ts lib/schemas.ts"]);
    assert_eq!(c, 1, "success that broke a locked contract can never look green: {err}");
    assert!(err.contains("GATE DRIFT") && err.contains("lib/schemas.ts"), "drift is stated and names the file: {err}");
    assert!(err.contains("VIOLATED"), "the post-check verdict is relayed: {err}");
    // CM-G-2: the command breaks the contract AND fails; its own code wins.
    let d2 = holding("gf4b");
    let (c, _out, err) = gate(&d2, &[], &["sh", "-c", "cp lib/schemas_v2.ts lib/schemas.ts; exit 9"]);
    assert_eq!(c, 9, "a failing command is never masked by the post-check: {err}");
    assert!(err.contains("GATE DRIFT"), "{err}");
}

#[test]
fn g_f5_proof_flag_passes_through_to_both_checks() {
    // A producer emits; a consumer imports; the gate runs with --proof.
    let d = holding("gf5p");
    let (c, out) = invar(&d, &["lock", "--emit-proof", "egress:OrderResponse"]);
    assert_eq!(c, 0, "{out}");
    let p = std::fs::read_to_string(d.join("invar.proof")).unwrap();
    let dg = p.lines().find_map(|l| l.strip_prefix("digest = \"")).unwrap().trim_end_matches('"').to_string();
    let toml = format!(
        "[[proof.import]]\nproof = \"vendor/orders.proof\"\ncontract_id = \"egress:OrderResponse\"\nanchor = \"GET /v1/orders\"\nfields = \"id total_minor\"\naccepted_epoch = \"1\"\naccepted_digest = \"{dg}\"\n"
    );
    let cons = ws("gf5c", &[("vendor/orders.proof", &p), ("invar.toml", &toml)]);
    let (c, _out, err) = gate(&cons, &["--proof"], &[bin(), "explain", "0"]);
    assert_eq!(c, 0, "{err}");
    // Tampered proof: the pre-check refuses at the proof's own exit 3.
    let cons2 = ws("gf5t", &[("vendor/orders.proof", &p.replace("id nested", "id owner nested")), ("invar.toml", &toml)]);
    let (c, _out, err) = gate(&cons2, &["--proof"], &[bin(), "explain", "0"]);
    assert_eq!(c, 3, "{err}");
    assert!(err.contains("GATE REFUSED"), "{err}");
}

#[test]
fn g_f6_wrapped_stdout_is_byte_identical_and_gate_lines_stay_on_stderr() {
    let d = holding("gf6");
    let bare = Command::new(bin()).args(["explain", "0"]).current_dir(&d).output().unwrap();
    let (c, out, err) = gate(&d, &[], &[bin(), "explain", "0"]);
    assert_eq!(c, 0, "{err}");
    assert_eq!(out.as_bytes(), &bare.stdout[..], "stdout must pass through byte-identical");
    assert!(!out.contains("GATE "), "no gate line may pollute stdout");
    assert!(err.contains("GATE "), "{err}");
}

#[test]
fn g_f7_argv_reaches_the_command_verbatim_no_shell() {
    let d = holding("gf7");
    let (c, out, _err) = gate(&d, &[], &[bin(), "explain", "NOT A CODE"]);
    assert_eq!(c, 3);
    assert!(out.contains("UNKNOWN-EXPLAIN-CODE: NOT A CODE is not"), "one argument must stay one argument: {out}");
}

#[test]
fn g_f8_empty_command_is_exit_3_and_nothing_runs() {
    let d = holding("gf8");
    let o = Command::new(bin()).args(["gate", "--"]).current_dir(&d).output().unwrap();
    assert_eq!(o.status.code(), Some(3), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(String::from_utf8_lossy(&o.stderr).contains("usage"), "the usage is stated");
    // Without `--`, clap refuses with its CLI-wide usage error (G-INT-1).
    let o2 = Command::new(bin()).args(["gate", "echo"]).current_dir(&d).output().unwrap();
    assert_ne!(o2.status.code(), Some(0));
}

#[test]
fn g_n1_missing_manifest_refuses_like_check() {
    let d = ws("gn1", &[]);
    let marker = d.join("marker.txt");
    let (c, _out, err) = gate(&d, &[], &["sh", "-c", "echo ran > marker.txt"]);
    assert_eq!(c, 3, "{err}");
    assert!(err.contains("GATE REFUSED"), "{err}");
    assert!(!marker.exists());
}

#[test]
fn g_m1_gate_stderr_is_deterministic() {
    let d = holding("gm1");
    // SECURITY-SPEC r0, S1: an unsigned lock is no longer reused locally, so
    // the first run evaluates and warms the keyed cache. Determinism is
    // compared on the two runs after that.
    let _ = gate(&d, &[], &[bin(), "explain", "0"]);
    let (c1, _o1, e1) = gate(&d, &[], &[bin(), "explain", "0"]);
    let (c2, _o2, e2) = gate(&d, &[], &[bin(), "explain", "0"]);
    assert_eq!((c1, e1), (c2, e2), "the gate's own lines must be byte-identical across runs");
}

#[test]
fn g_m2_gate_module_exists_with_no_network_or_shell_capability() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/gate.rs"))
        .expect("src/gate.rs must exist (Class 4)");
    for needle in ["std::net", "TcpStream", "reqwest", "CurlTransport", "http://", "https://", "sh -c", "\"sh\"", "\"bash\"", "cmd.exe"] {
        assert!(!src.contains(needle), "gate.rs must not reference {needle}");
    }
}

// ----------------------------------------------------- L: the diagnostic link

fn init_out(tag: &str, files: &[(&str, &str)]) -> String {
    let d = ws(tag, files);
    let o = Command::new(bin()).args(["init", "--stdout", "--root"]).arg(&d).output().unwrap();
    assert_eq!(o.status.code(), Some(0));
    String::from_utf8(o.stdout).unwrap()
}

/// A repo whose init output carries ABSTAIN lines (H2, single package).
const ABSTAINING: &[(&str, &str)] = &[
    ("lib/order.ts", "import { z } from \"zod\";\nexport const OrderIn = z.object({\n  total_minor: z.number(),\n  currency_code: z.string(),\n  unread_flag: z.boolean(),\n});\n"),
    ("src/consume.js", "import { OrderIn } from \"../lib/order.js\";\nexport function f(row) {\n  return [row.total_minor, row.currency_code];\n}\n"),
];

#[test]
fn l_f1_init_prints_exactly_one_trailing_pointer_only_with_abstentions() {
    // Option B (ruling of 2026-10-18): the pointer is the PENULTIMATE line,
    // immediately before the SUMMARY terminator, which stays last forever as
    // the machine-readable boundary. The repo carries TWO abstentions (H2
    // plus an unreadable file) so a per-abstention pointer mutant dies.
    // H2 (single package) plus a K3 AMBIG-RECEIVER (two qualifying shapes).
    let d = ws(
        "lf1a",
        &[
            ("package.json", "{\"name\":\"app\",\"private\":true}\n"),
            ("src/token.ts", "module.exports = {\n  attributes: {\n    accessKey: { type: 'string' },\n    lastUsed: { type: 'datetime' },\n    permissions: { type: 'json' },\n    expiresAt: { type: 'datetime' },\n  },\n};\n"),
            ("src/token2.ts", "import { z } from \"zod\";\nexport const ApiToken = z.object({\n  accessKey: z.string(),\n  lastUsed: z.string(),\n  permissions: z.array(z.string()),\n  expiresAt: z.string(),\n});\n"),
            ("src/service.ts", "const SELECT_FIELDS = ['id', 'lastUsed', 'accessKey', 'permissions', 'expiresAt'];\nexport function list(db) { return db.query({ select: SELECT_FIELDS }); }\n"),
        ],
    );
    let o = Command::new(bin()).args(["init", "--stdout", "--root"]).arg(&d).output().unwrap();
    assert_eq!(o.status.code(), Some(0));
    let with = String::from_utf8(o.stdout).unwrap();
    assert!(with.matches("# ABSTAIN ").count() >= 2, "pre-condition, two abstentions: {with}");
    assert_eq!(with.matches("codes: run").count(), 1, "exactly one pointer: {with}");
    let lines: Vec<&str> = with.trim_end().lines().collect();
    assert!(lines.last().unwrap().starts_with("# SUMMARY "), "SUMMARY stays the terminator: {with}");
    assert!(lines[lines.len() - 2].starts_with("# codes: run `invar explain"), "the pointer sits immediately before it: {with}");
    // A workspace with two packages and one clean pair: zero abstentions.
    let clean = init_out(
        "lf1b",
        &[
            ("package.json", "{\"name\":\"root\",\"private\":true,\"workspaces\":[\"a\",\"b\"]}\n"),
            ("a/package.json", "{\"name\":\"a\"}\n"),
            ("b/package.json", "{\"name\":\"b\"}\n"),
            ("a/src/order.ts", "import { z } from \"zod\";\nexport const OrderIn = z.object({\n  total_minor: z.number(),\n  currency_code: z.string(),\n});\n"),
            ("b/src/use.js", "import { OrderIn } from \"../../a/src/order.js\";\nexport function f(r) {\n  return [r.total_minor, r.currency_code];\n}\n"),
        ],
    );
    if !clean.contains("# ABSTAIN ") {
        assert!(!clean.contains("codes: run"), "zero abstentions, zero pointer: {clean}");
    } else {
        // If this synthetic repo abstains anywhere, the pointer must still be
        // exactly one and last; the zero-pointer arm is then asserted by the
        // existing suites' no-abstention fixtures at Gate 4.
        assert_eq!(clean.matches("codes: run").count(), 1, "{clean}");
    }
}

#[test]
fn l_f2_check_proof_prints_the_bare_pointer_only_on_abstain() {
    let toml = "[[proof.import]]\nproof = \"vendor/missing.proof\"\ncontract_id = \"egress:X\"\nanchor = \"GET /v1/x\"\nfields = \"id\"\naccepted_epoch = \"1\"\naccepted_digest = \"a\"\n";
    let cons = ws("lf2", &[("invar.toml", toml)]);
    let (c, out) = invar(&cons, &["check", "--proof"]);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("ABSTAIN PROOF-NOT-FOUND"), "{out}");
    assert_eq!(out.matches("codes: run").count(), 1, "exactly one pointer: {out}");
    assert!(out.trim_end().lines().last().unwrap().starts_with("codes: run `invar explain"), "final line in check --proof, where no terminator convention exists: {out}");
}

#[test]
fn l_f3_init_output_stays_byte_deterministic() {
    let a = init_out("lf3a", ABSTAINING);
    let b = init_out("lf3b", ABSTAINING);
    let strip = |s: &str| s.replace("lf3a", "X").replace("lf3b", "X");
    assert_eq!(strip(&a), strip(&b));
}

#[test]
fn l_f4_the_pointer_names_invar_explain_verbatim() {
    let with = init_out("lf4", ABSTAINING);
    assert!(with.contains("invar explain"), "{with}");
    assert!(with.contains("for any ABSTAIN code above"), "{with}");
}

/// CHECK-INCR-1 (K-2): these tests exercise evaluation, so `check` runs with
/// `--no-reuse`; reuse is tested in check_incr1_test.rs.
fn no_reuse<'a>(args: &[&'a str]) -> Vec<&'a str> {
    let mut v = args.to_vec();
    if v.first() == Some(&"check") && !v.contains(&"--no-reuse") {
        v.push("--no-reuse");
    }
    v
}
