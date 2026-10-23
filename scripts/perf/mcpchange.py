#!/usr/bin/env python3
"""Warm-server correctness: the same `invar mcp` process answers check
before a producer edit, after it, and after the revert; each answer must
equal a fresh process's answer. Usage: mcpchange.py <invar> <dir> <producer file> <old> <new>"""
import json, subprocess, sys
from collections import Counter
B, d, f, old, new = sys.argv[1:6]
def server():
    p = subprocess.Popen([B, "mcp"], cwd=d, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
    n = [0]
    def call(m, pa=None):
        n[0] += 1
        p.stdin.write(json.dumps({"jsonrpc": "2.0", "id": n[0], "method": m, "params": pa or {}}) + "\n"); p.stdin.flush()
        return json.loads(p.stdout.readline())
    call("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}})
    return p, lambda: call("tools/call", {"name": "check", "arguments": {}})["result"]["content"][0]["text"]
warm, check = server()
text = open(f"{d}/{f}").read()
ok = True
for label, body in [("before", text), ("edited", text.replace(old, new)), ("reverted", text)]:
    open(f"{d}/{f}", "w").write(body)
    w = check()
    cold, c2 = server(); c = c2(); cold.stdin.close(); cold.wait()
    st = Counter(x["status"] for x in json.loads(w)["contracts"])
    print(f"{label:9s} warm {dict(st)}  warm == fresh process: {w == c}")
    ok &= w == c
warm.stdin.close(); warm.wait()
sys.exit(0 if ok else 1)
