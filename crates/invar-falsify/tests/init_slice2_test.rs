//! INIT-DISCOVERY slice 2 fixtures (docs/architecture/INIT-DISCOVERY-SLICE2.md,
//! FROZEN revision 2, section D; cm6 amended by the 2026-10-15 slice 2b ruling). Twenty-three tests: the twenty-two named
//! fixtures K1-F1..F3, K1-N1..N3, K1-U1..U4, K2-F1..F4, K2-N1..N4, K2-U1,
//! K2-U2, CM-6, CM-7, plus the shared determinism and disclaimer scan
//! (D-SHARED). Every fixture is synthetic, built in a temporary directory, and
//! run through the release path twice (`invar init --stdout`); the two outputs
//! must be byte-identical. Lock and check fixtures run the real binary and
//! assert exit codes (0 holds, 1 violated, 4 skipped or not evaluated).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s2_{tag}_{}_{nanos}", std::process::id()));
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

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

#[derive(Debug, Clone)]
struct B {
    id: String,
    kind: String,
    keys: BTreeMap<String, String>,
    evidence: Vec<String>,
    evkeys: Vec<String>,
}

#[derive(Debug)]
struct Out {
    blocks: Vec<B>,
    abstain: Vec<String>,
    raw: String,
}

const EGRESS_STATUS: &str = "# status: PROPOSED (heuristic; response-position reference found; runtime serialization not verified)";
const K2_STATUS: &str = "# status: PROPOSED (heuristic; zod value producer, type-blind consumer; receiver of field reads not verified)";

fn parse(m: &str) -> Out {
    let mut blocks = Vec::new();
    let mut abstain = Vec::new();
    let mut ev: Option<(String, Vec<String>)> = None;
    let mut cur: Option<B> = None;
    for l in m.lines() {
        if let Some(id) = l.strip_prefix("# PROPOSED ") {
            ev = Some((id.to_string(), Vec::new()));
            continue;
        }
        if l == "[[invariant]]" {
            let (id, evidence) = ev.take().expect("a [[invariant]] block without a PROPOSED comment");
            cur = Some(B { id, kind: String::new(), keys: BTreeMap::new(), evidence, evkeys: Vec::new() });
            continue;
        }
        if let Some(b) = cur.as_mut() {
            if l.trim().is_empty() {
                blocks.push(cur.take().unwrap());
            } else if let Some((k, v)) = l.split_once(" = ") {
                let v = v.trim_matches('"').to_string();
                match k {
                    "kind" => b.kind = v,
                    "evidence" => b.evkeys.push(v),
                    _ => {
                        b.keys.insert(k.to_string(), v);
                    }
                }
            }
            continue;
        }
        if let Some((_, e)) = ev.as_mut() {
            if l.starts_with("# ") {
                e.push(l.to_string());
                continue;
            }
        }
        if let Some(a) = l.strip_prefix("# ABSTAIN ") {
            abstain.push(format!("ABSTAIN {a}"));
        }
    }
    if let Some(b) = cur.take() {
        blocks.push(b);
    }
    let n_inv = m.matches("[[invariant]]").count() - m.matches("# [[invariant]]").count();
    assert_eq!(n_inv, blocks.len(), "{m}");
    for b in &blocks {
        assert!(b.evidence.iter().any(|e| e.starts_with("# lock: run `invar lock` before `invar check`")), "{} lacks the lock note: {m}", b.id);
        assert!(b.evidence.iter().any(|e| e.starts_with("# category: ")), "{} lacks a category", b.id);
        // R-P-4 / R-K1-2(4): the heuristic disclaimer is on every proposal.
        match b.kind.as_str() {
            "egress" => {
                assert!(b.evidence.iter().any(|e| e == EGRESS_STATUS), "{} lacks the egress status line", b.id);
                for need in ["# X1 ", "# X2 ", "# X3 ", "# X4 ", "# X5 ", "# X6 DISCLAIMER: runtime serialization of "] {
                    assert!(b.evidence.iter().any(|e| e.starts_with(need)), "{} lacks {need}", b.id);
                }
                for k in ["shape", "boundary", "eligibility"] {
                    assert!(b.keys.contains_key(k), "{} lacks the {k} key", b.id);
                }
                assert!(!b.keys.contains_key("consumer"), "{}: an egress block carries no consumer", b.id);
            }
            "schema" => {
                if b.evidence.iter().any(|e| e.starts_with("# Z1 ")) {
                    assert!(b.evidence.iter().any(|e| e == K2_STATUS), "{} lacks the K2 status line", b.id);
                    for need in ["# Z1 ", "# Z2 ", "# Z3 ", "# Z4 ", "# Z5 "] {
                        assert!(b.evidence.iter().any(|e| e.starts_with(need)), "{} lacks {need}", b.id);
                    }
                    assert!(!b.evkeys.is_empty(), "{} lacks evidence keys", b.id);
                }
            }
            // Slice 2b (ruled 2026-10-15): strings blocks are asserted by init_slice2b_test.rs.
            "strings" => {}
            other => panic!("unexpected kind {other}"),
        }
    }
    Out { blocks, abstain, raw: m.to_string() }
}

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_invar")
}

