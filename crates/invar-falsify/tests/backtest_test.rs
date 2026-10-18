// INVAR-BACKTEST-002 (rulings B-D1 to B-D7): backtest replays a strings
// contract's producer with read_any_shape, the Zod key-set reader check
// already uses, so check and backtest can never disagree on the same bytes
// (STAGE3-EVAL-001 S3-F-05). A binding never present in the window says so
// (S3-F-06); a shallow history is stated (S3-F-07). Text only: the JSON
// surface and egress/schema replay are byte-identical.
//
// Every commit carries a fixed author and committer date, so hashes and
// dates in the output are deterministic and goldens can be byte-exact.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_invar")
}

fn repo(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("invar-bt2-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("src")).unwrap();
    git(&d, &["init", "-q"], 0);
    git(&d, &["config", "user.email", "t@t"], 0);
    git(&d, &["config", "user.name", "t"], 0);
    d
}

fn git(dir: &Path, args: &[&str], day: u32) {
    let date = format!("2026-01-{:02}T12:00:00Z", day.max(1));
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_AUTHOR_DATE", &date)
        .env("GIT_COMMITTER_DATE", &date)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

fn put(dir: &Path, rel: &str, body: &str) {
    std::fs::write(dir.join(rel), body).unwrap();
}

fn commit(dir: &Path, rel: &str, body: &str, day: u32) {
    put(dir, rel, body);
    git(dir, &["add", rel], day);
    git(dir, &["commit", "-qm", &format!("day {day}")], day);
}

fn invar(dir: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(bin()).args(args).current_dir(dir).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

const SERVICE: &str = "const SELECT_FIELDS = ['id', 'lastUsed', 'accessKey', 'permissions', 'expiresAt'];\nexport function list(db) { return db.query({ select: SELECT_FIELDS }); }\n";

const TOML: &str = "[[invariant]]\nkind = \"strings\"\nid = \"strings:ApiToken@const:SELECT_FIELDS\"\nproducer = \"src/token.ts\"\nshape = \"ApiToken\"\nlist = \"src/service.ts\"\nlistat = \"const:SELECT_FIELDS\"\nscope = \"same-directory\"\noverlap = \"accessKey expiresAt lastUsed permissions\"\n";

fn token(extra: &str, with_last_used: bool) -> String {
    format!(
        "import {{ z }} from \"zod\";\nexport const ApiToken = z.object({{\n  accessKey: z.string(),\n{}  permissions: z.array(z.string()),\n  expiresAt: z.string(),\n{extra}}});\n",
        if with_last_used { "  lastUsed: z.string(),\n" } else { "" }
    )
}

/// Write the manifest and the list, commit the list, then lock.
fn lock_strings(d: &Path) {
    put(d, "invar.toml", TOML);
    commit(d, "src/service.ts", SERVICE, 20);
    let (c, out) = invar(d, &["lock"]);
    assert_eq!(c, 0, "{out}");
}

fn json(d: &Path) -> serde_json::Value {
    let (c, out) = invar(d, &["backtest", "--format", "json"]);
    assert!(c == 0 || c == 4, "{out}");
    serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"))
}

fn states(v: &serde_json::Value, binding: &str) -> Vec<String> {
    let b = v["backtest"][0]["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["binding"] == binding)
        .unwrap_or_else(|| panic!("no binding {binding} in {v}"));
    b["events"].as_array().unwrap().iter().map(|e| e["state"].as_str().unwrap().to_string()).collect()
}

/// The text block of one binding: its line and the lines under it.
fn block(text: &str, binding: &str) -> String {
    let head = format!("  {binding}");
    let mut out = String::new();
    let mut on = false;
    for l in text.lines() {
        if l == head {
            on = true;
        } else if on && (!l.starts_with("    ") || l.trim().is_empty()) {
            break;
        }
        if on {
            out.push_str(l);
            out.push('\n');
        }
    }
    assert!(!out.is_empty(), "no block for {binding} in:\n{text}");
    out
}

#[test]
fn bt_f1_a_field_present_at_every_commit_is_never_absent() {
    let d = repo("f1");
    commit(&d, "src/token.ts", &token("", true), 1);
    commit(&d, "src/token.ts", &token("  note: z.string(),\n", true), 2);
    commit(&d, "src/token.ts", &token("  note: z.string(),\n  tag: z.string(),\n", true), 3);
    lock_strings(&d);
    let v = json(&d);
    assert_eq!(v["backtest"][0]["commits_replayed"], 3, "{v}");
    assert_eq!(states(&v, "ApiToken.accessKey"), ["present"], "S3-F-05: check reads it present, so must backtest");
    assert_eq!(states(&v, "ApiToken.lastUsed"), ["present"]);
    let (c, text) = invar(&d, &["backtest"]);
    assert_eq!(c, 0, "{text}");
    let b = block(&text, "ApiToken.accessKey");
    assert!(b.contains("never absent after introduction"), "{b}");
    assert!(!b.contains("never present"), "a present binding is not never-present: {b}");
}

#[test]
fn bt_f2_a_removed_and_restored_field_is_one_window() {
    let d = repo("f2");
    commit(&d, "src/token.ts", &token("", true), 1);
    commit(&d, "src/token.ts", &token("", false), 2);
    commit(&d, "src/token.ts", &token("", true), 3);
    lock_strings(&d);
    let v = json(&d);
    assert_eq!(states(&v, "ApiToken.lastUsed"), ["present", "absent", "present"], "{v}");
    assert_eq!(states(&v, "ApiToken.accessKey"), ["present"], "{v}");
    let (c, text) = invar(&d, &["backtest"]);
    assert_eq!(c, 0, "{text}");
    assert!(block(&text, "ApiToken.lastUsed").contains("1 absence window(s) after introduction"), "{text}");
}

#[test]
fn bt_f3_absence_before_the_shape_existed_is_not_a_window() {
    let d = repo("f3");
    commit(&d, "src/token.ts", "import { z } from \"zod\";\nexport const Other = z.object({\n  a: z.string(),\n});\n", 1);
    commit(&d, "src/token.ts", &token("", true), 2);
    commit(&d, "src/token.ts", &token("  note: z.string(),\n", true), 3);
    lock_strings(&d);
    let v = json(&d);
    assert_eq!(states(&v, "ApiToken.accessKey"), ["absent", "present"], "B-D3: NotFound is absent, before introduction: {v}");
    let (c, text) = invar(&d, &["backtest"]);
    assert_eq!(c, 0, "{text}");
    assert!(block(&text, "ApiToken.accessKey").contains("never absent after introduction"), "{text}");
}

#[test]
fn bt_f4_a_dynamic_shape_is_a_stated_blind_spot() {
    let d = repo("f4");
    commit(&d, "src/token.ts", &token("", true), 1);
    commit(&d, "src/token.ts", &token("  ...base,\n", true), 2);
    commit(&d, "src/token.ts", &token("", true), 3);
    lock_strings(&d);
    let v = json(&d);
    assert_eq!(states(&v, "ApiToken.accessKey"), ["present", "unextractable", "present"], "B-D4: never guessed: {v}");
    let (c, text) = invar(&d, &["backtest"]);
    assert_eq!(c, 4, "a blind spot is exit 4: {text}");
    assert!(block(&text, "ApiToken.accessKey").contains("1 commit(s) not readable (stated, not guessed)"), "{text}");
}

#[test]
fn bt_f5_a_binding_never_present_in_the_window_says_so() {
    let d = repo("f5");
    commit(&d, "src/token.ts", &token("", true), 1);
    commit(&d, "src/token.ts", &token("  note: z.string(),\n", true), 2);
    // scopeTag exists only in the working tree when the lock is taken, so
    // no replayed commit ever carries it.
    put(&d, "src/token.ts", &token("  note: z.string(),\n  scopeTag: z.string(),\n", true));
    put(&d, "invar.toml", &TOML.replace("permissions\"", "permissions scopeTag\""));
    commit(&d, "src/service.ts", &SERVICE.replace("'expiresAt']", "'expiresAt', 'scopeTag']"), 20);
    let (c, out) = invar(&d, &["lock"]);
    assert_eq!(c, 0, "{out}");
    assert!(std::fs::read_to_string(d.join("invar.lock")).unwrap().contains("ApiToken.scopeTag"), "scopeTag must be bound");
    let v = json(&d);
    assert_eq!(states(&v, "ApiToken.scopeTag"), ["absent"], "{v}");
    let (c, text) = invar(&d, &["backtest"]);
    assert_eq!(c, 0, "{text}");
    let b = block(&text, "ApiToken.scopeTag");
    // The exact line, not containment: Gate 3's first run had a mutant
    // fall through to the blind-spot arm ("...; 0 commit(s) not
    // readable") and survive a contains() check (recorded in the ledger).
    assert!(b.lines().any(|l| l == "    never present in the replayed window"), "S3-F-06: {b}");
    assert!(!b.contains("never absent after introduction"), "no reassurance about a field never seen: {b}");
    assert!(block(&text, "ApiToken.accessKey").contains("never absent after introduction"), "{text}");
}

const SHALLOW_NOTE: &str = "note: shallow history; the oldest replayed commit is the shallow boundary, and its date is the cutoff, not a change";

fn shallow_pair(tag: &str) -> (PathBuf, PathBuf) {
    let d = repo(tag);
    commit(&d, "src/token.ts", &token("", true), 1);
    commit(&d, "src/token.ts", &token("", false), 2);
    commit(&d, "src/token.ts", &token("", true), 3);
    lock_strings(&d);
    let s = std::env::temp_dir().join(format!("invar-bt2-{tag}-shallow-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&s);
    let url = format!("file://{}", d.display());
    let o = Command::new("git").args(["clone", "-q", "--depth", "2", &url]).arg(&s).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    for f in ["invar.toml", "invar.lock"] {
        std::fs::copy(d.join(f), s.join(f)).unwrap();
    }
    (d, s)
}

#[test]
fn bt_f6_shallow_history_is_stated_and_full_history_is_not() {
    let (full, shallow) = shallow_pair("f6");
    let (_, st) = invar(&shallow, &["backtest"]);
    let lines: Vec<&str> = st.lines().collect();
    assert!(lines[0].starts_with("invar backtest ("), "{st}");
    assert_eq!(lines[1], SHALLOW_NOTE, "R-B2-3: one line right after the header: {st}");
    let (_, ft) = invar(&full, &["backtest"]);
    assert!(!ft.contains("shallow"), "a full history is never labelled shallow: {ft}");
}

// ---------------------------------------------------------------- controls

const ORDER_WITH: &str = "type Order struct {\n\tAmountCents int64 `json:\"amount_cents\"`\n\tCurrency string `json:\"currency\"`\n}\n";
const ORDER_WITHOUT: &str = "type Order struct {\n\tCurrency string `json:\"currency\"`\n}\n";

fn schema_repo(tag: &str, id: &str) -> PathBuf {
    let d = repo(tag);
    commit(&d, "order.go", ORDER_WITH, 1);
    commit(&d, "order.go", ORDER_WITHOUT, 2);
    commit(&d, "order.go", ORDER_WITH, 3);
    put(&d, "client.js", "const a = order.amount_cents;\n");
    put(&d, "invar.toml", &format!("[[invariant]]\nkind = \"schema\"\nid = \"{id}\"\nproducer = \"order.go\"\nconsumer = \"client.js\"\n"));
    let (c, out) = invar(&d, &["lock"]);
    assert_eq!(c, 0, "{out}");
    d
}

// Recorded at Gate 1 from the unchanged engine (b484d3d). Fixed commit
// dates make every hash stable, so egress/schema replay is asserted
// byte-for-byte (CM-B-7).
const GOLDEN_TEXT: &str = r#"invar backtest (invar.toml, 1 contract(s), depth 50)

orders: order.go (3 commit(s) replayed)
  Order.amount_cents
    2026-01-01 26438df29f75 present
    2026-01-02 0342ee67679d absent
    2026-01-03 c890901e6e44 present
    1 absence window(s) after introduction

Timelines replay today's extraction over past file contents. They are
not replayed verdicts: waivers, selectors and samples are today's, and a
past consumer may not have read the field yet. Unreadable commits are
stated, never guessed.
"#;
const GOLDEN_JSON: &str = r#"{
  "backtest": [
    {
      "bindings": [
        {
          "absence_windows": 1,
          "binding": "Order.amount_cents",
          "blind_spot_commits": 0,
          "events": [
            {
              "commit": "26438df29f75",
              "date": "2026-01-01",
              "state": "present"
            },
            {
              "commit": "0342ee67679d",
              "date": "2026-01-02",
              "state": "absent"
            },
            {
              "commit": "c890901e6e44",
              "date": "2026-01-03",
              "state": "present"
            }
          ]
        }
      ],
      "commits_replayed": 3,
      "contract": "orders",
      "producer": "order.go"
    }
  ],
  "coda": "Timelines replay today's extraction over past file contents. They are\nnot replayed verdicts: waivers, selectors and samples are today's, and a\npast consumer may not have read the field yet. Unreadable commits are\nstated, never guessed.",
  "depth": 50,
  "skipped": []
}
"#;

#[test]
fn bt_n1_schema_replay_is_byte_identical() {
    let d = schema_repo("n1", "orders");
    let (c, text) = invar(&d, &["backtest"]);
    assert_eq!(c, 0, "{text}");
    let (_, js) = invar(&d, &["backtest", "--format", "json"]);
    if std::env::var("BT2_RECORD").is_ok() {
        std::fs::write(std::env::temp_dir().join("bt2_golden_text.txt"), &text).unwrap();
        std::fs::write(std::env::temp_dir().join("bt2_golden_json.txt"), &js).unwrap();
    }
    assert_eq!(text, GOLDEN_TEXT, "egress/schema replay text must not change by one byte");
    assert_eq!(js, GOLDEN_JSON, "egress/schema replay JSON must not change by one byte");
}

fn key_shape(v: &serde_json::Value, path: &str, out: &mut Vec<String>) {
    match v {
        serde_json::Value::Object(m) => {
            for (k, x) in m {
                let p = format!("{path}.{k}");
                out.push(p.clone());
                key_shape(x, &p, out);
            }
        }
        serde_json::Value::Array(a) => {
            for x in a {
                key_shape(x, &format!("{path}[]"), out);
            }
        }
        _ => {}
    }
}

#[test]
fn bt_n2_strings_json_has_exactly_the_schema_key_set() {
    let (_, shallow) = shallow_pair("n2");
    let s = json(&shallow);
    let g = json(&schema_repo("n2s", "orders"));
    let (mut a, mut b) = (Vec::new(), Vec::new());
    key_shape(&s, "", &mut a);
    key_shape(&g, "", &mut b);
    a.sort();
    a.dedup();
    b.sort();
    b.dedup();
    assert_eq!(a, b, "B4: the JSON surface is frozen, shallow or not");
    assert!(!s.to_string().contains("shallow"), "B-D6: the shallow note is text only: {s}");
}

#[test]
fn bt_n3_only_the_strings_prefix_routes_to_the_strings_reader() {
    let d = schema_repo("n3", "legacy-strings:orders");
    let v = json(&d);
    assert_eq!(
        states(&v, "Order.amount_cents"),
        ["present", "absent", "present"],
        "an id that merely contains strings is not a strings contract: {v}"
    );
}

#[test]
fn bt_m1_two_runs_are_byte_identical() {
    let (full, _) = shallow_pair("m1");
    let a = invar(&full, &["backtest"]);
    let b = invar(&full, &["backtest"]);
    assert_eq!(a, b);
}
