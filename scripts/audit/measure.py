#!/usr/bin/env python3
# AUDIT-SPEC r0 measurement (section C): AU-1 and AU-3 applied to every
# witness line of every VIOLATED case in a corpus capture, against the
# consumer texts. Usage:
#   python3 -I scripts/audit/measure.py <corpus capture dir> <consumer texts dir>
import glob, os, re, sys
cap, ct = sys.argv[1], sys.argv[2]
texts = {os.path.basename(f).split(".")[0]: open(f, errors="replace").read().split("\n") for f in glob.glob(ct + "/*.consumer")}
def tokens(line):
    t = set(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", line))
    for g in re.findall(r"\"([^\"\\\\]*)\"|'([^'\\\\]*)'|`([^`\\\\]*)`", line):
        t |= {x for x in g if x}
    return t
ok = bad = cases = 0
for f in sorted(glob.glob(cap + "/*.out")):
    c = os.path.basename(f)[:-4]; L = open(f).read().split("\n")
    if not any("[VIOLATED]" in l for l in L): continue
    cases += 1
    for l in L:
        if not l.startswith("      ") or l.strip().startswith(("fetched", "measured")) or "[not-evaluated" in l: continue
        name = l.strip().split(" ")[0]
        if "no inbound read and no literal" in l:
            w = re.search(r"written at [^:]+:([\d, ]+)$", l)
            for n in [int(x) for x in w.group(1).split(",")]:
                if name in tokens(texts[c][n - 1]): ok += 1
                else: bad += 1; print("FAIL", c, name, n)
            continue
        m = re.search(r" read at (.*?)( · |$)", l)
        ra = re.search(r"naming differs, read as (.*?)( · |$)", l)
        forms = {name} | ({x.strip() for x in ra.group(1).split(",")} if ra else set())
        for s in [x.strip() for x in m.group(1).split(",")]:
            if s.startswith("+"): continue
            ln = int(s.rsplit(":", 1)[1]); line = texts[c][ln - 1] if 0 < ln <= len(texts[c]) else None
            if line is not None and forms & tokens(line): ok += 1
            else: bad += 1; print("FAIL", c, name, s)
print(f"violated cases {cases}; sites holding the field or a declared read form as a token: {ok}; failures: {bad}")