/// Run `invar init --stdout` twice; byte-identical (determinism).
fn init(dir: &Path) -> Out {
    let run = || {
        let o = Command::new(bin()).args(["init", "--stdout", "--root"]).arg(dir).output().unwrap();
        assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8(o.stdout).unwrap()
    };
    let first = run();
    let second = run();
    assert_eq!(first, second, "init output is not deterministic");
    parse(&first)
}

fn invar(dir: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(bin()).args(args).current_dir(dir).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

/// Write the proposed manifest (`init --force`), as a user would.
fn adopt(dir: &Path) {
    let (c, o) = invar(dir, &["init", "--force"]);
    assert_eq!(c, 0, "{o}");
}

fn binds(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = read(dir, "invar.lock")
        .lines()
        .filter_map(|l| l.strip_prefix("bind = \""))
        .map(|l| l.trim_end_matches('"').split(' ').nth(2).unwrap().to_string())
        .collect();
    v.sort();
    v
}

/// Slice 2 abstentions only: slice 1's own H2 line is not under test here.
fn s2(o: &Out) -> Vec<String> {
    o.abstain.iter().filter(|a| !a.starts_with("ABSTAIN H2 ")).cloned().collect()
}

fn blk<'a>(o: &'a Out, id: &str) -> &'a B {
    o.blocks.iter().find(|b| b.id == id).unwrap_or_else(|| panic!("no proposal {id}: {}", o.raw))
}

fn x3_fields(b: &B) -> Vec<String> {
    let l = b.evidence.iter().find(|e| e.starts_with("# X3 ")).unwrap();
    l.split("direct fields: ").nth(1).unwrap().split(", ").map(str::to_string).collect()
}

fn z4_fields(b: &B, shape: &str) -> Vec<String> {
    let p = format!("# Z4 {shape} fields read: ");
    let l = b.evidence.iter().find(|e| e.starts_with(&p)).unwrap();
    l[p.len()..].split(", ").map(|x| x.split(" (").next().unwrap().to_string()).collect()
}

fn category(b: &B) -> String {
    b.evidence.iter().find_map(|e| e.strip_prefix("# category: ")).unwrap().to_string()
}

fn has_abstain(o: &Out, code: &str, loc: &str) -> bool {
    o.abstain.iter().any(|a| a.starts_with(&format!("ABSTAIN {code} {loc}")))
}

// ----------------------------------------------------------- fixtures

const SCHEMAS: &str = "import { z } from \"zod\";\nexport const OrderResponse = z.object({\n  id: z.string(),\n  total_minor: z.number(),\n  currency_code: z.string(),\n  nested: z.object({ inner_ref: z.string() }),\n});\n";
const ROUTE: &str = "import { NextResponse } from \"next/server\";\nimport { OrderResponse } from \"../../../../lib/schemas\";\nexport async function GET(request: Request) {\n  const data = await load();\n  return NextResponse.json(OrderResponse.parse(data));\n}\n";
const ROUTE_PATH: &str = "app/api/v1/orders/route.ts";

fn k1_repo(tag: &str) -> PathBuf {
    ws(tag, &[("lib/schemas.ts", SCHEMAS), (ROUTE_PATH, ROUTE)])
}

const ORDER_IN: &str = "import { z } from \"zod\";\nexport const OrderIn = z.object({\n  total_minor: z.number(),\n  currency_code: z.string(),\n  \"ship-to\": z.string(),\n  unread_flag: z.boolean(),\n  meta: z.object({ owner_ref: z.string() }),\n});\nexport const Other = z.object({ mode: z.string(), kind_flag: z.string() });\n";

