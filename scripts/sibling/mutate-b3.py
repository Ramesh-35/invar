#!/usr/bin/env python3
"""SIBLING-BASE-SPEC r0 section E: the 6 pre-registered SIB-3 operators,
each one exact text replacement, run against tests/sibling_base_test.rs with
--no-fail-fast. A kill is a failing fixture; a build failure is never a
kill. Usage (from a scratch copy of the repository root):
    python3 scripts/sibling/mutate-b3.py <target dir>"""
import os, re, subprocess, sys
SH = "action/dependents.sh"
OPS = [
 ("B3-M1", "b3_f01", SH, 'if [ -n "$base" ]; then', 'if false; then'),
 ("B3-M2", "b3_f01", SH, '      if relative "$signers"; then signers="$tree/$signers"; fi\n', ''),
 ("B3-M3", "b3_f02", SH, '      if relative "$locks"; then locks="$tree/$locks"; fi\n', ''),
 ("B3-M4", "b3_f04", SH, '      exit 0\n    fi\n  fi\nfi\n', '    fi\n  fi\nfi\n'),
 ("B3-M5", "b3_f06", SH, 'relative() { [ -n "$1" ] && [ "${1#/}" = "$1" ]; }', 'relative() { [ -n "$1" ]; }'),
 ("B3-M6", "b3_f07", "action.yml", '        INVAR_BASE_REF: ${{ inputs.base_ref }}\n        INVAR_ACTION_PATH: ${{ github.action_path }}\n      run: bash "$INVAR_ACTION_PATH/action/dependents.sh"',
     '        INVAR_ACTION_PATH: ${{ github.action_path }}\n      run: bash "$INVAR_ACTION_PATH/action/dependents.sh"'),
]
target = sys.argv[1]
rows = []
for op, killer, f, old, new in OPS:
    orig = open(f).read()
    if orig.count(old) != 1:
        rows.append((op, killer, "UNAPPLIED", "")); continue
    try:
        open(f, "w").write(orig.replace(old, new))
        r = subprocess.run(["cargo", "test", "-p", "invar-falsify", "--test", "sibling_base_test", "--no-fail-fast"],
                           capture_output=True, text=True, env=dict(os.environ, CARGO_TARGET_DIR=target))
    finally:
        open(f, "w").write(orig)
    out = r.stdout + r.stderr
    if "test result" not in out:
        rows.append((op, killer, "BUILD-FAILED", "")); continue
    failed = sorted(set(re.findall(r"^test (b3_f\d+)\S* \.\.\. FAILED", out, re.M)))
    status = "KILLED" if killer in failed else ("SURVIVED" if not failed else "KILLED-NOT-BY-DECLARED")
    rows.append((op, killer, status, ",".join(failed)))
    print(op, status, failed, flush=True)
for row in rows:
    print("\t".join(row))
