//! SECURITY S2 fixtures (docs/falsification/SECURITY-SPEC.md r1, sections H
//! and I), frozen BEFORE the S2 implementation. SEC-D4: no control or
//! bidirectional-override character from a repository reaches a terminal
//! raw, and a repository value cannot forge an output line. SEC-D5: the
//! lock walk never follows symbolic links, is bounded in depth and entries,
//! and the MCP `dependents` tool reads only inside the configuration's
//! directory.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let d = std::env::temp_dir().join(format!("invar_s2_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run(d: &Path, args: &[&str]) -> (i32, Vec<u8>) {
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(d).env("HOME", d).output().unwrap();
    let mut out = o.stdout.clone();
    out.extend_from_slice(&o.stderr);
    (o.status.code().unwrap_or(-1), out)
}

/// Bytes a terminal would act on: C0 controls other than newline and tab,
/// DEL, C1 controls (as UTF-8), and the bidirectional overrides/isolates.
fn terminal_active(out: &[u8]) -> Vec<String> {
    let s = String::from_utf8_lossy(out);
    s.chars()
        .filter(|&c| {
            (c.is_control() && c != '\n' && c != '\t')
                || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
        .map(|c| format!("U+{:04X}", c as u32))
        .collect()
}

/// A JSON-schema producer that drops `evil` at head; the consumer reads it.
fn violated(d: &Path, evil: &str) {
    let base = serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, evil: {"type": "string"}}});
    let head = serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}});
    std::fs::write(d.join("base.json"), base.to_string()).unwrap();
    std::fs::write(d.join("head.json"), head.to_string()).unwrap();
    // The raw characters inside a single-quoted literal: INVAR reads the
    // literal's bytes and does not decode escapes.
    std::fs::write(d.join("c.js"), format!("const w = payload['{evil}'];\n")).unwrap();
    std::fs::write(d.join("invar.toml"), "[[invariant]]\nkind = \"schema\"\nid = \"t\"\nproducer = \"head.json\"\nproducer_base = \"base.json\"\nconsumer = \"c.js\"\n").unwrap();
}

const EVIL: &str = "x\u{1b}]0;pwned\u{7}\u{1b}[2J\rfake";

// ------------------------------------------------------------- SEC-D4

#[test]
fn s2_f01_control_bytes_in_a_witness_are_escaped() {
    let d = ws("f01");
    violated(&d, EVIL);
    let (code, out) = run(&d, &["check", "--no-reuse"]);
    assert_eq!(code, 1, "{}", String::from_utf8_lossy(&out));
    assert!(terminal_active(&out).is_empty(), "raw: {:?}\n{}", terminal_active(&out), String::from_utf8_lossy(&out));
    assert!(String::from_utf8_lossy(&out).contains("x\\u{1b}]0;pwned\\u{7}\\u{1b}[2J\\rfake read at c.js:1"), "{}", String::from_utf8_lossy(&out));
}

#[test]
fn s2_f02_a_newline_in_a_value_cannot_forge_a_line() {
    // No source-file read carries a raw newline into a witness, so the
    // renderer's contract is driven directly: every repository value it
    // prints stays on its own line.
    use invar_falsify::check::{outcome_lines, ContractOutcome, Witness};
    let forged = "x\n  [HOLDS]    forged (schema)";
    let o = ContractOutcome {
        id: "t\n  [HOLDS]    forged-id (schema)".to_string(),
        kind: "schema".to_string(),
        residue: vec![forged.to_string()],
        provenance: Vec::new(),
        witnesses: vec![Witness { residue: forged.to_string(), sites: vec!["c.js:1".to_string()], missing_from: Some("head\n  [HOLDS]    forged-where".to_string()), type_change: None, read_as: Vec::new() }],
        skipped: None,
        waived: Vec::new(),
        expired: Vec::new(),
        notes: vec!["n\n  [HOLDS]    forged-note".to_string()],
        unevaluated: vec!["u\n  [HOLDS]    forged-statement".to_string()],
        reused: None,
    };
    let (_, text) = outcome_lines(&o);
    assert!(!text.lines().any(|l| l.trim_start().starts_with("[HOLDS]")), "{text}");
    assert!(text.contains("\\n"), "the newline is shown escaped: {text}");
}

#[test]
fn s2_f03_a_contract_id_cannot_inject_on_a_holding_run() {
    let d = ws("f03");
    std::fs::write(d.join("head.yaml"), "properties:\n  id: {type: string}\n").unwrap();
    std::fs::write(d.join("c.py"), "x = d[\"id\"]\n").unwrap();
    // A raw ESC in the id (the config parser does not decode escapes).
    std::fs::write(d.join("invar.toml"), "[[invariant]]\nkind = \"schema\"\nid = \"t\u{1b}[31mred\"\nproducer = \"head.yaml\"\nproducer_base = \"head.yaml\"\nconsumer = \"c.py\"\n").unwrap();
    let (code, out) = run(&d, &["check", "--no-reuse"]);
    assert_eq!(code, 0, "{}", String::from_utf8_lossy(&out));
    assert!(terminal_active(&out).is_empty(), "{:?}", terminal_active(&out));
}