// ------------------------------------------------------------- K1 positives

#[test]
fn k1_f1_versioned_route_one_egress_contract_lock_and_additive_check() {
    let d = k1_repo("k1f1");
    let o = init(&d);
    assert_eq!(o.blocks.len(), 1, "{}", o.raw);
    assert!(s2(&o).is_empty(), "{:?}", s2(&o));
    let b = blk(&o, "egress:OrderResponse");
    assert_eq!(b.kind, "egress");
    assert_eq!(b.keys["shape"], "OrderResponse");
    assert_eq!(b.keys["boundary"], ROUTE_PATH);
    assert_eq!(b.keys["eligibility"], "versioned-path");
    assert_eq!(b.keys["producer"], "lib/schemas.ts");
    // Depth-1 members only: `nested` yes, its inner_ref no.
    assert_eq!(x3_fields(b), vec!["id", "total_minor", "currency_code", "nested"]);
    assert!(b.evidence.iter().any(|e| e == &format!("# X5 response-position reference(s): {ROUTE_PATH}:5")), "{:?}", b.evidence);
    assert!(b.evidence.iter().any(|e| e.starts_with("# X2 boundary file app/api/v1/orders/route.ts is eligible by versioned path segment \"v1\"")));
    assert!(o.raw.contains("kind = \"egress\""));
    assert!(o.raw.contains("SUMMARY packages=0 candidates=0 proposals=1 abstentions=1 inventory=0 status=COMPLETE"), "{}", o.raw);

    adopt(&d);
    // R-K1-5: checked before `invar lock`: exit 4, never a HOLDS.
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("SKIPPED") && !out.contains("HOLDS"), "{out}");
    let (c, out) = invar(&d, &["lock"]);
    assert_eq!(c, 0, "{out}");
    assert_eq!(binds(&d), vec!["OrderResponse.currency_code", "OrderResponse.id", "OrderResponse.nested", "OrderResponse.total_minor"]);
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("[HOLDS]"), "{out}");

    // R-K1-3: an ADDED field holds.
    let added = SCHEMAS.replace("  nested:", "  extra_flag: z.boolean(),\n  nested:");
    put(&d, "lib/schemas.ts", &added);
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("1 field(s) added since lock, tolerated (extra_flag)"), "{out}");

    // A REMOVED field is VIOLATED and named.
    put(&d, "lib/schemas.ts", &SCHEMAS.replace("  currency_code: z.string(),\n", ""));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 1, "{out}");
    assert!(out.contains("[VIOLATED]") && out.contains("OrderResponse.currency_code"), "{out}");

    // A RENAMED field is VIOLATED (the old name is gone).
    put(&d, "lib/schemas.ts", &SCHEMAS.replace("total_minor", "total_cents"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 1, "{out}");
    assert!(out.contains("OrderResponse.total_minor"), "{out}");
}

#[test]
fn k1_f2_explicit_marking_is_the_eligibility_evidence() {
    let d = ws(
        "k1f2",
        &[
            ("lib/schemas.ts", SCHEMAS),
            ("app/api/orders/route.ts", &ROUTE.replace("../../../../lib/schemas", "../../../lib/schemas")),
            ("invar.toml", "[egress]\nboundary = \"app/api/orders\"\n"),
        ],
    );
    let o = init(&d);
    assert_eq!(o.blocks.len(), 1, "{}", o.raw);
    let b = blk(&o, "egress:OrderResponse");
    assert_eq!(b.keys["eligibility"], "marked");
    assert_eq!(b.keys["boundary"], "app/api/orders/route.ts");
    assert!(b.evidence.iter().any(|e| e.contains("marked under [egress] in invar.toml (\"app/api/orders\")")), "{:?}", b.evidence);
    // The marking is written back, so regenerating does not lose it.
    assert!(o.raw.contains("[egress]\nboundary = \"app/api/orders\"\n"), "{}", o.raw);
    adopt(&d);
    let o2 = init(&d);
    assert_eq!(o2.blocks.len(), 1);
    assert_eq!(o2.blocks[0].keys["eligibility"], "marked");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    assert_eq!(invar(&d, &["check"]).0, 0);
}

