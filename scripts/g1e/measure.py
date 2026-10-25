#!/usr/bin/env python3
# G1-E-SPEC r0 measurement counter: the frozen grammar of sections B and C
# over the corpus texts (consumer reads approximated by whole-word mention,
# a superset of the engine's recognized reads). Usage:
#   python3 -I scripts/g1e/measure.py <producer texts dir> <consumer texts dir>
import os, re, sys, glob
FORMS = [".json(", ".send(", "jsonify(", "JsonResponse(", "json.dumps(", "Response(", "send_result("]
KW = {"null", "None", "true", "false", "True", "False", "undefined"}
IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
def comment(l):
    t = l.lstrip(); return t.startswith(("//", "#", "*", "/*"))
def scan_args(text, i):
    """text[i] is just after '('. Return list of top-level argument strings, and stop index."""
    depth, args, cur, j = 0, [], [], i
    while j < len(text):
        c = text[j]
        if c in "\"'`":
            k = j + 1
            while k < len(text) and text[k] != c:
                k += 2 if text[k] == "\\" else 1
            cur.append(text[j:k + 1]); j = k + 1; continue
        if c in "([{": depth += 1
        elif c in ")]}":
            if depth == 0:
                args.append("".join(cur)); return args
            depth -= 1
        elif c == "," and depth == 0:
            args.append("".join(cur)); cur = []; j += 1; continue
        cur.append(c); j += 1
    return None
def top_keys(obj):
    s = obj.strip()
    assert s.startswith("{")
    body, depth, keys, j = s[1:], 0, [], 0
    tok_start = True
    while j < len(body):
        c = body[j]
        if c in "\"'`":
            k = j + 1
            while k < len(body) and body[k] != c:
                k += 2 if body[k] == "\\" else 1
            if depth == 0 and tok_start and body[k + 1:].lstrip().startswith(":"):
                keys.append(body[j + 1:k])
            j = k + 1; tok_start = False; continue
        if c in "([{": depth += 1
        elif c in ")]}": depth -= 1
        elif c == "," and depth == 0: tok_start = True; j += 1; continue
        elif depth == 0 and tok_start and (c.isalpha() or c == "_"):
            m = IDENT.match(body, j)
            if body[m.end():].lstrip().startswith(":") and not body[m.end():].lstrip().startswith("::"):
                keys.append(m.group())
            j = m.end(); tok_start = False; continue
        if not c.isspace(): tok_start = False
        j += 1
    return keys
def emissions(text):
    out = []
    lines = text.split("\n"); off = 0
    starts = []
    for n, l in enumerate(lines):
        starts.append(off); off += len(l) + 1
    for n, l in enumerate(lines):
        if comment(l): continue
        for f in FORMS:
            p = 0
            while True:
                q = l.find(f, p)
                if q < 0: break
                p = q + len(f)
                if f[0] != "." and q > 0 and (l[q-1].isalnum() or l[q-1] == "_"): continue
                args = scan_args(text, starts[n] + q + len(f))
                if not args: continue
                idx = 1 if f == "send_result(" else 0
                if len(args) <= idx: continue
                a = args[idx].strip()
                if a.startswith("{"):
                    out.append((f, "L", n + 1, top_keys(a)))
                elif IDENT.fullmatch(a) and a not in KW:
                    out.append((f, "O", n + 1, a))
    return out
def words(text):
    return set(IDENT.findall("\n".join(l for l in text.split("\n") if not comment(l))))
pt, ct = sys.argv[1], sys.argv[2]
cases = sorted({os.path.basename(f).split(".")[0] for f in glob.glob(ct + "/*.consumer")})
for c in cases:
    cons = open(f"{ct}/{c}.consumer", errors="replace").read()
    w = words(cons)
    for b, h in (("producer_base", "producer"), ("producer_code_base", "producer_code")):
        bp, hp = f"{pt}/{c}.{b}", f"{pt}/{c}.{h}"
        if not (os.path.exists(bp) and os.path.exists(hp)): continue
        be, he = emissions(open(bp, errors="replace").read()), emissions(open(hp, errors="replace").read())
        for f in FORMS:
            lb = [e for e in be if e[0] == f and e[1] == "L"]; ob = [e for e in be if e[0] == f and e[1] == "O"]
            lh = [e for e in he if e[0] == f and e[1] == "L"]; oh = [e for e in he if e[0] == f and e[1] == "O"]
            if len(lh) < len(lb) and len(oh) > len(ob):
                keys = sorted({k for e in lb for k in e[3] if k in w})
                print(f"{c}\t{h}\t{f}\tL {len(lb)}->{len(lh)}\tO {len(ob)}->{len(oh)}\tread-keys={','.join(keys) or '-'}\thead-opaque={[(e[2], e[3]) for e in oh]}")
