//! INIT-DISCOVERY slice 2b REV1 fixtures (docs/architecture/INIT-DISCOVERY-SLICE2B-REV1.md,
//! RATIFIED 2026-10-15 20:05, section E). Written BEFORE any engine edit. Synthetic
//! repositories in temporary directories, run through the real binary (`invar init
//! --stdout` twice, byte-identical; lock and check by exit code).
//!
//! A (TSC-GOVERNED): a1..a11. B (test-support categories): b1..b7. C (same-file
//! collapse): c1..c8. M: m2 determinism, m3 lock grammar, m4 SUMMARY counts (M1, the
//! unchanged 2b and slice 2 suites, is the existing test files). meta_*: the
//! metamorphic invariance suite ruled for Gate 4 (CRLF, BOM, trailing comments, file
//! order): output must be byte-identical to the base.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s2b_{tag}_{}_{nanos}", std::process::id()));
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

#[derive(Debug, Clone)]
struct B {
    id: String,
    kind: String,
    keys: BTreeMap<String, String>,
    evidence: Vec<String>,
}

#[derive(Debug)]
struct Out {
    blocks: Vec<B>,
    abstain: Vec<String>,
    raw: String,
}

const STRINGS_STATUS: &str = "# status: PROPOSED (heuristic; string list in a query position overlaps one shape; runtime store binding not verified)";

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
            cur = Some(B { id, kind: String::new(), keys: BTreeMap::new(), evidence });
            continue;
        }
        if let Some(b) = cur.as_mut() {
            if l.trim().is_empty() {
                blocks.push(cur.take().unwrap());
            } else if let Some((k, v)) = l.split_once(" = ") {
                let v = v.trim_matches('"').to_string();
                if k == "kind" {
                    b.kind = v;
                } else {
                    b.keys.insert(k.to_string(), v);
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
    for b in &blocks {
        assert_eq!(b.kind, "strings", "unexpected kind in {m}");
        assert!(b.evidence.iter().any(|e| e == STRINGS_STATUS), "{} lacks the status line", b.id);
        for need in ["# S1 ", "# S2 ", "# S3 ", "# S4 ", "# S5 ", "# S6 DISCLAIMER: that this list reaches the store", "# category: ", "# lock: run `invar lock` before `invar check`. Without a lock"] {
            assert!(b.evidence.iter().any(|e| e.starts_with(need)), "{} lacks {need}", b.id);
        }
        for k in ["shape", "list", "listat", "scope", "overlap"] {
            assert!(b.keys.contains_key(k), "{} lacks the {k} key", b.id);
        }
        assert!(!b.keys.contains_key("consumer"), "{}: a strings block carries no consumer", b.id);
    }
    Out { blocks, abstain, raw: m.to_string() }
}

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_invar")
}

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

fn adopt(dir: &Path) {
    let (c, o) = invar(dir, &["init", "--force"]);
    assert_eq!(c, 0, "{o}");
}

/// Abstentions other than slice 1's own H2 line.
fn k3(o: &Out) -> Vec<String> {
    o.abstain.iter().filter(|a| !a.starts_with("ABSTAIN H2 ")).cloned().collect()
}

const PKG: &str = "{\"name\":\"app\",\"private\":true}\n";

const TOKEN_ATTRS: &str = "module.exports = {\n  attributes: {\n    name: { type: 'string' },\n    description: { type: 'text' },\n    accessKey: { type: 'string' },\n    lastUsed: { type: 'datetime' },\n    permissions: { type: 'json' },\n    expiresAt: { type: 'datetime' },\n    owner: { type: 'string' },\n  },\n};\n";

const SERVICE: &str = "const SELECT_FIELDS = ['id', 'name', 'description', 'lastUsed', 'accessKey', 'permissions', 'expiresAt', 'type'];\nexport function list(db) { return db.query({ select: SELECT_FIELDS }); }\n";

fn repo(tag: &str, shape_file: &str, shape: &str, service: &str) -> PathBuf {
    ws(tag, &[("package.json", PKG), (shape_file, shape), ("src/service.ts", service)])
}

fn only(o: &Out) -> &B {
    assert_eq!(o.blocks.len(), 1, "{}", o.raw);
    &o.blocks[0]
}

// ------------------------------------------------------------- helpers

fn init_raw(dir: &Path) -> String {
    let o = Command::new(bin()).args(["init", "--stdout", "--root"]).arg(dir).output().unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout).unwrap()
}

