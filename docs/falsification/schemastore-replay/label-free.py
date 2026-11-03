#!/usr/bin/env python3
"""JUDGE-R3 r0 label-free gate (docs/falsification/JUDGE-R3-SPEC.md).

Runs `invar judge --contracts` on every commit of the replay corpus,
without reading its labels, and checks every printed counterexample with
Python jsonschema (valid against the parent's file, invalid against the
commit's). Exit 1 if any printed counterexample fails.

  python3 label-free.py <invar binary> <SchemaStore clone at the pinned commit's history>
"""
import json
import os
import subprocess
import sys

import jsonschema

here = os.path.dirname(os.path.abspath(__file__))
invar, ss = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
rows = [l.rstrip("\n").split("\t") for l in open(os.path.join(here, "MANIFEST.tsv")) if not l.startswith("#") and not l.startswith("commit\t")]
commits = sorted({r[0] for r in rows})
work = os.path.join(os.environ.get("TMPDIR", "/tmp"), "invar-labelfree-worktree")
subprocess.run(["git", "-C", ss, "worktree", "remove", "--force", work], capture_output=True)
subprocess.run(["git", "-C", ss, "worktree", "add", "-q", "--detach", work, commits[0]], check=True, capture_output=True)


def valid(schema, inst):
    return jsonschema.validators.validator_for(schema)(schema).is_valid(inst)


violated, confirmed, failed, printed = [], [], 0, 0
unconfirmed = []
for c in commits:
    subprocess.run(["git", "-C", work, "checkout", "-q", "--detach", c], check=True, capture_output=True)
    out = subprocess.run([invar, "judge", "--contracts", "src/schemas/json/*.json", "--base", c + "^", "--format", "json"], cwd=work, capture_output=True, text=True).stdout
    try:
        result = json.loads(out)
    except ValueError:
        print(f"ERROR {c[:9]}: {out[:200]}")
        sys.exit(3)
    hit = False
    proof = False
    why = []
    for k in result["contracts"]:
        if k["verdict"] != "VIOLATED":
            continue
        hit = True
        pre = json.loads(subprocess.run(["git", "-C", work, "show", f"{c}^:{k['contract']}"], capture_output=True, text=True, check=True).stdout)
        post = json.load(open(os.path.join(work, k["contract"])))
        for w in k["witnesses"]:
            if w["counterexample"] is None:
                why.append(f"{k['contract'].split('/')[-1]} {w['kind']} {w['path']}: {w['counterexample_reason']}")
                continue
            printed += 1
            if valid(pre, w["counterexample"]) and not valid(post, w["counterexample"]):
                proof = True
            else:
                failed += 1
                print(f"FAIL {c[:9]} {k['contract']} {w['kind']} {w['path']}: {json.dumps(w['counterexample'])[:200]}")
    if hit:
        violated.append(c)
        (confirmed if proof else unconfirmed).append((c, why))
subprocess.run(["git", "-C", ss, "worktree", "remove", "--force", work], capture_output=True)
print(f"commits {len(commits)}: VIOLATED {len(violated)}, confirmed by a checked counterexample {len(confirmed)}, unconfirmed {len(unconfirmed)}")
print(f"counterexamples printed {printed}, failed the independent check {failed}")
for c, why in unconfirmed:
    print(f"  unconfirmed {c[:9]}: " + "; ".join(why[:3]))
print("LABEL-FREE GATE: " + ("PASS" if failed == 0 else "FAIL"))
sys.exit(1 if failed else 0)
