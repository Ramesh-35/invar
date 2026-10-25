//! REQUEST-SIDE r2 repair fixture (docs/falsification/REQUEST-SIDE-SPEC.md
//! r2), added under the founder's ruling of 2026-10-25 after the r1
//! mutation run: RS-M2 (RS-1 condition 2 removed) survived as a
//! non-equivalent false-conviction path. Its r1 killer, rs_f02, keeps the
//! head read as `r.FormValue("pdfFormat")`, a line that also holds the
//! literal, so condition 3 masked the mutation. The B2 JavaScript member
//! form reads the key with NO literal, so only condition 2 can see it.
//! This fixture is ADDED; tests/rs_test.rs (r1) is unchanged byte-for-byte.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_rsr2_{tag}_{}_{nanos}", std::process::id()));
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

#[test]
fn rs_f02b_a_surviving_member_form_read_prevents_conviction() {
    // Base and head both read `req.body.pdfFormat` (B2 member form, no
    // string literal anywhere in the head), and the consumer writes the
    // key. Condition 3 holds (no literal), so ONLY condition 2 (a surviving
    // head inbound read) prevents conviction. (Kills RS-M2.)
    let base = "app.post(\"/convert\", (req, res) => {\n  const fmt = req.body.pdfFormat;\n  res.json({ ok: true });\n});\n";
    let head = "app.post(\"/convert\", (req, res) => {\n  const fmt = req.body.pdfFormat;\n  log(fmt);\n  res.json({ ok: true });\n});\n";
    let consumer = "body = {}\nbody[\"pdfFormat\"] = \"PDF/A-2b\"\n";
    let toml = "[[invariant]]\nkind = \"schema\"\nid = \"rs\"\nproducer = \"head.js\"\nproducer_base = \"base.js\"\nconsumer = \"c.py\"\n";
    let d = ws("f02b", &[("head.js", head), ("base.js", base), ("c.py", consumer), ("invar.toml", toml)]);
    let (code, out) = invar(&d, &["check", "--no-reuse"]);
    assert_eq!(code, 0, "the key is still read at head (member form):\n{out}");
    assert!(!out.contains("no inbound read and no literal"), "{out}");
    assert!(!out.contains("[VIOLATED]"), "{out}");
}
