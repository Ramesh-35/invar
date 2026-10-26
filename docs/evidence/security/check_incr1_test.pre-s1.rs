//! CHECK-INCR-1 fixtures (founder rulings I-D1 to I-D6 of 2026-10-23),
//! frozen at Gate 1 before any implementation. `invar check` reuses a HOLDS
//! only when the engine, the contract's invar.toml block, its invar.lock
//! entry and the bytes of every file its evaluation read are all unchanged,
//! and no waiver or locked sample names it. Records live in invar.lock
//! (`held`, `held_input`, written by `invar lock`) and in the local cache
//! `.invar/cache/verdicts.json` (written by `invar check`).
//!
//! Every fixture first proves a record exists (so it cannot pass
//! vacuously), then changes one input and requires a real evaluation.
//! F17 passes on the baseline by construction; all others fail there.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_incr1_{tag}_{}_{nanos}", std::process::id()));
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
    std::fs::read_to_string(root.join(rel)).unwrap_or_default()
}

fn invar(dir: &Path, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(dir).output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

const CACHE: &str = ".invar/cache/verdicts.json";

/// The HOLDS line of contract `id` says its verdict was reused.
fn reused(out: &str, id: &str) -> bool {
    out.lines().any(|l| l.contains("[HOLDS]") && l.contains(&format!(" {id} (")) && l.contains("unchanged since"))
}

/// Some HOLDS line says it was reused. (Gate 1 amendment, K-1: the frozen
/// version matched the existing note "producer unchanged since lock (...)"
/// too, so it rejected every correct evaluated outcome.)
fn any_reuse(out: &str) -> bool {
    out.lines().any(|l| l.contains("[HOLDS]") && l.contains("unchanged since"))
}

/// The ids recorded in the local cache (empty when absent or unreadable).
fn cached(dir: &Path) -> Vec<String> {
    let v: serde_json::Value = match serde_json::from_str(&read(dir, CACHE)) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    v.get("records")
        .and_then(|r| r.as_object())
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

/// The `[[contract]]` block of `id` in invar.lock.
fn entry(dir: &Path, id: &str) -> String {
    let lock = read(dir, "invar.lock");
    lock.split("\n[[")
        .find(|b| b.starts_with("contract]]") && b.contains(&format!("\nid = \"{id}\"\n")))
        .unwrap_or("")
        .to_string()
}

fn has_held(dir: &Path, id: &str) -> bool {
    entry(dir, id).lines().any(|l| l.starts_with("held = \""))
}

/// Remove every lock record, so only the cache can supply one.
fn strip_held(dir: &Path) {
    let lock = read(dir, "invar.lock");
    let kept: Vec<&str> = lock.lines().filter(|l| !l.starts_with("held = ") && !l.starts_with("held_input = ")).collect();
    put(dir, "invar.lock", &(kept.join("\n") + "\n"));
}

const API: &str = r#"{"components":{"schemas":{"Order":{"properties":{"total":{"type":"integer"},"tax":{"type":"integer"}}},"User":{"properties":{"name":{"type":"string"}}}}}}"#;
const API_NO_TAX: &str = r#"{"components":{"schemas":{"Order":{"properties":{"total":{"type":"integer"}}},"User":{"properties":{"name":{"type":"string"}}}}}}"#;
const READS_BOTH: &str = "const t = order.total;\nconst x = order.tax;\n";
const READS_TOTAL: &str = "const t = order.total;\n";

fn orders_toml(pointer: &str) -> String {
    format!("[[invariant]]\nkind = \"schema\"\nid = \"orders\"\nproducer = \"api.json\"\npointer = \"{pointer}\"\nconsumer = \"client.js\"\n")
}

fn users_toml() -> &'static str {
    "\n[[invariant]]\nkind = \"schema\"\nid = \"users\"\nproducer = \"api.json\"\npointer = \"/components/schemas/User\"\nconsumer = \"users.js\"\n"
}

fn orders_ws(tag: &str) -> PathBuf {
    ws(tag, &[("api.json", API), ("client.js", READS_BOTH), ("invar.toml", &orders_toml("/components/schemas/Order"))])
}

fn two_ws(tag: &str) -> PathBuf {
    let toml = orders_toml("/components/schemas/Order") + users_toml();
    ws(tag, &[("api.json", API), ("client.js", READS_BOTH), ("users.js", "const n = user.name;\n"), ("invar.toml", &toml)])
}

// ------------------------------------------------------------------- F1

#[test]
fn f1_lock_then_check_reuses_and_exits_0() {
    let d = orders_ws("f1");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let held = entry(&d, "orders");
    let line = held.lines().find(|l| l.starts_with("held = \"")).unwrap_or_else(|| panic!("no held line:\n{held}"));
    let toks: Vec<&str> = line.trim_start_matches("held = \"").trim_end_matches('"').split(' ').collect();
    assert_eq!(toks.len(), 4, "held = \"<at> <engine> <block> <entry>\": {line}");
    assert!(toks[1].starts_with(&format!("{}-", env!("CARGO_PKG_VERSION"))), "engine is version-hash: {line}");
    assert!(held.contains("held_input = \"") && held.contains(" api.json\"") && held.contains(" client.js\""), "{held}");
    for _ in 0..2 {
        let (code, out) = invar(&d, &["check"]);
        assert_eq!(code, 0, "{out}");
        assert!(reused(&out, "orders") && out.contains("unchanged since lock 20"), "{out}");
        assert!(out.contains("summary: held 1 (1 reused) | violated 0"), "{out}");
    }
}

// ------------------------------------------------------------- F2 to F4

#[test]
fn f2_consumer_change_is_evaluated() {
    let d = orders_ws("f2");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    // Consumer stops reading tax, producer drops it: still HOLDS, evaluated,
    // and recorded in the cache.
    put(&d, "client.js", READS_TOTAL);
    put(&d, "api.json", API_NO_TAX);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(cached(&d).contains(&"orders".to_string()), "no cache record:\n{}", read(&d, CACHE));
    // Only the consumer changes: it reads tax again.
    put(&d, "client.js", READS_BOTH);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] orders") && !any_reuse(&out), "{out}");
}

