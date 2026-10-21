//! INIT-DISCOVERY-SLICE4 fixtures (rulings S4-D1 to S4-D6): bounded barrel
//! chains. Code-pair discovery follows re-exports up to five levels from an
//! import's resolved entry, each (file, name) once, and resolves only when
//! exactly one distinct file declares the shape. `export type * from` is a
//! star re-export; namespace re-exports are not followed. Each fixture is a
//! two-package workspace (package `a` produces, `b` consumes and reads two
//! fields), run through `invar init --stdout` twice, byte-identical.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const ROOT_AB: (&str, &str) = ("package.json", r#"{"private":true,"workspaces":["a","b"]}"#);
const PKG_A: (&str, &str) = ("a/package.json", r#"{"name":"a"}"#);
const PKG_B: (&str, &str) = ("b/package.json", r#"{"name":"b"}"#);
const ORDER: &str = "export interface Order {\n  total_minor: number;\n  currency_code: string;\n}\n";
const ONE: &str = "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE";
const LOUD: &str = "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE";

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s4_{tag}_{}_{nanos}", std::process::id()));
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    dir
}

/// One `invar init --stdout` run, killed and failed after `limit` (a cycle
/// that is not guarded must fail by assertion, not hang the suite).
fn run(dir: &Path, limit: Duration) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["init", "--stdout", "--root"])
        .arg(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if start.elapsed() > limit {
            let _ = child.kill();
            let _ = child.wait();
            panic!("init did not finish within {limit:?}: a re-export cycle is not guarded");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let o = child.wait_with_output().unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout).unwrap()
}

struct Out {
    proposals: usize,
    e4: Vec<String>,
    abstain: Vec<String>,
    summary: String,
    raw: String,
}

fn init(dir: &Path) -> Out {
    let limit = Duration::from_secs(20);
    let (a, b) = (run(dir, limit), run(dir, limit));
    assert_eq!(a, b, "D01: init output is not deterministic");
    Out {
        proposals: a.lines().filter(|l| *l == "[[invariant]]").count(),
        e4: a.lines().filter_map(|l| l.strip_prefix("# E4 ")).map(String::from).collect(),
        abstain: a.lines().filter_map(|l| l.strip_prefix("# ABSTAIN ")).map(|l| format!("ABSTAIN {l}")).collect(),
        summary: a.lines().last().unwrap_or("").trim_start_matches("# ").to_string(),
        raw: a,
    }
}

/// Package `a` with `files`, and `b/src/use.ts` importing `clause` from 'a'
/// and reading two fields on a receiver declared as the imported name.
///
/// Changed by INIT-DISCOVERY-SLICE5 (ruling S5-D3): the receiver was `o: any`,
/// which SLICE5 no longer counts and which says nothing about chains. Original
/// line, kept verbatim:
///     let use_ts = format!("import {clause} from 'a';\nexport function f(o: any) {{\n  return [o.total_minor, o.currency_code];\n}}\n");
fn case(tag: &str, files: &[(&str, &str)], clause: &str) -> Out {
    let ty = clause.trim_matches(|c| c == '{' || c == '}' || c == ' ');
    let use_ts = format!("import {clause} from 'a';\nexport function f(o: {ty}) {{\n  return [o.total_minor, o.currency_code];\n}}\n");
    let mut all: Vec<(&str, &str)> = vec![ROOT_AB, PKG_A, PKG_B, ("b/src/use.ts", &use_ts)];
    all.extend_from_slice(files);
    init(&ws(tag, &all))
}

fn proposed(o: &Out, e4: &str) {
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert!(o.abstain.is_empty(), "{}", o.raw);
    assert_eq!(o.e4, vec![e4.to_string()], "{}", o.raw);
    assert_eq!(o.summary, ONE, "{}", o.raw);
}

fn abstained(o: &Out, line: &str) {
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert_eq!(o.abstain, vec![line.to_string()], "{}", o.raw);
    assert_eq!(o.summary, LOUD, "{}", o.raw);
}

