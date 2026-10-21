//! INIT-DISCOVERY-SLICE6 fixtures (rulings S6-D1 to S6-D4): root tsconfig
//! `paths` in egress discovery (K1), and earned silence there. Zero yield:
//! no proposal changes. Each fixture is a synthetic workspace with an
//! eligible boundary file (a versioned route), run through
//! `invar init --stdout` twice, byte-identical.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s6_{tag}_{}_{nanos}", std::process::id()));
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    dir
}

struct Out {
    ids: Vec<String>,
    producers: Vec<String>,
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
    let val = |l: &str, k: &str| l.strip_prefix(k).map(|v| v.trim_matches('"').to_string());
    Out {
        ids: a.lines().filter_map(|l| val(l, "id = ")).collect(),
        producers: a.lines().filter_map(|l| val(l, "producer = ")).collect(),
        abstain: a.lines().filter_map(|l| l.strip_prefix("# ABSTAIN ")).filter(|l| !l.starts_with("H2 ")).map(|l| format!("ABSTAIN {l}")).collect(),
        summary: a.lines().last().unwrap_or("").trim_start_matches("# ").to_string(),
        raw: a,
    }
}

const SCHEMAS: &[u8] = b"import { z } from \"zod\";\nexport const OrderResponse = z.object({\n  id: z.string(),\n  total_minor: z.number(),\n  currency_code: z.string(),\n});\n";
const ROUTE_PATH: &str = "app/api/v1/orders/route.ts";
const PATHS: &[u8] = br#"{ "compilerOptions": { "baseUrl": ".", "paths": { "@/*": ["./src/*"] } } }"#;

fn route(spec: &str) -> Vec<u8> {
    format!("import {{ OrderResponse }} from \"{spec}\";\nexport async function GET() {{\n  const data = await load();\n  return Response.json(OrderResponse.parse(data));\n}}\n").into_bytes()
}

/// A route importing `names` from `spec` and using each in a non-response
/// position (values the route calls; none is an egress schema).
fn route_values(spec: &str, names: &str) -> Vec<u8> {
    format!("import {{ {names} }} from \"{spec}\";\nexport async function GET() {{\n  return Response.json({{ ok: [{names}].length }});\n}}\n").into_bytes()
}

fn proposes_order(o: &Out, producer: &str) {
    assert_eq!(o.ids, vec!["egress:OrderResponse".to_string()], "{}", o.raw);
    assert_eq!(o.producers, vec![producer.to_string()], "{}", o.raw);
    assert!(o.abstain.is_empty(), "{:?}", o.abstain);
}

fn silent(o: &Out) {
    assert!(o.ids.is_empty(), "{}", o.raw);
    assert!(o.abstain.is_empty(), "{:?}", o.abstain);
}

fn loud(o: &Out, line: &str) {
    assert!(o.ids.is_empty(), "{}", o.raw);
    assert_eq!(o.abstain, vec![line.to_string()], "{}", o.raw);
}

// ------------------------------------------------------------ paths (B2, B3)

#[test]
fn s6_f1_an_alias_resolves_and_proposes_as_a_relative_import_would() {
    let rel = init(&ws("f1r", &[("src/lib/schemas.ts", SCHEMAS), (ROUTE_PATH, &route("../../../../src/lib/schemas"))]));
    let o = init(&ws("f1", &[("tsconfig.json", PATHS), ("src/lib/schemas.ts", SCHEMAS), (ROUTE_PATH, &route("@/lib/schemas"))]));
    proposes_order(&rel, "src/lib/schemas.ts");
    proposes_order(&o, "src/lib/schemas.ts");
    assert_eq!(o.summary, rel.summary);
}

#[test]
fn s6_f2_the_longest_pattern_wins() {
    let paths = br#"{ "compilerOptions": { "paths": { "@/*": ["./src/*"], "@/lib/*": ["./alt/*"] } } }"#;
    let o = init(&ws("f2", &[("tsconfig.json", paths), ("src/lib/schemas.ts", SCHEMAS), ("alt/schemas.ts", SCHEMAS), (ROUTE_PATH, &route("@/lib/schemas"))]));
    proposes_order(&o, "alt/schemas.ts");
}

#[test]
fn s6_f3_targets_are_tried_in_order() {
    let paths = br#"{ "compilerOptions": { "paths": { "@/*": ["./missing/*", "./src/*", "./alt/*"] } } }"#;
    let o = init(&ws("f3", &[("tsconfig.json", paths), ("src/lib/schemas.ts", SCHEMAS), ("alt/lib/schemas.ts", SCHEMAS), (ROUTE_PATH, &route("@/lib/schemas"))]));
    proposes_order(&o, "src/lib/schemas.ts");
}

