#!/usr/bin/env python3
"""INIT-DISCOVERY-SLICE3 pre-draft evidence: simulate Node's exports
resolution over every exports-blocked import STAGE3-EVAL-001 recorded.

Usage: simulate_exports.py <git_dir_at_pinned_sha> <phase1 invar.toml> <out.json>
The git dir may be a blobless partial clone: files are listed with ls-tree,
manifests read with git show. Shares no code with the engine.

For each `# ABSTAIN UNRESOLVED <file>:<line> <Name> from "<spec>": package
<pkg> declares "exports"` line, resolve <spec> through <pkg>'s exports map
(PACKAGE_EXPORTS_RESOLVE: exact subpath key, else the single-* pattern key
with the longest prefix; conditions walked in the object's own key order,
first key in the active set wins; arrays are fallbacks; null excludes) and
classify where the target lands, probed the way init already probes
(.js maps to .ts/.tsx; extensionless tries .ts, .tsx, /index.ts, /index.tsx).
Variants:
  node   active conditions {types, import, require, default}
  src    {source, development} prepended (a declared custom-condition set)
  outdir node, then a tsconfig rootDir/outDir back-map for non-source hits
"""
import json, re, subprocess, sys
from collections import Counter

def git(d, *a):
    return subprocess.run(["git", "-C", d, *a], capture_output=True, text=True).stdout

