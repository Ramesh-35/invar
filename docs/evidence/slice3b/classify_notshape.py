#!/usr/bin/env python3
"""SLICE3B pre-draft evidence: classify every silent NotShape event.
Usage: classify_notshape.py <repo_dir> <census.err> <out.json>
Shares no code with the engine. Per event: is the import type-only (import
type {..} or an inline `type X`)? What does the TARGET package's raw text
declare for that name: a type (interface, type alias), a value (function,
const, let, var), or a class/enum (both)? Silence is EARNED only when the
name is provably a value."""
import json, os, re, sys
from collections import Counter, defaultdict
repo, err, outp = sys.argv[1:4]
SKIP = {"node_modules", ".git", "dist", "build", "out", "lib", "coverage", ".next"}
idx = {}
def pkg_index(pdir):
    if pdir in idx:
        return idx[pdir]
    decl = defaultdict(set)
    root = os.path.join(repo, pdir)
    for d, dirs, files in os.walk(root):
        dirs[:] = [x for x in dirs if x not in SKIP]
        for f in files:
            if not f.endswith((".ts", ".tsx")):
                continue
            try:
                t = open(os.path.join(d, f), encoding="utf-8", errors="replace").read()
            except Exception:
                continue
            for k, n in re.findall(r"\b(interface|type|function\*?|const|let|var|class|enum)\s+([A-Za-z_$][\w$]*)", t):
                kind = {"interface": "type", "type": "type", "class": "classenum", "enum": "classenum"}.get(k, "value")
                decl[n].add(kind)
    idx[pdir] = decl
    return decl
src_cache = {}
def import_stmt(rel, line):
    if rel not in src_cache:
        try:
            src_cache[rel] = open(os.path.join(repo, rel), encoding="utf-8", errors="replace").read().splitlines()
        except Exception:
            src_cache[rel] = []
    s = src_cache[rel]
    buf = []
    for i in range(line - 1, min(len(s), line + 40)):
        buf.append(s[i])
        if re.search(r"\bfrom\s*['\"]", s[i]):
            break
    return " ".join(buf)
cats = Counter()
examples = defaultdict(list)
for l in open(err):
    if not l.startswith("NOTSHAPE\t"):
        continue
    _, f, line, name, spec, target, tp = l.rstrip("\n").split("\t")
    st = import_stmt(f, int(line))
    type_only = bool(re.match(r"\s*import\s+type\b", st)) or bool(re.search(r"[{,]\s*type\s+" + re.escape(name) + r"\b", st))
    kinds = pkg_index(tp).get(name, set())
    if type_only:
        c = "type-only import: " + ("type declared in package" if "type" in kinds else "class/enum only" if kinds == {"classenum"} else "no declaration in package" if not kinds else "value only")
    else:
        if kinds == {"value"}:
            c = "value import: value declared only (EARNED silence)"
        elif "type" in kinds and "value" not in kinds:
            c = "value import: type declared only"
        elif "type" in kinds:
            c = "value import: both a type and a value declared"
        elif kinds == {"classenum"} or kinds == {"classenum", "value"}:
            c = "value import: class or enum (EARNED silence under v1 shape scope)"
        else:
            c = "value import: no declaration in package"
    cats[c] += 1
    if len(examples[c]) < 3:
        examples[c].append(f"{f}:{line} {name} -> {target}")
res = {"repo": os.path.basename(repo.rstrip("/")), "notshape": sum(cats.values()), "categories": dict(cats.most_common()), "examples": examples}
json.dump(res, open(outp, "w"), indent=1)
print(res["repo"], res["notshape"], dict(cats.most_common()))
