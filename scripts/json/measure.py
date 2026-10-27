#!/usr/bin/env python3
# JSON-SPEC r0 measurement (section C): the structured fields JSON-1 derives,
# applied to every item line of a full-corpus text capture, with each case's
# bound producer names read from its invar.toml. Usage:
#   python3 -I measure.py <corpus capture dir> <corpus-real dir>
import glob, os, re, sys, collections
cap, real = sys.argv[1], sys.argv[2]
C = collections.Counter()
def names(toml):
    out = []
    for k in ("producer", "producer_base", "producer_code", "producer_code_base"):
        for v in re.findall(rf'^{k}\s*=\s*"([^"]*)"', toml, re.M):
            out.append(v.rsplit(":", 1)[-1] if v.startswith("github:") else v)
    return out
fails = []
for f in sorted(glob.glob(cap + "/*.out")):
    case = os.path.basename(f)[:-4]
    P = names(open(f"{real}/{case}/invar.toml").read())
    status = None
    for l in open(f).read().split("\n"):
        m = re.match(r"  \[([A-Z ]+)\] ", l)
        if m:
            status = m.group(1); C["contract:" + status] += 1
            if status == "SKIPPED":
                why = l.split("): ", 1)[1]
                t = re.search(r"\[not-evaluated:([a-z0-9-]+)\]$", why)
                C["skip:" + (t.group(1) if t else "UNTAGGED")] += 1
                if not t: fails.append(("skip-untagged", case, why[:90]))
            continue
        if not l.startswith("      ") or l.strip().startswith(("fetched ", "measured")):
            continue
        s = l.strip()
        t = re.search(r"\[not-evaluated:([a-z0-9-]+)\]$", s)
        if status == "NOT EVALUATED" or (t and status != "VIOLATED"):
            C["abst:" + (t.group(1) if t else "UNTAGGED")] += 1
            if not t: fails.append(("abst-untagged", case, s[:90]))
            continue
        if status != "VIOLATED":
            C["other-line:" + status] += 1; continue
        if t:
            C["abst-under-violated:" + t.group(1)] += 1; continue
        C["witness"] += 1
        rs = re.search(r" · not in (\S+) at head: no inbound read and no literal of .* written at ([^:]+):([\d, ]+)$", s)
        tc = " · type " in s
        nm = " · not in " in s
        rule = "request-side" if rs else ("absent-and-retyped" if (tc and nm) else "retyped" if tc else "absent" if nm else "OTHER")
        C["rule:" + rule] += 1
        sites = re.search(r" read at (.*?)( · |$)", s)
        if sites:
            for x in sites.group(1).split(", "):
                if re.match(r"\+\d+ more$", x): C["sites:elided"] += 1; continue
                ok = re.match(r".+:\d+$", x) is not None
                C["site:" + ("ok" if ok else "BAD")] += 1
        if rs:
            C["rs-writes"] += len(rs.group(3).split(","))
        place = None
        mm = re.search(r" · not in (.*?)( · |$)", s)
        if mm: place = mm.group(1)
        elif tc: place = re.search(r" in (.*)$", s.split(" · type ", 1)[1]).group(1)
        ok = place is not None and any(place == p or place.startswith(p + " ") or place.startswith(p + ":") or place.startswith(p + ",") for p in P)
        C["producer:" + ("named" if ok else "UNNAMED")] += 1
        if not ok: fails.append(("producer", case, s[:120]))
        if " · candidate rename: " in s: C["candidate"] += 1
for k in sorted(C): print(f"{C[k]:5d}  {k}")
for x in fails: print("CHECK", *x)
