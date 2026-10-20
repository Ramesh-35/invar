#!/usr/bin/env python3
"""Receiver-verification prototype (throwaway, for the SLICE5 draft).

Rule under test: a field read `r.f` counts for shape S only when the
receiver r is verified as S in the consumer file:
  B1 r is annotated with S: `r: S`, `r?: S`, `r: readonly S[]`, `r: S[]`,
     `r: Array<S>`, `r: ReadonlyArray<S>`, `r: Readonly<S>`, `r: Partial<S>`,
     `r: Required<S>`, `r: S | null | undefined` (S first in the type);
  B2 r is assigned a value cast to S: `r = <expr> as S` in one statement;
  B3 r is an element of a binding verified as an array of S:
     `for (const r of a)`, `a.map((r` / `.filter` / `.forEach` / `.find` /
     `.some` / `.every` / `.flatMap` / `.findLast` / `.sort`, or `a[...]`;
and the read is `r.f`, `r?.f`, `this.r.f`, or `a[...].f` for an array a.
A write (`r.f =`, `+=`, `-=`, `++`, `--`) never counts. Shapes keep the
two-distinctive-field rule; a pair keeps the shapes that still pass.
Usage: receiver.py <labels.tsv> <out.tsv>
labels.tsv: repo, eval, id, final  (eval: eval003 or eval004)"""
import csv, os, re, sys
EXT = os.environ.get("RV_EXT") == "1"
SCOPED = os.environ.get("RV_SCOPE") == "1"

STOP = {"id", "name", "type", "description", "data", "value", "payload", "key", "label", "title",
        "status", "error", "message", "result", "items", "url", "path", "version", "createdat", "updatedat"}
nk = lambda s: s.lower().replace("_", "").replace("-", "")
BASE = "/tmp/claude-0/-home-claude/0ad2ce1c-afee-57af-87cf-f7a4a128779f/scratchpad/s1"

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

def proposals(path):
    props, cur = {}, None
    for l in open(path).read().splitlines():
        if l.startswith("# PROPOSED "):
            cur = {"id": l[len("# PROPOSED "):], "ev": []}
        elif cur is not None and l.startswith("# "):
            cur["ev"].append(l[2:])
        elif cur is not None and l.startswith("consumer = "):
            cur["consumer"] = l.split('"')[1]
            props[cur["id"]] = cur; cur = None
    return props

def bindings(m, S):
    s = re.escape(S)
    head = rf"(?:readonly\s+)?(?:(?:Readonly|Partial|Required|Array|ReadonlyArray)\s*<\s*)?{s}\b(?!\s*[.\[]\w)"
    one, arr = set(), set()
    for x in re.finditer(rf"(?<![\w$.])([A-Za-z_$][\w$]*)\s*\??\s*:\s*({head})(\s*\[\s*\])?", m):
        name, ty, br = x.group(1), x.group(2), x.group(3)
        if br or re.match(r"(?:readonly\s+)?(?:Array|ReadonlyArray)\s*<", ty):
            arr.add(name)
        else:
            one.add(name)
    for x in re.finditer(rf"(?<![\w$.])([A-Za-z_$][\w$]*)\s*=[^;=][^;]*?\bas\s+(?:readonly\s+)?({s})\b(\s*\[\s*\])?", m):
        (arr if x.group(3) else one).add(x.group(1))
    if EXT:
        # E1 an explicit type argument names S on the call that produces r:
        # `r = [await] f<S ...>(`, `const [r, setR] = useState<S | null>(`
        targ = rf"<\s*(?:readonly\s+)?{s}\b[^>()]*>\s*\("
        for x in re.finditer(rf"(?<![\w$.])([A-Za-z_$][\w$]*)\s*=\s*(?:await\s+)?[\w$.]+\s*{targ}", m):
            one.add(x.group(1))
        for x in re.finditer(rf"\[\s*([A-Za-z_$][\w$]*)\s*,[^\]]*\]\s*=\s*[\w$.]+\s*{targ}", m):
            one.add(x.group(1))
        # E2 a function in this file declares S (or Promise<S>) as its return
        # type; a binding assigned its call result is S
        ret = rf"\)\s*:\s*(?:Promise\s*<\s*)?{s}\b(?!\s*\[)"
        fns = set(x.group(1) for x in re.finditer(rf"\bfunction\s+([A-Za-z_$][\w$]*)\s*(?:<[^>]*>)?\s*\([^)]*{ret}", m))
        fns |= set(x.group(1) for x in re.finditer(rf"\b(?:const|let)\s+([A-Za-z_$][\w$]*)\s*=\s*(?:async\s*)?\([^)]*{ret}", m))
        fns |= set(x.group(1) for x in re.finditer(rf"^\s*(?:(?:private|public|protected|static|async)\s+)*([A-Za-z_$][\w$]*)\s*\([^)]*{ret}", m, re.M))
        for fn in fns:
            for x in re.finditer(rf"(?<![\w$.])([A-Za-z_$][\w$]*)\s*=\s*(?:await\s+)?(?:this\.)?{re.escape(fn)}\s*\(", m):
                one.add(x.group(1))
    for a in list(arr):
        for x in re.finditer(rf"\bfor\s*\(\s*(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s+of\s+{re.escape(a)}\b", m):
            one.add(x.group(1))
        for x in re.finditer(rf"(?<![\w$.]){re.escape(a)}\s*\??\.\s*(?:map|filter|forEach|find|some|every|flatMap|findLast|sort|findIndex)\s*\(\s*\(?\s*([A-Za-z_$][\w$]*)", m):
            one.add(x.group(1))
    return one, arr

