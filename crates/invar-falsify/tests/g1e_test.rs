//! G1-E fixtures (RECALL-ATTACK-1 phase 3, docs/falsification/G1-E-SPEC.md
//! r0), frozen with the spec BEFORE any implementation. Change-scoped
//! workspaces (a producer_base is declared), as the rule applies only there.
//!
//! G1-E states (exit 4, never VIOLATED) a producer whose literal emissions
//! of one frozen call form fell while its opaque emissions of that form
//! rose, when a top-level key of a base literal emission of that form is
//! read by the consumer and the contract has no residue.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_g1e_{tag}_{}_{nanos}", std::process::id()));
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    dir
}

fn invar(dir: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(dir).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn toml(producer: &str, base: &str, consumer: &str) -> String {
    format!("[[invariant]]\nkind = \"schema\"\nid = \"g1e\"\nproducer = \"{producer}\"\nproducer_base = \"{base}\"\nconsumer = \"{consumer}\"\n")
}

const MARK: &str = "[not-evaluated:opaque-emission]";

fn run(tag: &str, base: &str, head: &str, consumer: &str) -> (i32, String) {
    let t = toml("head.ts", "base.ts", "c.jsx");
    let d = ws(tag, &[("head.ts", head), ("base.ts", base), ("c.jsx", consumer), ("invar.toml", &t)]);
    invar(&d, &["check", "--no-reuse"])
}

/// The r1-107 shape. Base: two literal `.json(` emissions stating
/// `response` (lines 2 and 5). Head: the first became `.json(data)`
/// (line 3); the error path still states `response` literally.
const BASE: &str = "export default async function h(req, res) {\n  if (req.method === \"GET\") return res.status(200).json({ response: tags });\n  if (!req.body) {\n    return res.status(400).json({ response: \"bad request\" });\n  }\n}\n";
const HEAD: &str = "export default async function h(req, res) {\n  const { statusCode, ...data } = tags;\n  if (req.method === \"GET\") return res.status(statusCode).json(data);\n  if (!req.body) {\n    return res.status(400).json({ response: \"bad request\" });\n  }\n}\n";
/// The consumer reads `response` (line 2).
const READS: &str = "export default function C({ tagsStatsData }) {\n  const tags = tagsStatsData?.response ?? tagsStatsData;\n  return tags.length;\n}\n";

#[test]
fn g1e_f01_literal_emission_became_opaque_is_stated() {
    let (code, out) = run("f01", BASE, HEAD, READS);
    assert_eq!(code, 4, "{out}");
    assert!(!out.contains("[VIOLATED]"), "{out}");
    assert!(
        out.contains("'.json(' emission became opaque in head.ts: literal 2 -> 1, opaque 0 -> 1 (first opaque at head.ts:3, 'data'); base stated 'response' literally (base.ts:2), read in c.jsx; whether head still emits it is not evaluated [not-evaluated:opaque-emission]"),
        "{out}"
    );
}

#[test]
fn g1e_f02_an_added_opaque_emission_without_a_lost_literal_holds() {
    // Opaque rises (0 -> 1) but the literal count does not fall (2 -> 2).
    let head = BASE.replace("  if (!req.body) {\n", "  if (req.query.raw) return res.json(raw);\n  if (!req.body) {\n");
    let (code, out) = run("f02", BASE, &head, READS);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains(MARK), "{out}");
}

#[test]
fn g1e_f03_a_lost_literal_without_an_opaque_emission_holds() {
    // Literal falls (2 -> 1), opaque does not rise: the GET emission became
    // a member expression, which is neither literal nor opaque.
    let head = BASE.replace(".json({ response: tags })", ".json(tags.body)");
    let (code, out) = run("f03", BASE, &head, READS);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains(MARK), "{out}");
}

#[test]
fn g1e_f04_no_base_literal_key_read_by_the_consumer_holds() {
    let consumer = "export default function C({ s }) {\n  const n = s?.count ?? 0;\n  return n;\n}\n";
    let (code, out) = run("f04", BASE, HEAD, consumer);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains(MARK), "{out}");
}

#[test]
fn g1e_f05_a_commented_opaque_emission_is_no_emission() {
    // The GET literal is removed (literal 2 -> 1) and the only opaque
    // emission at head is on a comment line.
    let head = BASE.replace(
        "  if (req.method === \"GET\") return res.status(200).json({ response: tags });\n",
        "  // return res.status(statusCode).json(data);\n",
    );
    let (code, out) = run("f05", BASE, &head, READS);
    assert!(!out.contains(MARK), "{out}");
    assert_ne!(code, 4, "{out}");
}

#[test]
fn g1e_f06_a_statement_never_rides_on_a_conviction() {
    // Response side: base declares `digest` in a type the consumer reads;
    // head drops it (residue). G1-E's opaque switch is then not stated.
    let base = format!("export interface Out {{\n  digest: string;\n}}\n{BASE}");
    let head = format!("export interface Out {{\n}}\n{HEAD}");
    let consumer = format!("{READS}export const d = (x) => x.digest;\n");
    let (code, out) = run("f06", &base, &head, &consumer);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED]") && out.contains("\"digest\""), "{out}");
    assert!(!out.contains(MARK), "{out}");
}

