//! INIT-DISCOVERY slice 2b fixtures (docs/architecture/INIT-DISCOVERY-SLICE2B.md,
//! FROZEN revision 1, section E). Every fixture is synthetic, built in a
//! temporary directory, and run through the real binary (`invar init
//! --stdout` twice, byte-identical; lock and check by exit code: 0 holds, 1
//! violated, 4 skipped or not evaluated).
//!
//! F: K3-F1 attributes literal, K3-F2 zod shape, K3-F3 producer rename,
//! K3-F4 merge-skew reintroduction, K3-F5 consistent rename holds, K3-F6 test
//! category (CM-K3-6), K3-F7 partial subset (CM-K3-10).
//! N: K3-N1 (CM-K3-1), N2 (CM-K3-5), N3 (CM-K3-4), N4 (CM-K3-7 version
//! family), N5 (CM-K3-8), N6 (stoplist-only list).
//! U: K3-U1 (CM-K3-2), U2 (list turned dynamic after lock), U3 (CM-K3-3), U4
//! (ORM source only), U5 (CM-K3-9 display column list).
//! M: K3-M3 determinism is asserted by every `init` call; K3-M1 and K3-M2
//! (slice 1 and slice 2 fixtures and F01 byte-identical) are the existing
//! suites, which must stay green unchanged.

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

const TOKEN_ZOD: &str = "import { z } from \"zod\";\nexport const ApiToken = z.object({\n  name: z.string(),\n  description: z.string(),\n  accessKey: z.string(),\n  lastUsed: z.string(),\n  permissions: z.array(z.string()),\n  expiresAt: z.string(),\n  owner: z.string(),\n});\n";

const SERVICE: &str = "const SELECT_FIELDS = ['id', 'name', 'description', 'lastUsed', 'accessKey', 'permissions', 'expiresAt', 'type'];\nexport function list(db) { return db.query({ select: SELECT_FIELDS }); }\n";

fn repo(tag: &str, shape_file: &str, shape: &str, service: &str) -> PathBuf {
    ws(tag, &[("package.json", PKG), (shape_file, shape), ("src/service.ts", service)])
}

fn only(o: &Out) -> &B {
    assert_eq!(o.blocks.len(), 1, "{}", o.raw);
    &o.blocks[0]
}

// ------------------------------------------------------------ positives

#[test]
fn k3_f1_attributes_literal_and_string_list_one_strings_contract() {
    let d = repo("f1", "src/token.ts", TOKEN_ATTRS, SERVICE);
    let o = init(&d);
    let b = only(&o);
    assert!(k3(&o).is_empty(), "{:?}", k3(&o));
    assert_eq!(b.kind, "strings");
    assert_eq!(b.keys["producer"], "src/token.ts");
    assert_eq!(b.keys["shape"], "module_exports@attributes");
    assert_eq!(b.keys["list"], "src/service.ts");
    assert_eq!(b.keys["listat"], "const:SELECT_FIELDS");
    assert_eq!(b.keys["scope"], "same-directory");
    assert_eq!(b.keys["overlap"], "accessKey expiresAt lastUsed permissions");
    assert!(b.evidence.iter().any(|e| e == "# category: production"), "{:?}", b.evidence);
    assert!(b.evidence.iter().any(|e| e.starts_with("# S5 distinctive overlap (4 of 4 list entries): accessKey (src/service.ts:1)")), "{:?}", b.evidence);

    adopt(&d);
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("SKIPPED") && !out.contains("HOLDS"), "{out}");
    let (c, out) = invar(&d, &["lock"]);
    assert_eq!(c, 0, "{out}");
    let lock = std::fs::read_to_string(d.join("invar.lock")).unwrap();
    let binds: Vec<&str> = lock.lines().filter(|l| l.starts_with("bind = ")).collect();
    assert_eq!(binds.len(), 4, "{lock}");
    assert!(lock.contains("module_exports@attributes.lastUsed"), "{lock}");
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "{out}");
    assert!(out.contains("[HOLDS]"), "{out}");
}

#[test]
fn k3_f2_zod_shape_variant() {
    let d = repo("f2", "src/token.ts", TOKEN_ZOD, SERVICE);
    let o = init(&d);
    let b = only(&o);
    assert_eq!(b.keys["shape"], "ApiToken");
    assert_eq!(b.keys["overlap"], "accessKey expiresAt lastUsed permissions");
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    assert_eq!(invar(&d, &["check"]).0, 0);
}

