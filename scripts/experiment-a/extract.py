"""Static output-schema extractor for Experiment A (heuristic, disclosed).

Given source text, return {tool_name: schema} where schema is
{"fields": {name: {"type": str|None, "required": bool}}, "closed": bool}
or the string "UNRESOLVED:<why>" when the schema is not statically
extractable. Top-level fields only. Pattern matching, not parsing.
"""
import json
import re

TOOLNAME_PATTERNS = [
    re.compile(r"registerTool\(\s*['\"`]([\w\-.:/]+)['\"`]"),
    re.compile(r"\.tool\(\s*['\"`]([\w\-.:/]+)['\"`]"),
    re.compile(r"\bname\s*[:=]\s*['\"`]([\w\-.:/]+)['\"`]"),
    re.compile(r"['\"]name['\"]\s*:\s*['\"]([\w\-.:/]+)['\"]"),
]


def capture(text, i):
    """Capture a balanced expression starting at index i (string-aware)."""
    depth, j, n = 0, i, len(text)
    quote = None
    while j < n:
        c = text[j]
        if quote:
            if c == "\\":
                j += 2
                continue
            if c == quote:
                quote = None
        elif c in "'\"`":
            quote = c
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            if depth == 0:
                return text[i:j]
            depth -= 1
            if depth == 0:
                k = j + 1
                # zod chains: .strict(), .passthrough(), .describe(...)
                while k < n and text[k] == ".":
                    m = re.match(r"\.\w+\(", text[k:])
                    if not m:
                        break
                    e = capture(text, k + m.end() - 1)
                    k = k + m.end() - 1 + len(e) + 1 if e is not None else n
                    if k > n:
                        k = n
                return text[i:k]
        elif c in ",;\n" and depth == 0:
            return text[i:j]
        j += 1
    return text[i:j]


def top_level_entries(body):
    """Split an object body '{...}' into top-level 'key: value' strings."""
    body = body.strip()
    if not (body.startswith("{") and body.endswith("}")):
        return None
    inner = body[1:-1]
    out, depth, cur, quote, j = [], 0, [], None, 0
    while j < len(inner):
        c = inner[j]
        if quote:
            cur.append(c)
            if c == "\\" and j + 1 < len(inner):
                cur.append(inner[j + 1])
                j += 2
                continue
            if c == quote:
                quote = None
        elif c in "'\"`":
            quote = c
            cur.append(c)
        elif c in "([{":
            depth += 1
            cur.append(c)
        elif c in ")]}":
            depth -= 1
            cur.append(c)
        elif c == "," and depth == 0:
            out.append("".join(cur))
            cur = []
        else:
            cur.append(c)
        j += 1
    if "".join(cur).strip():
        out.append("".join(cur))
    return [e.strip() for e in out if e.strip() and not e.strip().startswith("//")]


def strip_comments(t):
    t = re.sub(r"/\*.*?\*/", "", t, flags=re.S)
    return "\n".join(l for l in t.split("\n") if not l.strip().startswith(("//", "#")))


def split_kv(entry):
    entry = strip_comments(entry).strip()
    m = re.match(r"^['\"]?([\w\-$]+)['\"]?\??\s*:\s*(.*)$", entry, re.S)
    return (m.group(1), m.group(2).strip()) if m else (None, None)


def from_zod_shape(body):
    entries = top_level_entries(body)
    if entries is None:
        return None
    fields = {}
    for e in entries:
        k, v = split_kv(e)
        if v is not None:
            v = re.sub(r"\s*\.\s*", ".", v)
        if not k or not v.startswith("z."):
            if k and not v.startswith("z."):
                return None  # not a zod raw shape
            continue
        t = re.match(r"z\.(\w+)", v)
        opt = bool(re.search(r"\.(optional|nullish|default)\(", v))
        fields[k] = {"type": t.group(1) if t else None, "required": not opt}
    return {"fields": fields, "closed": False}


def from_json_schema_obj(body):
    """JSON-Schema-like literal (JS object or JSON)."""
    entries = top_level_entries(body)
    if entries is None:
        return None
    kv = dict(split_kv(e) for e in entries if split_kv(e)[0])
    if "properties" not in kv:
        if re.search(r"['\"]?type['\"]?\s*:\s*['\"]object['\"]", body):
            return "VACUOUS"
        return None
    props = top_level_entries(kv["properties"])
    if props is None:
        return None
    req = set(re.findall(r"['\"]([\w\-$]+)['\"]", kv.get("required", "")))
    fields = {}
    for p in props:
        k, v = split_kv(p)
        if not k:
            continue
        t = re.search(r"['\"]?type['\"]?\s*:\s*['\"](\w+)['\"]", v)
        fields[k] = {"type": t.group(1) if t else None, "required": k in req}
    closed = "false" in kv.get("additionalProperties", "")
    return {"fields": fields, "closed": closed}