#[test]
fn f3_producer_change_is_evaluated() {
    let d = orders_ws("f3");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let (_, out) = invar(&d, &["check"]);
    assert!(reused(&out, "orders"), "{out}");
    put(&d, "api.json", API_NO_TAX);
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] orders") && !any_reuse(&out), "{out}");
}

#[test]
fn f4_companion_change_is_evaluated() {
    let api = r#"{"components":{"schemas":{"Order":{"properties":{"amount_cents":{"type":"integer"},"currency":{"type":"string"}}}}}}"#;
    let go = "type Order struct {\n\tAmountCents int64 `json:\"amount_cents\"`\n}\n";
    let toml = "[[invariant]]\nkind = \"schema\"\nid = \"orders\"\nproducer = \"api.json\"\npointer = \"/components/schemas/Order\"\nproducer_code = \"order.go\"\nconsumer = \"client.js\"\n";
    let d = ws("f4", &[("api.json", api), ("order.go", go), ("client.js", "const a = order.amount_cents;\nconst c = order.currency;\n"), ("invar.toml", toml)]);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let held = entry(&d, "orders");
    assert_eq!(held.matches("held_input = \"").count(), 3, "producer, companion, consumer:\n{held}");
    let (_, out) = invar(&d, &["check"]);
    assert!(reused(&out, "orders"), "{out}");
    put(&d, "order.go", "type Order struct {\n\tAmountCents string `json:\"amount_cents\"`\n}\n");
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] orders") && !any_reuse(&out), "{out}");
}

// ------------------------------------------------------------- F5 to F7