#[test]
fn s6_f4_a_matching_alias_without_a_target_abstains() {
    let o = init(&ws("f4", &[("tsconfig.json", PATHS), (ROUTE_PATH, &route("@/lib/nothing"))]));
    loud(&o, "ABSTAIN UNRESOLVED app/api/v1/orders/route.ts:1 import from \"@/lib/nothing\" in an eligible boundary file: path alias \"@/lib/nothing\" matches tsconfig paths \"@/*\" but no target file exists; not guessed");
}

#[test]
fn s6_f4b_a_target_only_inside_an_unscanned_directory_says_so() {
    let paths = br#"{ "compilerOptions": { "paths": { "@v/*": ["./vendor/*"] } } }"#;
    let o = init(&ws("f4b", &[("tsconfig.json", paths), ("vendor/schemas.ts", SCHEMAS), (ROUTE_PATH, &route("@v/schemas"))]));
    loud(&o, "ABSTAIN UNRESOLVED app/api/v1/orders/route.ts:1 import from \"@v/schemas\" in an eligible boundary file: path alias \"@v/schemas\" matches tsconfig paths \"@v/*\"; its target vendor/schemas.ts lies in a directory init does not scan; not guessed");
}

#[test]
fn s6_f9_extends_only_keeps_the_generic_reason() {
    let ext = br#"{ "extends": "./tsconfig.base.json" }"#;
    let base = br#"{ "compilerOptions": { "paths": { "@/*": ["./src/*"] } } }"#;
    let o = init(&ws("f9", &[("tsconfig.json", ext), ("tsconfig.base.json", base), ("src/lib/schemas.ts", SCHEMAS), (ROUTE_PATH, &route("@/lib/schemas"))]));
    loud(&o, "ABSTAIN UNRESOLVED app/api/v1/orders/route.ts:1 import from \"@/lib/schemas\" in an eligible boundary file: path alias (tsconfig paths are not read)");
}

#[test]
fn s6_f10_an_alias_matching_no_key_says_so() {
    let o = init(&ws("f10", &[("tsconfig.json", PATHS), ("src/lib/schemas.ts", SCHEMAS), (ROUTE_PATH, &route("~/lib/schemas"))]));
    loud(&o, "ABSTAIN UNRESOLVED app/api/v1/orders/route.ts:1 import from \"~/lib/schemas\" in an eligible boundary file: path alias \"~/lib/schemas\" matches no key of the root tsconfig paths; not guessed");
}

// ------------------------------------------------------- earned silence (B4)

#[test]
fn s6_f5_a_value_imported_through_an_alias_is_silent() {
    let fmt = b"export function formatOrder(x: number) {\n  return String(x);\n}\nexport type OrderView = { label: string };\n";
    let o = init(&ws("f5", &[("tsconfig.json", PATHS), ("src/lib/format.ts", fmt), (ROUTE_PATH, &route_values("@/lib/format", "formatOrder, OrderView"))]));
    silent(&o);
}

#[test]
fn s6_f6_a_value_re_exported_through_an_alias_is_silent() {
    let idx = b"export { helper } from \"@/lib/helpers\";\nexport * from \"@/lib/more\";\n";
    let o = init(&ws(
        "f6",
        &[
            ("tsconfig.json", PATHS),
            ("src/lib/index.ts", idx),
            ("src/lib/helpers.ts", b"export const helper = 1;\n"),
            ("src/lib/more.ts", b"export class Widget {}\n"),
            (ROUTE_PATH, &route_values("@/lib", "helper, Widget")),
        ],
    ));
    silent(&o);
}

#[test]
fn s6_f6b_an_imported_then_exported_value_is_silent() {
    let err = b"import { sanitize } from \"./sanitization\";\nexport { sanitize };\n";
    let o = init(&ws(
        "f6b",
        &[("tsconfig.json", PATHS), ("src/lib/error.ts", err), ("src/lib/sanitization.ts", b"export function sanitize(m: string) {\n  return m;\n}\n"), (ROUTE_PATH, &route_values("@/lib/error", "sanitize"))],
    ));
    silent(&o);
}

#[test]
fn s6_f7_a_zod_schema_that_is_not_an_object_abstains() {
    let s = b"import { z } from \"zod\";\nexport const createSchema = z\n  .unknown()\n  .transform((body) => body);\n";
    let o = init(&ws("f7", &[("tsconfig.json", PATHS), ("src/lib/schema.ts", s), (ROUTE_PATH, &route_values("@/lib/schema", "createSchema"))]));
    loud(&o, "ABSTAIN UNRESOLVED app/api/v1/orders/route.ts:1 createSchema from \"@/lib/schema\": declared as a Zod schema at src/lib/schema.ts:2 that is not an object shape; not guessed");
}

