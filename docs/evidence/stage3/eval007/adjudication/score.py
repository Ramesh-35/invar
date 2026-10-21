#!/usr/bin/env python3
"""STAGE3-EVAL-007 merge and score (S7-D5): TRUE only when both blind
adjudicators say TRUE. Reported for all shapes, for the faux pattern under
both readings (fauxProvider and registerFauxProvider together; only
registerFauxProvider), and excluding it.
Usage: score.py <items.tsv> <adjudicator-A.tsv> <adjudicator-B.tsv> <merged.tsv>"""
import csv, sys
items, a, b, out = sys.argv[1:5]
rd = lambda p: list(csv.DictReader(open(p), delimiter="\t"))
I, A, B = rd(items), rd(a), rd(b)
assert len(I) == len(A) == len(B), (len(I), len(A), len(B))
rows = []
for i, x, y in zip(I, A, B):
    assert (i["repo"], i["n"], i["shape"]) == (x["repo"], x["n"], x["shape"]) == (y["repo"], y["n"], y["shape"]), (i, x, y)
    v = "TRUE" if x["verdict"] == y["verdict"] == "TRUE" else "NOT TRUE"
    rows.append((i, x["verdict"], y["verdict"], v))
with open(out, "w") as w:
    w.write("repo\tn\tid\tshape\tfunctions\tA\tB\tmerged\n")
    for i, x, y, v in rows:
        w.write(f"{i['repo']}\t{i['n']}\t{i['id']}\t{i['shape']}\t{i['functions']}\tA:{x}\tB:{y}\t{v}\n")
def rep(name, sel):
    s = [r for r in rows if sel(r[0])]
    t = sum(r[3] == "TRUE" for r in s); ag = sum(r[1] == r[2] for r in s)
    pct = f"{100 * t / len(s):.1f}" if s else "n/a"
    print(f"{name}: {t} of {len(s)} TRUE ({pct} percent), agreement {ag} of {len(s)}")
faux = {"fauxProvider", "registerFauxProvider"}
fn = lambda i: set(i["functions"].split(","))
rep("all", lambda i: True)
rep("faux pattern (fauxProvider + registerFauxProvider)", lambda i: fn(i) & faux)
rep("registerFauxProvider only", lambda i: "registerFauxProvider" in fn(i))
rep("excluding the faux pattern", lambda i: not fn(i) & faux)
rep("excluding registerFauxProvider only", lambda i: "registerFauxProvider" not in fn(i))
for i, x, y, v in rows:
    if v != "TRUE" or x != y:
        print("NOT TRUE or split:", i["repo"], i["n"], i["shape"], x, y)
