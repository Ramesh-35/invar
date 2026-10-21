//! INIT-DISCOVERY-SLICE3 fixtures (rulings S3-D1 to S3-D8): package
//! "exports" maps resolved to the source the package itself declares.
//! Each fixture is a two-package workspace: package `a` declares the map
//! and `Order`, package `b` imports it and reads two of its fields. Every
//! run is `invar init --stdout` twice, byte-identical (D01), and asserts
//! the complete outcome: proposal pairs, every ABSTAIN line, the SUMMARY.

use std::path::{Path, PathBuf};
use std::process::Command;

const ROOT_AB: (&str, &str) = ("package.json", r#"{"private":true,"workspaces":["a","b"]}"#);
const PKG_B: (&str, &str) = ("b/package.json", r#"{"name":"b"}"#);
const ORDER: &str = "export interface Order {\n  total_minor: number;\n  currency_code: string;\n  id: string;\n}\n";
const HIT: &str = "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE";
const MISS: &str = "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE";

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s3_{tag}_{}_{nanos}", std::process::id()));
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    dir
}

struct Out {
    pairs: Vec<(String, String)>,
    abstain: Vec<String>,
    summary: String,
    raw: String,
}

/// `invar init --stdout` twice; byte-identical (D01).
fn init(dir: &Path) -> Out {
    let run = || {
        let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(["init", "--stdout", "--root"]).arg(dir).output().unwrap();
        assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8(o.stdout).unwrap()
    };
    let (a, b) = (run(), run());
    assert_eq!(a, b, "D01: init output is not deterministic");
    let mut pairs = Vec::new();
    let mut producer = None;
    for l in a.lines() {
        if let Some(v) = l.strip_prefix("producer = ") {
            producer = Some(v.trim_matches('"').to_string());
        }
        if let Some(v) = l.strip_prefix("consumer = ") {
            pairs.push((producer.take().unwrap(), v.trim_matches('"').to_string()));
        }
    }
    let abstain = a.lines().filter_map(|l| l.strip_prefix("# ABSTAIN ")).map(|l| format!("ABSTAIN {l}")).collect();
    let summary = a.lines().last().unwrap_or("").trim_start_matches("# ").to_string();
    Out { pairs, abstain, summary, raw: a }
}

/// Package `a` with `manifest`, its extra files, and a consumer in `b`
/// importing `Order` from `spec`.
fn case(tag: &str, manifest: &str, extra: &[(&str, &str)], spec: &str) -> Out {
    let a_pkg = format!(r#"{{"name":"a",{manifest}}}"#);
    let use_ts = format!("import {{ Order }} from '{spec}';\nexport function f(o: Order) {{\n  return [o.total_minor, o.currency_code];\n}}\n");
    let mut files: Vec<(&str, &str)> = vec![ROOT_AB, PKG_B, ("a/package.json", &a_pkg), ("b/src/use.ts", &use_ts)];
    files.extend_from_slice(extra);
    init(&ws(tag, &files))
}

fn proposes(o: &Out, producer: &str) {
    assert_eq!(o.pairs, vec![(producer.to_string(), "b/src/use.ts".to_string())], "proposal\n{}", o.raw);
    assert!(o.abstain.is_empty(), "abstentions\n{}", o.raw);
    assert_eq!(o.summary, HIT, "summary\n{}", o.raw);
}

fn abstains(o: &Out, spec: &str, reason: &str) {
    assert!(o.pairs.is_empty(), "no proposal may be made\n{}", o.raw);
    assert_eq!(o.abstain, vec![format!("ABSTAIN UNRESOLVED b/src/use.ts:1 Order from \"{spec}\": {reason}")], "abstentions\n{}", o.raw);
    assert_eq!(o.summary, MISS, "summary\n{}", o.raw);
}

const SRC_PAIR: (&str, &str) = ("a/tsconfig.json", r#"{"compilerOptions":{"rootDir":"./src","outDir":"./dist"}}"#);

fn not_source(t: &str) -> String {
    format!("exports target \"{t}\" is not a TypeScript source and no declared rootDir/outDir pair maps it back; not guessed")
}

// ----------------------------------------------------------------- positives

#[test]
fn s3_f1_string_exports_is_the_root_entry() {
    proposes(&case("f1", r#""exports":"./src/index.ts""#, &[("a/src/index.ts", ORDER)], "a"), "a/src/index.ts");
}

#[test]
fn s3_f2_exact_subpath_key() {
    proposes(&case("f2", r#""exports":{"./order":"./src/order.ts"}"#, &[("a/src/order.ts", ORDER)], "a/order"), "a/src/order.ts");
}

#[test]
fn s3_f3_dist_pattern_maps_back_through_root_dir() {
    // The founder's outline example: a pattern into build output.
    let m = r#""exports":{"./features/*":{"types":"./dist/features/*.d.ts","import":"./dist/features/*.js"}}"#;
    proposes(&case("f3", m, &[("a/src/features/order.ts", ORDER), SRC_PAIR], "a/features/order"), "a/src/features/order.ts");
}

#[test]
fn s3_f4_the_longest_pattern_prefix_wins() {
    let m = r#""exports":{"./*":"./lib/*.ts","./features/*":"./src/features/*.ts"}"#;
    let other = "export interface Order {\n  other_a: number;\n  other_b: string;\n}\n";
    let o = case("f4", m, &[("a/src/features/order.ts", ORDER), ("a/lib/features/order.ts", other)], "a/features/order");
    proposes(&o, "a/src/features/order.ts");
}

#[test]
fn s3_f5_every_star_is_substituted() {
    proposes(&case("f5", r#""exports":{"./x/*":"./src/*/*.ts"}"#, &[("a/src/order/order.ts", ORDER)], "a/x/order"), "a/src/order/order.ts");
}

#[test]
fn s3_f6_declaration_dir_maps_a_d_ts_leaf_back() {
    let t = ("a/tsconfig.json", r#"{"compilerOptions":{"rootDir":"./src","outDir":"./dist","declarationDir":"./types"}}"#);
    proposes(&case("f6", r#""exports":{".":{"types":"./types/index.d.ts"}}"#, &[("a/src/index.ts", ORDER), t], "a"), "a/src/index.ts");
}

#[test]
fn s3_f14_jsonc_tsconfig_is_read() {
    let t = ("a/tsconfig.json", "{\n  // build settings\n  \"x-note\": \"see https://example.com/a//b\",\n  \"compilerOptions\": {\n    /* emitted */ \"outDir\": \"./dist\",\n    \"rootDir\": \"./src\",\n  },\n}\n");
    proposes(&case("f14", r#""exports":{".":"./dist/index.js"}"#, &[("a/src/index.ts", ORDER), t], "a"), "a/src/index.ts");
}

// ----------------------------------------------------------- abstentions

#[test]
fn s3_f7_a_dist_target_without_a_declared_pair_is_never_guessed() {
    let o = case("f7", r#""exports":{".":"./dist/index.js"}"#, &[("a/src/index.ts", ORDER)], "a");
    abstains(&o, "a", &not_source("./dist/index.js"));
}

#[test]
fn s3_f8_root_dir_is_never_computed() {
    let t = ("a/tsconfig.json", r#"{"compilerOptions":{"outDir":"./dist"}}"#);
    let o = case("f8", r#""exports":{".":"./dist/index.js"}"#, &[("a/src/index.ts", ORDER), t], "a");
    abstains(&o, "a", &not_source("./dist/index.js"));
}

#[test]
fn s3_f9_disagreeing_branches_are_never_picked() {
    let m = r#""exports":{".":{"types":"./src/a.ts","import":"./src/b.ts"}}"#;
    let o = case("f9", m, &[("a/src/a.ts", ORDER), ("a/src/b.ts", ORDER)], "a");
    abstains(&o, "a", "exports branches name different sources (a/src/a.ts, a/src/b.ts); not guessed");
}

#[test]
fn s3_f10_a_null_subpath_is_excluded() {
    let o = case("f10", r#""exports":{"./order":null,"./*":"./src/*.ts"}"#, &[("a/src/order.ts", ORDER)], "a/order");
    abstains(&o, "a/order", "exports excludes \"./order\" (null)");
}

#[test]
fn s3_f11_no_subpath_matches() {
    let o = case("f11", r#""exports":{"./order":"./src/order.ts"}"#, &[("a/src/order.ts", ORDER)], "a/other");
    abstains(&o, "a/other", "no exports entry matches \"./other\"");
}

#[test]
fn s3_f12_a_mixed_map_is_invalid() {
    let o = case("f12", r#""exports":{".":"./src/index.ts","import":"./src/index.ts"}"#, &[("a/src/index.ts", ORDER)], "a");
    abstains(&o, "a", "the exports map mixes subpath keys and condition keys");
}

#[test]
fn s3_f13_a_traversal_capture_never_escapes() {
    let o = case("f13", r#""exports":{"./*":"./src/*.ts"}"#, &[("a/x.ts", ORDER)], "a/../x");
    abstains(&o, "a/../x", "exports target \"./src/../x.ts\" escapes the package");
}

#[test]
fn s3_f15_an_inactive_condition_is_never_picked() {
    let o = case("f15", r#""exports":{".":{"browser":"./src/index.ts"}}"#, &[("a/src/index.ts", ORDER)], "a");
    abstains(&o, "a", "no exports condition among types, import, require, default");
}

#[test]
fn s3_f16_extends_is_not_followed() {
    let t = ("a/tsconfig.json", r#"{"extends":"./tsconfig.base.json"}"#);
    let base = ("a/tsconfig.base.json", r#"{"compilerOptions":{"rootDir":"./src","outDir":"./dist"}}"#);
    let o = case("f16", r#""exports":{".":"./dist/index.js"}"#, &[("a/src/index.ts", ORDER), t, base], "a");
    abstains(&o, "a", &not_source("./dist/index.js"));
}

// --------------------------------------------------------------- controls

#[test]
fn s3_n1_a_committed_d_ts_is_never_a_source() {
    let o = case("n1", r#""exports":{".":{"types":"./src/index.d.ts"}}"#, &[("a/src/index.d.ts", ORDER)], "a");
    abstains(&o, "a", &not_source("./src/index.d.ts"));
}

// Recorded at Gate 1 from the unchanged engine (0866583): a workspace with
// no "exports" anywhere must not change by one byte (CM-S3-13).
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
const N2_GOLDEN: &str = r#"# invar.toml generated by `invar init`.
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
files_scanned = "5"
truncated = "false"
packages = "2"
candidates = "1"
contracts_proposed = "1"
abstentions = "0"
inventory = "0"

# PROPOSED code:order.ts->use.ts
# status: PROPOSED (heuristic; each counted read is on a receiver declared as the shape in this file; field writes excluded; types are not inferred)
# E1 producer package a at a/package.json (package.json "workspaces" entry "a")
# E2 Order declared at a/order.ts:1
# E3 consumer package b at b/package.json (package.json "workspaces" entry "b")
# E4 Order imported at b/src/use.ts:1 from "a/order", resolved to a/order.ts
# E5 Order fields read: currency_code (b/src/use.ts:3), total_minor (b/src/use.ts:3)
# E5 Order stoplisted reads: (none)
# E6 pair a/order.ts -> b/src/use.ts
# category: production
# lock: run `invar lock` before `invar check`. Without a lock, check evaluates only payload.<field> reads and says nothing about this proposal (a HOLDS would be vacuous). `invar lock` binds every field of the producer file that the consumer mentions in any form, which is wider than the evidence above.
[[invariant]]
kind = "schema"
id = "code:order.ts->use.ts"
producer = "a/order.ts"
consumer = "b/src/use.ts"
evidence = "Order currency_code total_minor"

# ---- Abstentions (init declined to propose; not negative findings) ----
# (none)
# ---- Found but NOT proposed (inventory only) ----
# (none)
# SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE
"#;

#[test]
fn s3_n2_a_workspace_without_exports_is_byte_identical() {
    let use_ts = "import { Order } from 'a/order';\nexport function f(o: Order) {\n  return [o.total_minor, o.currency_code];\n}\n";
    let d = ws("n2", &[ROOT_AB, PKG_B, ("a/package.json", r#"{"name":"a"}"#), ("a/order.ts", ORDER), ("b/src/use.ts", use_ts)]);
    let o = init(&d);
    if std::env::var("S3_RECORD").is_ok() {
        std::fs::write(std::env::temp_dir().join("s3_n2_golden.txt"), &o.raw).unwrap();
    }
    assert_eq!(o.raw, N2_GOLDEN);
}

#[test]
fn s3_m1_two_runs_are_byte_identical() {
    // init() itself runs twice and asserts byte identity; this pins it on
    // the back-mapped pattern case, the most state-heavy path.
    let m = r#""exports":{"./features/*":{"types":"./dist/features/*.d.ts","import":"./dist/features/*.js"}}"#;
    let o = case("m1", m, &[("a/src/features/order.ts", ORDER), SRC_PAIR], "a/features/order");
    assert_eq!(o.summary, HIT, "{}", o.raw);
}
