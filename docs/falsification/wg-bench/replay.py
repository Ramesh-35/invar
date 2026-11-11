#!/usr/bin/env python3
"""WG gate: 76 SchemaStore version pairs where a complete witness-generation
algorithm (Attouche et al., "Witness Generation for Classical JSON Schema",
ACM TODS 2026; dataset shipped with Baazizi et al., "JSON Schema Inclusion
through Refutational Normalization", 2026, src/test/resources/inclusion/
schemastore_containment/sat) found a document valid at the OLDER version and
rejected by the NEWER one. Every pair is a real break, independently of
INVAR. The gate: the judge says HOLDS on none of them. VIOLATED or NOT
EVALUATED are both honest here; HOLDS is a false HOLDS.

    python3 docs/falsification/wg-bench/replay.py <invar binary>
"""
import json, os, subprocess, sys, collections
here = os.path.dirname(os.path.abspath(__file__))
invar = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else "invar"
rows = [l.rstrip("\n").split("\t") for l in open(os.path.join(here, "MANIFEST.tsv"))][1:]
C = collections.Counter(); holds = []
for base, head, src in rows:
    p = subprocess.run([invar, "judge", os.path.join(here, "pairs", base), os.path.join(here, "pairs", head), "--format", "json"], capture_output=True, text=True)
    try:
        v = json.loads(p.stdout)["verdict"]
    except Exception:
        v = "ERROR"
    C[v] += 1
    if v == "HOLDS":
        holds.append(src)
print(f"WG pairs {len(rows)}: " + ", ".join(f"{k} {C[k]}" for k in ("VIOLATED", "NOT EVALUATED", "HOLDS", "ERROR") if C[k]))
for h in holds:
    print(f"  FALSE HOLDS: {h}")
print("WG GATE: " + ("PASS" if not holds and not C["ERROR"] else "FAIL"))
sys.exit(0 if not holds and not C["ERROR"] else 1)
