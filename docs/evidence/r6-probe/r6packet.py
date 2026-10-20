#!/usr/bin/env python3
"""R6 probe adjudication packets: every YIELD candidate, and a seeded sample
of 30 NO-ROUTE lines (random.Random("R6-PROBE-2026-10-20")). Each item
quotes the shape declaration and the consumer's reads of the shape's
fields; YIELD items also quote the receiver binding the probe relied on.
Usage: r6packet.py <clones> <probe.tsv> <out dir>"""
import csv, os, random, re, sys
clones, probe, outd = sys.argv[1:4]
rows = list(csv.DictReader(open(probe), delimiter="\t"))
yld = [r for r in rows if r["class"].startswith("YIELD")]
no = sorted([r for r in rows if r["class"] == "NO-ROUTE"], key=lambda r: (r["repo"], r["file"], r["shape"]))
sample = random.Random("R6-PROBE-2026-10-20").sample(no, 30)
def src(repo, rel):
    try: return open(f"{clones}/{repo}/{rel}", encoding="utf-8", errors="replace").read().splitlines()
    except Exception: return []
def emit(out, label, r, fields):
    out.append(f"### {label}: {r['shape']} ({r['decl']}) read in {r['file']}")
    out.append(f"- fields claimed: {', '.join(fields)}")
    s = src(r["repo"], r["decl"].rsplit(':', 1)[0] if r["decl"].count(":") else r["decl"])
    d = src(r["repo"], r["decl"])
    decl_lines = [i for i, l in enumerate(d) if re.search(rf"\b(interface|type)\s+{re.escape(r['shape'])}\b", l)]
    if decl_lines:
        i = decl_lines[0]; out.extend(["```", f"// {r['decl']}:{i + 1}"] + d[i:i + 12] + ["```"])
    c = src(r["repo"], r["file"]); shown = 0
    for f in fields:
        for i, l in enumerate(c):
            if re.search(rf"\.\s*{re.escape(f)}\b", l):
                out.extend(["```", f"// {r['file']}:{i + 1}"] + c[max(0, i - 2):i + 1] + ["```"]); shown += 1
                break
        if shown >= 4: break
    out.extend(["Judgment (TRUE / NOT TRUE, one sentence):", ""])
for name, items, fld in (("yield", yld, "verified_fields"), ("noroute", sample, None)):
    out = []
    for n, r in enumerate(items, 1):
        fields = r[fld].split(",") if fld else []
        if not fields:
            # NO-ROUTE: the R6 line's own fields
            fields = r.get("fields_r6", "").split(",") if r.get("fields_r6") else []
        emit(out, f"{r['repo']} {n}", r, fields)
    open(f"{outd}/packet-{name}.md", "w").write("\n".join(out) + "\n")
    print(name, len(items))