#[test]
fn s6_f8_a_name_declared_nowhere_abstains() {
    let o = init(&ws("f8", &[("tsconfig.json", PATHS), ("src/lib/format.ts", b"export const other = 1;\n"), (ROUTE_PATH, &route_values("@/lib/format", "ghost"))]));
    loud(&o, "ABSTAIN UNRESOLVED app/api/v1/orders/route.ts:1 ghost from \"@/lib/format\": not declared in src/lib/format.ts or along its re-exports; not guessed");
}

#[test]
fn s6_f11_an_unread_file_on_the_path_means_silence_is_not_earned() {
    let idx = b"export * from \"./ok\";\nexport * from \"./bad\";\n";
    let o = init(&ws(
        "f11",
        &[("tsconfig.json", PATHS), ("src/lib/index.ts", idx), ("src/lib/ok.ts", b"export const helper = 1;\n"), ("src/lib/bad.ts", b"export const \xff\xfe = 2;\n"), (ROUTE_PATH, &route_values("@/lib", "helper"))],
    ));
    loud(&o, "ABSTAIN UNRESOLVED app/api/v1/orders/route.ts:1 helper from \"@/lib\": src/lib/bad.ts on its re-export path could not be read; silence is not earned; not guessed");
}

// ------------------------------------------------------------ no regression

#[test]
fn s6_n1_a_workspace_without_tsconfig_is_byte_identical() {
    // Golden recorded at Gate 1 on the d4488ac binary.
    let o = init(&ws("n1", &[("lib/schemas.ts", SCHEMAS), (ROUTE_PATH, &route("../../../../lib/schemas"))]));
    assert_eq!(o.raw, N1_GOLDEN);
}

#[test]
fn s6_m1_key_order_in_paths_does_not_matter() {
    let ab = br#"{ "compilerOptions": { "paths": { "@/*": ["./src/*"], "@/lib/*": ["./alt/*"] } } }"#;
    let ba = br#"{ "compilerOptions": { "paths": { "@/lib/*": ["./alt/*"], "@/*": ["./src/*"] } } }"#;
    let one = |tag: &str, p: &[u8]| init(&ws(tag, &[("tsconfig.json", p), ("src/lib/schemas.ts", SCHEMAS), ("alt/schemas.ts", SCHEMAS), (ROUTE_PATH, &route("@/lib/schemas"))])).raw;
    assert_eq!(one("m1a", ab), one("m1b", ba));
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
files_scanned = "2"
truncated = "false"
packages = "0"
candidates = "0"
contracts_proposed = "1"
abstentions = "1"
inventory = "0"

# Slice 2: blocks with kind = "egress" are EMITTED-SHAPE contracts: a promise
# about the direct keys of a zod schema this repository emits at a boundary
# (versioned path or [egress] marking, plus a response-position reference).
# They carry no consumer evidence, and runtime serialization is NOT verified.

# PROPOSED egress:OrderResponse
# status: PROPOSED (heuristic; response-position reference found; runtime serialization not verified)
# X1 EMITTED-SHAPE contract, not a producer/consumer pair: a promise about what this repository emits at a boundary; no consumer evidence is used or claimed
# X2 boundary file app/api/v1/orders/route.ts is eligible by versioned path segment "v1"
# X3 OrderResponse declared at lib/schemas.ts:2 as a z.object literal with 3 direct fields: id, total_minor, currency_code
# X4 OrderResponse imported at app/api/v1/orders/route.ts:1 from "../../../../lib/schemas", resolved to lib/schemas.ts
# X5 response-position reference(s): app/api/v1/orders/route.ts:4
# X6 DISCLAIMER: runtime serialization of OrderResponse is not verified; a response-position reference does not prove the route emits this schema
# category: production
# lock: run `invar lock` before `invar check`. Without a lock an egress contract is SKIPPED (exit 4), never HOLDS. The lock binds every direct field of OrderResponse; at check a removed or renamed field is VIOLATED and an added field HOLDS.
[[invariant]]
kind = "egress"
id = "egress:OrderResponse"
producer = "lib/schemas.ts"
shape = "OrderResponse"
boundary = "app/api/v1/orders/route.ts"
eligibility = "versioned-path"

# ---- Abstentions (init declined to propose; not negative findings) ----
# ABSTAIN H2 . 0 qualifying package(s); a multi-package workspace needs at least 2
# ---- Found but NOT proposed (inventory only) ----
# (none)
# codes: run `invar explain <CODE>` for any ABSTAIN code above
# SUMMARY packages=0 candidates=0 proposals=1 abstentions=1 inventory=0 status=COMPLETE
"#;