def resolve_expr(expr, text, depth=0):
    expr = expr.strip().rstrip(",")
    if depth > 3:
        return "UNRESOLVED:depth"
    m = re.match(r"^z\.object\(", expr)
    if m:
        start = m.end() - 1
        depth, j, quote = 0, start, None
        while j < len(expr):
            c = expr[j]
            if quote:
                if c == "\\":
                    j += 2
                    continue
                if c == quote:
                    quote = None
            elif c in "'\"`":
                quote = c
            elif c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        inner_body = expr[start + 1:j]
        s = from_zod_shape(inner_body.strip()) if inner_body else None
        if s is None:
            return "UNRESOLVED:zod-object"
        if ".strict()" in expr:
            s["closed"] = True
        return s
    if expr.startswith("{"):
        s = from_json_schema_obj(expr)
        if s == "VACUOUS":
            return "UNRESOLVED:vacuous-no-properties"
        if s is not None:
            return s
        s = from_zod_shape(expr)
        if s is not None:
            return s
        return "UNRESOLVED:object-shape"
    m = re.match(r"^([A-Za-z_$][\w$]*)(\.shape)?$", expr)
    if m:
        ident = m.group(1)
        d = re.search(r"(?:const|let|var|export const)\s+" + re.escape(ident) + r"\s*(?::[^=]+)?=\s*", text)
        if not d:
            return "UNRESOLVED:identifier-not-in-file"
        return resolve_expr(capture(text, d.end()), text, depth + 1)
    return "UNRESOLVED:other-expression"


def tool_name_before(text, pos):
    window = text[max(0, pos - 3000):pos]
    best, best_at = None, -1
    for pat in TOOLNAME_PATTERNS:
        for m in pat.finditer(window):
            if m.start() > best_at:
                best, best_at = m.group(1), m.start()
    return best


def extract(path, text):
    out = {}
    if path.endswith(".json"):
        try:
            data = json.loads(text)
        except Exception:
            return {}
        def walk(o):
            if isinstance(o, dict):
                if "outputSchema" in o and isinstance(o.get("name"), str) and isinstance(o["outputSchema"], dict):
                    sch = o["outputSchema"]
                    props = sch.get("properties")
                    if not isinstance(props, dict) or not props:
                        out[o["name"]] = "UNRESOLVED:vacuous-no-properties"
                    else:
                        req = set(sch.get("required") or [])
                        out[o["name"]] = {
                            "fields": {k: {"type": (v.get("type") if isinstance(v, dict) and isinstance(v.get("type"), str) else None), "required": k in req} for k, v in props.items()},
                            "closed": sch.get("additionalProperties") is False,
                        }
                for v in o.values():
                    walk(v)
            elif isinstance(o, list):
                for v in o:
                    walk(v)
        walk(data)
        return out
    for m in re.finditer(r"\boutputSchema\s*:\s*|['\"]outputSchema['\"]\s*:\s*|\boutput_schema\s*=\s*", text):
        expr = capture(text, m.end())
        if expr is None:
            continue
        name = tool_name_before(text, m.start()) or f"@{path}#{len(out)}"
        if name in out:
            name = f"{name}#{m.start()}"
        if expr.strip().startswith(("undefined", "null", "None")):
            continue
        out[name] = resolve_expr(expr, text)
    return out


def classify(prev, cur):
    """Breaking changes between two extracted schemas for one tool."""
    breaks = []
    if isinstance(prev, str) or isinstance(cur, str):
        return breaks
    pf, cf = prev["fields"], cur["fields"]
    for k, v in pf.items():
        if v["required"] and (k not in cf or not cf[k]["required"]):
            breaks.append(("required-removed-or-optional", k))
        if k in cf and v["type"] and cf[k]["type"] and v["type"] != cf[k]["type"]:
            breaks.append(("type-changed", k, v["type"], cf[k]["type"]))
    if prev["closed"]:
        for k in cf:
            if k not in pf:
                breaks.append(("closed-object-addition", k))
    return breaks