fn tsc(o: &Out) -> Vec<String> {
    o.abstain.iter().filter(|a| a.starts_with("ABSTAIN TSC-GOVERNED ")).cloned().collect()
}

fn category_of(b: &B) -> String {
    b.evidence.iter().find_map(|e| e.strip_prefix("# category: ")).unwrap().to_string()
}

fn s7(b: &B) -> Option<&String> {
    b.evidence.iter().find(|e| e.starts_with("# S7 "))
}

/// A service file with one qualifying query list (6 entries, 4 distinctive names
/// plus `id` and `name`, which the stoplist removes).
fn svc(ann: &str, tail: &str) -> String {
    format!(
        "const SELECT_FIELDS{ann} = ['id', 'name', 'lastUsed', 'accessKey', 'permissions', 'expiresAt']{tail};\nexport function list(db) {{ return db.query({{ select: SELECT_FIELDS }}); }}\n"
    )
}

// ------------------------------------------------- A: TSC-GOVERNED (R-R1-1, R-R1-2)

#[test]
fn r1_a1_trailing_satisfies_keyof_array_abstains_tsc_governed() {
    let d = repo("r1a1", "src/token.ts", TOKEN_ATTRS, &svc("", " as const satisfies readonly (keyof Token)[]"));
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = k3(&o);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].starts_with("ABSTAIN TSC-GOVERNED src/service.ts:1 "), "{a:?}");
    assert!(a[0].contains("satisfies readonly (keyof Token)[]"), "{a:?}");
    assert!(a[0].contains("the compiler checks its names against Token, so a lock adds nothing; not proposed"), "{a:?}");
    assert!(o.raw.contains("proposals=0"), "{}", o.raw);
}

#[test]
fn r1_a2_annotation_keyof_array_abstains() {
    let d = repo("r1a2", "src/token.ts", TOKEN_ATTRS, &svc(": (keyof Token)[]", ""));
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = tsc(&o);
    assert_eq!(a.len(), 1, "{:?}", o.abstain);
    assert!(a[0].contains("annotation (keyof Token)[]"), "{a:?}");
}

#[test]
fn r1_a3_array_and_readonlyarray_generics_abstain_two_lists() {
    let svc2 = "const SELECT_FIELDS: ReadonlyArray<keyof Token> = ['lastUsed', 'accessKey', 'permissions', 'expiresAt'];\nconst PICK_FIELDS: Array<keyof Token> = ['lastUsed', 'accessKey', 'permissions', 'owner'];\n";
    let d = repo("r1a3", "src/token.ts", TOKEN_ATTRS, svc2);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = tsc(&o);
    assert_eq!(a.len(), 2, "{:?}", o.abstain);
    assert!(a[0].starts_with("ABSTAIN TSC-GOVERNED src/service.ts:1 ") && a[0].contains("annotation ReadonlyArray<keyof Token>"), "{a:?}");
    assert!(a[1].starts_with("ABSTAIN TSC-GOVERNED src/service.ts:2 ") && a[1].contains("annotation Array<keyof Token>"), "{a:?}");
}

#[test]
fn r1_a4_keyof_typeof_object_abstains() {
    let d = repo("r1a4", "src/token.ts", TOKEN_ATTRS, &svc("", " satisfies readonly (keyof typeof TOKEN_MAP)[]"));
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = tsc(&o);
    assert_eq!(a.len(), 1, "{:?}", o.abstain);
    assert!(a[0].contains("keyof typeof TOKEN_MAP"), "{a:?}");
}

#[test]
fn r1_a5_keyof_of_an_unrelated_type_still_abstains_conservatively() {
    // CM-R1-1: the constraint names Other, not the shape the names overlap. Abstaining
    // is the safe direction (RD2); the reason names the type so it can be audited.
    let d = repo("r1a5", "src/token.ts", TOKEN_ATTRS, &svc("", " satisfies readonly (keyof Other)[]"));
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = tsc(&o);
    assert_eq!(a.len(), 1, "{:?}", o.abstain);
    assert!(a[0].contains("(keyof Other)") && a[0].contains("against Other"), "{a:?}");
}

