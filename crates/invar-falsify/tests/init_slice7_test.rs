//! INIT-DISCOVERY-SLICE7 fixtures (rulings S7-D1 to S7-D5 of 2026-10-21):
//! a receiver assigned `r = [await] f(...)` counts as declared as shape S
//! when f is imported by name from S's package and that package declares f
//! exactly once, with a written return type of S, Promise<S> (awaited),
//! Readonly<S>, S[] or Array<S>. Applied uniformly (S7-D1), and only to a
//! shape that does not qualify without it, so no existing evidence moves.
//! Each fixture is a two-package workspace (`a` produces, `b` consumes), run
//! through `invar init --stdout` twice, byte-identical.

use std::path::{Path, PathBuf};
use std::process::Command;

const ROOT_AB: (&str, &str) = ("package.json", r#"{"private":true,"workspaces":["a","b"]}"#);
const PKG_A: (&str, &str) = ("a/package.json", r#"{"name":"a"}"#);
const PKG_B: (&str, &str) = ("b/package.json", r#"{"name":"b"}"#);
const RESULT: &str = "export interface Result {\n  stdout_text: string;\n  exit_code: number;\n}\n";

const ROUTE_STATUS: &str = "# status: PROPOSED (heuristic; receiver assigned from imported function return type; field writes excluded; types are not inferred)";
const MIXED_STATUS: &str = "# status: PROPOSED (heuristic; counted reads are on receivers declared as the shape in this file or assigned from an imported function's declared return type; field writes excluded; types are not inferred)";
const DECLARED_STATUS: &str = "# status: PROPOSED (heuristic; each counted read is on a receiver declared as the shape in this file; field writes excluded; types are not inferred)";

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s7_{tag}_{}_{nanos}", std::process::id()));
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    dir
}

struct Out {
    proposals: usize,
    status: Vec<String>,
    e4: Vec<String>,
    e5: Vec<String>,
    abstain: Vec<String>,
    summary: String,
    raw: String,
}

fn init(dir: &Path) -> Out {
    let run = || {
        let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(["init", "--stdout", "--root"]).arg(dir).output().unwrap();
        assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8(o.stdout).unwrap()
    };
    let (a, b) = (run(), run());
    assert_eq!(a, b, "init output is not deterministic");
    Out {
        proposals: a.lines().filter(|l| *l == "[[invariant]]").count(),
        status: a.lines().filter(|l| l.starts_with("# status: ")).map(String::from).collect(),
        e4: a.lines().filter_map(|l| l.strip_prefix("# E4 ")).map(String::from).collect(),
        e5: a.lines().filter_map(|l| l.strip_prefix("# E5 ")).filter(|l| l.contains(" fields read: ")).map(String::from).collect(),
        abstain: a.lines().filter_map(|l| l.strip_prefix("# ABSTAIN ")).map(|l| format!("ABSTAIN {l}")).collect(),
        summary: a.lines().last().unwrap_or("").trim_start_matches("# ").to_string(),
        raw: a,
    }
}

/// `a/run.ts` is `run`; `b/src/x.ts` is `x`; plus `extra` files.
fn case(tag: &str, run: &str, x: &str, extra: &[(&str, &str)]) -> Out {
    let mut files = vec![ROOT_AB, PKG_A, PKG_B, ("a/run.ts", run), ("b/src/x.ts", x)];
    files.extend_from_slice(extra);
    init(&ws(tag, &files))
}

fn exec_returning(ty: &str, asynch: bool) -> String {
    format!("{}export {}function exec(c: string): {ty} {{\n  return null as any;\n}}\n", RESULT, if asynch { "async " } else { "" })
}

const ONE: &str = "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE";
const R6_LOUD: &str = "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE";
const R6_LINE: &str = "ABSTAIN R6 b/src/x.ts:1 reads exit_code, stdout_text of Result (a/run.ts:1) without importing it by name (out of v1)";
const E5_45: &str = "Result fields read: exit_code (b/src/x.ts:4), stdout_text (b/src/x.ts:4)";

fn route_e4(decl_line: usize) -> String {
    format!("Result not imported by name; resolved via return type of exec() declared in a/run.ts:{decl_line}, exec imported at b/src/x.ts:1 from \"a/run\"")
}

fn proposed_by_route(o: &Out, e4: &str, e5: &str) {
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert_eq!(o.status, vec![ROUTE_STATUS.to_string()], "{}", o.raw);
    assert_eq!(o.e4, vec![e4.to_string()], "{}", o.raw);
    assert_eq!(o.e5, vec![e5.to_string()], "{}", o.raw);
    assert!(o.abstain.is_empty(), "{}", o.raw);
    assert_eq!(o.summary, ONE, "{}", o.raw);
}

fn stays_r6(o: &Out) {
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert_eq!(o.abstain, vec![R6_LINE.to_string()], "{}", o.raw);
    assert_eq!(o.summary, R6_LOUD, "{}", o.raw);
}

const X_PLAIN: &str = "import { exec } from 'a/run';\nexport const f = () => {\n  const r = exec('x');\n  return [r.stdout_text, r.exit_code];\n};\n";
const X_AWAIT: &str = "import { exec } from 'a/run';\nexport const f = async () => {\n  const r = await exec('x');\n  return [r.stdout_text, r.exit_code];\n};\n";

// ---------------------------------------------------------- the route (B2)

#[test]
fn s7_f1_the_u01_workspace_proposes_by_return_type() {
    // The slice 1 u01 workspace, verbatim (ruling S7-D4).
    let run = "export function exec(c: string): Result {\n  return null as any;\n}\nexport interface Result {\n  stdout_text: string;\n  exit_code: number;\n}\n";
    let o = case("f1", run, X_PLAIN, &[]);
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert_eq!(o.status, vec![ROUTE_STATUS.to_string()], "{}", o.raw);
    assert_eq!(o.e4, vec![route_e4(1)], "{}", o.raw);
    assert_eq!(o.e5, vec![E5_45.to_string()], "{}", o.raw);
    assert!(o.raw.contains("# E2 Result declared at a/run.ts:4\n"), "{}", o.raw);
    assert_eq!(o.summary, ONE, "{}", o.raw);
}

#[test]
fn s7_f2_an_awaited_promise_proposes() {
    let o = case("f2", &exec_returning("Promise<Result>", true), X_AWAIT, &[]);
    proposed_by_route(&o, &route_e4(5), E5_45);
}

#[test]
fn s7_f2b_a_promise_not_awaited_is_not_the_shape() {
    let o = case("f2b", &exec_returning("Promise<Result>", true), X_PLAIN, &[]);
    stays_r6(&o);
}

#[test]
fn s7_f3_readonly_proposes() {
    let o = case("f3", &exec_returning("Readonly<Result>", false), X_PLAIN, &[]);
    proposed_by_route(&o, &route_e4(5), E5_45);
}

#[test]
fn s7_f4_an_array_binds_its_elements() {
    let x = "import { exec } from 'a/run';\nexport const f = () => {\n  const rs = exec('x');\n  for (const r of rs) {\n    use(r.stdout_text, r.exit_code);\n  }\n};\n";
    let o = case("f4", &exec_returning("Result[]", false), x, &[]);
    proposed_by_route(&o, &route_e4(5), "Result fields read: exit_code (b/src/x.ts:5), stdout_text (b/src/x.ts:5)");
}

#[test]
fn s7_f4b_array_generic_and_callbacks() {
    let x = "import { exec } from 'a/run';\nexport const f = () => {\n  const rs = exec('x');\n  return rs.map((r) => [r.stdout_text, r.exit_code]);\n};\n";
    let o = case("f4b", &exec_returning("Array<Result>", false), x, &[]);
    proposed_by_route(&o, &route_e4(5), E5_45);
}

#[test]
fn s7_f11_a_relative_cross_package_import_proposes() {
    let x = "import { exec } from '../../a/run';\nexport const f = () => {\n  const r = exec('x');\n  return [r.stdout_text, r.exit_code];\n};\n";
    let o = case("f11", &exec_returning("Result", false), x, &[]);
    proposed_by_route(
        &o,
        "Result not imported by name; resolved via return type of exec() declared in a/run.ts:5, exec imported at b/src/x.ts:1 from \"../../a/run\"",
        E5_45,
    );
}

// ------------------------------------------------- refusals: R6 stays (B2)

#[test]
fn s7_f5_no_written_return_type_ties_nothing() {
    let run = format!("{RESULT}export function exec(c: string) {{\n  return null as any;\n}}\n");
    stays_r6(&case("f5", &run, X_PLAIN, &[]));
}

#[test]
fn s7_f6_a_function_declared_twice_ties_nothing() {
    let other = "export function exec(c: string): number {\n  return 1;\n}\n";
    stays_r6(&case("f6", &exec_returning("Result", false), X_PLAIN, &[("a/other.ts", other)]));
}

#[test]
fn s7_f7_a_type_argument_of_another_type_is_not_the_shape() {
    let run = format!("{}export interface Box<T> {{\n  inner: T;\n}}\nexport function exec(c: string): Box<Result> {{\n  return null as any;\n}}\n", RESULT);
    let o = case("f7", &run, X_PLAIN, &[]);
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert!(o.abstain.contains(&R6_LINE.to_string()), "{}", o.raw);
}

#[test]
fn s7_f8_a_property_of_the_call_result_is_not_the_shape() {
    for (i, call) in ["exec('x').inner", "exec('x')?.inner", "exec('x')[0]", "exec('x')!", "exec('x') ?? fallback", "exec('x') || fallback"].iter().enumerate() {
        let x = format!("import {{ exec }} from 'a/run';\nexport const f = () => {{\n  const r = {call};\n  return [r.stdout_text, r.exit_code];\n}};\n");
        stays_r6(&case(&format!("f8{i}"), &exec_returning("Result", false), &x, &[]));
    }
}

#[test]
fn s7_f9_one_read_through_the_route_is_not_enough() {
    let x = "import { exec } from 'a/run';\nexport const f = (o: any) => {\n  const r = exec('x');\n  return [r.stdout_text, o.exit_code];\n};\n";
    stays_r6(&case("f9", &exec_returning("Result", false), x, &[]));
}

#[test]
fn s7_f10_an_unsupported_return_form_ties_nothing() {
    for (i, ty) in ["Result | undefined", "Partial<Result>", "Result | null", "Promise<Result[]>"].iter().enumerate() {
        stays_r6(&case(&format!("f10{i}"), &exec_returning(ty, ty.starts_with("Promise")), X_AWAIT, &[]));
    }
}

#[test]
fn s7_f15_a_shape_name_declared_twice_in_the_package_ties_nothing() {
    let other = "export interface Result {\n  stdout_text: string;\n  exit_code: number;\n}\n";
    let o = case("f15", &exec_returning("Result", false), X_PLAIN, &[("a/other.ts", other)]);
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert!(o.abstain.iter().all(|a| a.starts_with("ABSTAIN R6 ")), "{}", o.raw);
}

#[test]
fn s7_f16_this_dot_calls_are_not_followed() {
    let x = "import { exec } from 'a/run';\nexport class C {\n  exec = exec;\n  f() {\n    const r = this.exec('x');\n    return [r.stdout_text, r.exit_code];\n  }\n}\n";
    let o = case("f16", &exec_returning("Result", false), x, &[]);
    assert_eq!(o.proposals, 0, "{}", o.raw);
}

// --------------------------------------------- uniform scope (S7-D1, D2)

#[test]
fn s7_f12_a_shape_imported_by_name_is_rescued_from_recv() {
    let x = "import { exec, Result } from 'a/run';\nexport const f = () => {\n  const r = exec('x');\n  return [r.stdout_text, r.exit_code];\n};\n";
    let o = case("f12", &exec_returning("Result", false), x, &[]);
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert_eq!(o.status, vec![ROUTE_STATUS.to_string()], "{}", o.raw);
    assert_eq!(
        o.e4,
        vec!["Result imported at b/src/x.ts:1 from \"a/run\", resolved to a/run.ts; receiver resolved via return type of exec() declared in a/run.ts:5".to_string()],
        "{}",
        o.raw
    );
    assert_eq!(o.e5, vec![E5_45.to_string()], "{}", o.raw);
    assert!(o.abstain.is_empty(), "{}", o.raw);
}

#[test]
fn s7_f13_a_mixed_block_carries_the_combined_line() {
    let run = format!("{RESULT}export interface Other {{\n  mode_flag: string;\n  kind_code: number;\n}}\nexport function make(): Other {{\n  return null as any;\n}}\n");
    let x = "import { make, Result } from 'a/run';\nexport const f = (o: Result) => {\n  const p = make();\n  return [o.stdout_text, o.exit_code, p.mode_flag, p.kind_code];\n};\n";
    let o = case("f13", &run, x, &[]);
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert_eq!(o.status, vec![MIXED_STATUS.to_string()], "{}", o.raw);
    assert_eq!(
        o.e4,
        vec![
            "Other not imported by name; resolved via return type of make() declared in a/run.ts:9, make imported at b/src/x.ts:1 from \"a/run\"".to_string(),
            "Result imported at b/src/x.ts:1 from \"a/run\", resolved to a/run.ts".to_string(),
        ],
        "{}",
        o.raw
    );
    assert_eq!(
        o.e5,
        vec!["Other fields read: kind_code (b/src/x.ts:4), mode_flag (b/src/x.ts:4)".to_string(), "Result fields read: exit_code (b/src/x.ts:4), stdout_text (b/src/x.ts:4)".to_string()],
        "{}",
        o.raw
    );
}

#[test]
fn s7_f14_the_route_never_adds_reads_to_a_shape_that_already_qualifies() {
    // Golden recorded at Gate 1 on the b0a8c79 binary: byte-identical.
    let run = "export interface Result {\n  stdout_text: string;\n  exit_code: number;\n  extra_field: string;\n}\nexport function exec(c: string): Result {\n  return null as any;\n}\n";
    let x = "import { exec, Result } from 'a/run';\nexport const f = (o: Result) => {\n  const r = exec('x');\n  return [o.stdout_text, o.exit_code, r.extra_field];\n};\n";
    let o = case("f14", run, x, &[]);
    assert_eq!(o.status, vec![DECLARED_STATUS.to_string()], "{}", o.raw);
    assert_eq!(o.raw, F14_GOLDEN);
}

// ------------------------------------------------------------ no regression

#[test]
fn s7_n1_a_declared_receiver_proposal_is_byte_identical() {
    // Golden recorded at Gate 1 on the b0a8c79 binary.
    let x = "import { Result } from 'a/run';\nexport const f = (r: Result) => [r.stdout_text, r.exit_code];\n";
    let o = case("n1", &exec_returning("Result", false), x, &[]);
    assert_eq!(o.raw, N1_GOLDEN);
}

// S7-D6 (INIT-DISCOVERY-SLICE7) changed the header sentence of this golden
// and nothing else. Original lines 3 to 6 of the header, kept verbatim:
//     # [[invariant]] block below is a PROPOSAL: a TypeScript shape exported by one
//     # workspace package, imported by name in another, with at least 2 of its
//     # direct fields read there, each on a receiver declared as that shape in
//     # the consumer file (field writes excluded; types are not inferred).
// S8-D3 (INIT-DISCOVERY-SLICE8) changed the header sentence of this golden
// and nothing else. Its S7-D6 lines 4 to 7, kept verbatim:
//     # workspace package, with at least 2 of its direct fields read in a file of
//     # another, each on a receiver declared as that shape in that file or
//     # assigned from a function imported from the shape's package whose declared
//     # return type is the shape (field writes excluded; types are not inferred).
const F14_GOLDEN: &str = r#"# invar.toml generated by `invar init`.
#
# HONEST SCOPE: discovery is HEURISTIC (text patterns, not a parser). Each
# [[invariant]] block below is a PROPOSAL: a TypeScript shape exported by one
# workspace package, with at least 2 of its direct fields read in a file of
# another, each on a receiver declared as that shape in that file, assigned
# from a function imported from the shape's package whose declared return
# type is the shape, or read through a property, at most two hops from a
# receiver typed in that file, whose declared type is the shape (field
# writes excluded; types are not inferred).
# Review every block before trusting a verdict. Schema files are inventory
# only. ABSTAIN lines are refusals to propose, not negative findings. Paths
# are relative to this file.

[discovery]
files_scanned = "5"
truncated = "false"
packages = "2"
candidates = "2"
contracts_proposed = "1"
abstentions = "0"
inventory = "0"

# PROPOSED code:run.ts->x.ts
# status: PROPOSED (heuristic; each counted read is on a receiver declared as the shape in this file; field writes excluded; types are not inferred)
# E1 producer package a at a/package.json (package.json "workspaces" entry "a")
# E2 Result declared at a/run.ts:1
# E3 consumer package b at b/package.json (package.json "workspaces" entry "b")
# E4 Result imported at b/src/x.ts:1 from "a/run", resolved to a/run.ts
# E5 Result fields read: exit_code (b/src/x.ts:4), stdout_text (b/src/x.ts:4)
# E5 Result stoplisted reads: (none)
# E6 pair a/run.ts -> b/src/x.ts
# category: production
# lock: run `invar lock` before `invar check`. Without a lock, check evaluates only payload.<field> reads and says nothing about this proposal (a HOLDS would be vacuous). `invar lock` binds every field of the producer file that the consumer mentions in any form, which is wider than the evidence above.
[[invariant]]
kind = "schema"
id = "code:run.ts->x.ts"
producer = "a/run.ts"
consumer = "b/src/x.ts"
evidence = "Result exit_code stdout_text"

# ---- Abstentions (init declined to propose; not negative findings) ----
# (none)
# ---- Found but NOT proposed (inventory only) ----
# (none)
# SUMMARY packages=2 candidates=2 proposals=1 abstentions=0 inventory=0 status=COMPLETE
"#;
// S7-D6 (INIT-DISCOVERY-SLICE7) changed the header sentence of this golden
// and nothing else. Original lines 3 to 6 of the header, kept verbatim:
//     # [[invariant]] block below is a PROPOSAL: a TypeScript shape exported by one
//     # workspace package, imported by name in another, with at least 2 of its
//     # direct fields read there, each on a receiver declared as that shape in
//     # the consumer file (field writes excluded; types are not inferred).
// S8-D3 (INIT-DISCOVERY-SLICE8) changed the header sentence of this golden
// and nothing else. Its S7-D6 lines 4 to 7, kept verbatim:
//     # workspace package, with at least 2 of its direct fields read in a file of
//     # another, each on a receiver declared as that shape in that file or
//     # assigned from a function imported from the shape's package whose declared
//     # return type is the shape (field writes excluded; types are not inferred).
const N1_GOLDEN: &str = r#"# invar.toml generated by `invar init`.
#
# HONEST SCOPE: discovery is HEURISTIC (text patterns, not a parser). Each
# [[invariant]] block below is a PROPOSAL: a TypeScript shape exported by one
# workspace package, with at least 2 of its direct fields read in a file of
# another, each on a receiver declared as that shape in that file, assigned
# from a function imported from the shape's package whose declared return
# type is the shape, or read through a property, at most two hops from a
# receiver typed in that file, whose declared type is the shape (field
# writes excluded; types are not inferred).
# Review every block before trusting a verdict. Schema files are inventory
# only. ABSTAIN lines are refusals to propose, not negative findings. Paths
# are relative to this file.

[discovery]
files_scanned = "5"
truncated = "false"
packages = "2"
candidates = "1"
contracts_proposed = "1"
abstentions = "0"
inventory = "0"

# PROPOSED code:run.ts->x.ts
# status: PROPOSED (heuristic; each counted read is on a receiver declared as the shape in this file; field writes excluded; types are not inferred)
# E1 producer package a at a/package.json (package.json "workspaces" entry "a")
# E2 Result declared at a/run.ts:1
# E3 consumer package b at b/package.json (package.json "workspaces" entry "b")
# E4 Result imported at b/src/x.ts:1 from "a/run", resolved to a/run.ts
# E5 Result fields read: exit_code (b/src/x.ts:2), stdout_text (b/src/x.ts:2)
# E5 Result stoplisted reads: (none)
# E6 pair a/run.ts -> b/src/x.ts
# category: production
# lock: run `invar lock` before `invar check`. Without a lock, check evaluates only payload.<field> reads and says nothing about this proposal (a HOLDS would be vacuous). `invar lock` binds every field of the producer file that the consumer mentions in any form, which is wider than the evidence above.
[[invariant]]
kind = "schema"
id = "code:run.ts->x.ts"
producer = "a/run.ts"
consumer = "b/src/x.ts"
evidence = "Result exit_code stdout_text"

# ---- Abstentions (init declined to propose; not negative findings) ----
# (none)
# ---- Found but NOT proposed (inventory only) ----
# (none)
# SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE
"#;

// ------------------------- added at Gate 3, after the Gate 1 freeze (K)
// The bytes above are the frozen file (9b1864b7538e). These two close
// fixture gaps that Gate 3 mutants 03 and 14 survived.

#[test]
fn s7_f6b_a_duplicate_declaration_is_refused_in_any_file_order() {
    let later = "export function exec(c: string): Result {\n  return null as any;\n}\n";
    stays_r6(&case("f6b", &exec_returning("Result", false), X_PLAIN, &[("a/z.ts", later)]));
}

#[test]
fn s7_f17_a_function_imported_from_another_package_ties_nothing() {
    let root = ("package.json", r#"{"private":true,"workspaces":["a","b","c"]}"#);
    let pkg_c = ("c/package.json", r#"{"name":"c"}"#);
    let run = format!("{}export const other = 1;\n", exec_returning("Result", false));
    let c_run = "export function exec(c: string): number {\n  return 1;\n}\n";
    let x = "import { other } from 'a/run';\nimport { exec } from 'c/run';\nexport const f = () => {\n  const r = exec('x');\n  return [r.stdout_text, r.exit_code, other];\n};\n";
    let o = init(&ws("f17", &[root, PKG_A, PKG_B, pkg_c, ("a/run.ts", &run), ("c/run.ts", c_run), ("b/src/x.ts", x)]));
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert!(o.abstain.contains(&R6_LINE.to_string()), "{}", o.raw);
}