#[test]
fn f5_block_change_is_evaluated() {
    let d = orders_ws("f5");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let (_, out) = invar(&d, &["check"]);
    assert!(reused(&out, "orders"), "{out}");
    put(&d, "invar.toml", &orders_toml("/components/schemas/User"));
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 3, "the existing different-producer error, never a reuse:\n{out}");
    assert!(out.contains("different producer than invar.lock") && !any_reuse(&out), "{out}");
}

#[test]
fn f6_lock_entry_edit_is_evaluated() {
    let d = orders_ws("f6");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let (_, out) = invar(&d, &["check"]);
    assert!(reused(&out, "orders"), "{out}");
    put(&d, "invar.lock", &read(&d, "invar.lock").replace("Order.total integer", "Order.total string"));
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] orders") && !any_reuse(&out), "{out}");
}

#[test]
fn f7_engine_change_is_evaluated() {
    let d = orders_ws("f7");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let lock = read(&d, "invar.lock");
    let line = lock.lines().find(|l| l.starts_with("held = \"")).unwrap_or_else(|| panic!("no held line:\n{lock}")).to_string();
    // held = "<at> <engine> <block> <entry>": the engine is the fourth token.
    let engine = line.split(' ').nth(3).unwrap().to_string();
    let edited = line.replacen(&format!(" {engine} "), " 0.0.0-other ", 1);
    put(&d, "invar.lock", &lock.replace(&line, &edited));
    assert!(read(&d, "invar.lock").contains(" 0.0.0-other "), "edit applied");
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("[HOLDS]    orders (schema)") && !any_reuse(&out), "{out}");
}

// ------------------------------------------------------------- F8, F9

#[test]
fn f8_a_waived_contract_is_never_reused() {
    use invar_falsify::lock::{format_date, today_utc};
    let d = two_ws("f8");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let waiver = format!(
        "\n[[waiver]]\ncontract = \"orders\"\nfield = \"Order.tax\"\nowner = \"nikhil\"\nreason = \"moving to tax_minor\"\nexpires = \"{}\"\n",
        format_date(today_utc() + 30)
    );
    put(&d, "invar.lock", &(read(&d, "invar.lock") + &waiver));
    for _ in 0..2 {
        let (code, out) = invar(&d, &["check"]);
        assert_eq!(code, 0, "{out}");
        assert!(reused(&out, "users"), "control: users is reused:\n{out}");
        assert!(!reused(&out, "orders"), "a waived contract is evaluated:\n{out}");
    }
    assert!(!cached(&d).contains(&"orders".to_string()), "{}", read(&d, CACHE));
    assert_eq!(invar(&d, &["lock", "--update"]).0, 0);
    assert!(has_held(&d, "users") && !has_held(&d, "orders"), "{}", read(&d, "invar.lock"));
}

