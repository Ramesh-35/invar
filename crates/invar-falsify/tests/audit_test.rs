//! AUDIT fixtures (AGENT-1 P2, docs/falsification/AUDIT-SPEC.md r0), frozen
//! with the spec BEFORE any implementation. A correct engine never builds a
//! bad witness, so each rule is exercised on a constructed witness through
//! `audit_witness`, and one CLI fixture shows a real conviction is untouched.

use invar_falsify::check::{audit_witness, Witness};
use std::process::Command;

fn w(residue: &str, sites: &[&str], missing_from: Option<&str>) -> Witness {
    Witness {
        residue: residue.to_string(),
        sites: sites.iter().map(|s| s.to_string()).collect(),
        missing_from: missing_from.map(|s| s.to_string()),
        type_change: None,
        read_as: Vec::new(),
    }
}

const CONSUMER: &str = "def f(data):\n    a = data[\"legacy\"]\n    b = data.size\n    m = metadata\n    return a, b, m\n";

fn producers() -> Vec<String> {
    vec!["head.yaml".to_string(), "base.yaml".to_string()]
}

fn audit(x: &Witness) -> Result<(), String> {
    audit_witness(x, "client.py", CONSUMER, &producers())
}

#[test]
fn au_f01_a_witness_whose_site_holds_the_field_passes() {
    assert_eq!(audit(&w("size", &["client.py:3"], Some("head.yaml"))), Ok(()));
}

#[test]
fn au_f02_a_site_past_the_end_fails() {
    let e = audit(&w("size", &["client.py:99"], Some("head.yaml"))).unwrap_err();
    assert!(e.contains("client.py:99"), "{e}");
}

#[test]
fn au_f03_a_site_whose_line_lacks_the_field_fails() {
    let e = audit(&w("size", &["client.py:2"], Some("head.yaml"))).unwrap_err();
    assert!(e.contains("client.py:2"), "{e}");
}

#[test]
fn au_f04_a_declared_read_as_form_is_accepted() {
    let mut x = w("Size", &["client.py:3"], Some("head.yaml"));
    x.read_as = vec!["size".to_string()];
    assert_eq!(audit(&x), Ok(()));
}

#[test]
fn au_f05_a_substring_is_not_a_token() {
    // Line 4 holds `metadata`, which contains `data` only as a substring.
    let e = audit(&w("meta", &["client.py:4"], Some("head.yaml"))).unwrap_err();
    assert!(e.contains("client.py:4"), "{e}");
}

#[test]
fn au_f06_a_site_naming_another_file_fails() {
    let e = audit(&w("size", &["other.py:3"], Some("head.yaml"))).unwrap_err();
    assert!(e.contains("other.py:3"), "{e}");
}

#[test]
fn au_f07_a_string_literal_token_counts() {
    assert_eq!(audit(&w("legacy", &["client.py:2"], Some("head.yaml"))), Ok(()));
}

#[test]
fn au_f08_missing_from_naming_no_bound_producer_fails() {
    let e = audit(&w("size", &["client.py:3"], Some("elsewhere.yaml"))).unwrap_err();
    assert!(e.contains("elsewhere.yaml"), "{e}");
}

#[test]
fn au_f09_a_type_change_in_no_bound_producer_fails() {
    let mut x = w("size", &["client.py:3"], None);
    x.type_change = Some(("string".to_string(), "integer".to_string(), "elsewhere.yaml".to_string()));
    let e = audit(&x).unwrap_err();
    assert!(e.contains("elsewhere.yaml"), "{e}");
    x.type_change = Some(("string".to_string(), "integer".to_string(), "head.yaml".to_string()));
    assert_eq!(audit(&x), Ok(()));
}

const RS1: &str = "head.yaml at head: no inbound read and no literal of legacy remain (inbound read at base.yaml:6 at base); written at client.py:2";

#[test]
fn au_f10_an_rs1_witness_whose_write_lines_hold_the_key_passes() {
    assert_eq!(audit(&w("legacy", &[], Some(RS1))), Ok(()));
}

#[test]
fn au_f11_an_rs1_write_line_lacking_the_key_fails() {
    let bad = RS1.replace("written at client.py:2", "written at client.py:2, 3");
    let e = audit(&w("legacy", &[], Some(&bad))).unwrap_err();
    assert!(e.contains("client.py:3"), "{e}");
}

#[test]
fn au_f12_a_real_conviction_is_untouched() {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_au_f12_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("base.yaml"), "properties:\n  id: {type: string}\n  legacy: {type: string}\n").unwrap();
    std::fs::write(dir.join("head.yaml"), "properties:\n  id: {type: string}\n").unwrap();
    std::fs::write(dir.join("client.py"), CONSUMER).unwrap();
    std::fs::write(dir.join("invar.toml"), "[[invariant]]\nkind = \"schema\"\nid = \"s\"\nproducer = \"head.yaml\"\nproducer_base = \"base.yaml\"\nconsumer = \"client.py\"\n").unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(["check", "--no-reuse"]).current_dir(&dir).output().unwrap();
    let out = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    assert_eq!(o.status.code(), Some(1), "{out}");
    assert!(out.contains("legacy read at client.py:2 · not in head.yaml"), "{out}");
    assert!(!out.contains("self-audit"), "{out}");
}
