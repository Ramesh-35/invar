#!/usr/bin/env python3
"""SECURITY S0 probe, path 4: can repository text put raw control bytes
(ESC sequences, carriage returns) into INVAR's plain-text output, where a
terminal would execute them? A producer JSON schema drops a property whose
name carries ESC and CR; the consumer reads it by that name. Reports
whether stdout carries a raw ESC (0x1b) or CR (0x0d) byte from the repo.
Usage: python3 -I scripts/security/terminal-injection.py <invar binary>"""
import json, os, subprocess, sys, tempfile
binary = sys.argv[1]
evil = "x\x1b]0;pwned\x07\x1b[2J\rfake"
with tempfile.TemporaryDirectory() as d:
    base = {"type": "object", "properties": {"id": {"type": "string"}, evil: {"type": "string"}}}
    head = {"type": "object", "properties": {"id": {"type": "string"}}}
    json.dump(base, open(os.path.join(d, "base.json"), "w")); json.dump(head, open(os.path.join(d, "head.json"), "w"))
    open(os.path.join(d, "c.js"), "w").write("const v = payload[" + json.dumps(evil) + "];\nconst w = payload['" + evil + "'];\n")
    open(os.path.join(d, "invar.toml"), "w").write('[[invariant]]\nkind = "schema"\nid = "t"\nproducer = "head.json"\nproducer_base = "base.json"\nconsumer = "c.js"\n')
    ESC, CR, BEL = bytes([0x1b]), bytes([0x0d]), bytes([0x07])
    p = subprocess.run([binary, "check", "--no-reuse"], cwd=d, capture_output=True)
    out = p.stdout + p.stderr
    raw = [n for n, b in (("ESC", ESC), ("CR", CR), ("BEL", BEL)) if b in out]
    lines = [l for l in out.split(b"\n") if ESC in l or CR in l or BEL in l]
    print(f"check: exit {p.returncode}; raw control bytes from the repository in the output: {raw or 'none'}")
    for l in lines:
        print("  carried by:", repr(l[:120]))