#[test]
fn k1_f3_two_routes_two_schemas_two_contracts_with_unique_ids() {
    let route = |name: &str, spec: &str| {
        format!("import {{ {name} }} from \"{spec}\";\nexport function GET() {{\n  return Response.json({name}.parse(load()));\n}}\n")
    };
    let schema = |name: &str, a: &str, b: &str| {
        format!("import {{ z }} from \"zod\";\nexport const {name} = z.object({{\n  {a}: z.string(),\n  {b}: z.number(),\n}});\n")
    };
    let d = ws(
        "k1f3",
        &[
            ("lib/a/schema.ts", &schema("Payload", "alpha_ref", "alpha_count")),
            ("lib/b/schema.ts", &schema("Payload", "beta_ref", "beta_count")),
            ("app/api/v1/a/route.ts", &route("Payload", "../../../../lib/a/schema")),
            ("app/api/v1/b/route.ts", &route("Payload", "../../../../lib/b/schema")),
        ],
    );
    let o = init(&d);
    let ids: Vec<&str> = o.blocks.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, vec!["egress:Payload", "egress:Payload#2"], "{}", o.raw);
    assert_eq!(o.blocks[0].keys["producer"], "lib/a/schema.ts");
    assert_eq!(o.blocks[1].keys["producer"], "lib/b/schema.ts");
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    assert_eq!(binds(&d).len(), 4);
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("held 2 | violated 0"), "{out}");
}

// ------------------------------------------------------ K1 negatives, ambiguous

