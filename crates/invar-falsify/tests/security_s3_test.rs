//! SECURITY-1 S3 fixtures (docs/falsification/SECURITY-SPEC.md r3, sections
//! K and L), frozen with the spec BEFORE any S3 code. S3 closes threat path
//! 6 (programs INVAR runs) and starts path 7 (supply chain):
//! - S3-1: `invar run` and `invar report` execute a composite.toml
//!   `bin_path` only when the operator names the same program on the
//!   command line (`--semgrep-bin`); the tool's own name, resolved through
//!   the operator's PATH, needs no flag.
//! - S3-2: `invar lock --emit-proof` hashes the working tree without git's
//!   filters, so no filter program named by the repository runs.
//! - S3-3, S3-4: CI audits the dependency tree, and release archives are
//!   byte-reproducible.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ws(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s3_{tag}_{}_{nanos}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn put(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_in(dir: &Path, args: &[&str], path_prefix: Option<&Path>) -> (i32, String) {
    let mut c = Command::new(env!("CARGO_BIN_EXE_invar"));
    c.args(args).current_dir(dir).env("HOME", dir);
    if let Some(p) = path_prefix {
        let old = std::env::var("PATH").unwrap_or_default();
        c.env("PATH", format!("{}:{old}", p.display()));
    }
    let o = c.output().unwrap();
    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
    s.push_str(&String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), s)
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        let q = to.join(e.file_name());
        if p.is_dir() {
            copy_dir(&p, &q);
        } else {
            std::fs::copy(&p, &q).unwrap();
        }
    }
}

/// A harness workspace: the repository's corpus, rules and manifest, and a
/// script at `name` that leaves the marker `EXECUTED-<name>` when run.
fn harness(tag: &str, scripts: &[&str], composite: &str) -> PathBuf {
    let d = ws(tag);
    copy_dir(&repo().join("corpus"), &d.join("corpus"));
    copy_dir(&repo().join("rules"), &d.join("rules"));
    std::fs::copy(repo().join("corpus_manifest.json"), d.join("corpus_manifest.json")).unwrap();
    for s in scripts {
        script(&d, s);
    }
    put(&d, "composite.toml", composite);
    d
}

fn script(dir: &Path, rel: &str) {
    let marker = dir.join(format!("EXECUTED-{}", rel.replace('/', "_")));
    put(dir, rel, &format!("#!/bin/sh\ntouch '{}'\necho '{{\"results\": []}}'\n", marker.display()));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir.join(rel), std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn executed(dir: &Path, rel: &str) -> bool {
    dir.join(format!("EXECUTED-{}", rel.replace('/', "_"))).exists()
}

fn composite(bin_path: Option<&str>) -> String {
    let bp = bin_path.map(|b| format!("bin_path = \"{b}\"\n")).unwrap_or_default();
    format!("[semgrep]\nenabled = true\n{bp}rules = \"./rules/semgrep/falsification_rules.yml\"\n")
}

/// The semgrep leg of every case in run_results.json: (state, summary).
fn semgrep_legs(dir: &Path) -> Vec<(String, String)> {
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("out/run_results.json")).unwrap()).unwrap();
    v["cases"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| c["legs"].as_array().unwrap().iter().filter(|l| l["tool"] == "semgrep"))
        .map(|l| (l["state"].as_str().unwrap().to_string(), l["summary"].as_str().unwrap_or("").to_string()))
        .collect()
}

const RUN: &[&str] = &["run", "--manifest", "corpus_manifest.json", "--composite", "composite.toml", "--output", "out"];
const REPORT: &[&str] = &["report", "--manifest", "corpus_manifest.json", "--composite", "composite.toml", "--format", "json"];

// ---------------------------------------------- S3-1: composite bin_path

#[test]
fn s3_f01_run_does_not_execute_a_repository_named_binary() {
    let d = harness("f01", &["evil.sh"], &composite(Some("./evil.sh")));
    let (code, out) = run_in(&d, RUN, None);
    assert_eq!(code, 0, "{out}");
    assert!(!executed(&d, "evil.sh"), "composite.toml's bin_path ran without the operator naming it:\n{out}");
    let legs = semgrep_legs(&d);
    assert!(!legs.is_empty());
    for (state, summary) in &legs {
        assert_eq!(state, "ERROR", "a leg that did not run is never PASS or SKIPPED: {summary}");
        assert!(summary.contains("--semgrep-bin"), "the refusal names the flag that would allow it: {summary}");
    }
}

