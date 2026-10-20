#!/usr/bin/env python3
"""STAGE3-EVAL-004 adjudication packet: every proposal pair new against
STAGE3-EVAL-003 (the +29 SLICE4 delta, no cap), then every shape new
inside an existing pair, listed separately. Each item quotes the producer
declaration of every judged shape and up to four consumer reads, verbatim
from the pinned clone. Differs from build_packet.py of EVAL-002 only in
selecting by delta and quoting every judged declaration, not the last.
Usage: build_packet.py <repo_dir> <eval003 invar.toml> <eval004 invar.toml> <out.md>"""
import re, sys
repo, old_toml, new_toml, outp = sys.argv[1:5]
name = repo.rstrip("/").rsplit("/", 1)[-1]

def parse(path):
    props, cur = [], None
    for l in open(path).read().splitlines():
        if l.startswith("# PROPOSED "):
            cur = {"id": l[len("# PROPOSED "):], "ev": []}
        elif cur is not None and l.startswith("# "):
            cur["ev"].append(l[2:])
        elif cur is not None and l.startswith("consumer = "):
            pair = re.search(r"^E6 pair (\S+) -> (\S+)$", "\n".join(cur["ev"]), re.M).groups()
            cur["pair"] = pair
            cur["shapes"] = [e.split(" ")[1] for e in cur["ev"] if e.startswith("E2 ")]
            props.append(cur); cur = None
    return props

def src(rel):
    try:
        return open(f"{repo}/{rel}", encoding="utf-8", errors="replace").read().splitlines()
    except Exception:
        return []

old = {p["pair"]: p for p in parse(old_toml)}
new = parse(new_toml)
items = [(p, p["shapes"]) for p in new if p["pair"] not in old]
grown = [(p, [s for s in p["shapes"] if s not in old[p["pair"]]["shapes"]]) for p in new if p["pair"] in old]
grown = [(p, s) for p, s in grown if s]

def emit(out, label, p, shapes):
    out.append(f"### {label}: {p['id']}" + ("" if shapes == p["shapes"] else f" (judge only: {', '.join(shapes)})"))
    ev = [e for e in p["ev"] if e.startswith(("E2 ", "E4 ", "E5 ")) and e.split(" ")[1] in shapes]
    out.extend(f"- {e}" for e in ev)
    for e in ev:
        m = re.match(r"E2 \S+ declared at (\S+):(\d+)$", e)
        if m:
            f, n = m.group(1), int(m.group(2)); s = src(f)
            out.extend(["```", f"// {f}:{n}"] + s[n - 1:n + 11] + ["```"])
    reads = re.findall(r"\((\S+):(\d+)\)", " ".join(e for e in ev if e.startswith("E5 ") and "fields read" in e))
    seen = set()
    for f, n in reads:
        if (f, n) in seen or len(seen) == 4:
            continue
        seen.add((f, n)); s = src(f); n = int(n)
        out.extend(["```", f"// {f}:{n}"] + s[max(0, n - 2):n + 1] + ["```"])
    out.extend(["Judgment (TRUE / NOT TRUE, one sentence):", ""])

out = [f"## {name}: {len(items)} new proposal(s), all adjudicated", ""]
for i, (p, s) in enumerate(items, 1):
    emit(out, f"{name} {i}", p, s)
if grown:
    out += [f"## {name}: {len(grown)} existing proposal(s) with new shapes (reported separately)", ""]
    for i, (p, s) in enumerate(grown, 1):
        emit(out, f"{name} G{i}", p, s)
open(outp, "w").write("\n".join(out) + "\n")
print(name, "new", len(items), "grown", len(grown))