#[test]
fn k1_n1_unversioned_unmarked_route_never_proposes() {
    let d = ws("k1n1", &[("lib/schemas.ts", SCHEMAS), ("app/api/orders/route.ts", &ROUTE.replace("../../../../lib/schemas", "../../../lib/schemas"))]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    assert!(s2(&o).is_empty(), "an ineligible file is outside slice 2 scope, not an ambiguity: {:?}", s2(&o));
}

#[test]
fn k1_n2_hoppscotch_verzod_family_zero_proposals_zero_alarms() {
    let v1 = "import { z } from \"zod\";\nexport const V1_SCHEMA = z.object({\n  v: z.literal(1),\n  name: z.string(),\n  folders: z.array(z.any()),\n  requests: z.array(z.any()),\n});\n";
    let v2 = "import { z } from \"zod\";\nimport { V1_SCHEMA } from \"../1\";\nexport const V2_SCHEMA = V1_SCHEMA.extend({\n  v: z.literal(2),\n  auth: z.any(),\n  headers: z.array(z.any()),\n});\n";
    let fam_route = "import { V1_SCHEMA } from \"../../../../../../packages/data/src/collection/v/1\";\nimport { V2_SCHEMA } from \"../../../../../../packages/data/src/collection/v/2\";\nexport async function GET() {\n  const old = await loadOld();\n  const cur = await loadCurrent();\n  if (old) return Response.json(V1_SCHEMA.parse(old));\n  return Response.json(V2_SCHEMA.parse(cur));\n}\n";
    let d = ws(
        "k1n2",
        &[
            ("packages/data/src/collection/v/1/index.ts", v1),
            ("packages/data/src/collection/v/2/index.ts", v2),
            ("apps/backend/app/api/v1/collections/route.ts", fam_route),
            ("lib/schemas.ts", SCHEMAS),
            (ROUTE_PATH, ROUTE),
        ],
    );
    let o = init(&d);
    // 0 cross-version proposals: only the non-family contract is proposed.
    let ids: Vec<&str> = o.blocks.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, vec!["egress:OrderResponse"], "{}", o.raw);
    assert_eq!(
        s2(&o),
        vec![
            "ABSTAIN VERSION-FAMILY apps/backend/app/api/v1/collections/route.ts:1 V1_SCHEMA at packages/data/src/collection/v/1/index.ts:2 lives in a schema version family (path layout v/1); compatibility machinery presumed, no egress lock proposed".to_string(),
            "ABSTAIN VERSION-FAMILY apps/backend/app/api/v1/collections/route.ts:2 V2_SCHEMA at packages/data/src/collection/v/2/index.ts:3 lives in a schema version family (path layout v/2); compatibility machinery presumed, no egress lock proposed".to_string(),
        ]
    );
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    assert_eq!(invar(&d, &["check"]).0, 0);
    // 0 alarms from reorganization: v/2 becomes v/3, constants renamed.
    std::fs::remove_dir_all(d.join("packages/data/src/collection/v/2")).unwrap();
    put(&d, "packages/data/src/collection/v/3/index.ts", &v2.replace("V2_SCHEMA", "V3_SCHEMA").replace("literal(2)", "literal(3)"));
    put(&d, "apps/backend/app/api/v1/collections/route.ts", &fam_route.replace("V2_SCHEMA", "V3_SCHEMA").replace("v/2", "v/3"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "reorganizing a version family must not alarm: {out}");
    assert!(out.contains("[HOLDS]") && !out.contains("VIOLATED"), "{out}");
}

#[test]
fn k1_n3_schema_imported_but_never_referenced_abstains() {
    let route = "import { OrderResponse } from \"../../../../lib/schemas\";\nexport function GET() {\n  return Response.json({ ok: true });\n}\n";
    let d = ws("k1n3", &[("lib/schemas.ts", SCHEMAS), (ROUTE_PATH, route)]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    assert_eq!(s2(&o).len(), 1, "{:?}", s2(&o));
    assert!(has_abstain(&o, "AMBIG-EGRESS", "app/api/v1/orders/route.ts:1"), "{:?}", o.abstain);
    assert!(s2(&o)[0].contains("imported but never referenced"), "{:?}", s2(&o));
}

#[test]
fn k1_u1_two_schemas_imported_none_referenced() {
    let two = format!("{SCHEMAS}export const UserResponse = z.object({{ user_ref: z.string(), user_count: z.number() }});\n");
    let route = "import { OrderResponse, UserResponse } from \"../../../../lib/schemas\";\nexport function GET() {\n  return Response.json({ ok: true });\n}\n";
    let d = ws("k1u1", &[("lib/schemas.ts", &two), (ROUTE_PATH, route)]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = s2(&o);
    assert_eq!(a.len(), 2, "{a:?}");
    assert!(a.iter().all(|x| x.starts_with("ABSTAIN AMBIG-EGRESS app/api/v1/orders/route.ts:1 ")), "{a:?}");
    assert!(a[0].contains("OrderResponse") && a[1].contains("UserResponse"), "{a:?}");
}

#[test]
fn k1_u2_locked_schema_turned_dynamic_is_not_evaluated() {
    let d = k1_repo("k1u2");
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    assert_eq!(invar(&d, &["check"]).0, 0);
    let dynamic = "import { z } from \"zod\";\nconst Base = z.object({ id: z.string() });\nexport const OrderResponse = Base.extend({\n  total_minor: z.number(),\n  currency_code: z.string(),\n});\n";
    put(&d, "lib/schemas.ts", dynamic);
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 4, "R-K1-4: unextractable is exit 4, never HOLDS: {out}");
    assert!(out.contains("SKIPPED") && out.contains("no longer statically extractable"), "{out}");
    assert!(!out.contains("[HOLDS]"), "{out}");
}

#[test]
fn k1_u3_no_lock_means_no_verdict() {
    let d = k1_repo("k1u3");
    adopt(&d);
    assert!(!d.join("invar.lock").exists());
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("an unlocked egress contract cannot hold"), "{out}");
    assert!(!out.contains("[HOLDS]"), "{out}");
}

#[test]
fn k1_u4_request_side_only_reference_abstains() {
    let route = "import { OrderResponse } from \"../../../../lib/schemas\";\nexport async function POST(request: Request) {\n  const parsed = OrderResponse.safeParse(await request.json());\n  if (!parsed.success) {\n    return fail();\n  }\n  return ok();\n}\n";
    let d = ws("k1u4", &[("lib/schemas.ts", SCHEMAS), (ROUTE_PATH, route)]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = s2(&o);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].starts_with("ABSTAIN AMBIG-EGRESS app/api/v1/orders/route.ts:1 "), "{a:?}");
    assert!(a[0].contains("request-side"), "{a:?}");
}


fn k1_abstain_ambig(tag: &str, route: &str) {
    let d = ws(tag, &[("lib/schemas.ts", SCHEMAS), (ROUTE_PATH, route)]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = s2(&o);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].starts_with("ABSTAIN AMBIG-EGRESS app/api/v1/orders/route.ts:1 "), "{a:?}");
}