#[test]
fn s2_f04_a_bidirectional_override_is_escaped() {
    let d = ws("f04");
    violated(&d, "abc\u{202e}fed");
    let (_, out) = run(&d, &["check", "--no-reuse"]);
    assert!(terminal_active(&out).is_empty(), "{:?}", terminal_active(&out));
    assert!(String::from_utf8_lossy(&out).contains("abc\\u{202e}fed"), "{}", String::from_utf8_lossy(&out));
}

#[test]
fn s2_f05_lock_output_is_neutralized() {
    let d = ws("f05");
    violated(&d, EVIL);
    let toml = std::fs::read_to_string(d.join("invar.toml")).unwrap().replace("id = \"t\"", "id = \"t\u{1b}[31m\"");
    std::fs::write(d.join("invar.toml"), toml).unwrap();
    std::fs::copy(d.join("base.json"), d.join("head.json")).unwrap();
    let (_, out) = run(&d, &["lock"]);
    assert!(terminal_active(&out).is_empty(), "{:?}\n{}", terminal_active(&out), String::from_utf8_lossy(&out));
}

#[test]
fn s2_f06_dependents_output_is_neutralized() {
    // A lock binding the hostile name, then a query for it: the walk reads
    // the lock and echoes both the query and the bound field.
    let d = ws("f06");
    violated(&d, EVIL);
    std::fs::copy(d.join("base.json"), d.join("head.json")).unwrap();
    assert_eq!(run(&d, &["lock"]).0, 0);
    let (_, out) = run(&d, &["dependents", EVIL, "--locks", "."]);
    assert!(terminal_active(&out).is_empty(), "{:?}\n{}", terminal_active(&out), String::from_utf8_lossy(&out));
}

#[test]
fn s2_f07_ordinary_output_is_unchanged() {
    // A witness with nothing to escape prints exactly as before S2.
    let d = ws("f07");
    violated(&d, "legacy_name");
    let (code, out) = run(&d, &["check", "--no-reuse"]);
    assert_eq!(code, 1);
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("  [VIOLATED] t (schema): [\"legacy_name\"]\n      legacy_name read at c.js:1 · not in head.json\n"), "{text}");
}

#[test]
fn s2_f08_the_markdown_report_carries_no_terminal_active_character() {
    let d = ws("f08");
    violated(&d, "abc\u{202e}fed\u{1b}[2J");
    let (_, out) = run(&d, &["check", "--no-reuse", "--report-md", "r.md"]);
    let md = std::fs::read(d.join("r.md")).unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(&out)));
    assert!(terminal_active(&md).is_empty(), "{:?}", terminal_active(&md));
}

// ------------------------------------------------------------- SEC-D5

fn link_loop(dir: &Path) {
    std::fs::create_dir_all(dir.join("a")).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("..", dir.join("a/up")).unwrap();
        std::os::unix::fs::symlink("..", dir.join("a/up2")).unwrap();
        std::os::unix::fs::symlink(".", dir.join("a/self")).unwrap();
    }
}

