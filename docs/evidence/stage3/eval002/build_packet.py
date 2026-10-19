#!/usr/bin/env python3
"""STAGE3-EVAL-002 adjudication packet: the first 20 proposals per
repository in file order (M-S-3 cap), each with the producer shape
declaration and the consumer's field reads, quoted verbatim from the
pinned clone. Usage: build_packet.py <repo_dir> <invar.toml> <out.md>"""
import re, sys
repo, toml, outp = sys.argv[1:4]
name = repo.rstrip("/").rsplit("/", 1)[-1]
lines = open(toml).read().splitlines()
props, cur = [], None
for l in lines:
    if l.startswith("# PROPOSED "):
        cur = {"id": l[len("# PROPOSED "):], "ev": []}
    elif cur is not None and l.startswith("# "):
        cur["ev"].append(l[2:])
    elif cur is not None and l.startswith("consumer = "):
        props.append(cur); cur = None
def src(rel):
    try:
        return open(f"{repo}/{rel}", encoding="utf-8", errors="replace").read().splitlines()
    except Exception:
        return []
out = [f"## {name}: {len(props)} proposal(s), first {min(20, len(props))} adjudicated", ""]
for i, p in enumerate(props[:20], 1):
    out.append(f"### {name} {i}: {p['id']}")
    ev = {e.split(' ', 1)[0]: e for e in p["ev"]}
    for e in p["ev"]:
        if e.startswith(("E2 ", "E4 ", "E5 ")):
            out.append(f"- {e}")
    m = re.search(r"declared at (\S+):(\d+)", ev.get("E2", ""))
    if m:
        f, n = m.group(1), int(m.group(2)); s = src(f)
        out += ["```", f"// {f}:{n}"] + s[n - 1:n + 11] + ["```"]
    reads = re.findall(r"\((\S+):(\d+)\)", " ".join(e for e in p["ev"] if e.startswith("E5 ") and "fields read" in e))
    seen = set()
    for f, n in reads[:4]:
        if (f, n) in seen: continue
        seen.add((f, n)); s = src(f); n = int(n)
        out += ["```", f"// {f}:{n}"] + s[max(0, n - 2):n + 1] + ["```"]
    out += ["Judgment (TRUE / NOT TRUE, one sentence):", ""]
open(outp, "w").write("\n".join(out) + "\n")
print(name, "proposals", len(props), "in packet", min(20, len(props)))
