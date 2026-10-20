#!/usr/bin/env python3
"""STAGE3-EVAL-005 M-S-3 scoring (ruling S5-D5). Two packets, each judged
by two independent blind agents under rubric D; TRUE only when both say
TRUE (a disagreement is UNSURE and scores NOT TRUE).
  out-of-sample (A, B): the 50 surviving proposals never adjudicated
    before. The registered test: precision >= 60 percent.
  in-sample (C, D): the 54 surviving proposals adjudicated in EVAL-003 or
    -004, re-judged on their new evidence. Reported, not tested.
Retention: of the 57 pairs adjudicated TRUE before, how many still propose.
Usage: score.py <adjudication dir> <labeled-pairs.tsv> <eval005 dir>"""
import csv, re, sys
D, labeled, ev = sys.argv[1:4]

def load(p):
    return {(r["repo"], r["n"]): r for r in csv.DictReader(open(p), delimiter="\t")}

def merge(a, b, n, out):
    A, B = load(f"{D}/{a}"), load(f"{D}/{b}")
    assert set(A) == set(B) and len(A) == n, (a, len(A), len(B))
    for k in A:
        assert A[k]["verdict"] in ("TRUE", "NOT TRUE") and B[k]["verdict"] in ("TRUE", "NOT TRUE"), k
        assert A[k]["id"] == B[k]["id"], k
    final = {k: "TRUE" if A[k]["verdict"] == B[k]["verdict"] == "TRUE" else "NOT TRUE" for k in A}
    agree = sum(A[k]["verdict"] == B[k]["verdict"] for k in A)
    ta = sum(A[k]["verdict"] == "TRUE" for k in A) / n
    tb = sum(B[k]["verdict"] == "TRUE" for k in B) / n
    pe = ta * tb + (1 - ta) * (1 - tb)
    kappa = (agree / n - pe) / (1 - pe) if pe < 1 else 1.0
    with open(f"{D}/{out}", "w") as w:
        w.write("repo\tn\tid\tfirst\tsecond\tfinal\tfirst_sentence\tsecond_sentence\n")
        for k in sorted(A, key=lambda k: (k[0], int(k[1]))):
            w.write("\t".join([k[0], k[1], A[k]["id"], A[k]["verdict"], B[k]["verdict"], final[k], A[k]["sentence"], B[k]["sentence"]]) + "\n")
    return A, final, agree, kappa

for label, a, b, n, out in (("out-of-sample (registered)", "adjudicator-A-out.tsv", "adjudicator-B-out.tsv", 50, "merged-out.tsv"),
                            ("in-sample (reported)", "adjudicator-C-in.tsv", "adjudicator-D-in.tsv", 54, "merged-in.tsv")):
    A, final, agree, kappa = merge(a, b, n, out)
    t = sum(v == "TRUE" for v in final.values())
    print(f"{label}: {t} of {n} TRUE ({100 * t / n:.1f} percent); agreement {agree} of {n}, kappa {kappa:.2f}")
    for repo in sorted({k[0] for k in A}):
        ks = [k for k in A if k[0] == repo]
        print(f"  {repo}: {sum(final[k] == 'TRUE' for k in ks)} of {len(ks)}")
    if label.startswith("out"):
        print(f"  M-S-3: {'PASS' if t / n >= 0.6 else 'FAIL'} against the 60 percent floor")

# retention of the pairs adjudicated TRUE before
def pairs(path):
    out, cur = set(), None
    for l in open(path).read().splitlines():
        m = re.match(r"# E6 pair (\S+) -> (\S+)$", l)
        if m:
            out.add(m.groups())
    return out
live = {}
true_before = [r for r in csv.DictReader(open(labeled), delimiter="\t") if r["verdict_then"] == "TRUE"]
kept = 0
for r in true_before:
    if r["repo"] not in live:
        live[r["repo"]] = pairs(f"{ev}/{r['repo']}.invar.toml")
    kept += (r["producer"], r["consumer"]) in live[r["repo"]]
print(f"retention: {kept} of {len(true_before)} pairs adjudicated TRUE before still propose")
