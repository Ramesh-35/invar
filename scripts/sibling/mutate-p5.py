#!/usr/bin/env python3
"""SIBLING-EVAL-SPEC r1 section F: the 11 pre-registered P5 operators, each
one exact text replacement, run against tests/sibling_eval_test.rs with
--no-fail-fast. A kill is a failing fixture; a build failure is never a
kill. Usage (from a scratch copy of the repository root):
    python3 scripts/sibling/mutate-p5.py <target dir>"""
import os, re, subprocess, sys
D = "crates/invar-falsify/src/dependents.rs"
OPS = [
 ("P5-M1", "p5_f01", D, "if !crate::backtest::present(&fields, b) {", "if false && !crate::backtest::present(&fields, b) {"),
 ("P5-M2", "p5_f02", D, "(Some(was), Some(now)) if *was != now => {", "(Some(was), Some(now)) if false && *was != now => {"),
 ("P5-M3", "p5_f03", D, "if b.scope.is_empty() && scoped.contains(&name_key(&b.field)) {", "if false && b.scope.is_empty() && scoped.contains(&name_key(&b.field)) {"),
 ("P5-M4", "p5_f04", D, 'Some(_) if !sig.exists() => Err("unsigned (no invar.lock.sig)"),', 'Some(_) if !sig.exists() => Ok("unsigned".to_string()),'),
 ("P5-M5", "p5_f05", D, 'Some(s) => signer(&bytes, &sig, s)?.ok_or("signature does not verify"),', 'Some(_) => Ok("unverified".to_string()),'),
 ("P5-M6", "p5_f07", D, 'format!("{}/{}", r.owner, r.repo).eq_ignore_ascii_case(repo).then_some(r.path)', '{ let _ = repo; Some(r.path) }'),
 ("P5-M7", "p5_f08", D, 'format!("{}/{}", r.owner, r.repo).eq_ignore_ascii_case(repo).then_some(r.path)', '(format!("{}/{}", r.owner, r.repo) == repo).then_some(r.path)'),
 ("P5-M8", "p5_f09", D, 'None => Err("no signers configured (--lock-signers)"),', 'None => Ok("nobody".to_string()),'),
 ("P5-M9", "p5_f10", D, 'let text = std::fs::read_to_string(&file).map_err(|_| format!("{path} not found in this repository"))?;', 'let text = std::fs::read_to_string(&file).unwrap_or_default();'),
 ("P5-M10", "p5_f04", D, "if locks_ne + contracts_ne > 0 { 4 } else { 0 }", "0"),
 ("P5-M11", "p5_f01", D, 'report.push_str(&format!("\\n{SIBLING_NOTE}\\n"));', 'report.push_str("\\n");'),
]
target = sys.argv[1]
rows = []
for op, killer, f, old, new in OPS:
    orig = open(f).read()
    if orig.count(old) != 1:
        rows.append((op, killer, "UNAPPLIED", "")); continue
    try:
        open(f, "w").write(orig.replace(old, new))
        r = subprocess.run(["cargo", "test", "-p", "invar-falsify", "--test", "sibling_eval_test", "--no-fail-fast"],
                           capture_output=True, text=True, env=dict(os.environ, CARGO_TARGET_DIR=target))
    finally:
        open(f, "w").write(orig)
    out = r.stdout + r.stderr
    if "test result" not in out:
        rows.append((op, killer, "BUILD-FAILED", "")); continue
    failed = sorted(set(re.findall(r"^test (p5_f\d+)\S* \.\.\. FAILED", out, re.M)))
    status = "KILLED" if killer in failed else ("SURVIVED" if not failed else "KILLED-NOT-BY-DECLARED")
    rows.append((op, killer, status, ",".join(failed)))
    print(op, status, failed, flush=True)
for row in rows:
    print("\t".join(row))
