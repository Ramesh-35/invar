#!/usr/bin/env python3
"""STAGE3-EVAL-005 adjudication packets (ruling S5-D5). Every surviving
code-pair proposal of the SLICE4 + SLICE5 engine is split by whether its
(producer, consumer) pair was adjudicated in STAGE3-EVAL-003 or -004:
  out-of-sample: never adjudicated; the registered M-S-3 test;
  in-sample: adjudicated before; re-judged on its new evidence, reported.
Each item quotes the producer declaration of every shape and up to four
consumer reads, verbatim from the pinned clone (same format as EVAL-004).
Usage: build_packet.py <clones dir> <eval005 dir> <labeled pairs tsv> <out dir>
labeled pairs tsv: repo, producer, consumer (one line per adjudicated pair)."""
import csv, os, re, sys
clones, ev, labeled, outd = sys.argv[1:5]
seen = {(r["repo"], r["producer"], r["consumer"]) for r in csv.DictReader(open(labeled), delimiter="\t")}

def parse(path):
    props, cur = [], None
    for l in open(path).read().splitlines():
        if l.startswith("# PROPOSED "):
            cur = {"id": l[len("# PROPOSED "):], "ev": []}
        elif cur is not None and l.startswith("# "):
            cur["ev"].append(l[2:])
        elif cur is not None and l.startswith("consumer = "):
            cur["pair"] = re.search(r"^E6 pair (\S+) -> (\S+)$", "\n".join(cur["ev"]), re.M).groups()
            cur["shapes"] = [e.split(" ")[1] for e in cur["ev"] if e.startswith("E2 ")]
            props.append(cur); cur = None
    return props

def emit(out, repo, label, p):
    root = f"{clones}/{repo}"
    def src(rel):
        try:
            return open(f"{root}/{rel}", encoding="utf-8", errors="replace").read().splitlines()
        except Exception:
            return []
    out.append(f"### {label}: {p['id']}")
    ev = [e for e in p["ev"] if e.startswith(("E2 ", "E4 ", "E5 "))]
    out.extend(f"- {e}" for e in ev)
    for e in ev:
        m = re.match(r"E2 \S+ declared at (\S+):(\d+)$", e)
        if m:
            f, n = m.group(1), int(m.group(2)); s = src(f)
            out.extend(["```", f"// {f}:{n}"] + s[n - 1:n + 11] + ["```"])
    reads = re.findall(r"\((\S+):(\d+)\)", " ".join(e for e in ev if e.startswith("E5 ") and "fields read" in e))
    shown = set()
    for f, n in reads:
        if (f, n) in shown or len(shown) == 4:
            continue
        shown.add((f, n)); s = src(f); n = int(n)
        out.extend(["```", f"// {f}:{n}"] + s[max(0, n - 2):n + 1] + ["```"])
    out.extend(["Judgment (TRUE / NOT TRUE, one sentence):", ""])

packets = {"out-of-sample": [], "in-sample": []}
counts = {}
for f in sorted(os.listdir(ev)):
    if not f.endswith(".invar.toml"):
        continue
    repo = f[: -len(".invar.toml")]
    for p in parse(f"{ev}/{f}"):
        if not p["id"].startswith("code:"):
            continue
        k = "in-sample" if (repo,) + p["pair"] in seen else "out-of-sample"
        packets[k].append((repo, p))
for k, items in packets.items():
    out, n = [], {}
    for repo, p in items:
        n[repo] = n.get(repo, 0) + 1
        emit(out, repo, f"{repo} {n[repo]}", p)
    open(f"{outd}/packet-{k}.md", "w").write("\n".join(out) + "\n")
    print(k, len(items), n)
