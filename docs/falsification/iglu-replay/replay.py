#!/usr/bin/env python3
"""Iglu Central replay: does each SchemaVer bump keep its promise?

For every schema in iglu-central, each consecutive pair of versions is
judged with `invar judge` (old version as the base, new version as the
change). Every printed counterexample is checked independently with
Python jsonschema (Draft 4, which Iglu's self-describing meta-schema
extends): valid under the old version, invalid under the new.

SchemaVer (MODEL-REVISION-ADDITION): an ADDITION bump promises the new
schema accepts all data the previous one did. So an ADDITION with a
confirmed counterexample is a broken SchemaVer promise, shown by an
instance. Pinned: snowplow/iglu-central at 7771e92d6e8a43f3bc9074393869ae9bcb08af1e.
Usage: replay.py <invar> <iglu-central clone at the pin> <out.tsv>
"""
import json, os, subprocess, sys, tempfile
import jsonschema

invar, root, out = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2]), sys.argv[3]
PIN = "7771e92d6e8a43f3bc9074393869ae9bcb08af1e"
head = subprocess.run(["git", "-C", root, "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
if head != PIN:
    sys.exit(f"iglu-central clone is at {head or 'no commit'}, not the pin {PIN}; check out the pin first")

def ver(v):
    return tuple(int(x) for x in v.split("-"))

def bump(a, b):
    if b[0] != a[0]: return "MODEL"
    if b[1] != a[1]: return "REVISION"
    return "ADDITION"

def judge(pre, post):
    with tempfile.TemporaryDirectory() as d:
        env = dict(os.environ, GIT_AUTHOR_NAME="r", GIT_AUTHOR_EMAIL="r@r", GIT_COMMITTER_NAME="r", GIT_COMMITTER_EMAIL="r@r", HOME=d)
        run = lambda *a: subprocess.run(a, cwd=d, env=env, check=True, capture_output=True)
        run("git", "init", "-q", "-b", "main")
        open(os.path.join(d, "contract.json"), "w").write(pre)
        run("git", "add", "-A"); run("git", "commit", "-q", "-m", "base")
        open(os.path.join(d, "contract.json"), "w").write(post)
        p = subprocess.run([invar, "judge", "--contract", "contract.json", "--base", "main", "--format", "json"], cwd=d, env=env, capture_output=True, text=True)
        return json.loads(p.stdout)

def valid(schema, inst):
    return jsonschema.Draft4Validator(schema).is_valid(inst)

rows, failed = [], 0
base = os.path.join(root, "schemas")
for vendor in sorted(os.listdir(base)):
    for name in sorted(os.listdir(os.path.join(base, vendor))):
        d = os.path.join(base, vendor, name, "jsonschema")
        if not os.path.isdir(d): continue
        vs = sorted((v for v in os.listdir(d) if v.count("-") == 2), key=ver)
        for a, b in zip(vs, vs[1:]):
            pre, post = open(os.path.join(d, a)).read(), open(os.path.join(d, b)).read()
            try:
                r = judge(pre, post)
            except Exception as e:
                rows.append((vendor, name, a, b, bump(ver(a), ver(b)), "ERROR", 0, 0, str(e)[:80])); continue
            ws = r.get("witnesses", [])
            built = [w for w in ws if w.get("counterexample") is not None]
            ok = 0
            for w in built:
                if valid(json.loads(pre), w["counterexample"]) and not valid(json.loads(post), w["counterexample"]):
                    ok += 1
                else:
                    failed += 1
                    print(f"FAIL {vendor}/{name} {a}->{b} {w['kind']} {w['path']}: {json.dumps(w['counterexample'])[:160]}")
            first = next((w for w in built), None)
            ex = f"{first['kind']} {first['path']}: {json.dumps(first['counterexample'], separators=(',', ':'))}" if first else ""
            rows.append((vendor, name, a, b, bump(ver(a), ver(b)), r["verdict"], len(ws), ok, ex))

with open(out, "w") as f:
    f.write("vendor\tname\tfrom\tto\tbump\tverdict\twitnesses\tconfirmed\texample\n")
    for r in rows: f.write("\t".join(str(x) for x in r) + "\n")
from collections import Counter
c = Counter((r[4], r[5]) for r in rows)
print(f"pairs {len(rows)}; by bump and verdict:")
for k in sorted(c): print(f"  {k[0]:9} {k[1]:14} {c[k]}")
broken = [r for r in rows if r[4] == "ADDITION" and r[5] == "VIOLATED"]
proven = [r for r in broken if r[7] > 0]
print(f"ADDITION bumps VIOLATED: {len(broken)}; with a counterexample confirmed by jsonschema: {len(proven)}")
errors = sum(1 for r in rows if r[5] == "ERROR")
print(f"counterexamples failing the independent check: {failed}; pairs the judge could not run: {errors}")
# A pair the judge could not run is never a pass.
print("IGLU GATE: " + ("PASS" if failed == 0 and errors == 0 and rows else "FAIL"))
sys.exit(0 if failed == 0 and errors == 0 and rows else 1)