#[test]
fn k3_f3_producer_rename_is_a_stale_reference() {
    let d = repo("f3", "src/token.ts", TOKEN_ATTRS, SERVICE);
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    put(&d, "src/token.ts", &TOKEN_ATTRS.replace("lastUsed:", "lastUsedAt:"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 1, "{out}");
    assert!(out.contains("[VIOLATED]") && out.contains("module_exports@attributes.lastUsed"), "{out}");
    assert!(out.contains("src/service.ts:1"), "{out}");
}

#[test]
fn k3_f4_merge_skew_reintroducing_a_stale_string_is_violated() {
    let d = repo("f4", "src/token.ts", TOKEN_ATTRS, SERVICE);
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    // The clean rename lands on both sides: holds.
    put(&d, "src/token.ts", &TOKEN_ATTRS.replace("lastUsed:", "lastUsedAt:"));
    put(&d, "src/service.ts", &SERVICE.replace("'lastUsed'", "'lastUsedAt'"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "{out}");
    // A concurrent branch merges the old spelling back into the list.
    put(&d, "src/service.ts", &SERVICE.replace("'lastUsed'", "'lastUsedAt', 'lastUsed'"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 1, "{out}");
    assert!(out.contains("module_exports@attributes.lastUsed"), "{out}");
}

#[test]
fn k3_f5_consistent_rename_and_list_narrowing_hold() {
    let d = repo("f5", "src/token.ts", TOKEN_ATTRS, SERVICE);
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    put(&d, "src/token.ts", &TOKEN_ATTRS.replace("lastUsed:", "lastUsedAt:"));
    put(&d, "src/service.ts", &SERVICE.replace("'lastUsed', ", ""));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "{out}");
    // Additions to the shape hold too.
    put(&d, "src/token.ts", &TOKEN_ATTRS.replace("lastUsed:", "lastUsedAt:").replace("    owner:", "    extra: { type: 'string' },\n    owner:"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "{out}");
}

#[test]
fn k3_f6_test_lists_are_labeled_never_production() {
    let d = ws(
        "f6",
        &[("package.json", PKG), ("src/token.ts", TOKEN_ATTRS), ("src/__tests__/service.test.ts", SERVICE)],
    );
    let o = init(&d);
    let b = only(&o);
    assert!(b.evidence.iter().any(|e| e == "# category: test"), "{:?}", b.evidence);
    assert_eq!(b.keys["scope"], "same-package");
}

#[test]
fn k3_f7_partial_subset_list_holds_when_an_unlisted_field_is_renamed() {
    let d = repo("f7", "src/token.ts", TOKEN_ATTRS, SERVICE);
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    put(&d, "src/token.ts", &TOKEN_ATTRS.replace("owner:", "ownerId:"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 0, "{out}");
}

// ------------------------------------------------------------ negatives

#[test]
fn k3_n1_receiver_collision_below_the_distinctive_bar_never_proposes() {
    // (a) two shared distinctive names only: a different store that happens to share them.
    // (b) three shared names but under half of the list: the list is mostly some other store.
    for (tag, list) in [
        ("two", "const SELECT_FIELDS = ['email', 'lastUsed', 'accessKey'];\nexport const q = { select: SELECT_FIELDS };\n"),
        ("cover", "const SELECT_FIELDS = ['lastUsed', 'accessKey', 'permissions', 'email', 'phone', 'lastLogin', 'role'];\nexport const q = { select: SELECT_FIELDS };\n"),
    ] {
        let d = repo(&format!("n1{tag}"), "src/token.ts", TOKEN_ATTRS, list);
        let o = init(&d);
        assert!(o.blocks.is_empty(), "{tag}: {}", o.raw);
        let a = k3(&o);
        assert_eq!(a.len(), 1, "{tag}: {a:?}");
        assert!(a[0].starts_with("ABSTAIN NON-DISTINCTIVE-KEY src/service.ts:1 "), "{tag}: {a:?}");
    }
}

#[test]
fn k3_n2_stoplisted_overlap_never_proposes() {
    let list = "const SELECT_FIELDS = ['id', 'name', 'status', 'created_at', 'lastUsed'];\nexport const q = { select: SELECT_FIELDS };\n";
    let shape = TOKEN_ATTRS.replace("    name:", "    id: { type: 'integer' },\n    status: { type: 'string' },\n    created_at: { type: 'datetime' },\n    name:");
    let d = repo("n2", "src/token.ts", &shape, list);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = k3(&o);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].starts_with("ABSTAIN NON-DISTINCTIVE-KEY src/service.ts:1 "), "{a:?}");
}

#[test]
fn k3_n3_generic_helper_list_is_not_a_query_position() {
    let list = "const USER_KEYS = ['lastUsed', 'accessKey', 'permissions', 'expiresAt'];\nexport const q = buildQuery(USER_KEYS);\n";
    let d = repo("n3", "src/token.ts", TOKEN_ATTRS, list);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = k3(&o);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].starts_with("ABSTAIN AMBIG-RECEIVER src/service.ts:1 "), "{a:?}");
    assert!(a[0].contains("not in a query position"), "{a:?}");
}

#[test]
fn k3_n4_version_family_never_pairs() {
    let shapes = "import { z } from \"zod\";\nexport const AccountV1 = z.object({ accessKey: z.string(), lastUsed: z.string(), permissions: z.string(), expiresAt: z.string() });\nexport const AccountV2 = z.object({ accessKey: z.string(), lastUsed: z.string(), permissions: z.string(), expiresAt: z.string(), tier: z.string() });\n";
    let d = repo("n4", "src/token.ts", shapes, SERVICE);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = k3(&o);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].starts_with("ABSTAIN VERSION-FAMILY src/service.ts:1 "), "{a:?}");
}