#[test]
fn k1_u5_schema_factory_bare_return_abstains() {
    k1_abstain_ambig("k1u5", "import { OrderResponse } from \"../../../../lib/schemas\";\nexport function makeSchema() {\n  return OrderResponse;\n}\n");
}

#[test]
fn k1_u6_returned_client_config_with_schema_abstains() {
    k1_abstain_ambig("k1u6", "import { OrderResponse } from \"../../../../lib/schemas\";\nexport const config = () => {\n  return { client: \"x\", schema: OrderResponse };\n};\n");
}

// ------------------------------------------------------------- K2 positives

fn consumer_js(reads: &str) -> String {
    format!("import {{ OrderIn }} from \"../lib/order.js\";\nexport function f(row) {{\n{reads}\n}}\n")
}

#[test]
fn k2_f1_extraction_exactness() {
    let d = ws(
        "k2f1",
        &[
            ("lib/order.ts", ORDER_IN),
            ("src/consume.js", &consumer_js("  const a = row.total_minor + row['ship-to'];\n  const b = row.currency_code;\n  return [a, b, row.owner_ref];")),
        ],
    );
    let o = init(&d);
    assert_eq!(o.blocks.len(), 1, "{}", o.raw);
    let b = blk(&o, "code:order.ts->consume.js");
    assert_eq!(b.kind, "schema");
    assert_eq!(b.keys["producer"], "lib/order.ts");
    assert_eq!(b.keys["consumer"], "src/consume.js");
    // Depth-1 keys only: owner_ref (inside meta) is not a field of OrderIn.
    assert_eq!(z4_fields(b, "OrderIn"), vec!["currency_code", "ship-to", "total_minor"]);
    assert_eq!(b.evkeys, vec!["OrderIn currency_code ship-to total_minor"]);
    assert_eq!(category(b), "production");
}

