#!/usr/bin/env python3
"""R6 measurement probe (throwaway). For every R6 abstention of the landed
engine (d4488ac), ask whether the consumer file holds a receiver that text
alone can tie to the shape WITHOUT importing it by name, by one of:
  NS   a namespace import from the producer's package, `r: ns.S` (and the
       SLICE5 wrappers and arrays);
  IMP  an inline `r: import("spec").S` annotation;
  RET  `r = [await] f(...)` where f is imported by name from the producer's
       package and exactly one declaration of f in that package's sources
       declares a return type whose head is S, `Promise<S>`, `S | null`,
       `S[]`, `Array<S>` or `Promise<S[]>` (arrays bind elements: for-of,
       the array methods, `a[i]`);
and whether at least two distinctive fields of S are read through such a
receiver (dot reads, writes excluded, the frozen stoplist), i.e. whether a
SLICE5-style proposal would follow. Receivers are names, not scopes, as in
SLICE5 (B4). Every R6 line gets exactly one class:
  YIELD-<route>      >= 2 distinctive verified reads (a candidate proposal)
  ROUTE-THIN-<route> a route exists but < 2 verified reads
  NO-ROUTE           no receiver tied to S by any of the three routes
Usage: r6probe.py <clones dir> <eval dir> <out.tsv>"""
import json, os, re, sys
from collections import Counter

clones, evd, outp = sys.argv[1:4]
STOP = {"id", "name", "type", "description", "data", "value", "payload", "key", "label", "title",
        "status", "error", "message", "result", "items", "url", "path", "version", "createdat", "updatedat"}
nk = lambda s: s.lower().replace("_", "").replace("-", "")
SKIP = {"node_modules", ".git", "dist", "build", "out", ".next", "coverage"}
R6 = re.compile(r"^# ABSTAIN R6 (\S+):(\d+) reads (.+) of (\S+) \((\S+):(\d+)\) without importing it by name")

def mask(t):
    out, i, n = list(t), 0, len(t)
    def blank(a, b):
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "
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

def read(p):
    try:
        if os.path.getsize(p) > 1_000_000:
            return None
        return open(p, encoding="utf-8").read()
    except Exception:
        return None

def pkg_root(root, rel):
    d = os.path.dirname(rel)
    while True:
        pj = os.path.join(root, d, "package.json")
        if d and os.path.exists(pj):
            try:
                return d, json.load(open(pj)).get("name")
            except Exception:
                return d, None
        if not d:
            return "", None
        d = os.path.dirname(d)

src_cache = {}
def pkg_sources(root, d):
    key = (root, d)
    if key not in src_cache:
        out = {}
        for dd, ds, fs in os.walk(os.path.join(root, d)):
            ds[:] = sorted(x for x in ds if x not in SKIP)
            for f in sorted(fs):
                if f.endswith((".ts", ".tsx")) and not f.endswith(".d.ts"):
                    t = read(os.path.join(dd, f))
                    if t is not None:
                        out[os.path.relpath(os.path.join(dd, f), root)] = mask(t)
        src_cache[key] = out
    return src_cache[key]

def head_of(ty, S):
    """Is S the head of type text ty? Returns 'one', 'arr' or None."""
    t = ty.strip()
    t = re.sub(r"^Promise\s*<\s*(.*)>$", r"\1", t, flags=re.S).strip()
    t = re.sub(r"\s*\|\s*(null|undefined)\b", "", t).strip()
    t = re.sub(r"^(readonly\s+)?", "", t)
    m = re.match(rf"^(?:Readonly|Partial|Required)\s*<\s*{re.escape(S)}\b(?:<[^>]*>)?\s*>$", t)
    if m or re.match(rf"^{re.escape(S)}(?:<[^>]*>)?$", t):
        return "one"
    if re.match(rf"^(?:Array|ReadonlyArray)\s*<\s*{re.escape(S)}\b(?:<[^>]*>)?\s*>$", t) or re.match(rf"^{re.escape(S)}(?:<[^>]*>)?\s*\[\s*\]$", t):
        return "arr"
    return None

ret_cache = {}
def returns(root, d, fname, S):
    """Declared return types of fname in the package's sources."""
    key = (root, d, fname)
    if key not in ret_cache:
        tys = []
        f = re.escape(fname)
        for rel, t in pkg_sources(root, d).items():
            if fname not in t:
                continue
            for m in re.finditer(rf"\bfunction\s+{f}\s*(?:<[^>]*>)?\s*\(", t):
                tys.append(after_params(t, m.end() - 1))
            for m in re.finditer(rf"\b(?:const|let)\s+{f}\s*=\s*(?:async\s*)?(?:<[^>]*>)?\s*\(", t):
                tys.append(after_params(t, m.end() - 1))
        ret_cache[key] = [x for x in tys if x is not None]
    tys = ret_cache[key]
    if len(tys) != 1:
        return None
    return head_of(tys[0], S)

def after_params(t, open_at):
    d, i = 0, open_at
    while i < len(t):
        if t[i] == "(":
            d += 1
        elif t[i] == ")":
            d -= 1
            if d == 0:
                break
        i += 1
    m = re.match(r"\s*:\s*([^{=;]+?)\s*(?:=>|\{|;|$)", t[i + 1:i + 300])
    return m.group(1) if m else None

def imports(m):
    named, ns = [], []
    for x in re.finditer(r"import\s+(?:type\s+)?\{([^}]*)\}\s*from\s*['\"]([^'\"]+)['\"]", m, re.S):
        pass
    return named, ns