#[test]
fn r1_a6_a_locked_contract_whose_list_gains_keyof_is_still_checked() {
    let d = repo("r1a6", "src/token.ts", TOKEN_ATTRS, &svc("", ""));
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    put(&d, "src/service.ts", &svc("", " as const satisfies readonly (keyof Token)[]"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("[HOLDS]"), "{out}");
    // A stale locked name is still VIOLATED: check semantics are unchanged.
    put(&d, "src/token.ts", &TOKEN_ATTRS.replace("lastUsed:", "lastUsedAt:"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 1, "{out}");
    assert!(out.contains("[VIOLATED]"), "{out}");
}

#[test]
fn r1_a7_satisfies_a_plain_string_array_is_not_governed() {
    let d = repo("r1a7", "src/token.ts", TOKEN_ATTRS, &svc("", " as const satisfies readonly string[]"));
    let o = init(&d);
    assert_eq!(only(&o).keys["listat"], "const:SELECT_FIELDS");
    assert!(tsc(&o).is_empty(), "{:?}", o.abstain);
}

#[test]
fn r1_a8_a_keyof_cast_is_an_assertion_not_a_check() {
    let d = repo("r1a8", "src/token.ts", TOKEN_ATTRS, &svc("", " as (keyof Token)[]"));
    let o = init(&d);
    assert_eq!(only(&o).keys["listat"], "const:SELECT_FIELDS");
    assert!(tsc(&o).is_empty(), "{:?}", o.abstain);
}

#[test]
fn r1_a9_comment_string_and_next_statement_do_not_govern() {
    // CM-R1-4 (comment and string text) and CM-R1-5 (a later statement's satisfies).
    let s = "const SELECT_FIELDS = ['id', 'name', 'lastUsed', 'accessKey', 'permissions', 'expiresAt'] // satisfies readonly (keyof Token)[]\nconst NOTE = \"as const satisfies readonly (keyof Token)[]\"\nconst CHECK = [] satisfies readonly (keyof Token)[];\nexport function list(db) { return db.query({ select: SELECT_FIELDS }); }\n";
    let d = repo("r1a9", "src/token.ts", TOKEN_ATTRS, s);
    let o = init(&d);
    assert_eq!(only(&o).keys["listat"], "const:SELECT_FIELDS");
    assert!(tsc(&o).is_empty(), "{:?}", o.abstain);
}

#[test]
fn r1_a10_a_governed_list_below_the_bar_keeps_its_own_abstention_code() {
    // R-R1-2: TSC-GOVERNED is decided last and only for would-be proposals.
    let s = "const SELECT_FIELDS = ['id', 'name', 'lastUsed', 'accessKey'] as const satisfies readonly (keyof Token)[];\nexport function list(db) { return db.query({ select: SELECT_FIELDS }); }\n";
    let d = repo("r1a10", "src/token.ts", TOKEN_ATTRS, s);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = k3(&o);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].starts_with("ABSTAIN NON-DISTINCTIVE-KEY src/service.ts:1 "), "{a:?}");
}

#[test]
fn r1_a11_trailing_form_on_a_call_site_list() {
    let s = "export function list(db) { return db.query({ select: ['id', 'lastUsed', 'accessKey', 'permissions', 'expiresAt'] satisfies readonly (keyof Token)[] }); }\n";
    let d = repo("r1a11", "src/token.ts", TOKEN_ATTRS, s);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = tsc(&o);
    assert_eq!(a.len(), 1, "{:?}", o.abstain);
    assert!(a[0].contains("list prop:select#1 ") && a[0].contains("satisfies readonly (keyof Token)[]"), "{a:?}");
}

// ----------------------------------------------- B: test-support categories (R-R1-3)

fn cat_case(tag: &str, list_path: &str) -> String {
    let d = ws(tag, &[("package.json", PKG), ("src/token.ts", TOKEN_ATTRS), (list_path, SERVICE)]);
    let o = init(&d);
    category_of(only(&o))
}

#[test]
fn r1_b1_testutils_directory_is_test() {
    assert_eq!(cat_case("r1b1", "src/testUtils/service.ts"), "test");
}

#[test]
fn r1_b2_mocks_directory_is_test() {
    assert_eq!(cat_case("r1b2", "src/__mocks__/service.ts"), "test");
}

#[test]
fn r1_b3_fixtures_directory_is_test() {
    assert_eq!(cat_case("r1b3", "src/fixtures/service.ts"), "test");
}

#[test]
fn r1_b4_nested_forms_are_test_not_production() {
    // `/src/` made these production before; the test-support segment now wins.
    assert_eq!(cat_case("r1b4a", "packages/a/src/testUtils/service.ts"), "test");
    assert_eq!(cat_case("r1b4b", "src/deep/fixtures/seed.ts"), "test");
}