#[test]
fn k2_f2_js_consumer_lock_binds_exactly_the_evidence() {
    let d = ws(
        "k2f2",
        &[
            ("lib/order.ts", ORDER_IN),
            ("src/consume.js", &format!("import {{ OrderIn, Other }} from \"../lib/order\";\nexport function f(row, x) {{\n  return [row.total_minor, row.currency_code, x.mode];\n}}\n")),
        ],
    );
    let o = init(&d);
    assert_eq!(o.blocks.len(), 1, "{}", o.raw);
    assert_eq!(z4_fields(&o.blocks[0], "OrderIn"), vec!["currency_code", "total_minor"]);
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    // Exactly the evidence, scoped: not unread_flag, not Other.mode, no unscoped twin.
    assert_eq!(binds(&d), vec!["OrderIn.currency_code", "OrderIn.total_minor"]);
    assert_eq!(invar(&d, &["check"]).0, 0);
    // A field outside the evidence may change without an alarm.
    put(&d, "lib/order.ts", &ORDER_IN.replace("mode: z.string(), ", ""));
    assert_eq!(invar(&d, &["check"]).0, 0);
    // A bound field renamed: VIOLATED naming it.
    put(&d, "lib/order.ts", &ORDER_IN.replace("total_minor", "total_cents"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 1, "{out}");
    assert!(out.contains("OrderIn.total_minor"), "{out}");
}

#[test]
fn k2_f3_typescript_consumer_reading_only_through_bracket_literals() {
    let ts = "import { OrderIn } from \"../lib/order\";\nexport function f(p: any) {\n  OrderIn.parse(p);\n  return [p[\"total_minor\"], p[\"currency_code\"]];\n}\n";
    let d = ws("k2f3", &[("lib/order.ts", ORDER_IN), ("src/c.ts", ts)]);
    let o = init(&d);
    assert_eq!(o.blocks.len(), 1, "{}", o.raw);
    let b = &o.blocks[0];
    assert_eq!(z4_fields(b, "OrderIn"), vec!["currency_code", "total_minor"]);
    assert!(b.evidence.iter().any(|e| e.contains("TypeScript file reading only through bracket literals")), "{:?}", b.evidence);
    assert!(s2(&o).is_empty(), "{:?}", s2(&o));
}

// ------------------------------------------------- K2 negatives, ambiguous

#[test]
fn k2_n1_nested_members_do_not_count() {
    let d = ws("k2n1", &[("lib/order.ts", ORDER_IN), ("src/c.js", &consumer_js("  return [row.total_minor, row.owner_ref];"))]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    assert!(s2(&o).is_empty(), "{:?}", s2(&o));
}

#[test]
fn k2_n2_stoplist_and_minimum() {
    let schema = "import { z } from \"zod\";\nexport const OrderIn = z.object({ id: z.string(), name: z.string(), status: z.string(), total_minor: z.number() });\n";
    let d = ws("k2n2", &[("lib/order.ts", schema), ("src/c.js", &consumer_js("  return [row.id, row.name, row.status, row.total_minor];"))]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "one distinctive field is below the minimum: {}", o.raw);
}

#[test]
fn k2_n3_two_schemas_in_one_file_never_pool() {
    let schema = "import { z } from \"zod\";\nexport const A = z.object({ a_field: z.string(), b_field: z.string() });\nexport const B = z.object({ c_field: z.string(), d_field: z.string() });\n";
    let c = "import { A, B } from \"../lib/two\";\nexport function f(r) {\n  return [r.a_field, r.c_field];\n}\n";
    let d = ws("k2n3", &[("lib/two.ts", schema), ("src/c.js", c)]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "A has 1 read, B has 1 read; pooled they would be 2: {}", o.raw);
}

#[test]
fn k2_n4_typed_consumer_is_tsc_governed() {
    let typed = "import { z } from \"zod\";\nimport { OrderIn } from \"../lib/order\";\ntype T = z.infer<typeof OrderIn>;\nexport function f(v: T) {\n  return [v.total_minor, v.currency_code];\n}\n";
    let d = ws("k2n4", &[("lib/order.ts", ORDER_IN), ("src/typed.ts", typed)]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    assert_eq!(s2(&o).len(), 1, "{:?}", s2(&o));
    assert!(has_abstain(&o, "TSC-GOVERNED", "src/typed.ts:2"), "{:?}", o.abstain);
}

#[test]
fn k2_u1_extend_chain_is_unsupported_dynamic() {
    let schema = "import { z } from \"zod\";\nconst Base = z.object({ total_minor: z.number(), currency_code: z.string() });\nexport const OrderExt = Base.extend({ extra_ref: z.string() });\nexport const Chained = z.object({ total_minor: z.number(), currency_code: z.string() }).extend({ x_ref: z.string() });\n";
    let c = "import { OrderExt, Chained } from \"../lib/ext\";\nexport function f(r) {\n  return [r.total_minor, r.currency_code, r.extra_ref, r.x_ref];\n}\n";
    let d = ws("k2u1", &[("lib/ext.ts", schema), ("src/c.js", c)]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = s2(&o);
    assert_eq!(a.len(), 2, "{a:?}");
    assert!(a.iter().all(|x| x.starts_with("ABSTAIN UNSUPPORTED-DYNAMIC src/c.js:1 ")), "{a:?}");
    assert!(a.iter().any(|x| x.contains("OrderExt") && x.contains("derived from Base via .extend()")), "{a:?}");
    assert!(a.iter().any(|x| x.contains("Chained") && x.contains(".extend() chain")), "{a:?}");
}

#[test]
fn k2_u2_spread_and_computed_members_are_unsupported_dynamic() {
    let schema = "import { z } from \"zod\";\nconst Base = z.object({ base_ref: z.string() });\nexport const Spread = z.object({ ...Base.shape, total_minor: z.number(), currency_code: z.string() });\nexport const Computed = z.object({ [key]: z.string(), total_minor: z.number(), currency_code: z.string() });\n";
    let c = "import { Spread, Computed } from \"../lib/dyn\";\nexport function f(r) {\n  return [r.total_minor, r.currency_code];\n}\n";
    let d = ws("k2u2", &[("lib/dyn.ts", schema), ("src/c.js", c)]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = s2(&o);
    assert_eq!(a.len(), 2, "{a:?}");
    assert!(a.iter().any(|x| x.contains("Spread") && x.contains("spread member")), "{a:?}");
    assert!(a.iter().any(|x| x.contains("Computed") && x.contains("computed key")), "{a:?}");
}

#[test]
fn cm6_string_field_list_is_a_k3_positive_since_slice_2b() {
    // Amended by the founder's ruling of 2026-10-15 (slice 2b Gate 4): before
    // slice 2b this fixture asserted that slice 2 does not absorb string lists.
    // With K3 implemented it is an exact K3 positive: three distinctive names,
    // all of the list, a query-named list, import-linked to the shape.
    let c = "import { OrderIn } from \"../lib/order\";\nexport const SELECT_FIELDS = ['total_minor', 'currency_code', 'unread_flag'];\nexport function pick() {\n  return SELECT_FIELDS.join(',');\n}\n";
    let d = ws("cm6", &[("lib/order.ts", ORDER_IN), ("src/c.js", c)]);
    let o = init(&d);
    assert_eq!(o.blocks.len(), 1, "{}", o.raw);
    let b = &o.blocks[0];
    assert_eq!(b.kind, "strings");
    assert_eq!(b.keys["shape"], "OrderIn");
    assert_eq!(b.keys["list"], "src/c.js");
    assert_eq!(b.keys["listat"], "const:SELECT_FIELDS");
    assert_eq!(b.keys["overlap"], "currency_code total_minor unread_flag");
    assert!(s2(&o).is_empty(), "{:?}", s2(&o));
}

#[test]
fn cm7_test_fixture_consumers_are_category_labeled() {
    let d = ws(
        "cm7",
        &[
            ("lib/order.ts", ORDER_IN),
            ("tests/order.test.js", "import { OrderIn } from \"../lib/order\";\ntest('x', () => {\n  const r = load();\n  expect(r.total_minor).toBe(1);\n  expect(r.currency_code).toBe('EUR');\n});\n"),
        ],
    );
    let o = init(&d);
    assert_eq!(o.blocks.len(), 1, "{}", o.raw);
    assert_eq!(category(&o.blocks[0]), "test");
}

#[test]
fn k2_f4_typed_and_js_consumers_exactly_one_pair_for_the_js_file() {
    let typed = "import { z } from \"zod\";\nimport { OrderIn } from \"../lib/order\";\ntype T = z.infer<typeof OrderIn>;\nexport const g = (v: T) => [v.total_minor, v.currency_code];\n";
    let d = ws(
        "k2f4",
        &[
            ("lib/order.ts", ORDER_IN),
            ("src/typed.ts", typed),
            ("src/blind.js", &consumer_js("  return [row.total_minor, row.currency_code];")),
        ],
    );
    let o = init(&d);
    assert_eq!(o.blocks.len(), 1, "{}", o.raw);
    assert_eq!(o.blocks[0].keys["consumer"], "src/blind.js");
    let a = s2(&o);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(has_abstain(&o, "TSC-GOVERNED", "src/typed.ts:2"), "{a:?}");
}

// ------------------------------------------------------------- shared scan

#[test]
fn d_shared_every_proposal_is_deterministic_and_carries_its_disclaimer() {
    let d = ws(
        "dshared",
        &[
            ("lib/schemas.ts", SCHEMAS),
            (ROUTE_PATH, ROUTE),
            ("lib/order.ts", ORDER_IN),
            ("src/consume.js", &consumer_js("  return [row.total_minor, row.currency_code];")),
            ("src/typed.ts", "import { z } from \"zod\";\nimport { OrderIn } from \"../lib/order\";\ntype T = z.infer<typeof OrderIn>;\nexport const g = (v: T) => v.total_minor;\n"),
        ],
    );
    // `init` itself asserts byte-identical output across two runs and parse()
    // asserts the status line, evidence lines and lock note on every block.
    let o = init(&d);
    assert_eq!(o.blocks.len(), 2, "{}", o.raw);
    let kinds: Vec<&str> = o.blocks.iter().map(|b| b.kind.as_str()).collect();
    assert_eq!(kinds, vec!["egress", "schema"]);
    for b in &o.blocks {
        assert!(b.evidence.iter().any(|e| e.starts_with("# status: PROPOSED (heuristic; ")), "{}", b.id);
    }
    let egress = blk(&o, "egress:OrderResponse");
    assert!(egress.evidence.iter().any(|e| e.contains("a response-position reference does not prove the route emits this schema")));
    // The two contract classes stay distinct in the output text.
    assert!(o.raw.contains("EMITTED-SHAPE contract, not a producer/consumer pair"));
    assert!(o.raw.contains("A proposal never claims the consumer is correct."));
}