def receivers(src, m, S, spec_ok, fn_ok):
    """(single, array, routes) receivers tied to S without naming it."""
    one, arr, route = set(), set(), {}
    s = re.escape(S)
    nss = [x.group(1) for x in re.finditer(r"import\s+\*\s+as\s+([A-Za-z_$][\w$]*)\s+from\s*['\"]([^'\"]+)['\"]", src) if spec_ok(x.group(2))]
    for ns in nss:
        for x in re.finditer(rf"(?<![\w$.])([A-Za-z_$][\w$]*)\s*\??\s*:\s*(?:readonly\s+)?(?:(Readonly|Partial|Required|Array|ReadonlyArray)\s*<\s*)?{re.escape(ns)}\s*\.\s*{s}\b(?:<[^>]*>)?(\s*\[\s*\])?", m):
            a = bool(x.group(3)) or x.group(2) in ("Array", "ReadonlyArray")
            (arr if a else one).add(x.group(1)); route[x.group(1)] = "NS"
    for x in re.finditer(rf"(?<![\w$.])([A-Za-z_$][\w$]*)\s*\??\s*:\s*import\s*\(\s*['\"]([^'\"]+)['\"]\s*\)\s*\.\s*{s}\b(\s*\[\s*\])?", src):
        if spec_ok(x.group(2)):
            (arr if x.group(3) else one).add(x.group(1)); route[x.group(1)] = "IMP"
    for x in re.finditer(r"(?<![\w$.])([A-Za-z_$][\w$]*)\s*=\s*(?:await\s+)?([A-Za-z_$][\w$]*)\s*(?:<[^>]*>)?\s*\(", m):
        r, f = x.group(1), x.group(2)
        k = fn_ok(f)
        if k:
            (arr if k == "arr" else one).add(r); route[r] = "RET"
    for a in list(arr):
        for x in re.finditer(rf"\bfor\s*\(\s*(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s+of\s+{re.escape(a)}\s*\)", m):
            one.add(x.group(1)); route[x.group(1)] = route[a]
        for x in re.finditer(rf"(?<![\w$.]){re.escape(a)}\s*\??\.\s*(?:map|filter|forEach|find|findLast|findIndex|some|every|flatMap|sort)\s*\(\s*\(?\s*([A-Za-z_$][\w$]*)", m):
            one.add(x.group(1)); route[x.group(1)] = route[a]
    return one, arr, route

def verified(m, field, one, arr):
    f = re.escape(field)
    w = r"(?!\s*(?:=(?!=)|\+=|-=|\*=|/=|\+\+|--))"
    hits = []
    for r in one:
        for x in re.finditer(rf"(?<![\w$.]){re.escape(r)}\s*\??\.\s*{f}\b{w}", m):
            hits.append(r)
    for a in arr:
        for x in re.finditer(rf"(?<![\w$.]){re.escape(a)}\s*\[[^\]\n]*\]\s*\??\.\s*{f}\b{w}", m):
            hits.append(a)
    return hits

rows = []
for tf in sorted(os.listdir(evd)):
    if not tf.endswith(".invar.toml"):
        continue
    repo = tf[: -len(".invar.toml")]
    root = os.path.join(clones, repo)
    if not os.path.isdir(root):
        continue
    for l in open(os.path.join(evd, tf)):
        x = R6.match(l)
        if not x:
            continue
        cf, cl, fields, S, g, gl = x.groups()
        fields = [y.strip() for y in fields.split(",")]
        src = read(os.path.join(root, cf)) or ""
        m = mask(src)
        d, pname = pkg_root(root, g)
        def spec_ok(spec, pname=pname):
            return bool(pname) and (spec == pname or spec.startswith(pname + "/"))
        named = {}
        for y in re.finditer(r"import\s+(?:type\s+)?\{([^}]*)\}\s*from\s*['\"]([^'\"]+)['\"]", src, re.S):
            if spec_ok(y.group(2)):
                for b in y.group(1).split(","):
                    b = re.sub(r"^\s*type\s+", "", b).strip()
                    if b:
                        orig, _, loc = b.partition(" as ")
                        named[(loc or orig).strip()] = orig.strip()
        def fn_ok(f, d=d, root=root, S=S, named=named):
            return returns(root, d, named[f], S) if f in named else None
        one, arr, route = receivers(src, m, S, spec_ok, fn_ok)
        if not (one or arr):
            rows.append((repo, cf, cl, S, g, "NO-ROUTE", "", "")); continue
        ok, via = [], set()
        for fl in fields:
            if nk(fl) in STOP:
                continue
            h = verified(m, fl, one, arr)
            if h:
                ok.append(fl); via |= {route[r] for r in h}
        rr = "+".join(sorted(set(route.values())))
        if len(ok) >= 2:
            rows.append((repo, cf, cl, S, g, "YIELD-" + "+".join(sorted(via)), ",".join(ok), ",".join(sorted(one | arr))))
        else:
            rows.append((repo, cf, cl, S, g, "ROUTE-THIN-" + rr, ",".join(ok), ",".join(sorted(one | arr))))
with open(outp, "w") as w:
    w.write("repo\tfile\tline\tshape\tdecl\tclass\tverified_fields\treceivers\n")
    for r in rows:
        w.write("\t".join(r) + "\n")
c = Counter(r[5] for r in rows)
print("R6 lines", len(rows))
for k, v in sorted(c.items()):
    print(f"  {k}: {v}")
by = Counter((r[0], r[5].split("-")[0]) for r in rows)
for k, v in sorted(by.items()):
    print(" ", k, v)