fn timed(d: &Path, args: &[&str]) -> (i32, Duration) {
    let t = Instant::now();
    let mut c = Command::new(env!("CARGO_BIN_EXE_invar")).args(args).current_dir(d).env("HOME", d).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
    loop {
        if let Some(st) = c.try_wait().unwrap() {
            return (st.code().unwrap_or(-1), t.elapsed());
        }
        if t.elapsed() > Duration::from_secs(10) {
            let _ = c.kill();
            return (-9, t.elapsed());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(unix)]
#[test]
fn s2_f09_dependents_over_a_link_loop_returns() {
    let d = ws("f09");
    link_loop(&d.join("locks"));
    let (code, dt) = timed(&d, &["dependents", "Order.total", "--locks", "locks"]);
    assert_ne!(code, -9, "still running after 10 s");
    assert!(dt < Duration::from_secs(5), "{dt:?}");
}

#[cfg(unix)]
#[test]
fn s2_f10_a_repository_config_cannot_hang_check() {
    let d = ws("f10");
    std::fs::write(d.join("head.yaml"), "properties:\n  id: {type: string}\n").unwrap();
    std::fs::write(d.join("c.py"), "x = d[\"id\"]\n").unwrap();
    std::fs::write(d.join("invar.toml"), "[dependents]\nlocks = \"deps\"\n\n[[invariant]]\nkind = \"schema\"\nid = \"s\"\nproducer = \"head.yaml\"\nproducer_base = \"head.yaml\"\nconsumer = \"c.py\"\n").unwrap();
    link_loop(&d.join("deps"));
    let (code, dt) = timed(&d, &["check"]);
    assert_eq!(code, 0, "the verdict is unchanged ({dt:?})");
    assert!(dt < Duration::from_secs(5), "{dt:?}");
}

#[cfg(unix)]
#[test]
fn s2_f11_a_lock_reachable_only_through_a_link_is_not_read() {
    let d = ws("f11");
    let real = d.join("elsewhere/team");
    std::fs::create_dir_all(&real).unwrap();
    std::fs::write(real.join("invar.lock"), "[lock]\nformat = \"1\"\n").unwrap();
    std::fs::create_dir_all(d.join("locks")).unwrap();
    std::os::unix::fs::symlink(d.join("elsewhere"), d.join("locks/linked")).unwrap();
    let found = invar_falsify::dependents::locks_under(&d.join("locks"), 64, 200_000).unwrap();
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn s2_f12_the_walk_is_bounded_in_depth_and_entries() {
    let d = ws("f12");
    let mut p = d.join("locks");
    for i in 0..8 {
        p = p.join(format!("d{i}"));
    }
    std::fs::create_dir_all(&p).unwrap();
    std::fs::write(p.join("invar.lock"), "[lock]\nformat = \"1\"\n").unwrap();
    for i in 0..30 {
        std::fs::write(d.join("locks").join(format!("f{i}.txt")), "").unwrap();
    }
    assert_eq!(invar_falsify::dependents::locks_under(&d.join("locks"), 64, 200_000).unwrap().len(), 1, "within bounds: found");
    let e = invar_falsify::dependents::locks_under(&d.join("locks"), 4, 200_000).unwrap_err();
    assert!(e.contains("depth"), "{e}");
    let e = invar_falsify::dependents::locks_under(&d.join("locks"), 64, 10).unwrap_err();
    assert!(e.contains("entries"), "{e}");
}

#[test]
fn s2_f13_mcp_dependents_reads_only_inside_the_configuration_directory() {
    let d = ws("f13");
    let cfg = d.join("repo");
    std::fs::create_dir_all(cfg.join("locks")).unwrap();
    std::fs::write(cfg.join("invar.toml"), "").unwrap();
    let outside = d.join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_invar"))
        .args(["mcp", "--offline"])
        .current_dir(&cfg)
        .env("HOME", &d)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let msg = |id: u32, locks: &str| {
        format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"tools/call\",\"params\":{{\"name\":\"dependents\",\"arguments\":{{\"query\":\"Order.total\",\"locks\":{}}}}}}}\n",
            serde_json::to_string(locks).unwrap()
        )
    };
    let input = format!(
        "{}\n{}{}{}",
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#,
        msg(2, outside.to_str().unwrap()),
        msg(3, "../outside"),
        msg(4, "locks")
    );
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    let replies: Vec<serde_json::Value> = String::from_utf8(out.stdout).unwrap().lines().map(|s| serde_json::from_str(s).unwrap()).collect();
    let by_id = |id: u64| replies.iter().find(|r| r["id"] == id).cloned().unwrap_or_default().to_string();
    assert!(by_id(2).contains("outside the configuration directory"), "absolute path outside: {}", by_id(2));
    assert!(by_id(3).contains("outside the configuration directory"), "relative escape: {}", by_id(3));
    assert!(!by_id(4).contains("outside the configuration directory"), "inside is allowed: {}", by_id(4));
}

// ------------------------------------------------- r2 (before any S2 code)

#[test]
fn s2_f14_an_unreadable_lock_path_is_neutralized() {
    // `dependents.rs` prints only for a lock it cannot read ("skipping
    // <path>: <error>"); the path is repository-controlled. (Kills S2-M9:
    // s2_f06's output goes through main.rs, not these prints.)
    let d = ws("f14");
    let bad = d.join("locks").join("t\u{1b}]0;pwned\u{7}\u{202e}x");
    std::fs::create_dir_all(&bad).unwrap();
    std::fs::write(bad.join("invar.lock"), "this is not a lock\n").unwrap();
    let ok = d.join("locks/ok");
    std::fs::create_dir_all(&ok).unwrap();
    std::fs::write(ok.join("invar.lock"), "[lock]\nformat = \"1\"\n").unwrap();
    let (_, out) = run(&d, &["dependents", "Order.total", "--locks", "locks"]);
    assert!(String::from_utf8_lossy(&out).contains("skipping"), "the unreadable lock is reported:\n{}", String::from_utf8_lossy(&out));
    assert!(terminal_active(&out).is_empty(), "{:?}\n{}", terminal_active(&out), String::from_utf8_lossy(&out));
}