#[test]
fn f9_a_sampled_contract_is_never_reused() {
    let d = two_ws("f9");
    put(&d, "payload.json", r#"{"total": 5, "tax": 1}"#);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let (code, out) = invar(&d, &["lock", "--sample", "payload.json", "--contract", "orders"]);
    assert_eq!(code, 0, "{out}");
    for _ in 0..2 {
        let (_, out) = invar(&d, &["check"]);
        assert!(reused(&out, "users"), "control: users is reused:\n{out}");
        assert!(!reused(&out, "orders"), "a sampled contract is evaluated:\n{out}");
    }
    assert!(!cached(&d).contains(&"orders".to_string()), "{}", read(&d, CACHE));
    let (_, out) = invar(&d, &["check", "--sample", "payload.json", "--contract", "orders"]);
    assert!(!any_reuse(&out), "a --sample run reuses nothing:\n{out}");
}

// ------------------------------------------------------------------ F10

#[test]
fn f10_only_holds_is_recorded() {
    let toml = orders_toml("/components/schemas/Order")
        + users_toml()
        + "\n[[invariant]]\nkind = \"schema\"\nid = \"zero\"\nproducer = \"api.json\"\npointer = \"/components/schemas/User\"\nconsumer = \"none.js\"\n";
    let d = ws("f10", &[("api.json", API), ("client.js", READS_BOTH), ("users.js", "const n = user.name;\n"), ("none.js", "const z = thing.other;\n"), ("invar.toml", &toml)]);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    assert!(has_held(&d, "orders") && has_held(&d, "users"), "{}", read(&d, "invar.lock"));
    assert!(!has_held(&d, "zero"), "0 bindings is NOT EVALUATED, never recorded:\n{}", entry(&d, "zero"));
    // orders becomes VIOLATED, and a skipped contract is added.
    put(&d, "api.json", API_NO_TAX);
    put(&d, "client.c", "int x;\n");
    put(&d, "invar.toml", &(toml + "\n[[invariant]]\nkind = \"schema\"\nid = \"skip\"\nproducer = \"api.json\"\npointer = \"/components/schemas/Order\"\nconsumer = \"client.c\"\n"));
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("[VIOLATED] orders") && out.contains("[SKIPPED]  skip") && out.contains("[NOT EVALUATED] zero"), "{out}");
    assert_eq!(cached(&d), vec!["users".to_string()], "{}", read(&d, CACHE));
}

// ------------------------------------------------------------ F11, F12

#[test]
fn f11_ci_neither_reads_nor_writes_the_cache() {
    let d = orders_ws("f11");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    strip_held(&d);
    assert_eq!(invar(&d, &["check"]).0, 0);
    assert!(cached(&d).contains(&"orders".to_string()), "control: a plain check writes the cache");
    let (_, out) = invar(&d, &["check"]);
    assert!(reused(&out, "orders") && out.contains("unchanged since last run"), "control: a plain check reads it:\n{out}");
    let before = read(&d, CACHE);
    let (code, out) = invar(&d, &["check", "--ci"]);
    assert_eq!(code, 0, "{out}");
    assert!(!any_reuse(&out), "--ci never reads the cache:\n{out}");
    assert_eq!(read(&d, CACHE), before, "--ci never writes the cache");
    std::fs::remove_dir_all(d.join(".invar")).unwrap();
    assert_eq!(invar(&d, &["check", "--ci"]).0, 0);
    assert!(!d.join(CACHE).exists(), "--ci never creates the cache");
}

#[test]
fn f12_ci_reuses_a_lock_record_only_when_its_signature_verified() {
    if Command::new("ssh-keygen").arg("-h").output().is_err() {
        eprintln!("SKIPPED f12: ssh-keygen not on PATH");
        return;
    }
    let d = orders_ws("f12");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    assert!(has_held(&d, "orders"), "{}", read(&d, "invar.lock"));
    let key = d.join("k");
    assert!(Command::new("ssh-keygen").args(["-t", "ed25519", "-N", "", "-q", "-f"]).arg(&key).status().unwrap().success());
    assert!(Command::new("ssh-keygen")
        .args(["-Y", "sign", "-n", "invar-lock", "-f"])
        .arg(&key)
        .arg(d.join("invar.lock"))
        .current_dir(&d)
        .status()
        .unwrap()
        .success());
    put(&d, "signers", &format!("release@invar {}", read(&d, "k.pub")));
    let (code, out) = invar(&d, &["check", "--ci"]);
    assert_eq!(code, 0, "{out}");
    assert!(!any_reuse(&out), "unsigned CI evaluates everything:\n{out}");
    let (code, out) = invar(&d, &["check", "--ci", "--lock-signers", "signers", "--lock-principal", "release@invar"]);
    assert_eq!(code, 0, "{out}");
    assert!(reused(&out, "orders") && out.contains("unchanged since lock"), "a verified lock record is reused:\n{out}");
}

// ------------------------------------------------------------ F13, F14

