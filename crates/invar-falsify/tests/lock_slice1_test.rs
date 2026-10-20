//! LOCK-SLICE1 fixtures (founder ruling of 2026-10-20, after
//! STAGE3-EVAL-005 finding S5-F-01): check never reports HOLDS for a
//! contract whose lock binds no field (L1), and lock reads TypeScript
//! members written on a declaration's own line (L2). Each fixture is a
//! producer TypeScript file, a consumer, and one schema contract; lock, then
//! check, run as the CLI.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_lk1_{tag}_{}_{nanos}", std::process::id()));
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
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(dir).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

/// lock then check, each twice, byte-identical (L-M1).
fn lock_check(d: &Path) -> ((i32, String), (i32, String)) {
    let l = invar(d, &["lock"]);
    assert_eq!(l.0, 0, "{}", l.1);
    let lock_a = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    let l2 = invar(d, &["lock"]);
    assert_eq!(l2.0, 0, "{}", l2.1);
    let lock_b = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    assert_eq!(lock_a.replace(recorded(&lock_a).as_str(), ""), lock_b.replace(recorded(&lock_b).as_str(), ""), "lock is not deterministic");
    let c = invar(d, &["check"]);
    assert_eq!(c, invar(d, &["check"]), "check is not deterministic");
    (l, c)
}

fn recorded(lock: &str) -> String {
    lock.lines().find(|l| l.starts_with("recorded_at = ")).unwrap_or("").to_string()
}

fn binds(d: &Path) -> Vec<String> {
    let t = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    t.lines().filter_map(|l| l.strip_prefix("bind = \"code ")).map(|l| l.splitn(2, ' ').nth(1).unwrap_or("").trim_end_matches('"').to_string()).collect()
}

const TOML: &str = "[[invariant]]\nkind = \"schema\"\nid = \"code:limits\"\nproducer = \"lib/limits.ts\"\nconsumer = \"src/use.ts\"\nevidence = \"Limits maxBytes maxLines\"\n";
const USE: &str = "import type { Limits } from \"../lib/limits\";\nexport function f(l: Limits) {\n  return [l.maxBytes, l.maxLines];\n}\n";
const ONE_LINE_TYPE: &str = "export type Limits = { readonly maxBytes: number; readonly maxLines: number; readonly retain: \"head\" | \"tail\" };\n";

const REASON: &str = "the lock binds no field of this contract (0 bindings), so there is nothing to compare and a HOLDS would be vacuous; make the producer's fields readable and run `invar lock --update`";

// ------------------------------------------------------- L2: one-line members

#[test]
fn l_f1_a_one_line_type_binds_its_members_and_holds() {
    let d = ws("f1", &[("lib/limits.ts", ONE_LINE_TYPE), ("src/use.ts", USE), ("invar.toml", TOML)]);
    let ((_, lock_out), (code, out)) = lock_check(&d);
    assert!(lock_out.contains("+ contract code:limits (2 bindings)"), "{lock_out}");
    assert_eq!(binds(&d), vec!["Limits.maxBytes number".to_string(), "Limits.maxLines number".to_string()]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("  [HOLDS]    code:limits (schema)\n"), "{out}");
    assert!(out.contains("  durability: held since "), "{out}");
}

#[test]
fn l_f2_a_one_line_interface_binds_its_members() {
    let iface = "export interface Limits { maxBytes: number; maxLines: number }\n";
    let d = ws("f2", &[("lib/limits.ts", iface), ("src/use.ts", USE), ("invar.toml", TOML)]);
    let ((_, lock_out), (code, out)) = lock_check(&d);
    assert!(lock_out.contains("(2 bindings)"), "{lock_out}");
    assert_eq!(binds(&d), vec!["Limits.maxBytes number".to_string(), "Limits.maxLines number".to_string()]);
    assert_eq!(code, 0, "{out}");
}

#[test]
fn l_f3_nested_and_generic_members_stay_whole() {
    // `pending` is a member of the nested object, never of Limits; the comma
    // inside Map<string, number> does not split `byKey`.
    let ty = "export type Limits = { inner: { pending: number }; byKey: Map<string, number>; maxBytes: number };\n";
    let reads = "import type { Limits } from \"../lib/limits\";\nexport function f(l: Limits) {\n  return [l.inner, l.byKey, l.maxBytes, l.pending];\n}\n";
    let toml = TOML.replace("Limits maxBytes maxLines", "Limits byKey inner maxBytes");
    let d = ws("f3", &[("lib/limits.ts", ty), ("src/use.ts", reads), ("invar.toml", &toml)]);
    let ((_, lock_out), (code, out)) = lock_check(&d);
    assert!(lock_out.contains("(3 bindings)"), "{lock_out}");
    // Corrected by ruling L-D1 (2026-10-20): lock's member reader cuts a
    // type's text at its first comma, as it does for every multi-line member
    // today (L2: "read exactly as a member line is read today"); whole type
    // text would retype existing bindings, which L3 forbids. What this
    // fixture proves is unchanged: the comma does not split the member.
    // Original assertion, kept verbatim:
    //     vec!["Limits.byKey Map<string, number>".to_string(), "Limits.inner { pending: number }".to_string(), "Limits.maxBytes number".to_string()]
    assert_eq!(
        binds(&d),
        vec!["Limits.byKey Map<string".to_string(), "Limits.inner { pending: number }".to_string(), "Limits.maxBytes number".to_string()]
    );
    assert_eq!(code, 0, "{out}");
}

