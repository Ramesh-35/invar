#!/usr/bin/env python3
"""HOOK-SPEC r0 section E: the 9 pre-registered HOOK-1 operators, each one
exact text replacement, run against tests/hook_test.rs with --no-fail-fast.
A kill is a failing fixture; a build failure is never a kill. Usage (from a
scratch copy of the repository root):
    python3 scripts/hook/mutate-hook.py <target dir>"""
import os, re, subprocess, sys
M = "crates/invar-falsify/src/main.rs"
OPS = [
 ("HK-M1", "hk_f02", M, "let violated: Vec<_> = outcomes.iter().filter(|o| o.violated()).collect();",
     "let violated: Vec<_> = outcomes.iter().filter(|o| false && o.violated()).collect();"),
 ("HK-M2", "hk_f02", M, 'eprint!("{}", invar_falsify::term::neutralize(&text));', "let _ = text;"),
 ("HK-M3", "hk_f03", M, '    if !matches!(tool, "Write" | "Edit" | "MultiEdit" | "NotebookEdit") {\n        return Ok(0);\n    }\n',
     "    let _ = tool;\n"),
 ("HK-M4", "hk_f04", M, "    if !config.exists() {\n        return Ok(0);\n    }\n", ""),
 ("HK-M5", "hk_f05", M, 'serde_json::from_str(&input).map_err(|e| anyhow::anyhow!("hook event is not JSON: {e}"))?;',
     'match serde_json::from_str(&input) { Ok(v) => v, Err(_) => { eprintln!("bad event"); return Ok(2); } };'),
 ("HK-M6", "hk_f11", M, "let sections = invar_falsify::config::parse_sections(&config.display().to_string(), &raw)?;",
     'let sections = match invar_falsify::config::parse_sections(&config.display().to_string(), &raw) { Ok(s) => s, Err(e) => { eprintln!("{e}"); return Ok(2); } };'),
 ("HK-M10", "hk_f06", M, "let outcomes = evaluate_with(&sections, &resolver, lock_file.as_ref(), lock::today_utc())?;",
     'let outcomes = match evaluate_with(&sections, &resolver, lock_file.as_ref(), lock::today_utc()) { Ok(o) => o, Err(e) => { eprintln!("{e}"); return Ok(2); } };'),
 ("HK-M7", "hk_f07", M, '                    "matcher": "Write|Edit|MultiEdit|NotebookEdit",\n', ""),
 ("HK-M8", "hk_f08", M, "    if violated.is_empty() {\n        return Ok(0);\n    }\n",
     "    if violated.is_empty() {\n        return Ok(if outcomes.iter().any(|o| o.skipped.is_some() || !o.unevaluated.is_empty()) { 2 } else { 0 });\n    }\n"),
 ("HK-M9", "hk_f09", M, 'let cwd = event\n        .get("cwd")\n        .and_then(|c| c.as_str())\n        .map(PathBuf::from)\n        .unwrap_or_else(|| PathBuf::from("."));',
     'let cwd = PathBuf::from(".");\n    let _ = &event;'),
]
target = sys.argv[1]
rows = []
for op, killer, f, old, new in OPS:
    orig = open(f).read()
    if orig.count(old) != 1:
        rows.append((op, killer, "UNAPPLIED", "")); continue
    try:
        open(f, "w").write(orig.replace(old, new))
        r = subprocess.run(["cargo", "test", "-p", "invar-falsify", "--test", "hook_test", "--no-fail-fast"],
                           capture_output=True, text=True, env=dict(os.environ, CARGO_TARGET_DIR=target))
    finally:
        open(f, "w").write(orig)
    out = r.stdout + r.stderr
    if "test result" not in out:
        rows.append((op, killer, "BUILD-FAILED", "")); continue
    failed = sorted(set(re.findall(r"^test (hk_f\d+)\S* \.\.\. FAILED", out, re.M)))
    status = "KILLED" if killer in failed else ("SURVIVED" if not failed else "KILLED-NOT-BY-DECLARED")
    rows.append((op, killer, status, ",".join(failed)))
    print(op, status, failed, flush=True)
for row in rows:
    print("\t".join(row))