#[test]
fn s3_f02_report_does_not_execute_a_repository_named_binary() {
    let d = harness("f02", &["evil.sh"], &composite(Some("./evil.sh")));
    let (code, out) = run_in(&d, REPORT, None);
    assert_eq!(code, 0, "{out}");
    assert!(!executed(&d, "evil.sh"), "{out}");
    assert!(out.contains("--semgrep-bin"), "{out}");
}

#[test]
fn s3_f03_the_operator_naming_the_same_program_runs_it() {
    let d = harness("f03", &["tools/semgrep.sh"], &composite(Some("./tools/semgrep.sh")));
    let mut args = RUN.to_vec();
    args.extend_from_slice(&["--semgrep-bin", "./tools/semgrep.sh"]);
    let (code, out) = run_in(&d, &args, None);
    assert_eq!(code, 0, "{out}");
    assert!(executed(&d, "tools/semgrep.sh"), "the operator named it; it runs:\n{out}");
}

#[test]
fn s3_f04_a_flag_naming_a_different_program_runs_neither() {
    let d = harness("f04", &["evil.sh", "mine.sh"], &composite(Some("./evil.sh")));
    let mut args = RUN.to_vec();
    args.extend_from_slice(&["--semgrep-bin", "./mine.sh"]);
    let (code, out) = run_in(&d, &args, None);
    assert_eq!(code, 0, "{out}");
    assert!(!executed(&d, "evil.sh") && !executed(&d, "mine.sh"), "{out}");
    assert!(semgrep_legs(&d).iter().all(|(s, m)| s == "ERROR" && m.contains("--semgrep-bin")), "{out}");
}

#[test]
fn s3_f05_no_bin_path_runs_semgrep_from_the_operators_path() {
    let d = harness("f05", &["fakebin/semgrep"], &composite(None));
    let (code, out) = run_in(&d, RUN, Some(&d.join("fakebin")));
    assert_eq!(code, 0, "{out}");
    assert!(executed(&d, "fakebin/semgrep"), "unchanged: the tool's own name resolves through PATH:\n{out}");
}

#[test]
fn s3_f06_the_shipped_composite_still_runs_semgrep() {
    // The repository's own composite.toml names bin_path = "semgrep".
    let d = harness("f06", &["fakebin/semgrep"], &std::fs::read_to_string(repo().join("composite.toml")).unwrap());
    let (code, out) = run_in(&d, RUN, Some(&d.join("fakebin")));
    assert_eq!(code, 0, "{out}");
    assert!(executed(&d, "fakebin/semgrep"), "{out}");
}

// --------------------------------------------- S3-2: proof without filters

