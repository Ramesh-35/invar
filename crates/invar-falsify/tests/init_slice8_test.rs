//! INIT-DISCOVERY-SLICE8 fixtures (rulings S8-D0 to S8-D3 of 2026-10-21):
//! a field read counts for shape S through a property whose written type
//! is S, at most two hops from a receiver with a written type T in the
//! consumer file (`r.p.f`, `r.a.p.f`), T resolved through the consumer's
//! own imports or declared in that file, each member type exactly S,
//! Readonly<S>, S[] or Array<S> and resolved to that very S. Applied to
//! every shape, only where it does not qualify without it. Each fixture is
//! a workspace run through `invar init --stdout` twice, byte-identical.

use std::path::{Path, PathBuf};
use std::process::Command;

const ROOT_AB: (&str, &str) = ("package.json", r#"{"private":true,"workspaces":["a","b"]}"#);
const PKG_A: (&str, &str) = ("a/package.json", r#"{"name":"a"}"#);
const PKG_B: (&str, &str) = ("b/package.json", r#"{"name":"b"}"#);
const RESULT: &str = "export interface Result {\n  stdout_text: string;\n  exit_code: number;\n}\n";

const PROP_STATUS: &str = "# status: PROPOSED (heuristic; receiver reached through a property declared as the shape, at most two hops; field writes excluded; types are not inferred)";
const PROP_MIXED: &str = "# status: PROPOSED (heuristic; counted reads are on receivers declared as the shape in this file, assigned from an imported function's declared return type, or reached through a property declared as the shape, at most two hops; field writes excluded; types are not inferred)";
const DECLARED_STATUS: &str = "# status: PROPOSED (heuristic; each counted read is on a receiver declared as the shape in this file; field writes excluded; types are not inferred)";
const HEADER_S8: &str = "# [[invariant]] block below is a PROPOSAL: a TypeScript shape exported by one
# workspace package, with at least 2 of its direct fields read in a file of
# another, each on a receiver declared as that shape in that file, assigned
# from a function imported from the shape's package whose declared return
# type is the shape, or read through a property, at most two hops from a
# receiver typed in that file, whose declared type is the shape (field
# writes excluded; types are not inferred).
";

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s8_{tag}_{}_{nanos}", std::process::id()));
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
    e2: Vec<String>,
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
        e2: a.lines().filter_map(|l| l.strip_prefix("# E2 ")).map(String::from).collect(),
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

/// Result (lines 1 to 4) then `extra` declarations from line 5.
fn run_with(extra: &str) -> String {
    format!("{RESULT}{extra}")
}

const JOB: &str = "export interface Job {\n  result: Result;\n}\n";
const R6_LINE: &str = "ABSTAIN R6 b/src/x.ts:1 reads exit_code, stdout_text of Result (a/run.ts:1) without importing it by name (out of v1)";
const ONE: &str = "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE";
const E5_2: &str = "Result fields read: exit_code (b/src/x.ts:2), stdout_text (b/src/x.ts:2)";
const X_JOB: &str = "import { Job } from 'a/run';\nexport const f = (j: Job) => [j.result.stdout_text, j.result.exit_code];\n";
const JOB_ORIGIN: &str = "Job imported at b/src/x.ts:1 from \"a/run\"";

fn prop_e4(path: &str, decl_line: usize, origin: &str) -> String {
    format!("Result not imported by name; resolved via property {path} declared in a/run.ts:{decl_line}, {origin}")
}

fn proposed_by_property(o: &Out, e4: &str, e5: &str) {
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert_eq!(o.status, vec![PROP_STATUS.to_string()], "{}", o.raw);
    assert_eq!(o.e2, vec!["Result declared at a/run.ts:1".to_string()], "{}", o.raw);
    assert_eq!(o.e4, vec![e4.to_string()], "{}", o.raw);
    assert_eq!(o.e5, vec![e5.to_string()], "{}", o.raw);
    assert!(o.abstain.is_empty(), "{}", o.raw);
}

fn stays_r6(o: &Out) {
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert!(o.abstain.contains(&R6_LINE.to_string()), "{}", o.raw);
}

/// Everything from `[discovery]` on: the proposal sections, independent of
/// the header sentence (S8-D3 changes the header, not the blocks).
fn body(raw: &str) -> &str {
    &raw[raw.find("[discovery]").unwrap()..]
}

// --------------------------------------------------------- the route (B2)

#[test]
fn s8_f1_one_hop_proposes() {
    let o = case("f1", &run_with(JOB), X_JOB, &[]);
    proposed_by_property(&o, &prop_e4("Job.result", 6, JOB_ORIGIN), E5_2);
    assert_eq!(o.summary, ONE, "{}", o.raw);
}

#[test]
fn s8_f1b_a_type_declared_in_the_consumer_file_rescues_a_named_shape() {
    // Result imported by name, read only through a local type's property:
    // RECV before this slice, a proposal after it (uniform scope).
    let x = "import type { Result } from 'a/run';\ninterface Job {\n  result: Result;\n}\nexport const f = (j: Job) => [j.result.stdout_text, j.result.exit_code];\n";
    let o = case("f1b", RESULT, x, &[]);
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert_eq!(o.status, vec![PROP_STATUS.to_string()], "{}", o.raw);
    assert_eq!(
        o.e4,
        vec!["Result imported at b/src/x.ts:1 from \"a/run\", resolved to a/run.ts; receiver resolved via property Job.result declared in b/src/x.ts:3".to_string()],
        "{}",
        o.raw
    );
    assert_eq!(o.e5, vec!["Result fields read: exit_code (b/src/x.ts:5), stdout_text (b/src/x.ts:5)".to_string()], "{}", o.raw);
    assert!(o.abstain.is_empty(), "{}", o.raw);
}

#[test]
fn s8_f2_an_alias_of_the_property_value_proposes() {
    let x = "import { Job } from 'a/run';\nexport const f = (j: Job) => {\n  const r = j.result;\n  return [r.stdout_text, r.exit_code];\n};\n";
    let o = case("f2", &run_with(JOB), x, &[]);
    proposed_by_property(&o, &prop_e4("Job.result", 6, JOB_ORIGIN), "Result fields read: exit_code (b/src/x.ts:4), stdout_text (b/src/x.ts:4)");
}

#[test]
fn s8_f3_an_array_property_binds_its_elements() {
    let jobs = "export interface Job {\n  results: Result[];\n}\n";
    let x = "import { Job } from 'a/run';\nexport const f = (j: Job) => {\n  for (const r of j.results) {\n    use(r.stdout_text, r.exit_code);\n  }\n};\n";
    let o = case("f3", &run_with(jobs), x, &[]);
    proposed_by_property(&o, &prop_e4("Job.results", 6, JOB_ORIGIN), "Result fields read: exit_code (b/src/x.ts:4), stdout_text (b/src/x.ts:4)");
}

#[test]
fn s8_f3b_array_callbacks_and_indexing() {
    let jobs = "export interface Job {\n  results: Array<Result>;\n}\n";
    let x = "import { Job } from 'a/run';\nexport const f = (j: Job) => [j.results.map((r) => r.stdout_text), j.results[0].exit_code];\n";
    let o = case("f3b", &run_with(jobs), x, &[]);
    proposed_by_property(&o, &prop_e4("Job.results", 6, JOB_ORIGIN), E5_2);
}

#[test]
fn s8_f3c_readonly_proposes() {
    let jobs = "export interface Job {\n  result: Readonly<Result>;\n}\n";
    let o = case("f3c", &run_with(jobs), X_JOB, &[]);
    proposed_by_property(&o, &prop_e4("Job.result", 6, JOB_ORIGIN), E5_2);
}

#[test]
fn s8_f4_an_array_receiver_binds_its_elements() {
    for (i, x) in [
        "import { Job } from 'a/run';\nexport const f = (js: Job[]) => [js[0].result.stdout_text, js[0].result.exit_code];\n",
        "import { Job } from 'a/run';\nexport const f = (js: Job[]) => js.map((j) => [j.result.stdout_text, j.result.exit_code]);\n",
    ]
    .iter()
    .enumerate()
    {
        let o = case(&format!("f4{i}"), &run_with(JOB), x, &[]);
        proposed_by_property(&o, &prop_e4("Job.result", 6, JOB_ORIGIN), E5_2);
    }
}

#[test]
fn s8_f5b_an_optional_member_and_optional_reads_propose() {
    // Corrected after the Gate 1 freeze (S8-F-01): the frozen text read
    // `j.result!.exit_code`, which the field reader does not count as a
    // read at all (`!.`), so no R6 line exists for the route to convert.
    let jobs = "export interface Job {\n  result?: Result;\n}\n";
    let x = "import { Job } from 'a/run';\nexport const f = (j: Job) => [j.result?.stdout_text, j.result?.exit_code];\n";
    let o = case("f5b", &run_with(jobs), x, &[]);
    proposed_by_property(&o, &prop_e4("Job.result", 6, JOB_ORIGIN), E5_2);
}

#[test]
fn s8_f11_two_hops_propose() {
    let decls = format!("{JOB}export interface Wrap {{\n  job: Job;\n}}\n");
    let x = "import { Wrap } from 'a/run';\nexport const f = (w: Wrap) => [w.job.result.stdout_text, w.job.result.exit_code];\n";
    let o = case("f11", &run_with(&decls), x, &[]);
    proposed_by_property(&o, &prop_e4("Wrap.job.result", 6, "Wrap imported at b/src/x.ts:1 from \"a/run\""), E5_2);
}

#[test]
fn s8_f11c_two_hops_through_an_inline_object_type_propose() {
    let decls = "export interface Box2 {\n  inner: {\n    result: Result;\n  };\n}\n";
    let x = "import { Box2 } from 'a/run';\nexport const f = (b: Box2) => [b.inner.result.stdout_text, b.inner.result.exit_code];\n";
    let o = case("f11c", &run_with(decls), x, &[]);
    proposed_by_property(&o, &prop_e4("Box2.inner.result", 7, "Box2 imported at b/src/x.ts:1 from \"a/run\""), E5_2);
}

#[test]
fn s8_f7_the_same_name_from_another_package_is_not_the_shape() {
    let root = ("package.json", r#"{"private":true,"workspaces":["a","b","c"]}"#);
    let pkg_c = ("c/package.json", r#"{"name":"c"}"#);
    let run = format!("{RESULT}export const other = 1;\n");
    let c_job = "import type { Result } from './res';\nexport interface Job {\n  result: Result;\n}\n";
    let x = "import { Job } from 'c/job';\nimport { other } from 'a/run';\nexport const f = (j: Job) => [j.result.stdout_text, j.result.exit_code, other];\n";
    let o = init(&ws("f7", &[root, PKG_A, PKG_B, pkg_c, ("a/run.ts", &run), ("c/res.ts", RESULT), ("c/job.ts", c_job), ("b/src/x.ts", x)]));
    // c's Result is the one Job.result names: it proposes; a's Result stays R6.
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert_eq!(o.e2, vec!["Result declared at c/res.ts:1".to_string()], "{}", o.raw);
    assert_eq!(
        o.e4,
        vec!["Result not imported by name; resolved via property Job.result declared in c/job.ts:3, Job imported at b/src/x.ts:1 from \"c/job\"".to_string()],
        "{}",
        o.raw
    );
    assert!(
        o.abstain.contains(&"ABSTAIN R6 b/src/x.ts:2 reads exit_code, stdout_text of Result (a/run.ts:1) without importing it by name (out of v1)".to_string()),
        "{}",
        o.raw
    );
}

// -------------------------------------------------- refusals: R6 stays (B2)

#[test]
fn s8_f5_unpermitted_member_types_tie_nothing() {
    for (i, ty) in ["Result | undefined", "Result | null", "Partial<Result>", "Box<Result>", "Promise<Result>"].iter().enumerate() {
        let decls = format!("export interface Box<T> {{\n  inner: T;\n}}\nexport interface Job {{\n  result: {ty};\n}}\n");
        stays_r6(&case(&format!("f5{i}"), &run_with(&decls), X_JOB, &[]));
    }
}

#[test]
fn s8_f6_an_ambiguous_type_ties_nothing() {
    // Job reaches two declarations through a barrel: the resolver abstains,
    // and nothing is tied through it.
    let index = "export * from './one';\nexport * from './two';\nexport * from './run';\n";
    let one = "import type { Result } from './run';\nexport interface Job {\n  result: Result;\n}\n";
    let x = "import { Job } from 'a/index';\nexport const f = (j: Job) => [j.result.stdout_text, j.result.exit_code];\n";
    let o = case("f6", RESULT, x, &[("a/index.ts", index), ("a/one.ts", one), ("a/two.ts", one)]);
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert!(o.abstain.iter().any(|a| a.starts_with("ABSTAIN R6 ") && a.contains(" of Result (a/run.ts:1) ")), "{}", o.raw);
}

#[test]
fn s8_f6b_an_unresolved_type_ties_nothing() {
    let x = "import { Job } from 'a/nowhere';\nimport { other } from 'a/run';\nexport const f = (j: Job) => [j.result.stdout_text, j.result.exit_code, other];\n";
    let run = format!("{RESULT}export const other = 1;\n");
    let o = case("f6b", &run, x, &[]);
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert!(o.abstain.iter().any(|a| a.starts_with("ABSTAIN R6 ") && a.contains(" of Result (a/run.ts:1) ")), "{}", o.raw);
}

#[test]
fn s8_f8_one_read_through_the_property_is_not_enough() {
    let x = "import { Job } from 'a/run';\nexport const f = (j: Job, o: any) => [j.result.stdout_text, o.exit_code];\n";
    stays_r6(&case("f8", &run_with(JOB), x, &[]));
}

#[test]
fn s8_f9_a_write_does_not_count() {
    let x = "import { Job } from 'a/run';\nexport const f = (j: Job) => {\n  j.result.stdout_text = 'x';\n  return j.result.exit_code;\n};\n";
    stays_r6(&case("f9", &run_with(JOB), x, &[]));
}

#[test]
fn s8_f10_an_untyped_receiver_ties_nothing() {
    for (i, x) in [
        "import { Job } from 'a/run';\nexport const f = (j: any) => [j.result.stdout_text, j.result.exit_code];\n",
        "import { Job } from 'a/run';\nexport const f = () => {\n  const j = make();\n  return [j.result.stdout_text, j.result.exit_code];\n};\n",
    ]
    .iter()
    .enumerate()
    {
        stays_r6(&case(&format!("f10{i}"), &run_with(JOB), x, &[]));
    }
}

#[test]
fn s8_f11b_three_hops_tie_nothing() {
    let decls = format!("{JOB}export interface Wrap {{\n  job: Job;\n}}\nexport interface Outer {{\n  wrap: Wrap;\n}}\n");
    let x = "import { Outer } from 'a/run';\nexport const f = (o: Outer) => [o.wrap.job.result.stdout_text, o.wrap.job.result.exit_code];\n";
    stays_r6(&case("f11b", &run_with(&decls), x, &[]));
}

#[test]
fn s8_f16_a_member_of_the_property_value_is_not_the_property() {
    // `j.result.inner` is not `j.result`: an alias of a longer path ties nothing.
    let x = "import { Job } from 'a/run';\nexport const f = (j: Job) => {\n  const r = j.result.stdout_text;\n  const s = j.result.exit_code;\n  return [r.stdout_text, s.exit_code];\n};\n";
    let o = case("f16", &run_with(JOB), x, &[]);
    // the two direct reads `j.result.stdout_text` and `j.result.exit_code` count;
    // the aliases r and s are not tied to Result
    assert_eq!(o.e5, vec!["Result fields read: exit_code (b/src/x.ts:4), stdout_text (b/src/x.ts:3)".to_string()], "{}", o.raw);
}

// ---------------------------------------- uniform scope and status (S8-D2)

#[test]
fn s8_f12_a_mixed_block_carries_the_combined_line() {
    let decls = format!("{JOB}export interface Other {{\n  mode_flag: string;\n  kind_code: number;\n}}\n");
    let x = "import { Job, Other } from 'a/run';\nexport const f = (j: Job, o: Other) => [j.result.stdout_text, j.result.exit_code, o.mode_flag, o.kind_code];\n";
    let o = case("f12", &run_with(&decls), x, &[]);
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert_eq!(o.status, vec![PROP_MIXED.to_string()], "{}", o.raw);
    assert_eq!(
        o.e4,
        vec![
            "Other imported at b/src/x.ts:1 from \"a/run\", resolved to a/run.ts".to_string(),
            prop_e4("Job.result", 6, "Job imported at b/src/x.ts:1 from \"a/run\""),
        ],
        "{}",
        o.raw
    );
}

#[test]
fn s8_f13_the_route_never_adds_reads_to_a_shape_that_already_qualifies() {
    // Golden body recorded at Gate 1 on the SLICE7 binary (681fc1463275).
    let decls = "export interface Job {\n  result: Result;\n}\n";
    let run = "export interface Result {\n  stdout_text: string;\n  exit_code: number;\n  extra_field: string;\n}\n".to_string() + decls;
    let x = "import { Job, Result } from 'a/run';\nexport const f = (o: Result, j: Job) => [o.stdout_text, o.exit_code, j.result.extra_field];\n";
    let o = case("f13", &run, x, &[]);
    assert_eq!(o.status, vec![DECLARED_STATUS.to_string()], "{}", o.raw);
    assert_eq!(body(&o.raw), F13_BODY);
}

#[test]
fn s8_h1_the_header_names_the_property_route() {
    let o = case("h1", &run_with(JOB), X_JOB, &[]);
    assert!(o.raw.contains(HEADER_S8), "{}", o.raw);
}

// ------------------------------------------------------------ no regression

#[test]
fn s8_n1_a_slice7_proposal_is_byte_identical() {
    // The SLICE7 S7-F1 workspace; golden body recorded at Gate 1 on 681fc1463275.
    let run = "export function exec(c: string): Result {\n  return null as any;\n}\nexport interface Result {\n  stdout_text: string;\n  exit_code: number;\n}\n";
    let x = "import { exec } from 'a/run';\nexport const f = () => {\n  const r = exec('x');\n  return [r.stdout_text, r.exit_code];\n};\n";
    let o = case("n1", run, x, &[]);
    assert_eq!(body(&o.raw), N1_BODY);
}

const F13_BODY: &str = r#"[discovery]
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
# SUMMARY packages=2 candidates=2 proposals=1 abstentions=0 inventory=0 status=COMPLETE
"#;
const N1_BODY: &str = r#"[discovery]
files_scanned = "5"
truncated = "false"
packages = "2"
candidates = "1"
contracts_proposed = "1"
abstentions = "0"
inventory = "0"

# PROPOSED code:run.ts->x.ts
# status: PROPOSED (heuristic; receiver assigned from imported function return type; field writes excluded; types are not inferred)
# E1 producer package a at a/package.json (package.json "workspaces" entry "a")
# E2 Result declared at a/run.ts:4
# E3 consumer package b at b/package.json (package.json "workspaces" entry "b")
# E4 Result not imported by name; resolved via return type of exec() declared in a/run.ts:1, exec imported at b/src/x.ts:1 from "a/run"
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
# SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE
"#;

// ------------------------- added at Gate 3, after the Gate 1 freeze (K)
// The bytes above are the frozen file, apart from the S8-F-01 correction
// of F5b. These close the gaps Gate 3 and the stop-condition check found.

#[test]
fn s8_f5c_optional_access_between_hops_proposes() {
    let decls = format!("{JOB}export interface Wrap {{\n  job?: Job;\n}}\n");
    let x = "import { Wrap } from 'a/run';\nexport const f = (w?: Wrap) => [w?.job?.result.stdout_text, w?.job?.result.exit_code];\n";
    let o = case("f5c", &run_with(&decls), x, &[]);
    proposed_by_property(&o, &prop_e4("Wrap.job.result", 6, "Wrap imported at b/src/x.ts:1 from \"a/run\""), E5_2);
}

#[test]
fn s8_f16b_an_alias_of_a_longer_expression_ties_nothing() {
    // `r = j.result.stdout_text` is not the property value: r.exit_code is
    // not a read of Result, so only one field is verified and R6 stays.
    let x = "import { Job } from 'a/run';\nexport const f = (j: Job, x: any) => {\n  const r = j.result.stdout_text;\n  return [r.exit_code, x.stdout_text];\n};\n";
    stays_r6(&case("f16b", &run_with(JOB), x, &[]));
}

#[test]
fn s8_f17_a_property_only_pair_numbers_after_existing_pairs() {
    // Two pairs share the file names run.ts -> x.ts: one through a declared
    // receiver (a2/run.ts, as before SLICE8), one only through a property
    // path (a/run.ts, which sorts first). The existing pair keeps the bare id.
    let root = ("package.json", r#"{"private":true,"workspaces":["a","a2","b"]}"#);
    let pkg_a2 = ("a2/package.json", r#"{"name":"a2"}"#);
    let other = "export interface Other {\n  mode_flag: string;\n  kind_code: number;\n}\n";
    let x = "import { Job } from 'a/run';\nimport { Other } from 'a2/run';\nexport const f = (j: Job, o: Other) => [j.result.stdout_text, j.result.exit_code, o.mode_flag, o.kind_code];\n";
    let o = init(&ws("f17", &[root, PKG_A, pkg_a2, PKG_B, ("a/run.ts", &run_with(JOB)), ("a2/run.ts", other), ("b/src/x.ts", x)]));
    assert_eq!(o.proposals, 2, "{}", o.raw);
    let ids: Vec<&str> = o.raw.lines().filter(|l| l.starts_with("# PROPOSED ")).collect();
    assert_eq!(ids, vec!["# PROPOSED code:run.ts->x.ts#2", "# PROPOSED code:run.ts->x.ts"], "{}", o.raw);
    assert!(o.raw.contains("id = \"code:run.ts->x.ts\"\nproducer = \"a2/run.ts\"\n"), "{}", o.raw);
}