#[test]
fn l_f4_a_member_removed_from_a_one_line_type_is_violated() {
    let d = ws("f4", &[("lib/limits.ts", ONE_LINE_TYPE), ("src/use.ts", USE), ("invar.toml", TOML)]);
    let _ = lock_check(&d);
    put(&d, "lib/limits.ts", "export type Limits = { readonly maxBytes: number; readonly retain: \"head\" | \"tail\" };\n");
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] code:limits (schema)"), "{out}");
    assert!(out.contains("Limits.maxLines"), "{out}");
}

#[test]
fn l_f6_a_type_parameter_constraint_is_not_a_member() {
    // Added at Gate 3 (finding L-F-01): the opening brace is the first one
    // outside `<...>`, so the constraint's `{ pending: number }` is never
    // read as members of Limits.
    let ty = "export type Limits<T extends { pending: number } = { pending: number }> = { maxBytes: number; maxLines: number; inner: T };\n";
    let reads = "import type { Limits } from \"../lib/limits\";\nexport function f(l: Limits) {\n  return [l.maxBytes, l.maxLines, l.pending];\n}\n";
    let d = ws("f6", &[("lib/limits.ts", ty), ("src/use.ts", reads), ("invar.toml", TOML)]);
    let ((_, lock_out), (code, out)) = lock_check(&d);
    assert!(lock_out.contains("(2 bindings)"), "{lock_out}");
    assert_eq!(binds(&d), vec!["Limits.maxBytes number".to_string(), "Limits.maxLines number".to_string()]);
    assert_eq!(code, 0, "{out}");
}

// ------------------------------------------------------- L1: no vacuous HOLDS

#[test]
fn l_f5_a_zero_binding_contract_is_not_evaluated() {
    let pick = "import type { Base } from \"./base\";\nexport type Limits = Pick<Base, \"maxBytes\" | \"maxLines\">;\n";
    let base = "export interface Base {\n  maxBytes: number;\n  maxLines: number;\n}\n";
    let d = ws("f5", &[("lib/limits.ts", pick), ("lib/base.ts", base), ("src/use.ts", USE), ("invar.toml", TOML)]);
    let ((_, lock_out), (code, out)) = lock_check(&d);
    assert!(lock_out.contains("+ contract code:limits (0 bindings)"), "{lock_out}");
    assert_eq!(code, 4, "{out}");
    assert!(!out.contains("[HOLDS]"), "{out}");
    assert!(out.contains("  [NOT EVALUATED] code:limits (schema)"), "{out}");
    assert!(out.contains(REASON), "{out}");
    assert!(!out.contains("durability:"), "{out}");
}

// ------------------------------------------------------------ no regression

#[test]
fn l_n1_a_multi_line_interface_is_byte_identical() {
    let iface = "export interface Limits {\n  maxBytes: number;\n  maxLines: number;\n}\n";
    let d = ws("n1", &[("lib/limits.ts", iface), ("src/use.ts", USE), ("invar.toml", TOML)]);
    let ((_, lock_out), (code, out)) = lock_check(&d);
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    let lock = lock.replace(recorded(&lock).as_str(), "recorded_at = R");
    let today = invar_falsify::lock::format_date(invar_falsify::lock::today_utc());
    assert_eq!(lock.replace(&today, "D"), N1_LOCK);
    assert_eq!(code, 0);
    let out = out
        .lines()
        .map(|l| if l.contains("producer unchanged since lock") { "      note: producer unchanged since lock (R)" } else { l })
        .collect::<Vec<_>>()
        .join("\n")
        .replace(&today, "D");
    assert_eq!(out, N1_CHECK, "{lock_out}");
}

const N1_LOCK: &str = r#"# invar.lock: the producer fields this repository depends on.
# Written by `invar lock`. Accept a reviewed producer change with
# `invar lock --update`. Waivers are written by hand and kept.

[lock]
format = "1"

[[contract]]
id = "code:limits"
producer = "lib/limits.ts"
blake3 = "ae219e153b109533b3c0645299c29c4919a6233eae01dfa8076f4a84eb348c8e"
recorded_at = R
bind = "code D Limits.maxBytes number"
bind = "code D Limits.maxLines number"
members = "code Limits maxBytes maxLines"
"#;
const N1_CHECK: &str = r#"INVAR check (1 contract(s))
  [HOLDS]    code:limits (schema)
      note: producer unchanged since lock (R)
  durability: held since D (0 days)
result: ALL 1 contract(s) hold
summary: held 1 | violated 0 | skipped or not evaluated 0
measured: 38 of 70 held-out real cross-repository breaks caught (batches 4-8; docs/epistemic/STATUS.md)"#;
