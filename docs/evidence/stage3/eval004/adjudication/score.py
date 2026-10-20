#!/usr/bin/env python3
"""STAGE3-EVAL-004 M-S-3 scoring. Merges two independent blind
adjudications (A, B) of the 29 new code-pair proposals of SLICE4 and the
one shape added to an existing pair (G1, reported separately), under
rubric D: TRUE only when BOTH say TRUE; a disagreement is UNSURE and
scores NOT TRUE. Same method as STAGE3-EVAL-003 (score.py there), so the
two are comparable. Threshold (pre-registered by the founder, 2026-10-20):
precision on the +29 delta >= 60 percent. Also reported: cumulative
precision with EVAL-003 (41 of 58) and the founder's EVAL-001 strings
verdicts (8 of 9).
Usage: score.py <dir holding adjudicator-A.tsv and adjudicator-B.tsv>"""
import csv, sys
D = sys.argv[1]
def load(p):
    return {(r["repo"], r["n"]): r for r in csv.DictReader(open(p), delimiter="\t")}
A, B = load(f"{D}/adjudicator-A.tsv"), load(f"{D}/adjudicator-B.tsv")
assert set(A) == set(B) and len(A) == 30, (len(A), len(B))
for k in A:
    assert A[k]["verdict"] in ("TRUE", "NOT TRUE") and B[k]["verdict"] in ("TRUE", "NOT TRUE"), k
    assert A[k]["id"] == B[k]["id"], k
new = [k for k in A if not k[1].startswith("G")]
grown = [k for k in A if k[1].startswith("G")]
assert len(new) == 29 and len(grown) == 1
final = {k: "TRUE" if A[k]["verdict"] == B[k]["verdict"] == "TRUE" else "NOT TRUE" for k in A}
n = len(new)
agree = sum(A[k]["verdict"] == B[k]["verdict"] for k in new)
ta = sum(A[k]["verdict"] == "TRUE" for k in new) / n
tb = sum(B[k]["verdict"] == "TRUE" for k in new) / n
pe = ta * tb + (1 - ta) * (1 - tb)
kappa = (agree / n - pe) / (1 - pe) if pe < 1 else 1.0
order = lambda k: (k[0], k[1].startswith("G"), int(k[1].lstrip("G")))
with open(f"{D}/merged.tsv", "w") as out:
    out.write("repo\tn\tid\tA\tB\tfinal\tA_sentence\tB_sentence\n")
    for k in sorted(A, key=order):
        out.write("\t".join([k[0], k[1], A[k]["id"], A[k]["verdict"], B[k]["verdict"], final[k], A[k]["sentence"], B[k]["sentence"]]) + "\n")
t = sum(final[k] == "TRUE" for k in new)
print(f"agreement {agree} of {n} ({100*agree/n:.0f} percent), Cohen's kappa {kappa:.2f}")
for repo in sorted({k[0] for k in new}):
    ks = [k for k in new if k[0] == repo]
    print(f"{repo}: {sum(final[k] == 'TRUE' for k in ks)} of {len(ks)} TRUE")
print(f"M-S-3 delta: {t} of {n} TRUE ({100*t/n:.1f} percent) against the 60 percent floor: {'PASS' if t/n >= 0.6 else 'FAIL'}")
print(f"G1 (shape added to an existing pair, reported separately): {final[grown[0]]}")
print(f"cumulative code pairs (EVAL-003 + EVAL-004): {41+t} of {58+n} ({100*(41+t)/(58+n):.1f} percent)")
print(f"pooled with the EVAL-001 strings verdicts: {49+t} of {67+n} ({100*(49+t)/(67+n):.1f} percent)")