#[test]
fn r1_b5_near_names_are_not_matched() {
    // CM-R1-7 and CM-R1-8: exact, case-sensitive segment equality only.
    for (i, p) in ["src/mytestUtils/service.ts", "src/testUtilsExtra/service.ts", "src/fixture/service.ts", "src/test-utils/service.ts", "src/__fixtures__/service.ts", "src/fixtures.ts"].iter().enumerate() {
        assert_eq!(cat_case(&format!("r1b5_{i}"), p), "production", "{p}");
    }
}

#[test]
fn r1_b6_a_k2_proposal_under_fixtures_changes_only_its_label() {
    // RD1: category() is shared. The proposal set must be identical; only the label moves.
    let order_in = "import { z } from \"zod\";\nexport const OrderIn = z.object({\n  total_minor: z.number(),\n  currency_code: z.string(),\n  \"ship-to\": z.string(),\n  unread_flag: z.boolean(),\n});\n";
    let consumer = |p: &str| {
        let d = ws("r1b6", &[("lib/order.ts", order_in), (p, "import { OrderIn } from \"../../lib/order.js\";\nexport function f(row) {\n  return [row.total_minor, row.currency_code, row['ship-to']];\n}\n")]);
        init_raw(&d)
    };
    let prod = consumer("src/live/consume.js");
    let fix = consumer("src/fixtures/consume.js");
    let blocks = |s: &str| s.lines().filter(|l| *l == "[[invariant]]").count();
    assert_eq!(blocks(&prod), 1, "{prod}");
    assert_eq!(blocks(&fix), 1, "{fix}");
    assert!(prod.contains("# category: production") && fix.contains("# category: test"), "{prod}\n{fix}");
    let norm = |s: &str| s.replace("src/live/", "src/X/").replace("src/fixtures/", "src/X/").replace("# category: production", "# category: C").replace("# category: test", "# category: C");
    assert_eq!(norm(&prod), norm(&fix), "only the path and the label may differ");
}

#[test]
fn r1_b7_a_test_labeled_proposal_is_still_emitted() {
    let d = ws("r1b7", &[("package.json", PKG), ("src/token.ts", TOKEN_ATTRS), ("src/service.ts", SERVICE), ("src/testUtils/service.ts", SERVICE)]);
    let o = init(&d);
    let mut cats: Vec<String> = o.blocks.iter().map(category_of).collect();
    cats.sort();
    assert_eq!(cats, vec!["production", "test"], "{}", o.raw);
}

// ------------------------------------------------------- C: collapse (R-R1-4)

const SIX: &str = "export const a = (u) => pick(u, { fieldsToPick: ['accessKey', 'lastUsed', 'permissions'] });\nexport const b = (u) => pick(u, { fieldsToPick: ['accessKey', 'expiresAt', 'lastUsed', 'permissions'] });\nexport const c = (u) => pick(u, { fieldsToPick: ['accessKey', 'expiresAt', 'lastUsed', 'permissions', 'owner'] });\nexport const d = (u) => pick(u, { fieldsToPick: ['lastUsed', 'permissions', 'owner'] });\nexport const e = (u) => pick(u, { fieldsToPick: ['accessKey', 'expiresAt', 'owner'] });\nexport const f = (u) => pick(u, { fieldsToPick: ['accessKey', 'lastUsed', 'permissions', 'id', 'name'] });\n";

#[test]
fn r1_c1_six_same_shape_lists_in_one_file_collapse_to_one_proposal() {
    let d = repo("r1c1", "src/token.ts", TOKEN_ATTRS, SIX);
    let o = init(&d);
    let b = only(&o);
    assert_eq!(b.keys["listat"], "prop:fieldsToPick#3");
    assert_eq!(b.keys["overlap"], "accessKey expiresAt lastUsed owner permissions");
    assert_eq!(
        s7(b).map(String::as_str),
        Some("# S7 also: prop:fieldsToPick#1@1 |O|=3; prop:fieldsToPick#2@2 |O|=4; prop:fieldsToPick#4@4 |O|=3; prop:fieldsToPick#5@5 |O|=3; prop:fieldsToPick#6@6 |O|=3 (other sites are not locked; add a block by hand to lock them)")
    );
    assert!(k3(&o).is_empty(), "{:?}", k3(&o));
}

