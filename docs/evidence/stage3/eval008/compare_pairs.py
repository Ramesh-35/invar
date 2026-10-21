#!/usr/bin/env python3
"""STAGE3-EVAL-008: compare two init outputs per repository keyed by the
proposal's (producer, consumer) pair (its E6 line), not by its id, so a
renumbered id shows as such. Reports pairs added, removed, renamed (same
pair, other id), and changed (lines added or removed inside the block,
the id line aside).
Usage: compare_pairs.py <before dir> <after dir>"""
import os, re, sys
a, b = sys.argv[1:3]

def blocks(t):
    out, cur = {}, None
    for l in t.splitlines():
        if l.startswith("# PROPOSED "):
            cur = [l]
        elif cur is not None:
            if l == "":
                pair = next((x for x in cur if x.startswith("# E6 pair ")), cur[0])
                out[pair] = cur
                cur = None
            else:
                cur.append(l)
    return out

ren = chg = add = 0
for f in sorted(os.listdir(b)):
    if not f.endswith(".invar.toml") or not os.path.exists(f"{a}/{f}"):
        continue
    x, y = blocks(open(f"{a}/{f}").read()), blocks(open(f"{b}/{f}").read())
    for k in sorted(set(x) | set(y)):
        if k not in y:
            print(f"{f}: REMOVED {k}")
            continue
        if k not in x:
            add += 1
            continue
        idx = lambda bl: [l for l in bl if l.startswith(("# PROPOSED ", "id = "))]
        rest = lambda bl: [l for l in bl if not l.startswith(("# PROPOSED ", "id = "))]
        if idx(x[k]) != idx(y[k]):
            ren += 1
            print(f"{f}: RENAMED {k[len('# E6 pair '):]}: {idx(x[k])[0][11:]} -> {idx(y[k])[0][11:]}")
        if rest(x[k]) != rest(y[k]):
            chg += 1
print(f"pairs added {add}, renamed {ren}, changed {chg}")