#[test]
fn f13_a_malformed_cache_is_ignored_and_rewritten() {
    let d = orders_ws("f13");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    strip_held(&d);
    for bad in ["", "{}", "not json at all", "{\"records\": 7}", "{\"records\":{\"orders\":{\"engine\":1}}}"] {
        put(&d, CACHE, bad);
        let (code, out) = invar(&d, &["check"]);
        assert_eq!(code, 0, "cache {bad:?}:\n{out}");
        assert!(out.contains("[HOLDS]    orders (schema)") && !any_reuse(&out), "cache {bad:?}:\n{out}");
        assert_eq!(cached(&d), vec!["orders".to_string()], "cache {bad:?} rewritten:\n{}", read(&d, CACHE));
    }
}

#[test]
fn f14_the_cache_ignores_itself_in_git() {
    let d = orders_ws("f14");
    assert_eq!(invar(&d, &["check"]).0, 0);
    assert!(d.join(CACHE).exists(), "the cache is written");
    assert_eq!(read(&d, ".invar/cache/.gitignore"), "*\n");
}

// ------------------------------------------------------------ F15, F16

#[test]
fn f15_a_missing_input_is_never_reused() {
    let d = orders_ws("f15");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let (_, out) = invar(&d, &["check"]);
    assert!(reused(&out, "orders"), "{out}");
    std::fs::remove_file(d.join("client.js")).unwrap();
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 3, "today's error path:\n{out}");
    assert!(!any_reuse(&out), "{out}");
}