fn git(dir: &Path, args: &[&str]) {
    let o = Command::new("git")
        .args(["-c", "user.email=t@t", "-c", "user.name=t", "-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
}

const SCHEMAS: &str = "import { z } from \"zod\";\nexport const OrderResponse = z.object({\n  id: z.string(),\n  total_minor: z.number(),\n});\n";
const ROUTE: &str = "import { OrderResponse } from \"../../../../lib/schemas\";\nexport async function GET() {\n  const order = load();\n  return Response.json(OrderResponse.parse(order));\n}\n";
const PRODUCER_TOML: &str = "[[invariant]]\nkind = \"egress\"\nid = \"egress:OrderResponse\"\nproducer = \"lib/schemas.ts\"\nshape = \"OrderResponse\"\nboundary = \"app/api/v1/orders/route.ts\"\neligibility = \"versioned-path\"\nanchor = \"GET /v1/orders\"\n";

#[test]
fn s3_f07_emit_proof_runs_no_repository_named_git_filter() {
    let d = ws("f07");
    put(&d, "lib/schemas.ts", SCHEMAS);
    put(&d, "app/api/v1/orders/route.ts", ROUTE);
    put(&d, "invar.toml", PRODUCER_TOML);
    put(&d, ".gitattributes", "*.ts filter=evil\n");
    git(&d, &["init", "-q"]);
    git(&d, &["add", "-A"]);
    git(&d, &["commit", "-q", "-m", "base"]);
    let (c, out) = run_in(&d, &["lock"], None);
    assert_eq!(c, 0, "{out}");
    // The repository's own .git/config now names filter programs.
    let marker = d.join("EXECUTED-filter");
    for k in ["clean", "smudge"] {
        git(&d, &["config", &format!("filter.evil.{k}"), &format!("touch '{}'; cat", marker.display())]);
    }
    git(&d, &["config", "filter.evil.process", &format!("sh -c \"touch '{}'\"", marker.display())]);
    let (c, out) = run_in(&d, &["lock", "--emit-proof", "egress:OrderResponse"], None);
    assert!(!marker.exists(), "a filter named by .git/config ran:\n{out}");
    assert_eq!(c, 0, "the working tree equals the committed blob byte for byte, so the proof is emitted:\n{out}");
    assert!(d.join("invar.proof").exists(), "{out}");
}

// ------------------------------------------------- S3-3, S3-4: supply chain

fn sha256(p: &Path) -> String {
    let o = Command::new("sh").arg("-c").arg(format!("(sha256sum '{0}' 2>/dev/null || shasum -a 256 '{0}') | cut -d' ' -f1", p.display())).output().unwrap();
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

#[test]
fn s3_f08_release_archives_are_byte_reproducible() {
    let pack = repo().join("scripts/release/pack.py");
    assert!(pack.is_file(), "scripts/release/pack.py packages every release binary");
    let (a, b) = (ws("f08a"), ws("f08b"));
    put(&a, "bin/invar", "the same binary bytes\n");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    put(&b, "elsewhere/deeper/invar", "the same binary bytes\n");
    for (dir, rel) in [(&a, "bin/invar"), (&b, "elsewhere/deeper/invar")] {
        let o = Command::new("python3").arg(&pack).arg(dir.join(rel)).arg(dir.join("out.tar.gz")).output().unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    }
    let (ha, hb) = (sha256(&a.join("out.tar.gz")), sha256(&b.join("out.tar.gz")));
    assert_eq!(ha.len(), 64);
    assert_eq!(ha, hb, "same bytes, different path and time, same archive");
    let o = Command::new("tar").arg("-tvzf").arg(a.join("out.tar.gz")).output().unwrap();
    let listing = String::from_utf8_lossy(&o.stdout).to_string();
    assert_eq!(listing.lines().count(), 1, "{listing}");
    assert!(listing.starts_with("-rwxr-xr-x") && listing.trim_end().ends_with(" invar"), "{listing}");
    let o = Command::new("tar").arg("-xzOf").arg(a.join("out.tar.gz")).output().unwrap();
    assert_eq!(o.stdout, b"the same binary bytes\n", "the archive holds the binary unchanged");
}

#[test]
fn s3_f09_release_packages_through_the_reproducible_packer() {
    let rel = std::fs::read_to_string(repo().join(".github/workflows/release.yml")).unwrap();
    assert!(rel.contains("python3 scripts/release/pack.py"), "{rel}");
    assert!(!rel.contains("tar -czf"), "the time-stamped packager is gone:\n{rel}");
}

#[test]
fn s3_f10_ci_audits_the_dependency_tree_with_a_pinned_auditor() {
    let ci = std::fs::read_to_string(repo().join(".github/workflows/ci.yml")).unwrap();
    assert!(ci.contains("cargo install cargo-audit --locked --version "), "{ci}");
    assert!(ci.contains("cargo audit"), "{ci}");
}

#[test]
fn s3_f11_ci_rebuilds_from_a_second_path_and_compares() {
    let ci = std::fs::read_to_string(repo().join(".github/workflows/ci.yml")).unwrap();
    assert!(ci.contains("reproducible-build"), "a reproducible-build job exists:\n{ci}");
    assert!(ci.contains("scripts/release/repro-check.sh"), "{ci}");
    assert!(repo().join("scripts/release/repro-check.sh").is_file());
}
