#!/usr/bin/env python3
"""SIBLING-REPORT-SPEC r0 section E: the 10 pre-registered SIB-2 operators,
each one exact text replacement, run against tests/sibling_report_test.rs
with --no-fail-fast. A kill is a failing fixture; a build failure is never a
kill. Usage (from a scratch copy of the repository root):
    python3 scripts/sibling/mutate-s2.py <target dir>"""
import os, re, subprocess, sys
D = "crates/invar-falsify/src/dependents.rs"
M = "crates/invar-falsify/src/mcp.rs"
OPS = [
 ("S2-M1", "s2_f01", D, '"broken": broken.iter().map(|b| {', '"broken": broken.iter().take(0).map(|b| {'),
 ("S2-M2", "s2_f02", D, 'v["now_type"] = json!(now);', 'let _ = now;'),
 ("S2-M3", "s2_f03", D, 'Consumer::NotEvaluated { lockfile, reason, text } => {\n                json!({"lockfile": lockfile, "state": "not_evaluated", "reason": reason, "text": text})',
     'Consumer::NotEvaluated { lockfile, reason, text } => {\n                json!({"lockfile": lockfile, "state": "not_evaluated", "reason": reason, "text": text, "contracts": [{"broken": [{"field": "Limits.maxLines"}]}]})'),
 ("S2-M4", "s2_f04", M, 'self.dependents_signers.as_deref(),', 'self.dependents_signers.as_deref().or(Some(Path::new("signers"))),'),
 ("S2-M5", "s2_f06", M, '        if !inside {\n            return Err(anyhow!("locks path is outside the configuration directory: {locks}"));\n        }',
     '        if !inside && query.is_some() {\n            return Err(anyhow!("locks path is outside the configuration directory: {locks}"));\n        }'),
 ("S2-M6", "s2_f07", D, '"broken": e.broken,\n            "provided": e.provided,', '"broken": e.broken + e.provided,\n            "provided": e.provided,'),
 ("S2-M7", "s2_f09", M, '}, "required": ["locks"], "additionalProperties": false},', '}, "required": ["query", "locks"], "additionalProperties": false},'),
 ("S2-M8", "s2_f10", D, 'let mut md = format!("{DEPENDENTS_MARKER}\\n### INVAR', 'let mut md = format!("\\n### INVAR'),
 ("S2-M9", "s2_f11", D, '"| {} | {} | {} | {} | {} |\\n",\n                                    code(lockfile),\n                                    code(principal),\n                                    code(&k.id),',
     '"| {} | {} | {} | {} | {} |\\n",\n                                    code(lockfile),\n                                    code(principal),\n                                    k.id,'),
 ("S2-M10", "s2_f13", "action/dependents.sh", "fi\nexit 0\n", "fi\nexit \"$code\"\n"),
]
target = sys.argv[1]
rows = []
for op, killer, f, old, new in OPS:
    orig = open(f).read()
    if orig.count(old) != 1:
        rows.append((op, killer, "UNAPPLIED", "")); continue
    try:
        open(f, "w").write(orig.replace(old, new))
        r = subprocess.run(["cargo", "test", "-p", "invar-falsify", "--test", "sibling_report_test", "--no-fail-fast"],
                           capture_output=True, text=True, env=dict(os.environ, CARGO_TARGET_DIR=target))
    finally:
        open(f, "w").write(orig)
    out = r.stdout + r.stderr
    if "test result" not in out:
        rows.append((op, killer, "BUILD-FAILED", "")); continue
    failed = sorted(set(re.findall(r"^test (s2_f\d+)\S* \.\.\. FAILED", out, re.M)))
    status = "KILLED" if killer in failed else ("SURVIVED" if not failed else "KILLED-NOT-BY-DECLARED")
    rows.append((op, killer, status, ",".join(failed)))
    print(op, status, failed, flush=True)
for row in rows:
    print("\t".join(row))