/// Output with what reuse legitimately changes taken out: the reuse suffix,
/// the reuse count and the evaluation-only notes.
fn normal(out: &str) -> String {
    out.lines()
        .filter(|l| !l.trim_start().starts_with("note: "))
        .map(|l| match l.find(" unchanged since ") {
            Some(i) if l.contains("[HOLDS]") => l[..i].to_string(),
            _ => l.to_string(),
        })
        .map(|l| match (l.find(" ("), l.starts_with("summary: held ")) {
            (Some(i), true) => {
                let rest = &l[i..];
                let j = rest.find(") ").map(|j| j + 1).unwrap_or(0);
                format!("{}{}", &l[..i], &rest[j..])
            }
            _ => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn f16_reuse_equals_full_evaluation() {
    let api = r#"{"components":{"schemas":{"Order":{"properties":{"total":{"type":"integer"},"tax":{"type":"integer"}}},"User":{"properties":{"name":{"type":"string"},"email":{"type":"string"}}},"Item":{"properties":{"sku":{"type":"string"},"qty":{"type":"integer"}}}}}}"#;
    let mut toml = String::new();
    let mut files: Vec<(String, String)> = vec![("api.json".into(), api.into())];
    for (id, ty, reads) in [
        ("a", "Order", "const t = order.total;\n"),
        ("b", "Order", "const x = order.tax;\n"),
        ("c", "User", "const n = user.name;\n"),
        ("d", "User", "const e = user.email;\nconst n = user.name;\n"),
        ("e", "Item", "const s = item.sku;\n"),
        ("f", "Item", "const q = item.qty;\n"),
    ] {
        toml.push_str(&format!("[[invariant]]\nkind = \"schema\"\nid = \"{id}\"\nproducer = \"api.json\"\npointer = \"/components/schemas/{ty}\"\nconsumer = \"{id}.js\"\n\n"));
        files.push((format!("{id}.js"), reads.into()));
    }
    files.push(("invar.toml".into(), toml));
    let refs: Vec<(&str, &str)> = files.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let d = ws("f16", &refs);
    assert_eq!(invar(&d, &["lock"]).0, 0);
    // b's consumer moves to a field the producer never had: still evaluated.
    put(&d, "b.js", "const x = order.tax;\nconst y = order.discount;\n");
    let (rc, reuse) = invar(&d, &["check"]);
    let (fc, full) = invar(&d, &["check", "--no-reuse"]);
    assert_eq!(rc, fc, "exit codes\nreuse:\n{reuse}\nfull:\n{full}");
    assert!(reuse.contains("(5 reused)"), "five contracts unchanged since lock:\n{reuse}");
    assert!(!any_reuse(&full), "--no-reuse reuses nothing:\n{full}");
    assert_eq!(normal(&reuse), normal(&full), "reuse:\n{reuse}\nfull:\n{full}");
}

// ------------------------------------------------------------------ F17

#[test]
fn f17_no_records_means_baseline_output() {
    use invar_falsify::lock::{parse_date, today_utc};
    let d = orders_ws("f17");
    let h = blake3::hash(API.as_bytes()).to_hex().to_string();
    let lock = format!(
        "[lock]\nformat = \"1\"\n\n[[contract]]\nid = \"orders\"\nproducer = \"api.json\"\npointer = \"/components/schemas/Order\"\nblake3 = \"{h}\"\nrecorded_at = \"2026-10-01T00:00:00Z\"\nbind = \"spec 2026-10-01 Order.tax integer\"\nbind = \"spec 2026-10-01 Order.total integer\"\nmembers = \"spec Order tax total\"\n"
    );
    put(&d, "invar.lock", &lock);
    let days = today_utc() - parse_date("2026-10-01").unwrap();
    let want = format!(
        "INVAR check (1 contract(s))\n  [HOLDS]    orders (schema)\n      note: producer unchanged since lock (2026-10-01T00:00:00Z)\n  durability: held since 2026-10-01 ({days} day{})\nresult: ALL 1 contract(s) hold\nsummary: held 1 | violated 0 | skipped or not evaluated 0\n",
        if days == 1 { "" } else { "s" }
    );
    let (code, out) = invar(&d, &["check"]);
    assert_eq!(code, 0, "{out}");
    let got: String = out.lines().filter(|l| !l.starts_with("measured: ")).map(|l| format!("{l}\n")).collect();
    assert_eq!(got, want);
}

// ------------------------------------------------------------ F18 to F20

#[test]
fn f18_an_unchanged_relock_rewrites_nothing() {
    let d = orders_ws("f18");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    assert!(has_held(&d, "orders"), "{}", read(&d, "invar.lock"));
    let first = read(&d, "invar.lock");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let (code, out) = invar(&d, &["lock", "--update"]);
    assert_eq!(code, 0, "{out}");
    assert_eq!(read(&d, "invar.lock"), first, "{out}");
    let (code, out) = invar(&d, &["lock"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("invar.lock is current"), "{out}");
}

#[test]
fn f19_a_receipt_run_reuses_nothing() {
    let d = orders_ws("f19");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let (_, out) = invar(&d, &["check"]);
    assert!(reused(&out, "orders"), "control:\n{out}");
    let (code, out) = invar(&d, &["check", "--receipt", "r.json"]);
    assert_eq!(code, 0, "{out}");
    assert!(!any_reuse(&out), "evidence comes from a full evaluation:\n{out}");
}

#[test]
fn f20_a_stale_held_record_is_not_lock_drift() {
    let d = orders_ws("f20");
    assert_eq!(invar(&d, &["lock"]).0, 0);
    let first = read(&d, "invar.lock");
    put(&d, "client.js", "// totals\nconst t = order.total;\nconst x = order.tax;\n");
    let (code, out) = invar(&d, &["lock"]);
    assert_eq!(code, 0, "only the held record is stale; that is not LOCK_DRIFT:\n{out}");
    assert!(out.contains("invar.lock is current") && !out.contains("LOCK_DRIFT"), "{out}");
    assert_eq!(read(&d, "invar.lock"), first, "plain lock never writes an existing lock");
    let (code, out) = invar(&d, &["lock", "--update"]);
    assert_eq!(code, 0, "{out}");
    assert_ne!(read(&d, "invar.lock"), first, "--update refreshes the held record");
    let (_, out) = invar(&d, &["check"]);
    assert!(reused(&out, "orders") && out.contains("unchanged since lock"), "{out}");
}
