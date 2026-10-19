#!/usr/bin/env python3
"""STAGE3-EVAL-003 M-S-3 scoring. Merges two independent blind
adjudications (A, B) of the 58 new code-pair proposals under rubric D:
a proposal counts TRUE only when BOTH say TRUE (a disagreement is UNSURE,
and UNSURE scores NOT TRUE, per rubric D). Reports Cohen's kappa for
agreement, precision for the 58, and the pooled precision with the 9
strings proposals the founder adjudicated in STAGE3-EVAL-001 (8 TRUE),
which are byte-identical in EVAL-003. Threshold (pre-registered, E of
STAGE3-EVAL-001): pooled precision >= 60 percent and at least one
repository with a TRUE proposal."""
import csv, sys
D = "/mnt/user-data/outputs/stage3/eval003/adjudication"
def load(p):
    rows = list(csv.DictReader(open(p), delimiter="\t"))
    return {(r["repo"], int(r["n"])): r for r in rows}
A, B = load(f"{D}/adjudicator-A.tsv"), load(f"{D}/adjudicator-B.tsv")
assert set(A) == set(B) and len(A) == 58, (len(A), len(B))
for k in A:
    assert A[k]["verdict"] in ("TRUE", "NOT TRUE") and B[k]["verdict"] in ("TRUE", "NOT TRUE"), k
    assert A[k]["id"] == B[k]["id"], k
both = sum(1 for k in A if A[k]["verdict"] == B[k]["verdict"] == "TRUE")
agree = sum(1 for k in A if A[k]["verdict"] == B[k]["verdict"])
pa = agree / 58
ta, tb = sum(A[k]["verdict"] == "TRUE" for k in A) / 58, sum(B[k]["verdict"] == "TRUE" for k in B) / 58
pe = ta * tb + (1 - ta) * (1 - tb)
kappa = (pa - pe) / (1 - pe) if pe < 1 else 1.0
per = {}
with open(f"{D}/merged.tsv", "w") as out:
    out.write("repo\tn\tid\tA\tB\tfinal\tA_sentence\tB_sentence\n")
    for k in sorted(A, key=lambda k: (k[0], k[1])):
        fin = "TRUE" if A[k]["verdict"] == B[k]["verdict"] == "TRUE" else "NOT TRUE"
        per.setdefault(k[0], [0, 0]); per[k[0]][0] += fin == "TRUE"; per[k[0]][1] += 1
        out.write(f"{k[0]}\t{k[1]}\t{A[k]['id']}\t{A[k]['verdict']}\t{B[k]['verdict']}\t{fin}\t{A[k]['sentence']}\t{B[k]['sentence']}\n")
print(f"A TRUE {round(ta*58)}  B TRUE {round(tb*58)}  agreement {agree}/58 ({pa:.0%})  kappa {kappa:.2f}")
for r, (t, n) in sorted(per.items()):
    print(f"  {r}: {t} of {n} TRUE")
print(f"new code-pair precision: {both}/58 = {both/58:.0%}")
pooled_t, pooled_n = both + 8, 58 + 9
ok = pooled_t / pooled_n >= 0.60 and any(t for t, _ in per.values())
print(f"pooled with EVAL-001 strings (8/9): {pooled_t}/{pooled_n} = {pooled_t/pooled_n:.0%}  M-S-3 {'PASS' if ok else 'FAIL'} (threshold 60 percent)")