const E4: &str = "Order imported at b/src/use.ts:1 from \"a\", resolved to a/index.ts";

/// `a/index.ts` re-exports `a/l1.ts`, which re-exports `a/l2.ts`, ...;
/// the last level declares Order. `levels` = number of barrel hops.
fn star_chain(levels: usize) -> Vec<(String, String)> {
    let name = |i: usize| if i == 0 { "a/index.ts".to_string() } else { format!("a/l{i}.ts") };
    let mut v = Vec::new();
    for i in 0..levels {
        let next = if i + 1 == levels { "./o".to_string() } else { format!("./l{}", i + 1) };
        v.push((name(i), format!("export * from \"{next}\";\n")));
    }
    v.push(("a/o.ts".to_string(), ORDER.to_string()));
    v
}

fn borrow(v: &[(String, String)]) -> Vec<(&str, &str)> {
    v.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
}

// ------------------------------------------------------------ depth (R-S4-1)

#[test]
fn s4_f1_depth_two_proposes() {
    // The slice 1 u02 workspace, which today abstains BARREL2 (S4-D1).
    let x = "import { Order } from 'a';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n";
    let o = init(&ws(
        "f1",
        &[
            ROOT_AB,
            PKG_A,
            PKG_B,
            ("a/index.ts", "export * from './models';\n"),
            ("a/models/index.ts", "export * from './order';\n"),
            ("a/models/order.ts", ORDER),
            ("b/src/x.ts", x),
        ],
    ));
    proposed(
        &o,
        "Order imported at b/src/x.ts:1 from \"a\", resolved to a/index.ts, re-exported through a/index.ts:1 -> a/models/index.ts:1 from a/models/order.ts",
    );
}

#[test]
fn s4_f2_depth_five_proposes() {
    let o = case("f2", &borrow(&star_chain(5)), "{ Order }");
    proposed(
        &o,
        &format!("{E4}, re-exported through a/index.ts:1 -> a/l1.ts:1 -> a/l2.ts:1 -> a/l3.ts:1 -> a/l4.ts:1 from a/o.ts"),
    );
}

#[test]
fn s4_f3_depth_six_abstains_barrel2() {
    let o = case("f3", &borrow(&star_chain(6)), "{ Order }");
    abstained(
        &o,
        "ABSTAIN BARREL2 b/src/use.ts:1 Order from \"a\" is re-exported through more than 5 barrel levels (from a/index.ts)",
    );
}

#[test]
fn s4_f4_a_cycle_ends_and_earned_silence_decides() {
    let files = [
        ("a/index.ts", "export * from \"./x\";\n"),
        ("a/x.ts", "export * from \"./y\";\nexport * from \"./index\";\n"),
        ("a/y.ts", "export * from \"./x\";\n"),
    ];
    let o = case("f4", &files, "{ Order }");
    abstained(&o, "ABSTAIN UNRESOLVED b/src/use.ts:1 Order from \"a\": not declared in the sources of package a; not guessed");
}

// ------------------------------------------------------ one declaration (R-S4-2)

#[test]
fn s4_f5_a_diamond_to_one_declaration_proposes() {
    let files = [
        ("a/index.ts", "export * from \"./p\";\nexport * from \"./q\";\n"),
        ("a/p.ts", "export * from \"./o\";\n"),
        ("a/q.ts", "export * from \"./o\";\n"),
        ("a/o.ts", ORDER),
    ];
    let o = case("f5", &files, "{ Order }");
    proposed(&o, &format!("{E4}, re-exported through a/index.ts:1 -> a/p.ts:1 from a/o.ts"));
}

#[test]
fn s4_f6_two_declarations_abstain_ambig_shape() {
    let files = [
        ("a/index.ts", "export * from \"./q\";\nexport * from \"./p\";\n"),
        ("a/p.ts", "export * from \"./o2\";\n"),
        ("a/q.ts", "export * from \"./o1\";\n"),
        ("a/o1.ts", ORDER),
        ("a/o2.ts", ORDER),
    ];
    let o = case("f6", &files, "{ Order }");
    abstained(&o, "ABSTAIN AMBIG-SHAPE b/src/use.ts:1 Order from \"a\" resolves to more than one declaration: a/o1.ts, a/o2.ts");
}