def blocks(m):
    """Innermost-block lookup: for every '{' its matching '}'."""
    st, spans = [], []
    for i, c in enumerate(m):
        if c == "{":
            st.append(i)
        elif c == "}" and st:
            spans.append((st.pop(), i))
    return spans

def scope_of(spans, pos):
    best = (0, 10**12)
    for a, b in spans:
        if a <= pos <= b and b - a < best[1] - best[0]:
            best = (a, b)
    return best

INTRO = r"(?:\b(?:const|let|var)\s+|[({,]\s*|\.\.\.\s*|^\s*(?:(?:private|public|protected|readonly|static)\s+)+){r}\b(?=\s*(?:[:=,)}\]]|\?\s*:|=>|\bof\b|\bin\b))|(?<![\w$.]){r}\s*=>"

def introductions(m, spans, r):
    """Every place the name r is (re)declared, with the block it governs."""
    out = []
    for x in re.finditer(INTRO.replace("{r}", re.escape(r)), m, re.M):
        pos = x.start() + x.group(0).rfind(r)
        out.append((pos, scope_of(spans, pos)))
    return out

def verified_reads(m, S, field):
    one, arr = bindings(m, S)
    if SCOPED:
        return scoped_reads(m, S, field, one, arr)
    f = re.escape(field)
    write = r"(?!\s*(?:=(?!=)|\+=|-=|\*=|/=|\+\+|--))"
    pats = []
    for r in one:
        pats.append(rf"(?<![\w$.]){re.escape(r)}\s*\??\.\s*{f}\b{write}")
        pats.append(rf"\bthis\s*\.\s*{re.escape(r)}\s*\??\.\s*{f}\b{write}")
    for a in arr:
        pats.append(rf"(?<![\w$.]){re.escape(a)}\s*\[[^\]\n]*\]\s*\??\.\s*{f}\b{write}")
    lines = set()
    for p in pats:
        for x in re.finditer(p, m):
            lines.add(m.count("\n", 0, x.start()) + 1)
    return sorted(lines)

def main():
    labels = list(csv.DictReader(open(sys.argv[1]), delimiter="\t"))
    cache = {}
    w = open(sys.argv[2], "w")
    w.write("repo\teval\tid\tfinal\tshapes_before\tshapes_after\tkept\tdetail\n")
    for r in labels:
        key = (r["repo"], r["eval"])
        if key not in cache:
            cache[key] = proposals(f"{BASE}/{r['eval']}/{r['repo']}.invar.toml")
        p = cache[key][r["id"]]
        src = open(f"{BASE}/e2c/{r['repo']}/{p['consumer']}", encoding="utf-8", errors="replace").read()
        m = mask(src)
        before, after, detail = [], [], []
        for e in p["ev"]:
            x = re.match(r"E5 (\S+) fields read: (.*)$", e)
            if not x:
                continue
            S = x.group(1); before.append(S)
            fields = [y.group(1) for y in re.finditer(r"(\w+) \(", x.group(2))]
            ok = [fl for fl in fields if nk(fl) not in STOP and verified_reads(m, S, fl)]
            detail.append(f"{S}:{len(ok)}/{len(fields)}")
            if len(ok) >= 2:
                after.append(S)
        w.write("\t".join([r["repo"], r["eval"], r["id"], r["final"], ",".join(before), ",".join(after), "yes" if after else "no", " ".join(detail)]) + "\n")

