#!/usr/bin/env python3
"""STAGE3-EVAL-001 M-S-4 abstention-honesty audit (independent detector).

Usage: audit_abstain.py <repo_dir> <phase1 invar.toml> <out.json>

Shares no code with the engine. From raw source it derives init's
declared candidate shape (INIT-DISCOVERY-SLICE1): a named import, in a
file of workspace package A, from another workspace package B, of a name
B exports as an interface, object type, or other type alias, where the
importing file reads at least 2 of that shape's direct members by
property access (any non-object alias counts as candidate-shaped since
init must abstain on it as UNSUPPORTED-SHAPE).

Recorded operationalizations:
  S3-INT-5 the sample is drawn from files carrying at least one
           cross-package named import (a file with none cannot test
           honesty); strictly harder than sampling all files.
  S3-INT-6 workspace packages = every package.json with a "name" outside
           node_modules/.git/build/dist, root excluded: a SUPERSET of
           init's qualifying set, so the detector over-flags, never
           under-flags; every flag is triaged by hand.
  S3-INT-7 a construct is accounted iff its file is a proposal endpoint
           or an ABSTAIN line names that file's path; a repository-level
           ABSTAIN (H2, SLICE-SCOPE) accounts for every construct in it.
  S3-INT-8 sample: 10 files per repository, random.Random seeded with
           "STAGE3-EVAL-001:<repo>", drawn without replacement from the
           sorted eligible list.
"""
import json, os, random, re, sys

SKIP = {"node_modules", ".git", "build", "dist", "out", "coverage", ".next"}
IMPORT = re.compile(r'import\s+(type\s+)?\{([^}]*)\}\s*from\s*[\'"]([^\'"]+)[\'"]', re.S)
REPO_LEVEL = ("H2 ", "SLICE-SCOPE")

def walk(root):
    for d, dirs, files in os.walk(root):
        dirs[:] = sorted(x for x in dirs if x not in SKIP and not x.startswith("."))
        for f in sorted(files):
            yield os.path.join(d, f)

def ts_files(root):
    return [p for p in walk(root) if (p.endswith(".ts") or p.endswith(".tsx")) and not p.endswith(".d.ts")]

def read(p):
    try:
        if os.path.getsize(p) > 1_000_000:
            return None
        return open(p, encoding="utf-8").read()
    except Exception:
        return None

def packages(repo):
    pk = {}
    for p in walk(repo):
        if os.path.basename(p) != "package.json":
            continue
        d = os.path.dirname(p)
        if os.path.abspath(d) == os.path.abspath(repo):
            continue
        try:
            n = json.load(open(p)).get("name")
        except Exception:
            continue
        if isinstance(n, str) and n:
            pk[os.path.relpath(d, repo)] = n
    return pk

def own_pkg(rel, pkdirs):
    best = None
    for d in pkdirs:
        if rel.startswith(d + "/") and (best is None or len(d) > len(best)):
            best = d
    return best

def shape_members(pkg_dir, name, cache):
    key = (pkg_dir, name)
    if key in cache:
        return cache[key]
    res = None
    rx = re.compile(r'export\s+(?:declare\s+)?(interface|type)\s+' + re.escape(name) + r'\b[^{=]*(=)?\s*(\{)?')
    for p in ts_files(pkg_dir):
        t = read(p)
        if not t or name not in t:
            continue
        m = rx.search(t)
        if not m:
            continue
        kind = m.group(1)
        if kind == "type" and not m.group(3):
            res = ("alias", set())
            break
        i = t.find("{", m.start())
        depth, j, body = 0, i, []
        while j < len(t):
            c = t[j]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    break
            elif depth == 1:
                body.append(c)
            j += 1
        mem = set(re.findall(r'(?m)^\s*(?:readonly\s+)?([A-Za-z_]\w*)\??\s*[:(]', "".join(body)))
        res = ("object", mem)
        break
    cache[key] = res
    return res

def main():
    repo, toml, outp = sys.argv[1], sys.argv[2], sys.argv[3]
    name = os.path.basename(os.path.normpath(repo))
    t = open(toml).read()
    abstains = [l for l in t.splitlines() if l.startswith("# ABSTAIN ")]
    repo_level = [l for l in abstains if l[len("# ABSTAIN "):].startswith(REPO_LEVEL)]
    endpoints = set(re.findall(r'(?m)^(?:producer|consumer|list) = "([^"]+)"', t))
    pk = packages(repo)
    by_name = {v: k for k, v in pk.items()}
    pkdirs = sorted(pk)
    eligible, constructs = [], {}
    cache = {}
    for d in pkdirs:
        for p in ts_files(os.path.join(repo, d)):
            rel = os.path.relpath(p, repo)
            if own_pkg(rel, pkdirs) != d or rel in endpoints:
                continue
            src = read(p)
            if not src:
                continue
            found = []
            for m in IMPORT.finditer(src):
                spec = m.group(3)
                base = spec if not spec.startswith("@") else "/".join(spec.split("/")[:2])
                base = base.split("/")[0] if not spec.startswith("@") else base
                tgt = by_name.get(base)
                if not tgt or tgt == d:
                    continue
                line = src.count("\n", 0, m.start()) + 1
                for b in m.group(2).split(","):
                    b = b.strip()
                    if not b:
                        continue
                    b = re.sub(r'^type\s+', '', b)
                    imported, _, local = b.partition(" as ")
                    imported, local = imported.strip(), (local.strip() or imported.strip())
                    found.append((line, spec, imported, local, tgt))
            if not found:
                continue
            eligible.append(rel)
            cs = []
            for line, spec, imported, local, tgt in found:
                sm = shape_members(os.path.join(repo, tgt), imported, cache)
                if sm is None:
                    continue
                kind, mem = sm
                reads = sorted(m for m in mem if re.search(r'\.' + re.escape(m) + r'\b', src))
                if kind == "alias" or len(reads) >= 2:
                    cs.append({"line": line, "spec": spec, "name": imported, "kind": kind, "reads": reads[:8]})
            if cs:
                constructs[rel] = cs
    rnd = random.Random("STAGE3-EVAL-001:" + name)
    sample = rnd.sample(sorted(eligible), min(10, len(eligible)))
    def accounted(rel):
        if repo_level:
            return "repo-level " + repo_level[0][:60]
        hits = [l for l in abstains if (rel + ":") in l or (" " + rel + " ") in l]
        return hits[0][:160] if hits else None
    rows, silent = [], 0
    for rel in sample:
        for c in constructs.get(rel, []):
            a = accounted(rel)
            if a is None:
                silent += 1
            rows.append({"file": rel, **c, "accounted_by": a})
    full_unaccounted = sum(1 for rel, cs in constructs.items() for _ in cs if accounted(rel) is None)
    full_total = sum(len(cs) for cs in constructs.values())
    out = {"repo": name, "packages_superset": len(pk), "eligible_files": len(eligible),
           "sample": sample, "sampled_constructs": len(rows), "sampled_silent": silent,
           "rows": rows, "exploratory_all_files_constructs": full_total,
           "exploratory_all_files_unaccounted": full_unaccounted,
           "repo_level_abstain": repo_level[:1]}
    json.dump(out, open(outp, "w"), indent=1)
    print(f"{name}: pkgs={len(pk)} eligible={len(eligible)} sampled_constructs={len(rows)} SILENT={silent} | all-files constructs={full_total} unaccounted={full_unaccounted}")

main()