#[test]
fn g1e_f07_send_result_payload_is_the_second_argument() {
    // Home Assistant websocket shape: connection.send_result(msg_id, payload).
    // One function, two emissions, so no field moves between scopes.
    let base = "def info(hass, connection, msg):\n    connection.send_result(msg[\"id\"], {\"url\": data.url, \"channel\": ch})\n    if msg.get(\"short\"):\n        connection.send_result(msg[\"id\"], {\"url\": data.url})\n";
    let head = "def info(hass, connection, msg):\n    connection.send_result(msg[\"id\"], result)\n    if msg.get(\"short\"):\n        connection.send_result(msg[\"id\"], {\"url\": data.url})\n";
    let consumer = "export const show = (info) => {\n  const u = info?.url ?? \"\";\n  return u;\n};\n";
    let t = toml("head.py", "base.py", "c.ts");
    let d = ws("f07", &[("head.py", head), ("base.py", base), ("c.ts", consumer), ("invar.toml", &t)]);
    let (code, out) = invar(&d, &["check", "--no-reuse"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("'send_result(' emission became opaque in head.py: literal 2 -> 1, opaque 0 -> 1 (first opaque at head.py:2, 'result')"), "{out}");
    assert!(out.contains("base stated 'url' literally (base.py:2), read in c.ts"), "{out}");
}

#[test]
fn g1e_f08_a_member_expression_is_not_opaque() {
    // Literal falls; the head expression `tags.data` is a member access,
    // neither literal nor opaque. Same as f03 but at the opaque side: a
    // second head emission passes `tags.data` where none existed.
    let head = BASE.replace(".json({ response: tags })", ".json(tags.data)");
    let (code, out) = run("f08", BASE, &head, READS);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains(MARK), "{out}");
}

#[test]
fn g1e_f09_a_keyword_is_not_opaque() {
    let head = BASE.replace(".json({ response: tags })", ".json(null)");
    let (code, out) = run("f09", BASE, &head, READS);
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains(MARK), "{out}");
}

#[test]
fn g1e_f10_a_form_matches_only_as_a_whole_call_name() {
    // `JsonResponse(` contains `Response(`; only the JsonResponse form may
    // fire, once.
    let base = "def a(request):\n    return JsonResponse({\"response\": rows})\n\ndef b(request):\n    return JsonResponse({\"response\": []})\n";
    let head = "def a(request):\n    return JsonResponse(payload)\n\ndef b(request):\n    return JsonResponse({\"response\": []})\n";
    let t = toml("head.py", "base.py", "c.jsx");
    let d = ws("f10", &[("head.py", head), ("base.py", base), ("c.jsx", READS), ("invar.toml", &t)]);
    let (code, out) = invar(&d, &["check", "--no-reuse"]);
    assert_eq!(code, 4, "{out}");
    assert_eq!(out.matches(MARK).count(), 1, "{out}");
    assert!(out.contains("'JsonResponse(' emission became opaque"), "{out}");
}

#[test]
fn g1e_f11_a_multi_line_literal_states_its_keys() {
    // The base literal spans lines 2 to 4; its key is still extracted.
    let base = "export default function h(req, res) {\n  return res.json({\n    response: rows,\n  });\n}\nexport function e(req, res) {\n  return res.json({ response: [] });\n}\n";
    let head = "export default function h(req, res) {\n  return res.json(rows);\n}\nexport function e(req, res) {\n  return res.json({ response: [] });\n}\n";
    let (code, out) = run("f11", base, head, READS);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("base stated 'response' literally (base.ts:2), read in c.jsx"), "{out}");
}

#[test]
fn g1e_f12_only_top_level_keys_count() {
    // `response` is nested under `data`; the top-level key `data` is not
    // read, so no base literal key is read by the consumer.
    let base = "export default function h(req, res) {\n  return res.json({ data: { response: rows } });\n}\nexport function e(req, res) {\n  return res.json({ data: { response: [] } });\n}\n";
    let head = "export default function h(req, res) {\n  return res.json(rows);\n}\nexport function e(req, res) {\n  return res.json({ data: { response: [] } });\n}\n";
    let (code, out) = run("f12", base, head, READS);
    assert!(!out.contains(MARK), "{out}");
    assert_ne!(code, 4, "{out}");
}

#[test]
fn g1e_f13_strict_mode_is_untouched() {
    let t = "[[invariant]]\nkind = \"schema\"\nid = \"g1e\"\nproducer = \"head.ts\"\nconsumer = \"c.jsx\"\n";
    let d = ws("f13", &[("head.ts", HEAD), ("c.jsx", READS), ("invar.toml", t)]);
    let (_, out) = invar(&d, &["check", "--no-reuse"]);
    assert!(!out.contains(MARK), "no producer_base, no G1-E:\n{out}");
}

#[test]
fn g1e_f14_identical_bytes_give_identical_output() {
    let t = toml("head.ts", "base.ts", "c.jsx");
    let d = ws("f14", &[("head.ts", HEAD), ("base.ts", BASE), ("c.jsx", READS), ("invar.toml", &t)]);
    let a = invar(&d, &["check", "--no-reuse"]);
    let b = invar(&d, &["check", "--no-reuse"]);
    assert_eq!(a, b);
}

#[test]
fn g1e_f15_the_code_companion_is_scanned() {
    // The producer pair is unchanged; the opaque switch is in the companion.
    let plain = "export interface Tag {\n  name: string;\n}\n";
    let t = toml("head.ts", "base.ts", "c.jsx") + "producer_code = \"api.ts\"\nproducer_code_base = \"api_base.ts\"\n";
    let d = ws("f15", &[("head.ts", plain), ("base.ts", plain), ("api.ts", HEAD), ("api_base.ts", BASE), ("c.jsx", READS), ("invar.toml", &t)]);
    let (code, out) = invar(&d, &["check", "--no-reuse"]);
    assert_eq!(code, 4, "{out}");
    assert!(out.contains("'.json(' emission became opaque in api.ts: literal 2 -> 1, opaque 0 -> 1 (first opaque at api.ts:3, 'data'); base stated 'response' literally (api_base.ts:2)"), "{out}");
}
