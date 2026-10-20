#!/usr/bin/env python3
"""Classify the K1 NotShape silences that tsconfig paths make reachable.
Input: the census lines `K1NOTSHAPE\tfile\tline\tname\tspec\ttarget` and
the repository root. Scope: the repository's whole source tree (aliases cross
top-level trees, so a re-exported name's declaration may sit in another
one).
Declarations are read on comment- and string-masked text.
  earned-value: declared as function/class/enum/const/let/var and NOT as a
    Zod schema (`= z.` or `= z\n`), and never as a type: egress discovery
    proposes Zod values only, so the import is not a schema.
  earned-type: declared only as type/interface: not a value at all.
  UNEARNED zod-not-object: declared as a Zod schema (`= z.`) in scope, yet
    K1 found no Zod object schema of that name from the target: a Zod
    schema that is not an object (both on OmniRoute are
    `z.unknown().transform(...)`), or one K1 did not reach.
  UNEARNED nowhere: declared nowhere in scope.
  UNEARNED unread: scope has a file that could not be read (>1MB or not UTF-8).
Usage: classify_k1.py <census> <repo root> <out.tsv>"""
import os, re, sys
census, root, outp = sys.argv[1:4]
SKIP = {"node_modules", ".git", "dist", "build", "out", ".next", "coverage"}

def mask(t):
    out, i, n = list(t), 0, len(t)
    def blank(a, b):
        for k in range(a, b):
            if out[k] != "\n": out[k] = " "
    while i < n:
        c = t[i]
        if t.startswith("//", i):
            j = t.find("\n", i); j = n if j < 0 else j; blank(i, j); i = j
        elif t.startswith("/*", i):
            j = t.find("*/", i + 2); j = n if j < 0 else j + 2; blank(i, j); i = j
        elif c in "'\"`":
            j = i + 1
            while j < n and t[j] != c:
                j += 2 if t[j] == "\\" else 1
            blank(i + 1, min(j, n)); i = j + 1
        else:
            i += 1
    return "".join(out)

cache = {}
def scope(top):
    if top in cache: return cache[top]
    files, unread = {}, []
    base = os.path.join(root, top) if top else root
    for d, ds, fs in os.walk(base):
        ds[:] = sorted(x for x in ds if x not in SKIP)
        for f in sorted(fs):
            if not f.endswith((".ts", ".tsx", ".js", ".mjs", ".cjs", ".jsx")): continue
            p = os.path.join(d, f); rel = os.path.relpath(p, root)
            try:
                if os.path.getsize(p) > 1_000_000: unread.append(rel); continue
                files[rel] = mask(open(p, encoding="utf-8").read())
            except Exception:
                unread.append(rel)
    cache[top] = (files, unread)
    return cache[top]

idx_cache = {}
def index(top):
    """name -> (first zod site, has type decl, has value decl), built once."""
    if top in idx_cache: return idx_cache[top]
    files, unread = scope(top)
    zod, typ, val = {}, set(), set()
    for rel in sorted(files):
        t = files[rel]
        for m in re.finditer(r"(?m)\b(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*(?::[^=\n]+)?=\s*z\s*\.", t):
            zod.setdefault(m.group(1), f"{rel}:{t.count(chr(10), 0, m.start()) + 1}")
        typ |= set(re.findall(r"\b(?:type|interface)\s+([A-Za-z_$][\w$]*)\b", t))
        val |= set(re.findall(r"\b(?:function\*?|class|enum|const|let|var)\s+([A-Za-z_$][\w$]*)\b", t))
    idx_cache[top] = (zod, typ, val, unread)
    return idx_cache[top]

rows = []
for l in open(census):
    if not l.startswith("K1NOTSHAPE\t"): continue
    _, f, line, name, spec, target = l.rstrip("\n").split("\t")
    zod, typ, val, unread = index("")
    if name in zod: k = "UNEARNED zod-not-object " + zod[name]
    elif name in val: k = "earned-value"
    elif name in typ: k = "earned-type"
    elif unread: k = "UNEARNED unread " + unread[0]
    else: k = "UNEARNED nowhere"
    rows.append((f, line, name, spec, target, k))
with open(outp, "w") as w:
    w.write("file\tline\tname\tspec\ttarget\tclass\n")
    for r in rows: w.write("\t".join(r) + "\n")
from collections import Counter
print(Counter(r[5].split(" ")[0] + (" " + r[5].split(" ")[1] if r[5].startswith("UNEARNED") else "") for r in rows))
for r in rows:
    if r[5].startswith("UNEARNED"): print("  ", r[0] + ":" + r[1], r[2], "|", r[5])