#[test]
fn r1_c2_largest_overlap_wins_and_ties_go_to_the_lowest_line() {
    // CM-R1-12: the larger list is later in the file.
    let later = "export const a = (u) => pick(u, { fieldsToPick: ['accessKey', 'lastUsed', 'permissions'] });\nexport const b = (u) => pick(u, { fieldsToPick: ['accessKey', 'expiresAt', 'lastUsed', 'permissions'] });\n";
    let o = init(&repo("r1c2a", "src/token.ts", TOKEN_ATTRS, later));
    assert_eq!(only(&o).keys["listat"], "prop:fieldsToPick#2");
    // Equal overlap: the lowest line is the representative.
    let tie = "export const a = (u) => pick(u, { fieldsToPick: ['accessKey', 'expiresAt', 'lastUsed', 'permissions'] });\nexport const b = (u) => pick(u, { fieldsToPick: ['permissions', 'lastUsed', 'expiresAt', 'accessKey'] });\n";
    let o = init(&repo("r1c2b", "src/token.ts", TOKEN_ATTRS, tie));
    let b = only(&o);
    assert_eq!(b.keys["listat"], "prop:fieldsToPick#1");
    assert!(s7(b).unwrap().starts_with("# S7 also: prop:fieldsToPick#2@2 |O|=4 "), "{:?}", s7(b));
}

const INVOICE_ZOD: &str = "import { z } from \"zod\";\nexport const ApiToken = z.object({\n  accessKey: z.string(),\n  lastUsed: z.string(),\n  permissions: z.array(z.string()),\n  expiresAt: z.string(),\n});\nexport const Invoice = z.object({\n  invoiceNumber: z.string(),\n  dueDate: z.string(),\n  lineItems: z.array(z.string()),\n  taxTotal: z.number(),\n});\n";

#[test]
fn r1_c3_lists_matching_different_shapes_are_not_collapsed() {
    let s = "const TOKEN_FIELDS = ['accessKey', 'lastUsed', 'permissions', 'expiresAt'];\nconst INVOICE_FIELDS = ['invoiceNumber', 'dueDate', 'lineItems', 'taxTotal'];\n";
    let o = init(&repo("r1c3", "src/models.ts", INVOICE_ZOD, s));
    assert_eq!(o.blocks.len(), 2, "{}", o.raw);
    assert!(o.blocks.iter().all(|b| s7(b).is_none()), "{}", o.raw);
}

#[test]
fn r1_c4_the_same_list_in_two_files_is_not_collapsed() {
    let d = ws("r1c4", &[("package.json", PKG), ("src/token.ts", TOKEN_ATTRS), ("src/a.ts", SERVICE), ("src/b.ts", SERVICE)]);
    let o = init(&d);
    assert_eq!(o.blocks.len(), 2, "{}", o.raw);
    assert!(o.blocks.iter().all(|b| s7(b).is_none()), "{}", o.raw);
}

const TWO: &str = "const SELECT_FIELDS = ['accessKey', 'expiresAt', 'lastUsed', 'permissions'];\nconst PICK_FIELDS = ['lastUsed', 'permissions', 'owner'];\n";

#[test]
fn r1_c5_a_name_present_only_in_a_site_is_not_locked_and_holds() {
    // The documented bound: the lock binds the representative only.
    let d = repo("r1c5", "src/token.ts", TOKEN_ATTRS, TWO);
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    assert_eq!(lock.lines().filter(|l| l.starts_with("bind = ")).count(), 4, "{lock}");
    put(&d, "src/token.ts", &TOKEN_ATTRS.replace("owner:", "ownerId:"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("[HOLDS]"), "{out}");
}

#[test]
fn r1_c6_a_single_list_prints_no_site_line() {
    let o = init(&repo("r1c6", "src/token.ts", TOKEN_ATTRS, SERVICE));
    assert!(s7(only(&o)).is_none(), "{}", o.raw);
}

#[test]
fn r1_c7_governed_siblings_leave_before_grouping() {
    let s = "const SELECT_FIELDS = ['accessKey', 'expiresAt', 'lastUsed', 'permissions', 'owner'] as const satisfies readonly (keyof Token)[];\nconst PICK_FIELDS = ['accessKey', 'expiresAt', 'lastUsed', 'permissions'];\nconst A_FIELDS = ['lastUsed', 'permissions', 'owner'];\n";
    let o = init(&repo("r1c7", "src/token.ts", TOKEN_ATTRS, s));
    assert_eq!(tsc(&o).len(), 1, "{:?}", o.abstain);
    let b = only(&o);
    assert_eq!(b.keys["listat"], "const:PICK_FIELDS");
    assert!(s7(b).unwrap().starts_with("# S7 also: const:A_FIELDS@3 |O|=3 "), "{:?}", s7(b));
}

#[test]
fn r1_c8_a_site_with_names_outside_the_representative_is_never_merged() {
    // CM-R1-13: owner appears only in the site; the lock key and the S5 line stay the representative's.
    let o = init(&repo("r1c8", "src/token.ts", TOKEN_ATTRS, TWO));
    let b = only(&o);
    assert_eq!(b.keys["overlap"], "accessKey expiresAt lastUsed permissions");
    assert!(!b.evidence.iter().any(|e| e.starts_with("# S5 ") && e.contains("owner")), "{:?}", b.evidence);
    assert!(s7(b).unwrap().contains("const:PICK_FIELDS@2 |O|=3"), "{:?}", s7(b));
}

// ------------------------------------------------------------------ M: shared

#[test]
fn r1_m2_collapse_and_abstention_output_is_deterministic() {
    let d = repo("r1m2", "src/token.ts", TOKEN_ATTRS, SIX);
    assert_eq!(init_raw(&d), init_raw(&d));
}

#[test]
fn r1_m3_a_collapsed_lock_has_the_unchanged_grammar_and_checks() {
    let d = repo("r1m3", "src/token.ts", TOKEN_ATTRS, SIX);
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    let binds: Vec<&str> = lock.lines().filter(|l| l.starts_with("bind = ")).collect();
    assert_eq!(binds.len(), 5, "{lock}");
    assert!(binds.iter().all(|b| b.contains("module_exports@attributes.")), "{lock}");
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "{out}");
}

