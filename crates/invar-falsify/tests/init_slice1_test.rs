//! INIT-DISCOVERY slice 1 fixtures (definition package revision 2, section 7,
//! plus U07, U08, F06, F07, M05, U09, and E01 for run errors). Every fixture
//! is synthetic, built in a temporary directory, and run through the release
//! path (`invar init --stdout`) twice; the two outputs must be byte-identical
//! (D01). Each fixture asserts its complete outcome: proposals (id, producer,
//! consumer, category, fields per shape, stoplisted reads), every ABSTAIN
//! line, every INVENTORY line and the SUMMARY line. Every proposal is checked
//! for E1 to E6, the status line and the lock note.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const STATUS: &str = "# status: PROPOSED (heuristic; receiver of field reads not verified)";

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s1_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (rel, body) in files {
        put(&dir, rel, body.as_bytes());
    }
    dir
}

fn put(root: &Path, rel: &str, body: &[u8]) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

const ROOT_AB: (&str, &str) = ("package.json", r#"{"private":true,"workspaces":["a","b"]}"#);
const PKG_A: (&str, &str) = ("a/package.json", r#"{"name":"a"}"#);
const PKG_B: (&str, &str) = ("b/package.json", r#"{"name":"b"}"#);
const ORDER: &str = "export interface Order {\n  total_minor: number;\n  currency_code: string;\n  id: string;\n}\n";
const USE_ORDER: &str = "import { Order } from 'a/order';\nexport function f(o: Order) {\n  return [o.total_minor, o.currency_code];\n}\n";

#[derive(Debug, PartialEq, Eq, Clone)]
struct P {
    id: String,
    producer: String,
    consumer: String,
    category: String,
    fields: BTreeMap<String, Vec<String>>,
    stoplisted: BTreeMap<String, Vec<String>>,
    evidence: Vec<String>,
}

#[derive(Debug)]
struct Out {
    proposals: Vec<P>,
    abstain: Vec<String>,
    inventory: Vec<String>,
    summary: String,
    raw: String,
}

fn parse(m: &str) -> Out {
    let mut proposals = Vec::new();
    let mut abstain = Vec::new();
    let mut inventory = Vec::new();
    let mut cur: Option<P> = None;
    for l in m.lines() {
        if let Some(id) = l.strip_prefix("# PROPOSED ") {
            cur = Some(P {
                id: id.to_string(),
                producer: String::new(),
                consumer: String::new(),
                category: String::new(),
                fields: BTreeMap::new(),
                stoplisted: BTreeMap::new(),
                evidence: Vec::new(),
            });
            continue;
        }
        if let Some(p) = cur.as_mut() {
            if l.starts_with("# ") {
                p.evidence.push(l.to_string());
                if let Some(c) = l.strip_prefix("# category: ") {
                    p.category = c.to_string();
                }
                if let Some(rest) = l.strip_prefix("# E5 ") {
                    let (shape, tail) = rest.split_once(' ').unwrap();
                    if let Some(list) = tail.strip_prefix("fields read: ") {
                        let names = list.split(", ").map(|x| x.split(" (").next().unwrap().to_string()).collect();
                        p.fields.insert(shape.to_string(), names);
                    } else if let Some(list) = tail.strip_prefix("stoplisted reads: ") {
                        let names: Vec<String> =
                            if list == "(none)" { Vec::new() } else { list.split(", ").map(str::to_string).collect() };
                        p.stoplisted.insert(shape.to_string(), names);
                    }
                }
                continue;
            }
            if let Some(v) = l.strip_prefix("producer = ") {
                p.producer = v.trim_matches('"').to_string();
            }
            if let Some(v) = l.strip_prefix("consumer = ") {
                p.consumer = v.trim_matches('"').to_string();
                proposals.push(cur.take().unwrap());
            }
            continue;
        }
        if let Some(a) = l.strip_prefix("# ABSTAIN ") {
            abstain.push(format!("ABSTAIN {a}"));
        }
        if let Some(i) = l.strip_prefix("# INVENTORY ") {
            inventory.push(format!("INVENTORY {i}"));
        }
    }
    let summary = m.lines().last().unwrap_or("").trim_start_matches("# ").to_string();
    // Every [[invariant]] block is a proposal with complete evidence.
    assert_eq!(m.matches("[[invariant]]").count() - m.matches("# [[invariant]]").count(), proposals.len(), "{m}");
    for p in &proposals {
        for need in ["# E1 ", "# E2 ", "# E3 ", "# E4 ", "# E5 ", "# E6 ", "# lock: run `invar lock` before `invar check`"] {
            assert!(p.evidence.iter().any(|e| e.starts_with(need)), "{} lacks {need}: {m}", p.id);
        }
        assert!(p.evidence.iter().any(|e| e == STATUS), "{} lacks the status line", p.id);
    }
    Out { proposals, abstain, inventory, summary, raw: m.to_string() }
}

/// Run `invar init --stdout` twice; outputs must be byte-identical (D01).
fn init(dir: &Path) -> Out {
    let bin = env!("CARGO_BIN_EXE_invar");
    let run = || {
        let o = Command::new(bin).args(["init", "--stdout", "--root"]).arg(dir).output().unwrap();
        assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8(o.stdout).unwrap()
    };
    let first = run();
    let second = run();
    assert_eq!(first, second, "D01: init output is not deterministic");
    parse(&first)
}

fn prop(id: &str, producer: &str, consumer: &str, category: &str, shapes: &[(&str, &[&str], &[&str])]) -> (String, String, String, String, BTreeMap<String, Vec<String>>, BTreeMap<String, Vec<String>>) {
    let mut f = BTreeMap::new();
    let mut s = BTreeMap::new();
    for (shape, fields, stop) in shapes {
        f.insert(shape.to_string(), fields.iter().map(|x| x.to_string()).collect());
        s.insert(shape.to_string(), stop.iter().map(|x| x.to_string()).collect());
    }
    (id.into(), producer.into(), consumer.into(), category.into(), f, s)
}

type Exp = (String, String, String, String, BTreeMap<String, Vec<String>>, BTreeMap<String, Vec<String>>);

fn expect(o: &Out, proposals: &[Exp], abstain: &[&str], inventory: &[&str], summary: &str) {
    let got: Vec<Exp> = o
        .proposals
        .iter()
        .map(|p| (p.id.clone(), p.producer.clone(), p.consumer.clone(), p.category.clone(), p.fields.clone(), p.stoplisted.clone()))
        .collect();
    assert_eq!(got, proposals.to_vec(), "proposals\n{}", o.raw);
    assert_eq!(o.abstain, abstain.iter().map(|s| s.to_string()).collect::<Vec<_>>(), "abstentions\n{}", o.raw);
    assert_eq!(o.inventory, inventory.iter().map(|s| s.to_string()).collect::<Vec<_>>(), "inventory\n{}", o.raw);
    assert_eq!(o.summary, summary, "summary\n{}", o.raw);
}

fn order_pair(consumer: &str, category: &str) -> Exp {
    let base = consumer.rsplit('/').next().unwrap();
    prop(&format!("code:order.ts->{base}"), "a/order.ts", consumer, category, &[("Order", &["currency_code", "total_minor"], &[])])
}

// ------------------------------------------------------------- positives

#[test]
fn f02_basic_pair() {
    let d = ws("f02", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER), ("b/src/use.ts", USE_ORDER)]);
    let o = init(&d);
    expect(&o, &[order_pair("b/src/use.ts", "production")], &[], &[], "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE");
    // Exact evidence for the representative case.
    assert_eq!(
        o.proposals[0].evidence,
        vec![
            STATUS.to_string(),
            "# E1 producer package a at a/package.json (package.json \"workspaces\" entry \"a\")".into(),
            "# E2 Order declared at a/order.ts:1".into(),
            "# E3 consumer package b at b/package.json (package.json \"workspaces\" entry \"b\")".into(),
            "# E4 Order imported at b/src/use.ts:1 from \"a/order\", resolved to a/order.ts".into(),
            "# E5 Order fields read: currency_code (b/src/use.ts:3), total_minor (b/src/use.ts:3)".into(),
            "# E5 Order stoplisted reads: (none)".into(),
            "# E6 pair a/order.ts -> b/src/use.ts".into(),
            "# category: production".into(),
            format!("# {}", invar_falsify::init::LOCK_NOTE),
        ]
    );
}

#[test]
fn f03_readonly_alias() {
    let order = "export type Order = Readonly<{\n  total_minor: number;\n  currency_code: string;\n  id: string;\n}>;\n";
    let d = ws("f03", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", order), ("b/src/use.ts", USE_ORDER)]);
    expect(&init(&d), &[order_pair("b/src/use.ts", "production")], &[], &[], "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn f04_one_barrel_level_keeps_declaring_file() {
    let d = ws(
        "f04",
        &[
            ROOT_AB,
            PKG_A,
            PKG_B,
            ("a/order.ts", ORDER),
            ("a/index.ts", "export * from './order';\n"),
            ("b/src/use.ts", "import { Order } from 'a';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n"),
        ],
    );
    let o = init(&d);
    expect(&o, &[order_pair("b/src/use.ts", "production")], &[], &[], "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE");
    assert!(o.proposals[0].evidence.contains(&"# E4 Order imported at b/src/use.ts:1 from \"a\", resolved to a/index.ts, re-exported by barrel a/index.ts:1 from a/order.ts".to_string()), "{}", o.raw);
}

#[test]
fn f05_test_consumer_labeled() {
    let d = ws("f05", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER), ("b/src/__tests__/order.test.ts", USE_ORDER)]);
    expect(&init(&d), &[order_pair("b/src/__tests__/order.test.ts", "test")], &[], &[], "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn f06_bare_import_through_main_index_ts() {
    let d = ws(
        "f06",
        &[
            ROOT_AB,
            ("a/package.json", r#"{"name":"a","main":"index.ts"}"#),
            PKG_B,
            ("a/index.ts", ORDER),
            ("b/src/use.ts", "import { Order } from 'a';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n"),
        ],
    );
    let p = prop("code:index.ts->use.ts", "a/index.ts", "b/src/use.ts", "production", &[("Order", &["currency_code", "total_minor"], &[])]);
    expect(&init(&d), &[p], &[], &[], "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn f07_js_specifier_maps_to_ts() {
    let d = ws(
        "f07",
        &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER), ("b/src/use.ts", "import type { Order } from '../../a/order.js';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n")],
    );
    expect(&init(&d), &[order_pair("b/src/use.ts", "production")], &[], &[], "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE");
}

// ------------------------------------------------------------- negatives

#[test]
fn n01_same_named_types_pair_by_import_not_name() {
    let d = ws(
        "n01",
        &[
            ("package.json", r#"{"workspaces":["a","b","c"]}"#),
            PKG_A,
            PKG_B,
            ("c/package.json", r#"{"name":"c"}"#),
            ("a/config.ts", "export interface Config {\n  alpha_mode: string;\n  beta_level: number;\n}\n"),
            ("c/config.ts", "export interface Config {\n  gamma_rate: number;\n  delta_scale: number;\n}\n"),
            ("b/src/x.ts", "import { Config } from 'a/config';\nexport const f = (k: Config, z: any) => [z.gamma_rate, z.delta_scale];\n"),
        ],
    );
    expect(&init(&d), &[], &[], &[], "SUMMARY packages=3 candidates=1 proposals=0 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn n02_overlap_without_import() {
    let d = ws("n02", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER), ("b/src/x.ts", "export const f = (z: any) => [z.total_minor, z.currency_code];\n")]);
    expect(&init(&d), &[], &[], &[], "SUMMARY packages=2 candidates=0 proposals=0 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn n03_one_distinctive_field() {
    let d = ws("n03", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER), ("b/src/x.ts", "import { Order } from 'a/order';\nexport const f = (o: Order) => [o.total_minor, o.id];\n")]);
    expect(&init(&d), &[], &[], &[], "SUMMARY packages=2 candidates=1 proposals=0 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn n04_stoplist_naming_form() {
    let d = ws(
        "n04",
        &[
            ROOT_AB,
            PKG_A,
            PKG_B,
            ("a/audit.ts", "export interface Audit {\n  createdAt: string;\n  updated_at: string;\n  amount_due: number;\n}\n"),
            ("b/src/x.ts", "import { Audit } from 'a/audit';\nexport const f = (a: Audit) => [a.createdAt, a.updated_at, a.amount_due];\n"),
        ],
    );
    expect(&init(&d), &[], &[], &[], "SUMMARY packages=2 candidates=1 proposals=0 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn n05_construction_is_not_reading() {
    let d = ws(
        "n05",
        &[
            ROOT_AB,
            PKG_A,
            PKG_B,
            ("a/order.ts", ORDER),
            ("b/src/x.ts", "import { Order } from 'a/order';\nexport const o: Order = {\n  total_minor: 1,\n  currency_code: 'x',\n  id: 'i',\n};\n"),
        ],
    );
    expect(&init(&d), &[], &[], &[], "SUMMARY packages=2 candidates=1 proposals=0 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn n06_nested_members_do_not_count() {
    let order = "export interface Order {\n  metadata: {\n    owner_ref: string;\n    region_code: string;\n  };\n  total_minor: number;\n}\n";
    let d = ws(
        "n06",
        &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", order), ("b/src/x.ts", "import { Order } from 'a/order';\nexport const f = (o: Order, m: any) => [m.owner_ref, m.region_code, o.total_minor];\n")],
    );
    expect(&init(&d), &[], &[], &[], "SUMMARY packages=2 candidates=1 proposals=0 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn n07_non_exported_type_is_not_a_shape() {
    let order = "interface Hidden {\n  h_one: string;\n  h_two: string;\n}\nexport interface Order {\n  total_minor: number;\n  currency_code: string;\n}\n";
    let d = ws(
        "n07",
        &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", order), ("b/src/x.ts", "import { Order } from 'a/order';\nexport const f = (o: Order, h: any) => [h.h_one, h.h_two];\n")],
    );
    expect(&init(&d), &[], &[], &[], "SUMMARY packages=2 candidates=1 proposals=0 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn n08_same_package_is_not_cross_package() {
    let d = ws(
        "n08",
        &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER), ("a/src/use.ts", "import { Order } from '../order';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n")],
    );
    expect(&init(&d), &[], &[], &[], "SUMMARY packages=2 candidates=0 proposals=0 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn n09_undeclared_directory_is_not_a_package() {
    let d = ws(
        "n09",
        &[
            ROOT_AB,
            PKG_A,
            PKG_B,
            ("c/package.json", r#"{"name":"c"}"#),
            ("c/order.ts", ORDER),
            ("b/src/x.ts", "import { Order } from 'c/order';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n"),
        ],
    );
    expect(&init(&d), &[], &[], &[], "SUMMARY packages=2 candidates=0 proposals=0 abstentions=0 inventory=0 status=COMPLETE");
}

#[test]
fn n10_receiver_not_verified_is_pinned() {
    let d = ws(
        "n10",
        &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER), ("b/src/x.ts", "import { Order } from 'a/order';\nexport const f = (o: Order, other: any) => [other.total_minor, other.currency_code];\n")],
    );
    let o = init(&d);
    expect(&o, &[order_pair("b/src/x.ts", "production")], &[], &[], "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE");
    assert!(o.proposals[0].evidence.contains(&STATUS.to_string()));
}

#[test]
fn n11_fields_never_pooled_across_shapes() {
    let d = ws(
        "n11",
        &[
            ROOT_AB,
            PKG_A,
            PKG_B,
            ("a/order.ts", "export interface Order {\n  total_minor: number;\n}\nexport interface Invoice {\n  due_date: string;\n}\n"),
            ("b/src/x.ts", "import { Order, Invoice } from 'a/order';\nexport const f = (o: Order, i: Invoice) => [o.total_minor, i.due_date];\n"),
        ],
    );
    expect(&init(&d), &[], &[], &[], "SUMMARY packages=2 candidates=2 proposals=0 abstentions=0 inventory=0 status=COMPLETE");
}

// ------------------------------------------------------- ambiguous, unsupported

#[test]
fn a01_indexed_alias_abstains_r5() {
    let d = ws(
        "a01",
        &[ROOT_AB, PKG_A, PKG_B, ("a/share.ts", "export type Share = components[\"schemas\"][\"Share\"];\n"), ("b/src/x.ts", "import { Share } from 'a/share';\nexport const f = (s: Share) => [s.counterparty_name, s.expires_at];\n")],
    );
    expect(
        &init(&d),
        &[],
        &["ABSTAIN R5 b/src/x.ts:1 Share resolves to an indexed alias into generated types at a/share.ts:1 (out of v1)"],
        &[],
        "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

#[test]
fn a02_ambiguous_shape_abstains() {
    let d = ws(
        "a02",
        &[
            ROOT_AB,
            PKG_A,
            PKG_B,
            ("a/index.ts", "export * from './one';\nexport * from './two';\n"),
            ("a/one.ts", ORDER),
            ("a/two.ts", ORDER),
            ("b/src/x.ts", "import { Order } from 'a';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n"),
        ],
    );
    expect(
        &init(&d),
        &[],
        &["ABSTAIN AMBIG-SHAPE b/src/x.ts:1 Order from \"a\" resolves to more than one declaration: a/one.ts, a/two.ts"],
        &[],
        "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

#[test]
fn u01_inferred_type_abstains_r6() {
    let run = "export function exec(c: string): Result {\n  return null as any;\n}\nexport interface Result {\n  stdout_text: string;\n  exit_code: number;\n}\n";
    let d = ws(
        "u01",
        &[ROOT_AB, PKG_A, PKG_B, ("a/run.ts", run), ("b/src/x.ts", "import { exec } from 'a/run';\nexport const f = () => {\n  const r = exec('x');\n  return [r.stdout_text, r.exit_code];\n};\n")],
    );
    expect(
        &init(&d),
        &[],
        &["ABSTAIN R6 b/src/x.ts:1 reads exit_code, stdout_text of Result (a/run.ts:4) without importing it by name (out of v1)"],
        &[],
        "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

#[test]
fn u02_barrel_depth_two_abstains() {
    let d = ws(
        "u02",
        &[
            ROOT_AB,
            PKG_A,
            PKG_B,
            ("a/index.ts", "export * from './models';\n"),
            ("a/models/index.ts", "export * from './order';\n"),
            ("a/models/order.ts", ORDER),
            ("b/src/x.ts", "import { Order } from 'a';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n"),
        ],
    );
    expect(
        &init(&d),
        &[],
        &["ABSTAIN BARREL2 b/src/x.ts:1 Order from \"a\" is re-exported through more than one barrel level (from a/index.ts)"],
        &[],
        "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

#[test]
fn u03_single_declared_package_abstains_h2() {
    let d = ws(
        "u03",
        &[
            ("package.json", r#"{"workspaces":["cli"]}"#),
            ("cli/package.json", r#"{"name":"cli"}"#),
            ("server/package.json", r#"{"name":"server"}"#),
            ("server/order.ts", ORDER),
            ("cli/src/x.ts", "import { Order } from 'server/order';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n"),
        ],
    );
    expect(
        &init(&d),
        &[],
        &["ABSTAIN H2 . 1 qualifying package(s) (cli); a multi-package workspace needs at least 2"],
        &[],
        "SUMMARY packages=1 candidates=0 proposals=0 abstentions=1 inventory=0 status=ABSTAINED",
    );
}

#[test]
fn u04_pnpm_settings_only_abstains_h2() {
    let d = ws("u04", &[("pnpm-workspace.yaml", "onlyBuiltDependencies:\n  - esbuild\n"), ("a/package.json", r#"{"name":"a"}"#)]);
    expect(
        &init(&d),
        &[],
        &["ABSTAIN H2 . 0 qualifying package(s); a multi-package workspace needs at least 2"],
        &[],
        "SUMMARY packages=0 candidates=0 proposals=0 abstentions=1 inventory=0 status=ABSTAINED",
    );
}

#[test]
fn u05_marker_outside_slice_abstains() {
    let d = ws("u05", &[("lerna.json", r#"{"packages":["a","b"]}"#), PKG_A, PKG_B]);
    expect(
        &init(&d),
        &[],
        &["ABSTAIN SLICE-SCOPE . workspace marker(s) lerna.json are outside slice 1 (package.json \"workspaces\" and pnpm-workspace.yaml only)"],
        &[],
        "SUMMARY packages=0 candidates=0 proposals=0 abstentions=1 inventory=0 status=ABSTAINED",
    );
}

#[test]
fn u06_exports_map_unresolved() {
    let d = ws(
        "u06",
        &[ROOT_AB, ("a/package.json", r#"{"name":"a","exports":{"./order":"./order.ts"}}"#), PKG_B, ("a/order.ts", ORDER), ("b/src/x.ts", USE_ORDER)],
    );
    expect(
        &init(&d),
        &[],
        &["ABSTAIN UNRESOLVED b/src/x.ts:1 Order from \"a/order\": package a declares \"exports\"; not supported in slice 1"],
        &[],
        "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

#[test]
fn u07_zod_inferred_alias_unsupported() {
    let order = "import { z } from 'zod';\nexport const OrderSchema = z.object({ total_minor: z.number(), currency_code: z.string() });\nexport type Order = z.infer<typeof OrderSchema>;\n";
    let d = ws("u07", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", order), ("b/src/x.ts", USE_ORDER)]);
    expect(
        &init(&d),
        &[],
        &["ABSTAIN UNSUPPORTED-SHAPE b/src/x.ts:1 Order at a/order.ts:3 is not an object type (z.infer<typeof OrderSchema>)"],
        &[],
        "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

#[test]
fn u08_dist_entry_is_not_guessed() {
    let d = ws(
        "u08",
        &[
            ROOT_AB,
            ("a/package.json", r#"{"name":"a","main":"dist/index.js"}"#),
            PKG_B,
            ("a/src/index.ts", ORDER),
            ("a/index.ts", ORDER),
            ("b/src/x.ts", "import { Order } from 'a';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n"),
        ],
    );
    expect(
        &init(&d),
        &[],
        &["ABSTAIN UNRESOLVED b/src/x.ts:1 Order from \"a\": entry \"dist/index.js\" is not a TypeScript source file; not guessed"],
        &[],
        "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

#[test]
fn u09_member_under_skipped_directory() {
    let d = ws(
        "u09",
        &[
            ("package.json", r#"{"workspaces":["a","b","build/c"]}"#),
            PKG_A,
            PKG_B,
            ("build/c/package.json", r#"{"name":"c"}"#),
            ("build/c/order.ts", ORDER),
            ("b/src/x.ts", "import { Order } from 'c/order';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n"),
        ],
    );
    expect(
        &init(&d),
        &[],
        &["ABSTAIN H2-MEMBER build/c declared member is inside a skipped directory (build); its files are not scanned"],
        &[],
        "SUMMARY packages=2 candidates=0 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

// ------------------------------------------------------------ malformed

#[test]
fn m01_unparsable_marker_abstains_not_crashes() {
    let d = ws("m01", &[("package.json", "{ \"workspaces\": ["), PKG_A, PKG_B]);
    expect(
        &init(&d),
        &[],
        &["ABSTAIN H2 . unparsable marker package.json (EOF while parsing a list at line 1 column 17)"],
        &[],
        "SUMMARY packages=0 candidates=0 proposals=0 abstentions=1 inventory=0 status=ABSTAINED",
    );
}

#[test]
fn m02_missing_producer_file_unresolved() {
    let d = ws("m02", &[ROOT_AB, PKG_A, PKG_B, ("a/other.ts", ORDER), ("b/src/x.ts", USE_ORDER)]);
    expect(
        &init(&d),
        &[],
        &["ABSTAIN UNRESOLVED b/src/x.ts:1 Order from \"a/order\": no TypeScript file for \"a/order\""],
        &[],
        "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

#[test]
fn m03_unreadable_source_abstains() {
    let d = ws("m03", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER)]);
    put(&d, "b/src/x.ts", &[0x69, 0x6d, 0xff, 0xfe, 0x0a]);
    expect(
        &init(&d),
        &[],
        &["ABSTAIN UNREADABLE b/src/x.ts not valid UTF-8 or not readable; not scanned"],
        &[],
        "SUMMARY packages=2 candidates=0 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

#[test]
fn m04_unrepresentable_pair_is_abstained_not_inventoried() {
    let d = ws(
        "m04",
        &[ROOT_AB, PKG_A, PKG_B, ("a/or\"der.ts", ORDER), ("b/src/x.ts", "import { Order } from 'a/or\"der';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n")],
    );
    expect(
        &init(&d),
        &[],
        &["ABSTAIN UNREPRESENTABLE a/or\"der.ts -> b/src/x.ts path not representable in flat TOML; not proposed"],
        &[],
        "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

#[test]
fn m05_oversized_consumer_abstains() {
    let d = ws("m05", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER)]);
    let mut big = USE_ORDER.as_bytes().to_vec();
    big.extend(std::iter::repeat(b' ').take(1_000_001));
    put(&d, "b/src/big.ts", &big);
    expect(
        &init(&d),
        &[],
        &["ABSTAIN UNREADABLE b/src/big.ts larger than the 1000000-byte scan limit; not scanned"],
        &[],
        "SUMMARY packages=2 candidates=0 proposals=0 abstentions=1 inventory=0 status=COMPLETE",
    );
}

// ------------------------------------------------------- inventory, limits

#[test]
fn i01_schema_files_are_inventory_only() {
    let schema = r#"{"type":"object","properties":{"alpha_one":{"type":"string"},"beta_two":{"type":"string"},"gamma_three":{"type":"string"}}}"#;
    let d = ws(
        "i01",
        &[
            ROOT_AB,
            PKG_A,
            PKG_B,
            ("a/x_builder/schema.json", schema),
            ("a/y_builder/schema.json", schema),
            ("b/z_builder/schema.json", schema),
            ("b/src/c.js", "export const f = (payload) => [payload.alpha_one, payload.beta_two, payload.gamma_three];\n"),
        ],
    );
    expect(
        &init(&d),
        &[],
        &[],
        &[
            "INVENTORY schema a/x_builder/schema.json  schema file; inventory only (H1), never proposed",
            "INVENTORY schema a/y_builder/schema.json  schema file; inventory only (H1), never proposed",
            "INVENTORY schema b/z_builder/schema.json  schema file; inventory only (H1), never proposed",
        ],
        "SUMMARY packages=2 candidates=0 proposals=0 abstentions=0 inventory=3 status=COMPLETE",
    );
}

#[test]
fn t01_truncation_is_reported() {
    let d = ws("t01", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER), ("b/src/use.ts", USE_ORDER)]);
    std::fs::create_dir_all(d.join("z")).unwrap();
    for i in 0..20_001 {
        std::fs::write(d.join(format!("z/f{i:05}.txt")), b"x").unwrap();
    }
    expect(&init(&d), &[order_pair("b/src/use.ts", "production")], &[], &[], "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=TRUNCATED");
}

#[test]
fn c01_category_precedence_and_id_collisions() {
    let d = ws(
        "c01",
        &[
            ROOT_AB,
            PKG_A,
            PKG_B,
            ("a/order.ts", ORDER),
            ("b/src/tests/x.ts", USE_ORDER),
            ("b/examples/src/x.ts", USE_ORDER),
            ("b/scripts/x.ts", USE_ORDER),
            ("b/x.ts", USE_ORDER),
        ],
    );
    let mk = |id: &str, c: &str, cat: &str| prop(id, "a/order.ts", c, cat, &[("Order", &["currency_code", "total_minor"], &[])]);
    expect(
        &init(&d),
        &[
            mk("code:order.ts->x.ts", "b/examples/src/x.ts", "example"),
            mk("code:order.ts->x.ts#2", "b/scripts/x.ts", "tooling"),
            mk("code:order.ts->x.ts#3", "b/src/tests/x.ts", "test"),
            mk("code:order.ts->x.ts#4", "b/x.ts", "unknown"),
        ],
        &[],
        &[],
        "SUMMARY packages=2 candidates=4 proposals=4 abstentions=0 inventory=0 status=COMPLETE",
    );
}

// ------------------------------------------------- 86428 correction passes

const CM_ORDER: &str = "export interface Order {\n    total_minor: number;\n    currency_code: string;\n    zoom: { render_mode: string } | { render_mode: string; crop_box: { x_off: number } };\n}\nexport type Reply = { render_flag: boolean; body_text: string };\n";
const CM_USE: &str = "import { Order } from 'a/order';\nexport function f(o: Order, z: any) {\n    const flag = z.render_mode === 'fast';\n    return { render_flag: flag, total: o.total_minor, code: o.currency_code };\n}\n";

// U10 (Pass 1, defect C): a pnpm flow list is outside the slice 1 contract;
// membership must abstain, never parse it into packages.
#[test]
fn u10_pnpm_flow_list_abstains() {
    let d = ws(
        "u10",
        &[
            ("pnpm-workspace.yaml", "packages: [a, b]\n"),
            PKG_A,
            PKG_B,
            ("a/order.ts", ORDER),
            ("b/src/x.ts", USE_ORDER),
        ],
    );
    expect(
        &init(&d),
        &[],
        &["ABSTAIN H2 . unsupported marker pnpm-workspace.yaml (packages is not a block list)"],
        &[],
        "SUMMARY packages=0 candidates=0 proposals=0 abstentions=1 inventory=0 status=ABSTAINED",
    );
}

// L01 (Pass 2, defect A): every locked binding of a slice 1 contract is one
// of the proposal's E5 fields, scoped to its shape. The countermodel offers
// a nested member (render_mode, read on an unrelated value) and a neighboring
// type (Reply.render_flag, constructed in the consumer); neither may bind.
#[test]
fn l01_lock_bindings_are_evidence_bounded() {
    let d = ws("l01", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", CM_ORDER), ("b/src/x.ts", CM_USE)]);
    let bin = env!("CARGO_BIN_EXE_invar");
    let run = |args: &[&str]| std::process::Command::new(bin).args(args).current_dir(&d).output().unwrap();
    assert_eq!(run(&["init"]).status.code(), Some(0));
    let toml = std::fs::read_to_string(d.join("invar.toml")).unwrap();
    assert!(toml.contains("evidence = \"Order currency_code total_minor\""), "{toml}");
    assert_eq!(run(&["lock"]).status.code(), Some(0));
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    let binds: Vec<&str> = lock
        .lines()
        .filter_map(|l| l.trim().strip_prefix("bind = \""))
        .map(|l| l.trim_end_matches('"'))
        .collect();
    let fields: Vec<&str> = binds.iter().map(|b| b.split(' ').nth(2).unwrap_or("")).collect();
    assert_eq!(
        fields,
        vec!["Order.currency_code", "Order.total_minor"],
        "bindings must equal the evidence, scoped: {lock}"
    );
    let _ = std::fs::remove_dir_all(&d);
}

// B01 (Pass 3, defect B): without a lock, a slice 1 contract is reported as
// not evaluated (exit 4), never as a vacuous HOLDS; after `invar lock` the
// same check holds with exit 0.
#[test]
fn b01_unlocked_check_is_not_evaluated() {
    let d = ws("b01", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER), ("b/src/use.ts", USE_ORDER)]);
    let bin = env!("CARGO_BIN_EXE_invar");
    let run = |args: &[&str]| std::process::Command::new(bin).args(args).current_dir(&d).output().unwrap();
    assert_eq!(run(&["init"]).status.code(), Some(0));
    let check = run(&["check"]);
    let out = String::from_utf8_lossy(&check.stdout).to_string();
    assert_eq!(check.status.code(), Some(4), "no lock must mean not evaluated, never HOLDS:\n{out}");
    assert!(out.contains("payload.<field>") && out.contains("invar lock"), "{out}");
    assert!(!out.contains("[HOLDS]"), "{out}");
    assert_eq!(run(&["lock"]).status.code(), Some(0));
    let again = run(&["check"]);
    assert_eq!(again.status.code(), Some(0), "{}", String::from_utf8_lossy(&again.stdout));
    let _ = std::fs::remove_dir_all(&d);
}

// --------------------------------------------------------------- run error

#[test]
fn e01_run_error_keeps_existing_behavior() {
    let d = ws("e01", &[ROOT_AB, PKG_A, PKG_B, ("invar.toml", "# mine\n")]);
    let bin = env!("CARGO_BIN_EXE_invar");
    let o = Command::new(bin).args(["init", "--root"]).arg(&d).output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stderr).starts_with("error: "));
    assert_eq!(std::fs::read_to_string(d.join("invar.toml")).unwrap(), "# mine\n", "no partial or replaced file");
}
