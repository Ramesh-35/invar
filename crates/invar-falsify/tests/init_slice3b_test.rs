//! INIT-DISCOVERY-SLICE3B fixtures (rulings S3B-D1 to S3B-D7): earned
//! silence. An import that resolves but finds no shape is silent only when
//! the target package's own sources declare the name as a value, class or
//! enum and nowhere as an interface or type alias; otherwise init abstains
//! (UNRESOLVED, S3B-D2) with a reason that names where to look. Each fixture
//! is a two-package workspace (package `a` produces, `b` consumes and reads
//! two fields), run through `invar init --stdout` twice, byte-identical.

use std::path::{Path, PathBuf};
use std::process::Command;

const ROOT_AB: (&str, &str) = ("package.json", r#"{"private":true,"workspaces":["a","b"]}"#);
const PKG_A: (&str, &str) = ("a/package.json", r#"{"name":"a"}"#);
const PKG_B: (&str, &str) = ("b/package.json", r#"{"name":"b"}"#);
const ORDER: &str = "export interface Order {\n  total_minor: number;\n  currency_code: string;\n}\n";
const SILENT: &str = "SUMMARY packages=2 candidates=1 proposals=0 abstentions=0 inventory=0 status=COMPLETE";
const LOUD: &str = "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE";
const ONE: &str = "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE";

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s3b_{tag}_{}_{nanos}", std::process::id()));
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    dir
}

struct Out {
    proposals: usize,
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
    assert_eq!(a, b, "D01: init output is not deterministic");
    Out {
        proposals: a.lines().filter(|l| *l == "[[invariant]]").count(),
        abstain: a.lines().filter_map(|l| l.strip_prefix("# ABSTAIN ")).map(|l| format!("ABSTAIN {l}")).collect(),
        summary: a.lines().last().unwrap_or("").trim_start_matches("# ").to_string(),
        raw: a,
    }
}

/// Package `a` with `files`, and `b/src/use.ts` importing `clause` from 'a'
/// and reading two fields.
fn case(tag: &str, files: &[(&str, &str)], clause: &str) -> Out {
    let use_ts = format!("import {clause} from 'a';\nexport function f(o: any) {{\n  return [o.total_minor, o.currency_code];\n}}\n");
    let mut all: Vec<(&str, &str)> = vec![ROOT_AB, PKG_A, PKG_B, ("b/src/use.ts", &use_ts)];
    all.extend_from_slice(files);
    init(&ws(tag, &all))
}

/// INIT-DISCOVERY-SLICE5 (ruling S5-D3): `case` with the receiver declared
/// as Order, for the four fixtures S4-D7 turned into proposals. SLICE5 counts
/// no read through `o: any`; the abstaining and silent fixtures keep `case`.
fn case_typed(tag: &str, files: &[(&str, &str)], clause: &str) -> Out {
    let use_ts = format!("import {clause} from 'a';\nexport function f(o: Order) {{\n  return [o.total_minor, o.currency_code];\n}}\n");
    let mut all: Vec<(&str, &str)> = vec![ROOT_AB, PKG_A, PKG_B, ("b/src/use.ts", &use_ts)];
    all.extend_from_slice(files);
    init(&ws(tag, &all))
}

fn silent(o: &Out) {
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert!(o.abstain.is_empty(), "earned silence must stay silent\n{}", o.raw);
    assert_eq!(o.summary, SILENT, "{}", o.raw);
}

fn loud(o: &Out, name: &str, reason: &str) {
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert_eq!(o.abstain, vec![format!("ABSTAIN UNRESOLVED b/src/use.ts:1 {name} from \"a\": {reason}")], "unearned silence must abstain\n{}", o.raw);
    assert_eq!(o.summary, LOUD, "{}", o.raw);
}

/// SLICE4 now follows this workspace's chain to the declaration.
fn proposed(o: &Out) {
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert!(o.abstain.is_empty(), "{}", o.raw);
    assert_eq!(o.summary, ONE, "{}", o.raw);
}

fn unreached(site: &str) -> String {
    format!("declared as a type at {site}, which init could not reach from a/index.ts; not guessed")
}

const NOT_DECLARED: &str = "not declared in the sources of package a; not guessed";

// --------------------------------------------------------- unearned: loud

#[test]
fn b_f1_a_type_three_barrels_deep_resolved_by_slice4() {
    // Superseded by INIT-DISCOVERY-SLICE4 (ruling S4-D7 under the standing
    // rule of S3-D1). Original name and assertion, kept verbatim:
    //   fn b_f1_a_type_three_barrels_deep_abstains_with_its_site()
    //   loud(&case("f1", &files, "{ Order }"), "Order", &unreached("a/l3.ts:1"));
    // Chains are now followed five levels, so this workspace proposes. The
    // unreached-site reason stays pinned on a route init still does not
    // follow: a namespace re-export at the third level.
    let files = [("a/index.ts", "export * from \"./l1\";\n"), ("a/l1.ts", "export * from \"./l2\";\n"), ("a/l2.ts", "export * from \"./l3\";\n"), ("a/l3.ts", ORDER)];
    // S5-D3: typed receiver. Line before SLICE5, kept verbatim:
    //     proposed(&case("f1", &files, "{ Order }"));
    proposed(&case_typed("f1", &files, "{ Order }"));
    let ns = [("a/index.ts", "export * from \"./l1\";\n"), ("a/l1.ts", "export * from \"./l2\";\n"), ("a/l2.ts", "export * as ns from \"./l3\";\n"), ("a/l3.ts", ORDER)];
    loud(&case("f1ns", &ns, "{ Order }"), "Order", &unreached("a/l3.ts:1"));
}

#[test]
fn b_f2_a_regex_literal_cannot_hide_a_type() {
    // The S3-F-11 shape, verbatim from excalidraw listMarker.ts:22: an
    // unbalanced `)` inside a character class shifts the scanner's bracket
    // depth, so the declaration after it is invisible to the shape reader.
    let lm = "const LIST_MARKER_REGEX =\n  /^(\\s*)(?:\\((\\d{1,9}|[a-zA-Z]+)\\)|\\[(\\d{1,9})\\]|(\\d{1,9}|[a-zA-Z]+)([.)]))(?=\\s|$)/;\n\nexport type Order = {\n  total_minor: number;\n  currency_code: string;\n};\n";
    let files = [("a/index.ts", "export * from \"./lm\";\n"), ("a/lm.ts", lm)];
    loud(&case("f2", &files, "{ Order }"), "Order", &unreached("a/lm.ts:4"));
}

#[test]
fn b_f3_an_export_type_star_chain_resolved_by_slice4() {
    // Superseded by INIT-DISCOVERY-SLICE4, which parses `export type *`
    // (S3-F-10; ruling S4-D7 under the standing rule of S3-D1). Original
    // name, comment and assertion, kept verbatim:
    //   fn b_f3_an_export_type_star_chain_abstains()
    //   // S3-F-10 is deferred (S3B-D5): the chain stays unparsed, and stated.
    //   loud(&case("f3", &files, "{ Order }"), "Order", &unreached("a/o.ts:1"));
    let files = [("a/index.ts", "export type * from \"./o\";\n"), ("a/o.ts", ORDER)];
    // S5-D3: typed receiver. Line before SLICE5, kept verbatim:
    //     proposed(&case("f3", &files, "{ Order }"));
    proposed(&case_typed("f3", &files, "{ Order }"));
}

#[test]
fn b_f6_a_name_declared_outside_the_package_abstains() {
    let files = [("a/index.ts", "export { Order } from \"some-external-lib\";\n")];
    loud(&case("f6", &files, "{ Order }"), "Order", NOT_DECLARED);
}

#[test]
fn b_f7_a_name_declared_both_ways_resolved_by_slice4() {
    // Superseded by INIT-DISCOVERY-SLICE4 (ruling S4-D7 under the standing
    // rule of S3-D1). Original name and assertion, kept verbatim:
    //   fn b_f7_a_name_declared_both_ways_abstains()
    //   loud(&case("f7", &files, "{ Order }"), "Order", "declared both as a type and as a value in package a; not guessed");
    // The chain now reaches the type alias, which proposes exactly as it
    // does one level down. The both-ways reason stays pinned on a route
    // init still does not follow: a namespace re-export at the third level.
    let l3 = "export const Order = { kind: 1 };\nexport type Order = {\n  total_minor: number;\n  currency_code: string;\n};\n";
    let files = [("a/index.ts", "export * from \"./l1\";\n"), ("a/l1.ts", "export * from \"./l2\";\n"), ("a/l2.ts", "export * from \"./l3\";\n"), ("a/l3.ts", l3)];
    // S5-D3: typed receiver. Line before SLICE5, kept verbatim:
    //     proposed(&case("f7", &files, "{ Order }"));
    proposed(&case_typed("f7", &files, "{ Order }"));
    let ns = [("a/index.ts", "export * from \"./l1\";\n"), ("a/l1.ts", "export * from \"./l2\";\n"), ("a/l2.ts", "export * as ns from \"./l3\";\n"), ("a/l3.ts", l3)];
    loud(&case("f7ns", &ns, "{ Order }"), "Order", "declared both as a type and as a value in package a; not guessed");
}

#[test]
fn b_f9_a_declaration_in_a_comment_or_string_is_not_evidence() {
    let idx = "// interface Order { total_minor: number }\nexport const note = \"type Order = 1\";\nexport function f() {\n  return 1;\n}\n";
    loud(&case("f9", &[("a/index.ts", idx)], "{ Order }"), "Order", NOT_DECLARED);
}

// ----------------------------------------------------------- earned: silent

#[test]
fn b_f4_a_function_import_stays_silent() {
    let idx = "export function makeOrder() {\n  return 1;\n}\n";
    silent(&case("f4", &[("a/index.ts", idx)], "{ makeOrder }"));
}

#[test]
fn b_f5_a_class_import_stays_silent() {
    let idx = "export class Order {\n  total_minor = 0;\n  currency_code = \"\";\n}\n";
    silent(&case("f5", &[("a/index.ts", idx)], "{ Order }"));
}

#[test]
fn b_f8_a_type_only_import_of_a_value_stays_silent() {
    let idx = "export function makeOrder() {\n  return 1;\n}\n";
    silent(&case("f8", &[("a/index.ts", idx)], "type { makeOrder }"));
}

#[test]
fn b_f10_an_unread_package_file_voids_earned_silence() {
    // Found at the STAGE3-EVAL-003 rehearsal (playwright-core): a `class`
    // in readable code and an `interface` of the same name in a file over
    // init's 1 MB scan limit. Silence needs proof, and a package with an
    // unread file cannot supply it (S3B-D1 applied to incomplete evidence).
    let mut big = String::from("export interface Order {\n  total_minor: number;\n  currency_code: string;\n}\n");
    while big.len() <= 1_000_000 {
        big.push_str("// padding to exceed the scan limit\n");
    }
    let idx = "export class Order {\n  total_minor = 0;\n  currency_code = \"\";\n}\n";
    let o = case("f10", &[("a/index.ts", idx), ("a/big.d.ts", &big)], "{ Order }");
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert_eq!(
        o.abstain,
        vec![
            "ABSTAIN UNREADABLE a/big.d.ts larger than the 1000000-byte scan limit; not scanned".to_string(),
            "ABSTAIN UNRESOLVED b/src/use.ts:1 Order from \"a\": package a has a source file init could not read (a/big.d.ts); silence is not earned; not guessed".to_string(),
        ],
        "{}",
        o.raw
    );
}

// ---------------------------------------------------------------- control

#[test]
fn b_m1_two_runs_are_byte_identical() {
    // init() runs twice and asserts byte identity; pinned on the deepest path.
    let files = [("a/index.ts", "export * from \"./l1\";\n"), ("a/l1.ts", "export * from \"./l2\";\n"), ("a/l2.ts", "export * from \"./l3\";\n"), ("a/l3.ts", ORDER)];
    // S5-D3: typed receiver. Line before SLICE5, kept verbatim:
    //     let o = case("m1", &files, "{ Order }");
    let o = case_typed("m1", &files, "{ Order }");
    // Changed by INIT-DISCOVERY-SLICE4 (ruling S4-D7): this chain now
    // proposes. Original assertion, kept verbatim:
    //   assert_eq!(o.summary, LOUD, "{}", o.raw);
    assert_eq!(o.summary, ONE, "{}", o.raw);
}
