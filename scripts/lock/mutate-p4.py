#!/usr/bin/env python3
"""LOCK-VERIFIED-SPEC section E: the 9 pre-registered P4 operators, each one
exact text replacement, run against tests/lock_verified_test.rs with
--no-fail-fast. A kill is a failing fixture; a build failure is never a
kill. Usage (from a scratch copy of the repository root):
    python3 scripts/lock/mutate-p4.py <target dir>"""
import os, re, subprocess, sys
MCP = "crates/invar-falsify/src/mcp.rs"
MAIN = "crates/invar-falsify/src/main.rs"
VERIFY = '    if !crate::receipt::verify_ssh(&bytes, &sig, &s.file, &s.principal, "invar-lock")? {'
OPS = [
 ("P4-M1", "p4_f02", MCP, 'Some(l) => (Some(l), "unsigned"),', 'Some(l) => (Some(l), "verified"),'),
 ("P4-M2", "p4_f01", MCP, 'None => (None, "absent"),', 'None => (None, "unsigned"),'),
 ("P4-M3", "p4_f06", MCP, VERIFY, VERIFY.replace("if !", "if false && !")),
 ("P4-M4", "p4_f08", MCP, VERIFY, '    static ONCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);\n'
     + VERIFY.replace("if !", "if !ONCE.swap(true, std::sync::atomic::Ordering::SeqCst) && !")),
 ("P4-M5", "p4_f05", MCP, 'return Err(anyhow!("lock not confirmed: {} is missing", sig.display()));',
     'return Ok((load_lock(config)?, "unsigned"));'),
 ("P4-M6", "p4_f09", MAIN, '(Some(_), None) => anyhow::bail!("--lock-signers needs --lock-principal"),', '(Some(_), None) => None,'),
 ("P4-M7", "p4_f04", MCP, '    let text = String::from_utf8(bytes)',
     '    println!("lock signature ok ({})", s.principal);\n    let text = String::from_utf8(bytes)'),
 ("P4-M8", "p4_f02", MCP, '"days_remaining": w.days_left(self.today),\n            })).collect::<Vec<_>>(),\n            "lock_state": lock_state,\n            "lock_verified": lock_state == "verified",\n',
     '"days_remaining": w.days_left(self.today),\n            })).collect::<Vec<_>>(),\n'),
 ("P4-M9", "p4_f03", MCP, 'Some(l) => (Some(l), "unsigned"),',
     'Some(l) => (Some(l), if base_dir(config).join("invar.lock.sig").exists() { "verified" } else { "unsigned" }),'),
]
target = sys.argv[1]
rows = []
for op, killer, f, old, new in OPS:
    orig = open(f).read()
    if orig.count(old) != 1:
        rows.append((op, killer, "UNAPPLIED", "")); continue
    try:
        open(f, "w").write(orig.replace(old, new))
        r = subprocess.run(["cargo", "test", "-p", "invar-falsify", "--test", "lock_verified_test", "--no-fail-fast"],
                           capture_output=True, text=True, env=dict(os.environ, CARGO_TARGET_DIR=target))
    finally:
        open(f, "w").write(orig)
    out = r.stdout + r.stderr
    if "test result" not in out:
        rows.append((op, killer, "BUILD-FAILED", "")); continue
    failed = sorted(set(re.findall(r"^test (p4_f\d+)\S* \.\.\. FAILED", out, re.M)))
    status = "KILLED" if killer in failed else ("SURVIVED" if not failed else "KILLED-NOT-BY-DECLARED")
    rows.append((op, killer, status, ",".join(failed)))
    print(op, status, failed, flush=True)
for row in rows:
    print("\t".join(row))
