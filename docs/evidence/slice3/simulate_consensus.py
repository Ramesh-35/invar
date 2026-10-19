#!/usr/bin/env python3
"""SLICE3 evidence, variant `consensus` (the drafted rule): order-free.
Every branch under an active condition {types, import, require, default}
is followed (arrays: first valid element, their order is preserved by any
JSON parser); each leaf target is probed directly, then back-mapped
through the package's own tsconfig.json or tsconfig.build.json
(outDir/declarationDir to rootDir, .d.ts and .js to .ts/.tsx). Resolve
iff every branch that reaches a source names the SAME file; two different
files is AMBIG-EXPORTS; none is UNRESOLVED with the reason of the first
leaf in sorted order. Usage: simulate_consensus.py <git_dir> <invar.toml> <out.json>
"""
import json, re, subprocess, sys
from collections import Counter
ACTIVE = {"types", "import", "require", "default"}

def git(d, *a):
    return subprocess.run(["git", "-C", d, *a], capture_output=True, text=True).stdout

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

def jsonc(txt):
    o, i, n, s = [], 0, len(txt), False
    while i < n:
        c = txt[i]
        if s:
            o.append(c)
            if c == "\\" and i + 1 < n:
                o.append(txt[i + 1]); i += 2; continue
            if c == '"':
                s = False
            i += 1; continue
        if c == '"':
            s = True; o.append(c); i += 1; continue
        if txt.startswith("//", i):
            j = txt.find("\n", i); i = n if j < 0 else j; continue
        if txt.startswith("/*", i):
            j = txt.find("*/", i + 2); i = n if j < 0 else j + 2; continue
        o.append(c); i += 1
    t = re.sub(r",(\s*[}\]])", r"\1", "".join(o))
    return json.loads(t)

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
            if isinstance(m.get("name"), str):
                pkgdir.setdefault(m["name"], []).append((f.rsplit("/", 1)[0] if "/" in f else "", m))
    rx = re.compile(r'^# ABSTAIN UNRESOLVED (\S+):(\d+) (\S+) from "([^"]+)": package (\S+) declares "exports"')
    rows = [m.groups() for m in map(rx.match, open(toml).read().splitlines()) if m]
    tsc = {}

    def maps(pdir):
        if pdir in tsc:
            return tsc[pdir]
        found = []
        for nm in ("tsconfig.json", "tsconfig.build.json"):
            p = (pdir + "/" if pdir else "") + nm
            if p not in files:
                continue
            try:
                co = jsonc(git(d, "show", f"HEAD:{p}")).get("compilerOptions", {}) or {}
            except Exception:
                continue
            rd = co.get("rootDir")
            pre = (pdir + "/") if pdir else ""
            for key in ("outDir", "declarationDir"):
                od = co.get(key)
                if isinstance(od, str) and isinstance(rd, str):
                    a, b = norm(pre + od), norm(pre + rd)
                    if a is not None and b is not None:
                        found.append((key, a, b))
        tsc[pdir] = found
        return found

    def probe(base):
        if base.endswith(".d.ts"):
            stem = base[:-5]; c = [stem + ".ts", stem + ".tsx"]
        elif base.endswith(".js"):
            stem = base[:-3]; c = [stem + ".ts", stem + ".tsx"]
        elif base.endswith((".ts", ".tsx")):
            c = [base]
        else:
            c = [base + ".ts", base + ".tsx", base + "/index.ts", base + "/index.tsx"]
        return next((x for x in c if x in files), None)

    def leaves(t, star, acc):
        if t is None:
            acc.append(("null-excluded", None)); return
        if isinstance(t, str):
            if not t.startswith("./"):
                acc.append(("invalid-target", None)); return
            v = t.replace("*", star) if star is not None else t
            if any(seg in ("..", "node_modules") for seg in v.split("/")):
                acc.append(("invalid-target", None)); return
            acc.append(("target", v)); return
        if isinstance(t, list):
            for x in t:
                sub = []
                leaves(x, star, sub)
                if any(k == "target" for k, _ in sub):
                    acc.extend(sub); return
            acc.append(("no-valid-fallback", None)); return
        if isinstance(t, dict):
            hit = False
            for k in sorted(t):
                if k in ACTIVE:
                    hit = True; leaves(t[k], star, acc)
            if not hit:
                acc.append(("no-active-condition", None))
            return
        acc.append(("invalid-target", None))

    def resolve(exp, sub):
        if isinstance(exp, (str, list)) or (isinstance(exp, dict) and exp and not any(k.startswith(".") for k in exp)):
            exp = {".": exp}
        if not isinstance(exp, dict) or (any(k.startswith(".") for k in exp) and not all(k.startswith(".") for k in exp)):
            return [("invalid-map", None)]
        acc = []
        if sub in exp:
            leaves(exp[sub], None, acc); return acc
        best = None
        for k in exp:
            if k.count("*") == 1:
                pre, suf = k.split("*")
                if sub.startswith(pre) and sub.endswith(suf) and len(sub) >= len(pre) + len(suf):
                    if best is None or len(pre) > len(best[1]) or (len(pre) == len(best[1]) and len(k) > len(best[0])):
                        best = (k, pre, suf)
        if best:
            k, pre, suf = best
            leaves(exp[k], sub[len(pre):len(sub) - len(suf)], acc); return acc
        return [("no-subpath-match", None)]

    out, pk = Counter(), Counter()
    hits = []
    for file, line, name, spec, pkg in rows:
        c = pkgdir.get(pkg, [])
        if len(c) != 1:
            out["pkg-not-unique"] += 1; continue
        pdir, man = c[0]
        sub = "." + spec[len(pkg):] if spec != pkg else "."
        srcs, reasons = set(), []
        for kind, t in resolve(man.get("exports"), sub):
            if kind != "target":
                reasons.append(kind); continue
            full = norm((pdir + "/" if pdir else "") + t[2:])
            if full is None:
                reasons.append("leaves-package"); continue
            found = set()
            direct = probe(full)
            if direct:
                found.add(direct)
            else:
                for key, od, rd in maps(pdir):
                    if (full.endswith(".d.ts") or key == "outDir") and full.startswith(od + "/"):
                        back = (rd + "/" if rd else "") + full[len(od) + 1:]
                        h = probe(back)
                        if h:
                            found.add(h)
            if found:
                srcs |= found
            else:
                reasons.append("declaration-d.ts" if full.endswith(".d.ts") else "non-source")
        if len(srcs) == 1:
            out["TS-SOURCE"] += 1; pk[pkg] += 1; hits.append((name, next(iter(srcs))))
        elif len(srcs) > 1:
            out["AMBIG-EXPORTS"] += 1
        else:
            out[sorted(reasons)[0] if reasons else "nothing"] += 1
    cache, named, suspects = {}, Counter(), []
    for nm, f in hits:
        if f not in cache:
            cache[f] = git(d, "show", f"HEAD:{f}")
        named["names-it" if re.search(r"\b" + re.escape(nm) + r"\b", cache[f]) else ("star-reexport-barrel" if re.search(r"export\s+\*\s+from", cache[f]) else "SUSPECT-neither")] += 1
        if not re.search(r"\b" + re.escape(nm) + r"\b", cache[f]) and not re.search(r"export\s+\*\s+from", cache[f]):
            suspects.append((nm, f))
    res = {"repo": d.rstrip("/").rsplit("/", 1)[-1], "blocked_imports": len(rows), "consensus": dict(out.most_common()), "resolved_by_package": dict(pk.most_common()), "sanity_resolved_file_names_the_import": dict(named), "suspects": suspects[:10]}
    json.dump(res, open(outp, "w"), indent=1)
    print(json.dumps({k: res[k] for k in ("repo", "blocked_imports", "consensus", "sanity_resolved_file_names_the_import")}))

main()
