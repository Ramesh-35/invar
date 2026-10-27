#!/usr/bin/env python3
"""SIB-1 (P5) kill gate, SIBLING-EVAL-SPEC section C. For each corpus case
OUTSIDE the sealed held-out set: lock the pinned consumer against the
pre-break producer (producer_base), lock it again against the post-break
producer, and compare bindings. Same consumer bytes both times, so a
binding that vanishes means the producer stopped providing that field
(SB-2 `absent`); a binding whose type changed is SB-2 `retyped`. A break is
ACTIONABLE when an absent or retyped binding is the case's expected
residue. Controls: any absent or retyped binding is a false alarm.
Usage: killgate.py <invar> <corpus-real> <invar-corpus.jsonl> <shim dir> <work dir>"""
import json, os, re, shutil, subprocess, sys
binp, corpus, jsonl, shim, work = sys.argv[1:6]
env = dict(os.environ, PATH=shim + ":" + os.environ["PATH"])
key = lambda s: re.sub(r"[_\-]", "", s).lower()

def lock(case_dir, side):
    w = os.path.join(work, os.path.basename(case_dir) + "-" + side)
    shutil.rmtree(w, ignore_errors=True); os.makedirs(w)
    t = open(os.path.join(case_dir, "invar.toml")).read()
    if side == "base":
        if "producer_base" not in t:
            return None, "no producer_base"
        # Each block's producer becomes that block's producer_base.
        blocks = re.split(r"(?m)^(?=\[\[)", t)
        out = []
        for b in blocks:
            m = re.search(r'(?m)^producer_base = (".*")$', b)
            if m:
                b = re.sub(r'(?m)^producer = ".*"$', "producer = " + m.group(1), b)
            out.append(b)
        t = "".join(out)
    open(os.path.join(w, "invar.toml"), "w").write(t)
    p = subprocess.run([binp, "lock"], cwd=w, capture_output=True, text=True, env=dict(env, HOME=w), timeout=300)
    lf = os.path.join(w, "invar.lock")
    if p.returncode != 0 or not os.path.exists(lf):
        return None, "lock exit %d: %s" % (p.returncode, (p.stdout + p.stderr).strip().splitlines()[-1:] )
    binds, cid = {}, None
    for line in open(lf):
        m = re.match(r'id = "(.*)"', line)
        if m: cid = m.group(1)
        m = re.match(r'bind = "(\S+) (\S+) (\S+)(?: (.*))?"', line)
        if m: binds[(cid, m.group(3))] = m.group(4) or ""
    return binds, None

rows = [json.loads(l) for l in open(jsonl)][1:]
cases = [r for r in rows if not r["held_out"]]
assert len(cases) == 37, len(cases)
actionable, false_alarms, results = 0, 0, []
for r in sorted(cases, key=lambda r: r["id"]):
    d = os.path.join(corpus, r["dir"])
    base, eb = lock(d, "base"); head, eh = lock(d, "head")
    if base is None or head is None:
        results.append((r["id"], r["class"], "NOT MEASURED", eb or eh, "")); continue
    absent = [k for k in base if k not in head]
    retyped = [k for k in base if k in head and base[k] != head[k]]
    named = sorted({k[1] for k in absent} | {k[1] for k in retyped})
    want = [key(x) for x in re.split(r"[,\s]+", r.get("expected", {}).get("residue", "")) if x]
    hit = any(key(n.split(".")[-1]) in want or key(n) in want for n in named)
    if r["class"] == "break":
        status = "ACTIONABLE" if hit else ("CHANGED-OTHER" if named else "SILENT")
        actionable += hit
    else:
        status = "FALSE-ALARM" if named else "QUIET"
        false_alarms += bool(named)
    detail = " ".join(["absent:" + k[1] for k in absent] + ["retyped:%s(%s->%s)" % (k[1], base[k], head[k]) for k in retyped])
    results.append((r["id"], r["class"], status, r.get("expected", {}).get("residue", ""), detail))
for row in results:
    print("\t".join(row))
breaks = sum(1 for r in cases if r["class"] == "break")
print("breaks %d actionable %d | controls %d false-alarms %d | not measured %d" % (
    breaks, actionable, len(cases) - breaks, false_alarms, sum(1 for x in results if x[2] == "NOT MEASURED")))
print("KILL GATE (fewer than 3 actionable kills P5):", "PASS" if actionable >= 3 else "KILLED")
