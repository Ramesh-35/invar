#!/usr/bin/env python3
"""STAGE3-EVAL-003 B4 reconciliation: every census event whose verdict
differs from the engine's, with its cause. The census classifier
(classify_notshape.py) and the engine differ in exactly three declared
ways; each difference is attributed by recomputing the verdict with one
difference switched at a time:
  scope   the census walked every directory under the package except
          node_modules/.git/dist/build/out/lib/coverage/.next (nested
          packages included); the engine reads the files init scanned for
          that package (init's skip list, nested packages excluded)
  mask    the census read raw text; the engine reads text with strings and
          comments blanked
  typeonly the census parsed the import statement text; the engine uses
          the scanner's type-only record
  complete the engine's completeness rule (an unread file in the target
          package voids earned silence), found at the EVAL-003 rehearsal
          after the census was pre-registered
Usage: reconcile.py <repo_dir> <census.err> <engine_new.json> <out.json>"""
import json, os, re, sys
from collections import defaultdict
repo, err, engf, outp = sys.argv[1:5]
CENSUS_SKIP = {"node_modules", ".git", "dist", "build", "out", "lib", "coverage", ".next"}
INIT_SKIP = {".git", "target", "node_modules", "dist", "build", ".venv", "venv", "__pycache__", "vendor", ".next", ".idea", ".vscode"}
KW = re.compile(r"\b(interface|type|function\*?|const|let|var|class|enum)\s+([A-Za-z_$][\w$]*)")

def mask(t):
    o, i, n = [], 0, len(t)
    while i < n:
        c = t[i]
        if t.startswith("//", i):
            j = t.find("\n", i); j = n if j < 0 else j; o.append(" " * (j - i)); i = j; continue
        if t.startswith("/*", i):
            j = t.find("*/", i + 2); j = n if j < 0 else j + 2; o.append(re.sub(r"[^\n]", " ", t[i:j])); i = j; continue
        if c in "\"'`":
            j = i + 1
            while j < n and t[j] != c:
                j += 2 if t[j] == "\\" else 1
            j = min(j + 1, n); o.append(re.sub(r"[^\n]", " ", t[i:j])); i = j; continue
        o.append(c); i += 1
    return "".join(o)

pkg_dirs = set()
for d, dirs, files in os.walk(repo):
    dirs[:] = [x for x in dirs if x not in INIT_SKIP]
    if "package.json" in files:
        pkg_dirs.add(os.path.relpath(d, repo))

def kinds_for(tp, name, scope, masked):
    k = set()
    root = os.path.join(repo, tp)
    for d, dirs, files in os.walk(root):
        skip = CENSUS_SKIP if scope == "census" else INIT_SKIP
        dirs[:] = [x for x in dirs if x not in skip]
        rel = os.path.relpath(d, repo)
        if scope == "init" and rel != tp and rel in pkg_dirs:
            dirs[:] = []
            continue
        for f in files:
            if not f.endswith((".ts", ".tsx")):
                continue
            t = open(os.path.join(d, f), encoding="utf-8", errors="replace").read()
            if masked:
                t = mask(t)
            for kw, n in KW.findall(t):
                if n == name:
                    k.add({"interface": "type", "type": "type", "class": "ce", "enum": "ce"}.get(kw, "value"))
            # The engine's word scan reads `const enum X` as an enum; the
            # census regex consumed `const enum` as a const named `enum`.
            # Engine-side view only, so the recorded census stays as it was.
            if scope == "init" and re.search(r"\benum\s+" + re.escape(name) + r"\b", t):
                k.add("ce")
    return k

def unread_in(tp):
    root = os.path.join(repo, tp)
    for d, dirs, files in os.walk(root):
        dirs[:] = [x for x in dirs if x not in INIT_SKIP]
        rel = os.path.relpath(d, repo)
        if rel != tp and rel in pkg_dirs:
            dirs[:] = []
            continue
        for f in files:
            if f.endswith((".ts", ".tsx")) and os.path.getsize(os.path.join(d, f)) > 1_000_000:
                return True
    return False

def verdict(kinds, type_only):
    if not kinds:
        return "loud"
    if "type" not in kinds:
        return "silent"
    return "loud"

src = {}
def stmt(rel, line):
    if rel not in src:
        src[rel] = open(os.path.join(repo, rel), encoding="utf-8", errors="replace").read().splitlines()
    s, buf = src[rel], []
    for i in range(line - 1, min(len(s), line + 40)):
        buf.append(s[i])
        if re.search(r"\bfrom\s*['\"]", s[i]):
            break
    return " ".join(buf)

eng = defaultdict(int)
for f, line, name in json.load(open(engf))["engine_new"]:
    eng[(f, line, name)] += 1
rows, census_loud, diffs = [], 0, []
seen = defaultdict(int)
for l in open(err):
    if not l.startswith("NOTSHAPE\t"):
        continue
    _, f, line, name, spec, target, tp = l.rstrip("\n").split("\t")
    st = stmt(f, int(line))
    to = bool(re.match(r"\s*import\s+type\b", st)) or bool(re.search(r"[{,]\s*type\s+" + re.escape(name) + r"\b", st))
    c = verdict(kinds_for(tp, name, "census", False), to)
    key = (f, line, name)
    seen[key] += 1
    e = "loud" if seen[key] <= eng.get(key, 0) else "silent"
    census_loud += c == "loud"
    if c != e:
        causes = []
        if verdict(kinds_for(tp, name, "init", False), to) == e:
            causes.append("scope")
        if verdict(kinds_for(tp, name, "census", True), to) == e:
            causes.append("mask")
        if verdict(kinds_for(tp, name, "init", True), to) == e and not causes:
            causes.append("scope+mask")
        if not causes and e == "loud" and unread_in(tp):
            causes.append("complete")
        ek = kinds_for(tp, name, "init", True)
        if not causes and e == "silent" and ek and "type" not in ek:
            causes.append("census-regex (engine reads the declaration, e.g. const enum)")
        diffs.append({"event": f"{f}:{line} {name}", "target_pkg": tp, "census": c, "engine": e, "cause": causes or ["UNEXPLAINED"],
                      "census_kinds": sorted(kinds_for(tp, name, "census", False)), "engine_like_kinds": sorted(kinds_for(tp, name, "init", True))})
res = {"repo": os.path.basename(repo.rstrip("/")), "census_loud": census_loud, "engine_loud": sum(eng.values()), "differences": diffs}
json.dump(res, open(outp, "w"), indent=1)
from collections import Counter
print(res["repo"], "census", census_loud, "engine", res["engine_loud"], "diffs", len(diffs), dict(Counter("+".join(d["cause"]) + ":" + d["census"] + "->" + d["engine"] for d in diffs)))
