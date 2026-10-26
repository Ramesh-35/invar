#!/usr/bin/env python3
"""SECURITY-SPEC r3 section M: the 10 pre-registered S3 operators, each one
exact text replacement, run against tests/security_s3_test.rs with
--no-fail-fast. A kill is a failing fixture; a build failure is never a
kill. Usage (from a scratch copy of the repository root):
    python3 scripts/security/mutate-s3.py <target dir>"""
import os, re, subprocess, sys
EV = "crates/invar-falsify/src/evaluator.rs"
OPS = [
 ("S3-M1", "s3_f01", EV, '(Some(b), None) => Err(refusal(b)),', '(Some(b), None) => Ok(b.to_string()),'),
 ("S3-M2", "s3_f02", "crates/invar-falsify/src/main.rs", 'fn cmd_report(args: ReportArgs) -> Result<i32> {\n    let report = build_report(\n        &args.manifest,\n        &args.composite,\n        Duration::from_secs(args.timeout),\n        args.semgrep_bin.clone(),', 'fn cmd_report(args: ReportArgs) -> Result<i32> {\n    let report = build_report(\n        &args.manifest,\n        &args.composite,\n        Duration::from_secs(args.timeout),\n        CompositeConfig::load(&args.composite).ok().and_then(|c| c.semgrep.bin_path),'),
 ("S3-M3", "s3_f04", EV, 'if b != f && b != "semgrep" => Err(refusal(b)),', 'if false && b != f => Err(refusal(b)),'),
 ("S3-M4", "s3_f03", EV, '(_, Some(f)) => Ok(f.to_string()),', '(_, Some(f)) => Err(refusal(f)),'),
 ("S3-M5", "s3_f06", EV, '(None | Some("semgrep"), None) => Ok("semgrep".to_string()),', '(None, None) => Ok("semgrep".to_string()),'),
 ("S3-M6", "s3_f07", "crates/invar-falsify/src/proof.rs", '&["hash-object", "--no-filters", &producer]', '&["hash-object", &producer]'),
 ("S3-M7", "s3_f08", "scripts/release/pack.py", '    info.mtime = 0\n', '    info.mtime = int(__import__("os").path.getmtime(binary))\n'),
 ("S3-M8", "s3_f09", ".github/workflows/release.yml", 'python3 scripts/release/pack.py "target/$TARGET/release/invar" "$asset"', 'tar -czf "$asset" -C "target/$TARGET/release" invar'),
 ("S3-M9", "s3_f10", ".github/workflows/ci.yml", '      - run: cargo install cargo-audit --locked --version 0.22.2\n      - run: cargo audit --deny warnings\n', ''),
 ("S3-M10", "s3_f11", ".github/workflows/ci.yml", '      - run: bash scripts/release/repro-check.sh "$RUNNER_TEMP/repro"\n', ''),
]
target = sys.argv[1]
rows = []
for op, killer, f, old, new in OPS:
    orig = open(f).read()
    if orig.count(old) != 1:
        rows.append((op, killer, "UNAPPLIED", "")); continue
    try:
        open(f, "w").write(orig.replace(old, new))
        r = subprocess.run(["cargo", "test", "--release", "--test", "security_s3_test", "--no-fail-fast"],
                           capture_output=True, text=True, env=dict(os.environ, CARGO_TARGET_DIR=target))
    finally:
        open(f, "w").write(orig)
    out = r.stdout + r.stderr
    if "test result" not in out:
        rows.append((op, killer, "BUILD-FAILED", "")); continue
    failed = sorted(set(re.findall(r"^test (s3_f\d+)\S* \.\.\. FAILED", out, re.M)))
    status = "KILLED" if killer in failed else ("SURVIVED" if not failed else "KILLED-NOT-BY-DECLARED")
    rows.append((op, killer, status, ",".join(failed)))
    print(op, status, failed, flush=True)
for row in rows:
    print("\t".join(row))
