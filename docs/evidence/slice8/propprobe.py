#!/usr/bin/env python3
"""Property-type measurement probe (throwaway, SLICE8 draft evidence).

For every R6 abstention of the SLICE7 engine, ask whether the consumer file
reads at least two distinctive fields of the shape S through a PROPERTY whose
written type is S, i.e. a path the engine cannot follow today:

  PROP1  r.p.f        r has a written type T in this file, T's declaration
                      (found by name in the repository) has member p of type S
  PROP2  r.a.p.f      the same with one more hop (T.a: U, U.p: S)
  THIS   this.p.f     a class in this file declares p: S
  plus aliases of a property value: `const y = r.p` then y.f, and the
  elements of an array-typed property: for-of, the array callbacks, y[i].f.

Types are read as written, never inferred. A member type counts when its head
is S, S[], Array<S>, Readonly<S>, or S | undefined / S | null (recorded
separately, since SLICE7 excludes unions). Names, not scopes, as SLICE5.
T is resolved by name: exactly one interface or type declaration named T in
the repository (UNIQ), or several (AMBIG, recorded, not counted).

Every R6 line gets exactly one class:
  YIELD-<routes>   >= 2 distinctive fields read through such paths
  THIN-<routes>    a path exists but < 2 fields
  NONE             no such path
Usage: propprobe.py <clones dir> <eval dir> <out.tsv>"""
import os, re, sys
from collections import Counter, defaultdict

clones, evd, outp = sys.argv[1:4]
STOP = {"id", "name", "type", "description", "data", "value", "payload", "key", "label", "title",
        "status", "error", "message", "result", "items", "url", "path", "version", "createdat", "updatedat"}
nk = lambda s: s.lower().replace("_", "").replace("-", "")
SKIP = {"node_modules", ".git", "dist", "build", "out", ".next", "coverage", "vendor"}
R6 = re.compile(r"^# ABSTAIN R6 (\S+):(\d+) reads (.+) of (\S+) \((\S+):(\d+)\) without importing it by name")
ID = r"[A-Za-z_$][\w$]*"
WRITE = r"(?!\s*(?:=(?!=)|\+=|-=|\*=|/=|\+\+|--))"


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


decl_cache = {}


def type_decls(root):
    """name -> list of (file, member dict {prop: type text}) for every
    interface / object type alias in the repository."""
    if root in decl_cache:
        return decl_cache[root]
    out = defaultdict(list)
    for dd, ds, fs in os.walk(root):
        ds[:] = sorted(x for x in ds if x not in SKIP)
        for f in sorted(fs):
            if not f.endswith((".ts", ".tsx")) or f.endswith(".d.ts"):
                continue
            t = read(os.path.join(dd, f))
            if t is None:
                continue
            m = mask(t)
            rel = os.path.relpath(os.path.join(dd, f), root)
            for x in re.finditer(rf"\b(?:interface\s+({ID})(?:\s*<[^>{{]*>)?(?:\s+extends\s+[^{{]+)?\s*\{{|type\s+({ID})(?:\s*<[^>=]*>)?\s*=\s*\{{)", m):
                name = x.group(1) or x.group(2)
                body = block(m, x.end() - 1)
                out[name].append((rel, members(body)))
    decl_cache[root] = out
    return out


def block(m, open_at):
    d, i = 0, open_at
    while i < len(m):
        if m[i] == "{":
            d += 1
        elif m[i] == "}":
            d -= 1
            if d == 0:
                return m[open_at + 1:i]
        i += 1
    return m[open_at + 1:]


def members(body):
    """Depth-1 members `p?: type` of a declaration body."""
    out, d, start = {}, 0, 0
    parts = []
    for i, c in enumerate(body):
        if c in "{([<":
            d += 1
        elif c in "})]>" and not (c == ">" and i and body[i - 1] == "="):
            d -= 1
        elif d == 0 and c in ";,\n":
            parts.append(body[start:i]); start = i + 1
    parts.append(body[start:])
    for p in parts:
        x = re.match(rf"^\s*(?:readonly\s+)?({ID})\s*\??\s*:\s*(.+?)\s*$", p, re.S)
        if x:
            out[x.group(1)] = " ".join(x.group(2).split())
    return out


