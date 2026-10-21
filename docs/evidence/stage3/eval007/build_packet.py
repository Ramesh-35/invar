#!/usr/bin/env python3
"""STAGE3-EVAL-007 adjudication packet (ruling S7-D5). One item per shape
the SLICE7 engine proposes that EVAL-006 did not: every shape whose E4
line says it was resolved via an imported function's return type. The
script also checks that this set equals the shapes added between the two
evaluations (by (repo, id, shape)), and stops if it does not.
Each item quotes the shape's declaration, the function's declaration and
up to four consumer reads, verbatim from the pinned clone.
Usage: build_packet.py <clones dir> <eval006 dir> <eval007 dir> <out dir>"""
import os, re, sys

clones, ev6, ev7, outd = sys.argv[1:5]


def parse(path):
    props, cur = [], None
    for l in open(path).read().splitlines():
        if l.startswith("# PROPOSED "):
            cur = {"id": l[len("# PROPOSED "):], "ev": []}
        elif cur is not None and l.startswith("# "):
            cur["ev"].append(l[2:])
        elif cur is not None and l.startswith("consumer = "):
            cur["shapes"] = [e.split(" ")[1] for e in cur["ev"] if e.startswith("E2 ")]
            props.append(cur)
            cur = None
    return props


def shapes_of(path):
    return {(p["id"], s) for p in parse(path) for s in p["shapes"]} if os.path.exists(path) else set()


items, added = [], set()
for f in sorted(os.listdir(ev7)):
    if not f.endswith(".invar.toml"):
        continue
    repo = f[: -len(".invar.toml")]
    added |= {(repo,) + k for k in shapes_of(f"{ev7}/{f}") - shapes_of(f"{ev6}/{f}")}
    for p in parse(f"{ev7}/{f}"):
        for s in p["shapes"]:
            e4 = [e for e in p["ev"] if e.startswith(f"E4 {s} ")]
            if any("via return type of" in e for e in e4):
                items.append((repo, p, s))
routed = {(r, p["id"], s) for r, p, s in items}
if routed != added:
    sys.exit(f"STOP: routed shapes != added shapes\nonly routed: {sorted(routed - added)}\nonly added: {sorted(added - routed)}")


def src(repo, rel):
    try:
        return open(f"{clones}/{repo}/{rel}", encoding="utf-8", errors="replace").read().splitlines()
    except Exception:
        return []


out = ["# STAGE3-EVAL-007 adjudication packet", "",
       f"{len(items)} items. Judge only the shape named in each heading.", ""]
for n, (repo, p, s) in enumerate(items, 1):
    out.append(f"### {repo} {n}: {p['id']} (judge only: {s})")
    ev = [e for e in p["ev"] if e.startswith((f"E2 {s} ", f"E4 {s} ", f"E5 {s} "))]
    out.extend(f"- {e}" for e in ev)
    sites = []
    for e in ev:
        m = re.match(r"E2 \S+ declared at (\S+):(\d+)$", e)
        if m:
            sites.append((m.group(1), int(m.group(2)), 12))
        for m in re.finditer(r"declared in (\S+):(\d+)", e):
            sites.append((m.group(1), int(m.group(2)), 3))
    for f, ln, k in sites:
        out.extend(["```", f"// {f}:{ln}"] + src(repo, f)[ln - 1:ln - 1 + k] + ["```"])
    shown = []
    for e in ev:
        if e.startswith(f"E5 {s} fields read"):
            for f, ln in re.findall(r"\((\S+):(\d+)\)", e):
                if (f, ln) not in shown and len(shown) < 4:
                    shown.append((f, ln))
                    ln = int(ln)
                    out.extend(["```", f"// {f}:{ln}"] + src(repo, f)[max(0, ln - 3):ln + 1] + ["```"])
    out.extend(["Judgment (TRUE / NOT TRUE, one sentence):", ""])
os.makedirs(outd, exist_ok=True)
open(f"{outd}/packet.md", "w").write("\n".join(out) + "\n")
with open(f"{outd}/items.tsv", "w") as w:
    w.write("repo\tn\tid\tshape\tfunctions\n")
    for n, (repo, p, s) in enumerate(items, 1):
        fns = sorted(set(re.findall(r"return type of (\w+)\(\)", " ".join(e for e in p["ev"] if e.startswith(f"E4 {s} ")))))
        w.write(f"{repo}\t{n}\t{p['id']}\t{s}\t{','.join(fns)}\n")
print(len(items), "items")
