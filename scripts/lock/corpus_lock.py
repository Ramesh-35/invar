#!/usr/bin/env python3
# LOCK-1 corpus check: the MCP `check` result of the P4 binary, minus
# lock_state and lock_verified, must equal the pre-P4 result exactly on
# every corpus case (errors included); the new keys are tallied.
# Usage: corpus_lock.py <baseline invar> <new invar> <corpus-real> <shim dir>
import json, os, subprocess, sys, collections
base, new, corpus, shim = sys.argv[1:5]
env = dict(os.environ, PATH=shim + ":" + os.environ["PATH"])
INIT = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}\n'
CALL = '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"check","arguments":{}}}\n'
def run(binp, toml):
    p = subprocess.run([binp, "mcp", "--config", toml], input=INIT + CALL, capture_output=True, text=True, env=env, timeout=120)
    return json.loads(p.stdout.splitlines()[-1])["result"]
C = collections.Counter(); diffs = []; cases = 0
for case in sorted(os.listdir(corpus)):
    if case == "_template": continue
    cases += 1
    t = os.path.join(corpus, case, "invar.toml")
    b, n = run(base, t), run(new, t)
    sc = n.get("structuredContent")
    if sc is not None:
        C["lock_state:%s lock_verified:%s" % (sc.pop("lock_state", "MISSING"), sc.pop("lock_verified", "MISSING"))] += 1
        # The text content is the same JSON; compare it parsed, keys stripped.
        for r in (b, n):
            t = json.loads(r["content"][0]["text"])
            t.pop("lock_state", None); t.pop("lock_verified", None)
            r["content"][0]["text"] = t
    else:
        C["tool-error"] += 1
    if n != b: diffs.append(case)
print("cases", cases)
for k, v in sorted(C.items()): print(k, v)
print("cases differing beyond the two new keys:", len(diffs), " ".join(diffs))