def head(ty, S):
    """'one', 'arr', 'one-u', 'arr-u' when S heads the type text, else None."""
    t = ty.strip()
    u = ""
    t2 = re.sub(r"\s*\|\s*(?:null|undefined)\b", "", t).strip()
    if t2 != t:
        u, t = "-u", t2
    t = re.sub(r"^readonly\s+", "", t)
    s = re.escape(S)
    if re.fullmatch(rf"(?:Readonly\s*<\s*)?{s}(?:\s*<[^>]*>)?(?:\s*>)?", t):
        return "one" + u
    if re.fullmatch(rf"(?:(?:Readonly)?Array\s*<\s*{s}(?:\s*<[^>]*>)?\s*>|{s}(?:\s*<[^>]*>)?\s*\[\s*\])", t):
        return "arr" + u
    return None


def tname(ty):
    """The type name of a written receiver type: T, T[], Array<T>, Readonly<T>."""
    t = re.sub(r"\s*\|\s*(?:null|undefined)\b", "", ty).strip()
    t = re.sub(r"^readonly\s+", "", t)
    x = re.fullmatch(rf"(?:Readonly\s*<\s*)?({ID})(?:\s*<[^>]*>)?(?:\s*>)?", t)
    if x:
        return x.group(1), False
    x = re.fullmatch(rf"(?:(?:Readonly)?Array\s*<\s*({ID})(?:\s*<[^>]*>)?\s*>|({ID})(?:\s*<[^>]*>)?\s*\[\s*\])", t)
    if x:
        return x.group(1) or x.group(2), True
    return None, False


def typed_receivers(m):
    """name -> set of (type name, is_array) written in this file."""
    out = defaultdict(set)
    for x in re.finditer(rf"(?<![\w$.])({ID})\s*\??\s*:\s*((?:readonly\s+)?{ID}(?:\s*<[^<>]*(?:<[^<>]*>)?[^<>]*>)?(?:\s*\[\s*\])?(?:\s*\|\s*(?:null|undefined))?)", m):
        n, arr = tname(x.group(2))
        if n:
            out[x.group(1)].add((n, arr))
    return out


def fields_through(m, base, S_paths, fields):
    """Fields of `fields` read through `base` (a regex) + one of S_paths
    (property chains ending in S). Returns {field: route}."""
    hit = {}
    for chain, form, route in S_paths:
        acc = base + "".join(rf"\s*[?!]?\.\s*{re.escape(c)}" for c in chain)
        exprs = [acc]
        # aliases `const y = <acc>;`
        for x in re.finditer(rf"\b(?:const|let|var)\s+({ID})\s*=\s*{acc}\s*[;\n]", m):
            exprs.append(re.escape(x.group(1)))
        elems = []
        if form.startswith("arr"):
            for e in list(exprs):
                for x in re.finditer(rf"\bfor\s*\(\s*(?:const|let|var)\s+({ID})\s+of\s+{e}\s*\)", m):
                    elems.append(re.escape(x.group(1)))
                for x in re.finditer(rf"{e}\s*\??\.\s*(?:map|filter|forEach|find|findLast|some|every|flatMap)\s*\(\s*\(?\s*({ID})", m):
                    elems.append(re.escape(x.group(1)))
            idx = [e + r"\s*\[[^\]\n]*\]" for e in exprs]
            singles = idx + [rf"(?<![\w$.]){e}" for e in elems]
        else:
            singles = [rf"(?<![\w$.]){e}" for e in exprs]
        for f in fields:
            if f in hit or nk(f) in STOP:
                continue
            for s in singles:
                if re.search(rf"{s}\s*[?!]?\.\s*{re.escape(f)}\b{WRITE}", m):
                    hit[f] = route
                    break
    return hit


