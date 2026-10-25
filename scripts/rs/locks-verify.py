#!/usr/bin/env python3
"""Corrected comparison for round-trip locks that differ only in time-derived
fields (the G3 method, recorded as a script this time). For every
[[contract]] entry of a baseline and a candidate lock:
1. the entry hash (held field 4) recomputes as blake3 of the rendered entry
   (the block from [[contract]] up to, not including, the held lines);
2. substituting the baseline recorded_at into the candidate entry makes its
   rendered text byte-identical to the baseline's, with the same hash;
3. held fields 3 (block hash) and every held_input line are identical.
Held fields 1 (time) and 2 (engine id) are the declared normalization.
Usage: locks-verify.py <baseline lock> <candidate lock>"""
import sys, blake3

def entries(path):
    out, cur = [], None
    for line in open(path).read().split("\n"):
        if line == "[[contract]]":
            cur = {"lines": [], "held": None, "inputs": []}
            out.append(cur)
        elif cur is not None and line.startswith("held = "):
            cur["held"] = line[8:-1].split(" ")
        elif cur is not None and line.startswith("held_input = "):
            cur["inputs"].append(line)
        elif cur is not None and line:
            cur["lines"].append(line)
    return out

def render(lines):
    return "\n[[contract]]\n" + "".join(l + "\n" for l in lines)

def rec(lines):
    return next(l for l in lines if l.startswith("recorded_at = "))

b, c = entries(sys.argv[1]), entries(sys.argv[2])
assert len(b) == len(c), "entry counts differ"
for i, (eb, ec) in enumerate(zip(b, c)):
    for e in (eb, ec):
        assert blake3.blake3(render(e["lines"]).encode()).hexdigest() == e["held"][3], f"entry {i}: hash does not recompute"
    sub = [rec(eb["lines"]) if l.startswith("recorded_at = ") else l for l in ec["lines"]]
    assert render(sub) == render(eb["lines"]), f"entry {i}: differs beyond recorded_at"
    assert blake3.blake3(render(sub).encode()).hexdigest() == eb["held"][3], f"entry {i}: substituted hash differs"
    assert eb["held"][2] == ec["held"][2], f"entry {i}: block hash differs"
    assert eb["inputs"] == ec["inputs"], f"entry {i}: held_input differs"
print(f"IDENTICAL-AFTER-SUBSTITUTION {len(b)} entries")
