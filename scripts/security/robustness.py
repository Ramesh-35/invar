#!/usr/bin/env python3
"""SECURITY S0 probe, path 2 (pathological input): run `invar check` on
generated producer/consumer pairs (random bytes, deep nesting, huge lines,
unterminated strings and brackets, NULs, invalid UTF-8) across the source
languages INVAR reads, and report crashes (exit 101 or a signal) and runs
over a time budget. Deterministic: a fixed seed.
Usage: python3 -I scripts/security/robustness.py <invar binary> [cases]"""
import os, random, subprocess, sys, tempfile, time
binary = sys.argv[1]; n = int(sys.argv[2]) if len(sys.argv) > 2 else 300
rng = random.Random(20261025)
EXTS = ["go", "py", "ts", "js", "rs", "json", "yaml"]
def junk():
    k = rng.randrange(8)
    if k == 0: return bytes(rng.randrange(256) for _ in range(rng.randrange(1, 4000)))
    if k == 1: return ("{" * 5000 + "}" * 4999).encode()
    if k == 2: return ("x" * 200000 + "\n").encode()
    if k == 3: return ('"unterminated ' + "a" * 1000 + "\n" + "res.json({ k: " * 300).encode()
    if k == 4: return ("\x00\x1b[31m" * 500).encode()
    if k == 5: return bytes([0xff, 0xfe, 0xc3]) * 1000
    if k == 6: return "".join(rng.choice(['{', '}', '(', ')', '[', ']', '"', "'", '`', ':', ',', '\n', 'a', ' ', '\\', '#', '/']) for _ in range(20000)).encode()
    return ("type T struct {\n" + "\tF string `json:\"f\"`\n" * 3000 + "}\n").encode()
crash = slow = 0; worst = 0.0; codes = {}
for i in range(n):
    with tempfile.TemporaryDirectory() as d:
        e1, e2 = rng.choice(EXTS), rng.choice(EXTS)
        for name, ext in (("head", e1), ("base", e1)):
            open(os.path.join(d, f"{name}.{ext}"), "wb").write(junk())
        open(os.path.join(d, f"c.{e2}"), "wb").write(junk())
        open(os.path.join(d, "invar.toml"), "w").write(f'[[invariant]]\nkind = "schema"\nid = "x"\nproducer = "head.{e1}"\nproducer_base = "base.{e1}"\nconsumer = "c.{e2}"\n')
        t = time.perf_counter()
        try:
            p = subprocess.run([binary, "check", "--no-reuse"], cwd=d, capture_output=True, timeout=20)
            code = p.returncode
        except subprocess.TimeoutExpired:
            code = "timeout"
        dt = time.perf_counter() - t; worst = max(worst, dt); codes[code] = codes.get(code, 0) + 1
        if code == 101 or (isinstance(code, int) and code < 0) or code == "timeout":
            crash += 1; print(f"CRASH case {i}: exit {code} ({e1}/{e2})", p.stderr[-300:] if code != "timeout" else b"")
        elif dt > 2.0:
            slow += 1; print(f"SLOW case {i}: {dt:.2f}s ({e1}/{e2})")
print(f"cases {n}; crashes {crash}; over 2 s {slow}; slowest {worst:.2f} s; exit codes {dict(sorted(codes.items(), key=str))}")