rows = []
for tf in sorted(os.listdir(evd)):
    if not tf.endswith(".invar.toml"):
        continue
    repo = tf[: -len(".invar.toml")]
    root = os.path.join(clones, repo)
    if not os.path.isdir(root):
        continue
    decls = type_decls(root)
    for l in open(os.path.join(evd, tf)):
        x = R6.match(l)
        if not x:
            continue
        cf, cl, fields, S, g, gl = x.groups()
        fields = [y.strip() for y in fields.split(",")]
        src = read(os.path.join(root, cf)) or ""
        m = mask(src)
        recv = typed_receivers(m)
        hits, routes, ambig = {}, set(), 0
        for r, types in recv.items():
            for T, r_arr in types:
                ds = decls.get(T, [])
                if len(ds) > 1:
                    ambig += 1
                    continue
                if len(ds) != 1:
                    continue
                paths = []
                for p, ty in ds[0][1].items():
                    h = head(ty, S)
                    if h:
                        paths.append(((p,), h, "PROP1" + ("-u" if h.endswith("-u") else "")))
                    if ty.startswith("{") and ty.endswith("}"):
                        for p2, ty2 in members(ty[1:-1]).items():
                            h2 = head(ty2, S)
                            if h2:
                                paths.append(((p, p2), h2, "PROP2" + ("-u" if h2.endswith("-u") else "")))
                    n2, a2 = tname(ty)
                    if n2 and n2 != S and len(decls.get(n2, [])) == 1:
                        for p2, ty2 in decls[n2][0][1].items():
                            h2 = head(ty2, S)
                            if h2 and not a2:
                                paths.append(((p, p2), h2, "PROP2" + ("-u" if h2.endswith("-u") or "|" in ty else "")))
                if not paths:
                    continue
                bases = [re.escape(r)] if not r_arr else [re.escape(r) + r"\s*\[[^\]\n]*\]"]
                if r_arr:
                    # elements of an array receiver
                    for y in re.finditer(rf"\bfor\s*\(\s*(?:const|let|var)\s+({ID})\s+of\s+{re.escape(r)}\s*\)", m):
                        bases.append(re.escape(y.group(1)))
                for b in bases:
                    h = fields_through(m, b, paths, fields)
                    for f, rt in h.items():
                        hits.setdefault(f, rt); routes.add(rt)
                if paths:
                    routes |= {"path:" + pp[2] for pp in paths}
        # this.p where a class in this file declares p: S
        tp = []
        for y in re.finditer(rf"(?:private|public|protected|readonly|\n)\s*(?:readonly\s+)?({ID})\s*[?!]?\s*:\s*([^;=\n]+)", m):
            h = head(y.group(2), S)
            if h:
                tp.append(((y.group(1),), h, "THIS" + ("-u" if h.endswith("-u") else "")))
        if tp:
            routes |= {"path:" + t[2] for t in tp}
            for f, rt in fields_through(m, "this", tp, fields).items():
                hits.setdefault(f, rt); routes.add(rt)
        got = sorted(hits)
        rts = sorted({hits[f] for f in got})
        paths_seen = sorted(r[5:] for r in routes if r.startswith("path:"))
        if len(got) >= 2:
            cls = "YIELD-" + "+".join(rts)
        elif paths_seen:
            cls = "THIN-" + "+".join(sorted(set(paths_seen)))
        else:
            cls = "NONE"
        rows.append((repo, cf, cl, S, g, cls, ",".join(got), str(ambig)))
with open(outp, "w") as w:
    w.write("repo\tfile\tline\tshape\tdecl\tclass\tverified_fields\tambiguous_receiver_types\n")
    for r in rows:
        w.write("\t".join(r) + "\n")
c = Counter(r[5].split("-")[0] for r in rows)
print("R6 lines", len(rows))
for k, v in sorted(Counter(r[5] for r in rows).items()):
    print(f"  {k}: {v}")
for k, v in sorted(Counter((r[0], r[5].split("-")[0]) for r in rows).items()):
    print(" ", k, v)