def scoped_reads(m, S, field, one, arr):
    """A read counts only when the innermost in-scope declaration of its
    receiver is a verified-S declaration (shadowed names do not count)."""
    spans = blocks(m)
    s = re.escape(S)
    f = re.escape(field)
    write = r"(?!\s*(?:=(?!=)|\+=|-=|\*=|/=|\+\+|--))"
    lines = set()
    names = one | arr
    good = {}
    for r in names:
        ver = verified_positions(m, S, r)
        intros = introductions(m, spans, r)
        good[r] = (ver, intros)
    def ok(r, q):
        ver, intros = good[r]
        live = [(p, sc) for p, sc in intros if p < q and sc[0] <= q <= sc[1]]
        if not live:
            return False
        p = max(live)[0]
        return any(abs(p - v) <= len(r) + 2 for v in ver)
    for r in one:
        for x in re.finditer(rf"(?<![\w$.]){re.escape(r)}\s*\??\.\s*{f}\b{write}", m):
            if ok(r, x.start()):
                lines.add(m.count("\n", 0, x.start()) + 1)
        for x in re.finditer(rf"\bthis\s*\.\s*{re.escape(r)}\s*\??\.\s*{f}\b{write}", m):
            lines.add(m.count("\n", 0, x.start()) + 1)
    for a in arr:
        for x in re.finditer(rf"(?<![\w$.]){re.escape(a)}\s*\[[^\]\n]*\]\s*\??\.\s*{f}\b{write}", m):
            if ok(a, x.start()):
                lines.add(m.count("\n", 0, x.start()) + 1)
    return sorted(lines)

def verified_positions(m, S, r):
    """Positions where r is introduced AS S (the binding forms of bindings())."""
    one, arr = set(), set()
    pos = []
    rr = re.escape(r)
    s = re.escape(S)
    head = rf"(?:readonly\s+)?(?:(?:Readonly|Partial|Required|Array|ReadonlyArray)\s*<\s*)?{s}\b(?!\s*[.\[]\w)"
    for x in re.finditer(rf"(?<![\w$.]){rr}\s*\??\s*:\s*{head}", m):
        pos.append(x.start())
    for x in re.finditer(rf"(?<![\w$.]){rr}\s*=[^;=][^;]*?\bas\s+(?:readonly\s+)?{s}\b", m):
        pos.append(x.start())
    if EXT:
        targ = rf"<\s*(?:readonly\s+)?{s}\b[^>()]*>\s*\("
        for x in re.finditer(rf"(?<![\w$.]){rr}\s*=\s*(?:await\s+)?[\w$.]+\s*{targ}", m):
            pos.append(x.start())
        for x in re.finditer(rf"\[\s*{rr}\s*,[^\]]*\]\s*=\s*[\w$.]+\s*{targ}", m):
            pos.append(x.start() + x.group(0).find(r))
        ret = rf"\)\s*:\s*(?:Promise\s*<\s*)?{s}\b(?!\s*\[)"
        fns = set(x.group(1) for x in re.finditer(rf"\bfunction\s+([A-Za-z_$][\w$]*)\s*(?:<[^>]*>)?\s*\([^)]*{ret}", m))
        fns |= set(x.group(1) for x in re.finditer(rf"\b(?:const|let)\s+([A-Za-z_$][\w$]*)\s*=\s*(?:async\s*)?\([^)]*{ret}", m))
        fns |= set(x.group(1) for x in re.finditer(rf"^\s*(?:(?:private|public|protected|static|async)\s+)*([A-Za-z_$][\w$]*)\s*\([^)]*{ret}", m, re.M))
        for fn in fns:
            for x in re.finditer(rf"(?<![\w$.]){rr}\s*=\s*(?:await\s+)?(?:this\.)?{re.escape(fn)}\s*\(", m):
                pos.append(x.start())
    # array element forms: r introduced by for-of / callback over a verified array
    _, arrs = bindings(m, S)
    for a in arrs:
        for x in re.finditer(rf"\bfor\s*\(\s*(?:const|let|var)\s+({rr})\s+of\s+{re.escape(a)}\b", m):
            pos.append(x.start(1))
        for x in re.finditer(rf"(?<![\w$.]){re.escape(a)}\s*\??\.\s*(?:map|filter|forEach|find|some|every|flatMap|findLast|sort|findIndex)\s*\(\s*\(?\s*({rr})\b", m):
            pos.append(x.start(1))
    return pos

main()