def main():
    d, toml, outp = sys.argv[1:4]
    files = set(git(d, "ls-tree", "-r", "--name-only", "HEAD").splitlines())
    pkgdir = {}
    for f in files:
        if f.endswith("package.json") and "node_modules/" not in f:
            try:
                m = json.loads(git(d, "show", f"HEAD:{f}"))
            except Exception:
                continue
            n = m.get("name")
            if isinstance(n, str):
                pkgdir.setdefault(n, []).append((f.rsplit("/", 1)[0] if "/" in f else "", m))
    rx = re.compile(r'^# ABSTAIN UNRESOLVED (\S+):(\d+) (\S+) from "([^"]+)": package (\S+) declares "exports"')
    rows = [m.groups() for m in map(rx.match, open(toml).read().splitlines()) if m]
    tsc_cache = {}

    def probe(base):
        if base.endswith(".d.ts"):
            return None
        if base.endswith(".js") or base.endswith(".mjs") or base.endswith(".cjs"):
            stem = base.rsplit(".", 1)[0]
            c = [stem + ".ts", stem + ".tsx", stem + ".mts", stem + ".cts"]
        elif base.endswith((".ts", ".tsx", ".mts", ".cts")):
            c = [base]
        else:
            c = [base + ".ts", base + ".tsx", base + "/index.ts", base + "/index.tsx"]
        return next((x for x in c if x in files), None)

    def norm(p):
        out = []
        for s in p.split("/"):
            if s in ("", "."):
                continue
            if s == "..":
                if not out:
                    return None
                out.pop()
            else:
                out.append(s)
        return "/".join(out)

    def target(t, conds, sub_star):
        if t is None:
            return ("null-excluded", None)
        if isinstance(t, str):
            if not t.startswith("./"):
                return ("invalid-target", t)
            return ("target", t.replace("*", sub_star) if sub_star is not None else t)
        if isinstance(t, list):
            for x in t:
                r = target(x, conds, sub_star)
                if r[0] == "target":
                    return r
            return ("no-valid-fallback", None)
        if isinstance(t, dict):
            for k, v in t.items():
                if k == "default" or k in conds:
                    r = target(v, conds, sub_star)
                    if r[0] != "no-condition":
                        return r
            return ("no-condition", None)
        return ("invalid-target", None)

    def resolve(exp, sub, conds):
        if isinstance(exp, (str, list)) or (isinstance(exp, dict) and exp and not any(k.startswith(".") for k in exp)):
            exp = {".": exp}
        if not isinstance(exp, dict):
            return ("invalid-map", None)
        if any(k.startswith(".") for k in exp) and not all(k.startswith(".") for k in exp):
            return ("invalid-map", None)
        if sub in exp:
            return target(exp[sub], conds, None)
        best = None
        for k in exp:
            if k.count("*") == 1:
                pre, suf = k.split("*")
                if sub.startswith(pre) and sub.endswith(suf) and len(sub) >= len(pre) + len(suf):
                    if best is None or len(pre) > len(best[1]) or (len(pre) == len(best[1]) and len(k) > len(best[0])):
                        best = (k, pre, suf)
        if best:
            k, pre, suf = best
            return target(exp[k], conds, sub[len(pre):len(sub) - len(suf)])
        if any(k.endswith("/") for k in exp):
            return ("folder-mapping", None)
        return ("no-subpath-match", None)

    def tsconfig_map(pdir, names):
        key = (pdir, names)
        if key in tsc_cache:
            return tsc_cache[key]
        r = None
        for nm in names:
          p = (pdir + "/" if pdir else "") + nm
          if r is None and p in files:
            txt = git(d, "show", f"HEAD:{p}")
            o = re.search(r'"outDir"\s*:\s*"([^"]+)"', txt)
            ro = re.search(r'"rootDir"\s*:\s*"([^"]+)"', txt)
            if o and ro:
                pre = (pdir + "/") if pdir else ""
                oo, rr = norm(pre + o.group(1)), norm(pre + ro.group(1))
                if oo is not None and rr is not None:
                    r = (oo, rr)
        tsc_cache[key] = r
        return r

    def classify(pdir, res):
        kind, t = res
        if kind != "target":
            return kind
        full = norm((pdir + "/" if pdir else "") + t[2:])
        if full is None:
            return "leaves-package"
        if probe(full):
            return "TS-SOURCE"
        if full.endswith(".d.ts"):
            return "declaration-d.ts"
        return "non-source(js/dist)"

    V = ("node", "src", "outdir-tsconfig", "outdir-tsconfig+build", "src+outdir")
    out = {v: Counter() for v in V}
    conds_seen = Counter()
    pkgs = Counter()
    def walk(x):
        if isinstance(x, dict):
            for k, v in x.items():
                if not k.startswith("."):
                    conds_seen[k] += 1
                walk(v)
        elif isinstance(x, list):
            for v in x: walk(v)
    seen_pkg = set()
    def backmap(pdir, rn, c, names):
        if c not in ("non-source(js/dist)", "declaration-d.ts"):
            return c
        tm = tsconfig_map(pdir, names)
        t2 = norm((pdir + "/" if pdir else "") + rn[1][2:])
        if tm and t2:
            o, r = tm
            if t2.startswith(o + "/"):
                back = (r + t2[len(o):]) if r else t2[len(o) + 1:]
                back = re.sub(r"\.d\.ts$", ".ts", back)
                if probe(back):
                    return "TS-SOURCE(via outDir)"
        return c
    for file, line, name, spec, pkg in rows:
        cands = pkgdir.get(pkg, [])
        if len(cands) != 1:
            for v in out: out[v]["pkg-not-unique"] += 1
            continue
        pdir, man = cands[0]
        sub = "." + spec[len(pkg):] if spec != pkg else "."
        exp = man.get("exports")
        if pkg not in seen_pkg:
            seen_pkg.add(pkg); walk(exp)
        pkgs[pkg] += 1
        base = {"types", "import", "require", "default"}
        rn = resolve(exp, sub, base)
        rs = resolve(exp, sub, base | {"source", "development"})
        cn, cs = classify(pdir, rn), classify(pdir, rs)
        out["node"][cn] += 1
        out["src"][cs] += 1
        out["outdir-tsconfig"][backmap(pdir, rn, cn, ("tsconfig.json",))] += 1
        out["outdir-tsconfig+build"][backmap(pdir, rn, cn, ("tsconfig.json", "tsconfig.build.json"))] += 1
        out["src+outdir"][backmap(pdir, rs, cs, ("tsconfig.json", "tsconfig.build.json"))] += 1
    res = {"repo": d.rstrip("/").rsplit("/", 1)[-1], "blocked_imports": len(rows),
           "packages": dict(pkgs.most_common()), "conditions_seen": dict(conds_seen.most_common()), **{v: dict(out[v].most_common()) for v in out}}
    json.dump(res, open(outp, "w"), indent=1)
    print(json.dumps({k: res[k] for k in ("repo", "blocked_imports", *V)}))

main()
