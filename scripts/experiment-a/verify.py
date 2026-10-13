"""Verification instrument (not the frozen extractor): depth-aware optionality."""
import re, extract as E

def top_level_calls(v):
    calls, depth, quote, i = [], 0, None, 0
    while i < len(v):
        c = v[i]
        if quote:
            if c == "\\": i += 2; continue
            if c == quote: quote = None
        elif c in "'\"`": quote = c
        elif c in "([{": depth += 1
        elif c in ")]}": depth -= 1
        elif c == "." and depth == 0:
            m = re.match(r"\.(\w+)\(", v[i:])
            if m: calls.append(m.group(1))
        i += 1
    return calls

def zod_shape_v2(body):
    entries = E.top_level_entries(body)
    if entries is None: return None
    fields = {}
    for e in entries:
        k, v = E.split_kv(e)
        if not k: continue
        v = re.sub(r"\s*\.\s*", ".", v)
        if not v.startswith("z."): return None
        t = re.match(r"z\.(\w+)", v)
        calls = top_level_calls(v[v.find("(") if "(" in v else 0:]) if False else top_level_calls(v)
        opt = any(c in ("optional", "nullish", "default") for c in calls)
        fields[k] = {"type": t.group(1) if t else None, "required": not opt}
    return {"fields": fields, "closed": False}

E.from_zod_shape = zod_shape_v2   # swap in for verification only
