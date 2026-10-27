#!/usr/bin/env python3
# JSON-1 corpus check: run the MCP `check` tool of two binaries on every
# corpus case; the new binary's JSON minus the JSON-1 keys must equal the
# baseline JSON exactly, and every JSON-1 field is tallied.
# Usage: corpus_json.py <baseline invar> <new invar> <corpus-real> <shim dir>
import json, os, subprocess, sys, collections
base, new, corpus, shim = sys.argv[1:5]
env = dict(os.environ, PATH=shim + ":" + os.environ["PATH"])
INIT = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}\n'
CALL = '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"check","arguments":{}}}\n'
def run(binp, toml):
    p = subprocess.run([binp, "mcp", "--config", toml], input=INIT + CALL, capture_output=True, text=True, env=env, timeout=120)
    lines = [json.loads(l) for l in p.stdout.splitlines() if l.strip()]
    r = lines[-1]["result"]
    return r.get("structuredContent"), r.get("isError")
NEW_R = ("rule", "consumer_sites", "producers", "candidate_rename")
C = collections.Counter(); diffs = []
for case in sorted(os.listdir(corpus)):
    if case == "_template": continue
    t = os.path.join(corpus, case, "invar.toml")
    b, be = run(base, t); n, ne = run(new, t)
    if be or ne:
        C["tool-error"] += 1; continue
    stripped = json.loads(json.dumps(n)); stripped.pop("exit_code", None)
    for c in stripped["contracts"]:
        c.pop("not_evaluated_items", None); c.pop("skip", None)
        for r in c["residue"]:
            for k in NEW_R: r.pop(k, None)
    if stripped != b: diffs.append(case)
    C["exit_code:%s" % n["exit_code"]] += 1
    for c in n["contracts"]:
        for r in c["residue"]:
            C["rule:" + r["rule"]] += 1
            C["producers:" + ("named" if r["producers"] else "EMPTY")] += 1
            C["sites"] += len(r["consumer_sites"])
            if r["candidate_rename"]: C["candidate"] += 1
        for it in c["not_evaluated_items"]:
            C["reason:" + it["reason"]] += 1
        if c["skip"]: C["skip:" + c["skip"]["reason"]] += 1
for k in sorted(C): print(f"{C[k]:5d}  {k}")
print("existing keys identical to baseline:", 122 - len(diffs) - C["tool-error"], "of 122", diffs)