#[test]
fn k3_n5_cross_package_with_no_import_is_not_paired() {
    let d = ws(
        "n5",
        &[
            ("packages/a/package.json", "{\"name\":\"a\"}\n"),
            ("packages/a/src/token.ts", TOKEN_ATTRS),
            ("packages/b/package.json", "{\"name\":\"b\"}\n"),
            ("packages/b/src/service.ts", SERVICE),
        ],
    );
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = k3(&o);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].starts_with("ABSTAIN AMBIG-RECEIVER packages/b/src/service.ts:1 "), "{a:?}");
    assert!(a[0].contains("no import path"), "{a:?}");
}

#[test]
fn k3_n6_list_of_only_generic_names_never_proposes() {
    let list = "const SELECT_FIELDS = ['id', 'name', 'type', 'description', 'title'];\nexport const q = { select: SELECT_FIELDS };\n";
    let shape = TOKEN_ATTRS.replace("    name:", "    id: { type: 'integer' },\n    type: { type: 'string' },\n    title: { type: 'string' },\n    name:");
    let d = repo("n6", "src/token.ts", &shape, list);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
}

// ----------------------------------------------------------- abstentions

#[test]
fn k3_u1_two_qualifying_shapes_abstain_ambig_receiver() {
    let d = ws(
        "u1",
        &[("package.json", PKG), ("src/token.ts", TOKEN_ATTRS), ("src/token2.ts", TOKEN_ZOD), ("src/service.ts", SERVICE)],
    );
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = k3(&o);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].starts_with("ABSTAIN AMBIG-RECEIVER src/service.ts:1 "), "{a:?}");
    assert!(a[0].contains("more than one shape"), "{a:?}");
}

#[test]
fn k3_u2_list_turned_dynamic_after_lock_is_not_evaluated() {
    let d = repo("u2", "src/token.ts", TOKEN_ATTRS, SERVICE);
    adopt(&d);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    put(&d, "src/service.ts", &SERVICE.replace("'type']", "'type', ...EXTRA]"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 4, "{out}");
    assert!(out.contains("no longer statically extractable") && !out.contains("[HOLDS]"), "{out}");
    // A shape turned dynamic is the same.
    put(&d, "src/service.ts", SERVICE);
    put(&d, "src/token.ts", &TOKEN_ATTRS.replace("    owner:", "    ...base,\n    owner:"));
    let (c, out) = invar(&d, &["check"]);
    assert_eq!(c, 4, "{out}");
}

#[test]
fn k3_u3_dynamic_string_lists_abstain() {
    for (tag, list) in [
        ("spread", "const SELECT_FIELDS = ['lastUsed', 'accessKey', 'permissions', 'expiresAt', ...EXTRA];\n"),
        ("concat", "const SELECT_FIELDS = ['last' + 'Used', 'accessKey', 'permissions', 'expiresAt'];\n"),
        ("template", "const SELECT_FIELDS = [`${p}Used`, 'accessKey', 'permissions', 'expiresAt'];\n"),
    ] {
        let d = repo(&format!("u3{tag}"), "src/token.ts", TOKEN_ATTRS, list);
        let o = init(&d);
        assert!(o.blocks.is_empty(), "{tag}: {}", o.raw);
        let a = k3(&o);
        assert_eq!(a.len(), 1, "{tag}: {a:?}");
        assert!(a[0].starts_with("ABSTAIN UNSUPPORTED-STRING-DYNAMIC src/service.ts:1 "), "{tag}: {a:?}");
    }
}

#[test]
fn k3_u4_orm_source_only_abstains() {
    let prisma = "model ApiToken {\n  id Int @id\n  accessKey String\n  lastUsed DateTime\n  permissions Json\n  expiresAt DateTime\n}\n";
    let d = ws("u4", &[("package.json", PKG), ("prisma/schema.prisma", prisma), ("src/service.ts", SERVICE)]);
    let o = init(&d);
    assert!(o.blocks.is_empty(), "{}", o.raw);
    let a = k3(&o);
    assert_eq!(a.len(), 1, "{a:?}");
    assert!(a[0].starts_with("ABSTAIN UNSUPPORTED-ORM-SOURCE src/service.ts:1 "), "{a:?}");
    assert!(a[0].contains("prisma/schema.prisma"), "{a:?}");
}

#[test]
fn k3_u5_display_column_list_is_proposed_and_labeled_residual_noise() {
    let ui = "export const table = {\n  columns: ['lastUsed', 'accessKey', 'permissions', 'expiresAt'],\n};\n";
    let d = repo("u5", "src/token.ts", TOKEN_ATTRS, ui);
    let o = init(&d);
    let b = only(&o);
    assert_eq!(b.keys["listat"], "prop:columns#1");
    assert!(b.evidence.iter().any(|e| e.starts_with("# S6 DISCLAIMER") && e.contains("display column lists")), "{:?}", b.evidence);
}
