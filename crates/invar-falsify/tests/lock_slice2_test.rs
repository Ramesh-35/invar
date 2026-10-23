//! LOCK-SLICE2 fixtures (founder ruling of 2026-10-21, after
//! STAGE3-EVAL-008 found excalidraw's one-line
//! `type RenderPositionOffset = Readonly<{ x: number; y: number }>` locking
//! zero fields): lock reads members written on the declaration's own line
//! inside `= Readonly<{ ... }>`, the one wrapped object form init proposes
//! (codepairs `type_kind`), exactly as LOCK-SLICE1 reads `= { ... }`.
//! Nothing else is opened. Each fixture is a producer TypeScript file, a
//! consumer and one schema contract; lock, then check, run as the CLI.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_lk2_{tag}_{}_{nanos}", std::process::id()));
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
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(no_reuse(args)).current_dir(dir).output().unwrap();
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



const R_USE: &str = "import type { P } from \"../lib/p\";\nexport function f(o: P) {\n  return [o.x, o.y];\n}\n";
const R_TOML: &str = "[[invariant]]\nkind = \"schema\"\nid = \"code:p\"\nproducer = \"lib/p.ts\"\nconsumer = \"src/use.ts\"\nevidence = \"P x y\"\n";
const ONE_LINE_READONLY: &str = "export type P = Readonly<{ x: number; y: number }>;\n";

// --------------------------------------------- L2b: one-line Readonly bodies

#[test]
fn l2_f1_a_one_line_readonly_type_binds_its_members_and_holds() {
    let d = ws("f1", &[("lib/p.ts", ONE_LINE_READONLY), ("src/use.ts", R_USE), ("invar.toml", R_TOML)]);
    let ((_, lock_out), (code, out)) = lock_check(&d);
    assert!(lock_out.contains("+ contract code:p (2 bindings)"), "{lock_out}");
    assert_eq!(binds(&d), vec!["P.x number".to_string(), "P.y number".to_string()]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("  [HOLDS]    code:p (schema)\n"), "{out}");
    assert!(out.contains("  durability: held since "), "{out}");
}

#[test]
fn l2_f2_a_member_removed_is_violated() {
    let d = ws("f2", &[("lib/p.ts", ONE_LINE_READONLY), ("src/use.ts", R_USE), ("invar.toml", R_TOML)]);
    let _ = lock_check(&d);
    put(&d, "lib/p.ts", "export type P = Readonly<{ x: number }>;\n");
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] code:p (schema)"), "{out}");
    assert!(out.contains("P.y"), "{out}");
}

#[test]
fn l2_f3_spacing_and_nesting() {
    // `Readonly <{`, a nested object member kept whole, a trailing `>`.
    let ty = "export type P = Readonly <{ inner: { q: number }; x: number; y: number }> ;\n";
    let d = ws("f3", &[("lib/p.ts", ty), ("src/use.ts", R_USE), ("invar.toml", R_TOML)]);
    let ((_, lock_out), (code, out)) = lock_check(&d);
    assert!(lock_out.contains("(2 bindings)"), "{lock_out}");
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    assert!(lock.contains("members = \"code P inner x y\""), "{lock}");
    assert_eq!(code, 0, "{out}");
}

#[test]
fn l2_f4_other_wrappers_stay_unread() {
    // Partial, Required and unknown generics are not object bodies init
    // proposes; they stay as before: 0 bindings, NOT EVALUATED.
    for (i, w) in ["Partial", "Required", "Box"].iter().enumerate() {
        let ty = format!("export type P = {w}<{{ x: number; y: number }}>;\n");
        let d = ws(&format!("f4{i}"), &[("lib/p.ts", &ty), ("src/use.ts", R_USE), ("invar.toml", R_TOML)]);
        let ((_, lock_out), (code, out)) = lock_check(&d);
        assert!(lock_out.contains("+ contract code:p (0 bindings)"), "{w}: {lock_out}");
        assert_eq!(code, 4, "{w}: {out}");
    }
}

#[test]
fn l2_f5_a_readonly_constraint_is_not_a_member() {
    // the opening brace after `= Readonly<` only: a Readonly in a type
    // parameter's constraint is never opened.
    let ty = "export type P<T extends Readonly<{ q: number }> = Readonly<{ q: number }>> = { x: number; y: number; t: T };\n";
    let d = ws("f5", &[("lib/p.ts", ty), ("src/use.ts", R_USE), ("invar.toml", R_TOML)]);
    let ((_, lock_out), _) = lock_check(&d);
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    assert!(lock_out.contains("(2 bindings)"), "{lock_out}");
    assert!(lock.contains("members = \"code P t x y\""), "{lock}");
}

#[test]
fn l2_f6_a_readonly_member_type_is_not_a_body() {
    // A member whose type is Readonly<{...}> is a member, not a new body.
    let ty = "export type P = { x: number; y: number; r: Readonly<{ q: number }> };\n";
    let d = ws("f6", &[("lib/p.ts", ty), ("src/use.ts", R_USE), ("invar.toml", R_TOML)]);
    let ((_, lock_out), _) = lock_check(&d);
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    assert!(lock_out.contains("(2 bindings)"), "{lock_out}");
    assert!(lock.contains("members = \"code P r x y\""), "{lock}");
}

// ------------------------------------------------------------ no regression

#[test]
fn l2_n1_a_multi_line_readonly_type_is_unchanged() {
    let ty = "export type P = Readonly<{\n  x: number;\n  y: number;\n}>;\n";
    let d = ws("n1", &[("lib/p.ts", ty), ("src/use.ts", R_USE), ("invar.toml", R_TOML)]);
    let ((_, lock_out), (code, out)) = lock_check(&d);
    assert!(lock_out.contains("(2 bindings)"), "{lock_out}");
    assert_eq!(binds(&d), vec!["P.x number".to_string(), "P.y number".to_string()]);
    assert_eq!(code, 0, "{out}");
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