#[test]
fn s4_f7_a_renamed_reexport_follows_the_original_name() {
    // `a/o.ts` is reached first under the imported name (where it declares
    // nothing), then under the original name through `a/mid.ts`.
    let files = [
        ("a/index.ts", "export * from \"./o\";\nexport * from \"./mid\";\n"),
        ("a/mid.ts", "export { Order as Ord } from \"./o\";\n"),
        ("a/o.ts", ORDER),
    ];
    let o = case("f7", &files, "{ Ord }");
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert!(o.raw.contains("producer = \"a/o.ts\"\n"), "{}", o.raw);
    proposed(&o, "Ord imported at b/src/use.ts:1 from \"a\", resolved to a/index.ts, re-exported through a/index.ts:2 -> a/mid.ts:1 from a/o.ts");
}

// ------------------------------------------------------------ syntax (R-S4-3)

#[test]
fn s4_f8_an_export_type_star_chain_proposes() {
    let files = [("a/index.ts", "export type * from \"./m\";\n"), ("a/m.ts", "export type * from \"./o\";\n"), ("a/o.ts", ORDER)];
    let o = case("f8", &files, "{ Order }");
    proposed(&o, &format!("{E4}, re-exported through a/index.ts:1 -> a/m.ts:1 from a/o.ts"));
}

#[test]
fn s4_f9_a_namespace_reexport_is_not_followed() {
    let files = [("a/index.ts", "export * as ns from \"./o\";\nexport type * as tns from \"./o\";\n"), ("a/o.ts", ORDER)];
    let o = case("f9", &files, "{ Order }");
    abstained(
        &o,
        "ABSTAIN UNRESOLVED b/src/use.ts:1 Order from \"a\": declared as a type at a/o.ts:1, which init could not reach from a/index.ts; not guessed",
    );
}

// ------------------------------------------------------------ evidence (R-S4-4)

#[test]
fn s4_f10_e4_names_the_whole_chain() {
    let files = [
        ("a/index.ts", "export * from \"./l1\";\n"),
        ("a/l1.ts", "// models\n\nexport { Order } from \"./l2\";\n"),
        ("a/l2.ts", "export * from \"./o\";\n"),
        ("a/o.ts", ORDER),
    ];
    let o = case("f10", &files, "{ Order }");
    proposed(&o, &format!("{E4}, re-exported through a/index.ts:1 -> a/l1.ts:3 -> a/l2.ts:1 from a/o.ts"));
}

// ------------------------------------------------------------- no regression

#[test]
fn s4_n1_a_single_level_proposal_is_byte_identical() {
    // Golden recorded at Gate 1 on the aa03983 release binary.
    let o = case("n1", &[("a/index.ts", "export * from \"./o\";\n"), ("a/o.ts", ORDER)], "{ Order }");
    assert_eq!(o.raw, N1_GOLDEN);
}

// ------------------------------------------------------- determinism (R-S4-5)