#[test]
fn r1_m4_summary_counts_proposals_after_collapse() {
    let o = init(&repo("r1m4", "src/token.ts", TOKEN_ATTRS, SIX));
    assert!(o.raw.contains("SUMMARY packages=0 candidates=0 proposals=1 abstentions=1 inventory=0 status=COMPLETE"), "{}", o.raw);
}

// ------------------------------------- metamorphic invariance suite (ruling, Gate 4)

type Files = Vec<(String, String)>;

fn bases() -> Vec<(&'static str, Files)> {
    let f = |v: Vec<(&str, &str)>| -> Files { v.into_iter().map(|(a, b)| (a.to_string(), b.to_string())).collect() };
    let gov = svc("", " as const satisfies readonly (keyof Token)[]");
    vec![
        ("plain", f(vec![("package.json", PKG), ("src/token.ts", TOKEN_ATTRS), ("src/service.ts", SERVICE)])),
        ("governed", f(vec![("package.json", PKG), ("src/token.ts", TOKEN_ATTRS), ("src/service.ts", &gov)])),
        ("collapsed", f(vec![("package.json", PKG), ("src/token.ts", TOKEN_ATTRS), ("src/service.ts", SIX)])),
        ("testutils", f(vec![("package.json", PKG), ("src/token.ts", TOKEN_ATTRS), ("src/testUtils/service.ts", SERVICE), ("src/b.ts", &gov)])),
    ]
}

fn run_variant(tag: &str, files: &Files) -> String {
    let refs: Vec<(&str, &str)> = files.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let d = ws(tag, &refs);
    init_raw(&d).replace(d.to_string_lossy().as_ref(), "<ROOT>")
}

fn meta(name: &str, tf: &dyn Fn(&str, &str) -> String, reverse: bool) {
    for (bname, base) in bases() {
        let want = run_variant(&format!("m_{name}_{bname}_b"), &base);
        let mut v: Files = base.iter().map(|(p, b)| (p.clone(), tf(p, b))).collect();
        if reverse {
            v.reverse();
        }
        let got = run_variant(&format!("m_{name}_{bname}_v"), &v);
        assert_eq!(want, got, "metamorphic variant {name} changed the output on base {bname}");
    }
}

#[test]
fn meta_crlf_line_endings_do_not_change_output() {
    meta("crlf", &|_, b| b.replace('\n', "\r\n"), false);
}

#[test]
fn meta_utf8_bom_on_source_files_does_not_change_output() {
    meta("bom", &|p, b| if p.ends_with(".json") { b.to_string() } else { format!("\u{feff}{b}") }, false);
}

#[test]
fn meta_trailing_comments_do_not_change_output() {
    meta(
        "cmt",
        &|p, b| if p.ends_with(".json") { b.to_string() } else { b.lines().map(|l| format!("{l} // note\n")).collect::<String>() },
        false,
    );
}

#[test]
fn meta_file_creation_order_does_not_change_output() {
    meta("order", &|_, b| b.to_string(), true);
}
