#!/usr/bin/env python3
"""STAGE3-EVAL-007 byte preservation and silent-drop audit: EVAL-006 vs
EVAL-007 init outputs, per repository. A proposal block runs from its
"# PROPOSED" line to the line before the next blank line. Reports blocks
added, removed, and changed (with the changed lines), and abstention lines
removed and added, by code.
Usage: compare.py <eval006 dir> <eval007 dir>"""
import os, re, sys
from collections import Counter
a, b = sys.argv[1:3]

def blocks(t):
    out, cur = {}, None
    for l in t.splitlines():
        if l.startswith("# PROPOSED "):
            cur = l[len("# PROPOSED "):]; out[cur] = [l]
        elif cur is not None:
            if l == "":
                cur = None
            else:
                out[cur].append(l)
    return out

def abst(t):
    return Counter(l for l in t.splitlines() if l.startswith("# ABSTAIN "))

tot = Counter()
for f in sorted(os.listdir(b)):
    if not f.endswith(".invar.toml"):
        continue
    x, y = open(f"{a}/{f}").read(), open(f"{b}/{f}").read()
    if x == y:
        print(f"{f}: byte-identical"); continue
    bx, by = blocks(x), blocks(y)
    add = sorted(set(by) - set(bx)); rem = sorted(set(bx) - set(by))
    chg = sorted(k for k in set(bx) & set(by) if bx[k] != by[k])
    ax, ay = abst(x), abst(y)
    gone, new = ax - ay, ay - ax
    code = lambda c: Counter(l.split()[2] for l in c.elements())
    print(f"{f}: blocks +{len(add)} -{len(rem)} changed {len(chg)}; abstentions removed {dict(code(gone))} added {dict(code(new))}")
    for k in chg:
        print(f"  changed {k}:")
        for l in by[k]:
            if l not in bx[k]:
                print(f"    + {l}")
        for l in bx[k]:
            if l not in by[k]:
                print(f"    - {l}")
    tot["add"] += len(add); tot["rem"] += len(rem); tot["chg"] += len(chg)
    # every other line outside blocks and abstentions must be equal apart from counts
    strip = lambda t, bl: [l for l in t.splitlines() if not l.startswith("# ABSTAIN ") and not any(l in v for v in bl.values())]
    ox, oy = strip(x, bx), strip(y, by)
    diff = [l for l in oy if l not in ox] + [l for l in ox if l not in oy]
    print("  other lines differing:", diff)
print("total", dict(tot))