#[test]
fn s4_m1_reordered_reexports_give_the_same_answer() {
    // init() already asserts two runs byte-identical. Reordering a barrel's
    // lines moves only that barrel's own line numbers.
    let diamond = |first: &str, second: &str| {
        let idx = format!("export * from \"./{first}\";\nexport * from \"./{second}\";\n");
        let files = [("a/index.ts", idx.as_str()), ("a/p.ts", "export * from \"./o\";\n"), ("a/q.ts", "export * from \"./o\";\n"), ("a/o.ts", ORDER)];
        case("m1", &files, "{ Order }")
    };
    let (pq, qp) = (diamond("p", "q"), diamond("q", "p"));
    assert_eq!(pq.e4, vec![format!("{E4}, re-exported through a/index.ts:1 -> a/p.ts:1 from a/o.ts")], "{}", pq.raw);
    assert_eq!(qp.e4, vec![format!("{E4}, re-exported through a/index.ts:2 -> a/p.ts:1 from a/o.ts")], "{}", qp.raw);
    assert_eq!(pq.raw.replace("a/index.ts:1 ->", "a/index.ts:N ->"), qp.raw.replace("a/index.ts:2 ->", "a/index.ts:N ->"));
    let ambig = |first: &str, second: &str| {
        let idx = format!("export * from \"./{first}\";\nexport * from \"./{second}\";\n");
        let files = [("a/index.ts", idx.as_str()), ("a/p.ts", "export * from \"./o2\";\n"), ("a/q.ts", "export * from \"./o1\";\n"), ("a/o1.ts", ORDER), ("a/o2.ts", ORDER)];
        case("m1b", &files, "{ Order }")
    };
    assert_eq!(ambig("p", "q").raw, ambig("q", "p").raw);
}

// S5-D4 (INIT-DISCOVERY-SLICE5) changed two lines of this golden, the header
// sentence and the status line, and nothing else. Originals, kept verbatim:
//     # direct fields read there. The receiver of those reads is NOT verified.
//     # status: PROPOSED (heuristic; receiver of field reads not verified)
// S7-D6 (INIT-DISCOVERY-SLICE7) changed the header sentence of this golden
// and nothing else. Original lines 3 to 6 of the header, kept verbatim:
//     # [[invariant]] block below is a PROPOSAL: a TypeScript shape exported by one
//     # workspace package, imported by name in another, with at least 2 of its
//     # direct fields read there, each on a receiver declared as that shape in
//     # the consumer file (field writes excluded; types are not inferred).
const N1_GOLDEN: &str = r#"# invar.toml generated by `invar init`.
#
# HONEST SCOPE: discovery is HEURISTIC (text patterns, not a parser). Each
# [[invariant]] block below is a PROPOSAL: a TypeScript shape exported by one
# workspace package, with at least 2 of its direct fields read in a file of
# another, each on a receiver declared as that shape in that file or
# assigned from a function imported from the shape's package whose declared
# return type is the shape (field writes excluded; types are not inferred).
# Review every block before trusting a verdict. Schema files are inventory
# only. ABSTAIN lines are refusals to propose, not negative findings. Paths
# are relative to this file.

[discovery]
files_scanned = "6"
truncated = "false"
packages = "2"
candidates = "1"
contracts_proposed = "1"
abstentions = "0"
inventory = "0"

# PROPOSED code:o.ts->use.ts
# status: PROPOSED (heuristic; each counted read is on a receiver declared as the shape in this file; field writes excluded; types are not inferred)
# E1 producer package a at a/package.json (package.json "workspaces" entry "a")
# E2 Order declared at a/o.ts:1
# E3 consumer package b at b/package.json (package.json "workspaces" entry "b")
# E4 Order imported at b/src/use.ts:1 from "a", resolved to a/index.ts, re-exported by barrel a/index.ts:1 from a/o.ts
# E5 Order fields read: currency_code (b/src/use.ts:3), total_minor (b/src/use.ts:3)
# E5 Order stoplisted reads: (none)
# E6 pair a/o.ts -> b/src/use.ts
# category: production
# lock: run `invar lock` before `invar check`. Without a lock, check evaluates only payload.<field> reads and says nothing about this proposal (a HOLDS would be vacuous). `invar lock` binds every field of the producer file that the consumer mentions in any form, which is wider than the evidence above.
[[invariant]]
kind = "schema"
id = "code:o.ts->use.ts"
producer = "a/o.ts"
consumer = "b/src/use.ts"
evidence = "Order currency_code total_minor"

# ---- Abstentions (init declined to propose; not negative findings) ----
# (none)
# ---- Found but NOT proposed (inventory only) ----
# (none)
# SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE
"#;
