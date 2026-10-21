#!/usr/bin/env python3
"""STAGE3-EVAL-008 merge and score: TRUE only when both blind adjudicators
say TRUE. Reported for all new shapes, for the 20 the draft's probe found
(judged before, in sample), for the others, and by kind (R6 conversion or
RECV rescue).
Usage: score.py <items.tsv> <A.tsv> <B.tsv> <probe.tsv> <merged.tsv>"""
import csv, re, sys
items, a, b, probe, out = sys.argv[1:6]
rd = lambda p: list(csv.DictReader(open(p), delimiter="\t"))
I, A, B = rd(items), rd(a), rd(b)
yields = {(r["repo"], r["file"], r["shape"]) for r in rd(probe) if r["class"].startswith("YIELD")}
pk = open(items.replace("items.tsv", "packet.md")).read().split("### ")[1:]
rows = []
for i, x, y in zip(I, A, B):
    assert (i["repo"], i["n"], i["shape"]) == (x["repo"], x["n"], x["shape"]) == (y["repo"], y["n"], y["shape"])
    blk = next(p for p in pk if p.startswith(f"{i['repo']} {i['n']}: "))
    e4 = next(l for l in blk.splitlines() if l.startswith(f"- E4 {i['shape']} "))
    named = " not imported by name" not in e4
    cons = re.search(r"\(([^)]+?):\d+\)", next(l for l in blk.splitlines() if l.startswith(f"- E5 {i['shape']} fields read"))).group(1)
    inprobe = (i["repo"], cons, i["shape"]) in yields
    v = "TRUE" if x["verdict"] == y["verdict"] == "TRUE" else "NOT TRUE"
    rows.append((i, x["verdict"], y["verdict"], v, named, inprobe))
with open(out, "w") as w:
    w.write("repo\tn\tid\tshape\tkind\tin_probe\tA\tB\tmerged\n")
    for i, x, y, v, named, ip in rows:
        w.write(f"{i['repo']}\t{i['n']}\t{i['id']}\t{i['shape']}\t{'RECV-rescue' if named else 'R6-conversion'}\t{'yes' if ip else 'no'}\t{x}\t{y}\t{v}\n")
def rep(name, sel):
    s = [r for r in rows if sel(r)]
    t = sum(r[3] == "TRUE" for r in s); ag = sum(r[1] == r[2] for r in s)
    print(f"{name}: {t} of {len(s)} TRUE, agreement {ag} of {len(s)}")
rep("all new shapes", lambda r: True)
rep("the probe's 20 (in sample)", lambda r: r[5])
rep("not found by the probe (out of sample)", lambda r: not r[5])
rep("R6 conversions", lambda r: not r[4])
rep("RECV rescues", lambda r: r[4])
