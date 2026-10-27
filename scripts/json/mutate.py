#!/usr/bin/env python3
"""JSON-SPEC r0 section F: the 16 pre-registered operators, each one exact
text replacement in the implementation, run against tests/json_test.rs
with --no-fail-fast. A kill is a failing fixture; a build failure is never
a kill. Usage (from a scratch copy of the repository root):
    python3 scripts/json/mutate.py <target dir>"""
import os, re, subprocess, sys
F = "crates/invar-falsify/src/mcp.rs"
OPS = [
 ("J-M1", "j_f03", '.is_some_and(|m| rs1_writes(m).is_some())', '.is_some_and(|_| false)'),
 ("J-M2", "j_f04", 'if kind == "lock_order" {\n        "lock-order"', 'if false {\n        "lock-order"'),
 ("J-M3", "j_f02", '    } else {\n        "retyped"\n    }', '    } else {\n        "absent"\n    }'),
 ("J-M4", "j_f03", 'out.extend(lines.into_iter().map(|l| json!({"file": name, "line": l, "access": "write"})));', 'let _ = (name, lines);'),
 ("J-M5", "j_f04", 'let access = if kind == "lock_order" { "trace" } else { "read" };', 'let access = "read";'),
 ("J-M6", "j_f01", '.filter(|n| p == n.as_str() ||', '.filter(|n| false && p == n.as_str() ||'),
 ("J-M7", "j_f01", 'match w.missing_from.as_deref().and_then(crate::check::rename_candidate)', 'match None::<(&str, &str, &str)>'),
 ("J-M8", "j_f06", 'if let Some(code) = text.rsplit_once("[not-evaluated:").and_then(|(_, t)| t.strip_suffix(\']\'))', 'if let Some(code) = None::<&str>'),
 ("J-M9", "j_f07", 'text.ends_with(", not evaluated (pattern not read)")', 'false'),
 ("J-M10", "j_f08", 'text.contains(" moved to ") && text.contains("; consumer read at ")', 'false && text.contains("; consumer read at ")'),
 ("J-M11", "j_f09", 'text.starts_with("the lock binds no field of this contract (0 bindings)")', 'false'),
 ("J-M12", "j_f10", 'text.starts_with("language not supported (")', 'false'),
 ("J-M13", "j_f11", 'text.starts_with("not evaluated: no invar.lock entry")', 'false'),
 ("J-M14", "j_f12", '"exit_code": crate::receipt::contracts_exit(&outcomes),', '"exit_code": if violated > 0 { 1 } else { 0 },'),
 ("J-M15", "j_f06", 'json!({"reason": reason_code(u), "text": u})', 'json!({"reason": reason_code(u), "text": u.rsplit_once(" [not-evaluated:").map_or(u.as_str(), |(t, _)| t)})'),
 ("J-M16", "j_f13", 'json!(o.unevaluated.iter().map(', 'json!(o.unevaluated.iter().filter(|_| !o.violated()).map('),
]
target = sys.argv[1]
orig = open(F).read()
rows = []
try:
    for op, killer, old, new in OPS:
        if orig.count(old) != 1:
            rows.append((op, killer, "UNAPPLIED", "", "")); continue
        open(F, "w").write(orig.replace(old, new))
        r = subprocess.run(["cargo", "test", "--release", "--test", "json_test", "--no-fail-fast"],
                           capture_output=True, text=True, env=dict(os.environ, CARGO_TARGET_DIR=target))
        out = r.stdout + r.stderr
        if "test result" not in out:
            rows.append((op, killer, "BUILD-FAILED", "", "")); continue
        failed = sorted(set(re.findall(r"^test (j_f\d+)\S* \.\.\. FAILED", out, re.M)))
        status = "KILLED" if killer in failed else ("SURVIVED" if not failed else "KILLED-NOT-BY-DECLARED")
        rows.append((op, killer, status, ",".join(failed), ""))
        print(op, status, failed, flush=True)
finally:
    open(F, "w").write(orig)
for row in rows:
    print("\t".join(row))
