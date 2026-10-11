#!/usr/bin/env python3
"""Export the real-break corpus as deterministic JSONL (invar-corpus.jsonl).

Zero dependencies (stdlib only, Python 3.8+). Reads every
corpus-real/<case>/case.toml and invar.toml with a minimal flat-TOML
parser (the same subset the engine itself parses: [section] headers,
key = "value" lines, # comments) and writes one JSON object per line:
first a meta record, then one record per case, sorted by id, keys
sorted. Running it twice produces identical bytes.

Data license: CDLA-Permissive-2.0 (LICENSE-DATA). The export contains
INVAR's own case metadata and pinned references only, never third-party
file contents.
"""
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CORPUS = os.path.join(ROOT, "corpus-real")
OUT = os.path.join(ROOT, "invar-corpus.jsonl")
HELD_OUT_BATCHES = {"4", "5", "6", "7", "8"}
SEAL = "docs/falsification/HELD-OUT-MANIFEST.txt"


def parse_flat_toml(path):
    """[section] -> {key: value}; [[section]] repeats collapse to the
    section name; later duplicate keys in one section are rejected so a
    malformed case file fails loudly, never silently."""
    sections = {}
    current = None
    with open(path, encoding="utf-8") as f:
        for n, raw in enumerate(f, 1):
            line = raw.split("#", 1)[0].strip() if not raw.lstrip().startswith("#") else ""
            # a '#' inside quotes would be mangled above; the corpus has
            # none, and the assertion below catches a parse that drops a
            # required key.
            if not line:
                continue
            m = re.match(r"^\[+([A-Za-z0-9_.-]+)\]+$", line)
            if m:
                current = m.group(1)
                sections.setdefault(current, {})
                continue
            if "=" not in line or current is None:
                raise SystemExit(f"{path}:{n}: unparseable line {raw!r}")
            k, v = line.split("=", 1)
            k, v = k.strip(), v.strip()
            if v.startswith('"') and v.endswith('"'):
                v = v[1:-1]
            if k in sections[current]:
                raise SystemExit(f"{path}:{n}: duplicate key {k!r}")
            sections[current][k] = v
    return sections


def main():
    cases = []
    for d in sorted(os.listdir(CORPUS)):
        case_path = os.path.join(CORPUS, d, "case.toml")
        inv_path = os.path.join(CORPUS, d, "invar.toml")
        if d == "_template":
            continue  # the scaffold, not a case (the historical 123 vs 122 miscount)
        if not os.path.isfile(case_path):
            continue  # strays carry no case.toml
        ct = parse_flat_toml(case_path)
        iv = parse_flat_toml(inv_path) if os.path.isfile(inv_path) else {}
        case = dict(ct.get("case", {}))
        if "id" not in case:
            raise SystemExit(f"{case_path}: no [case] id")
        batch = case.get("batch", "")
        rec = {
            "record": "case",
            "dir": d,
            "class": "control" if "-control-" in d else "break",
            "held_out": batch in HELD_OUT_BATCHES,
            "expected": dict(ct.get("expected", {})),
            "baseline": dict(ct.get("baseline", {})),
            "invariant": dict(iv.get("invariant", {})),
        }
        rec.update(case)
        cases.append(rec)
    cases.sort(key=lambda r: r["id"])
    held = [c for c in cases if c["held_out"]]
    meta = {
        "record": "meta",
        "corpus": "INVAR real cross-repository break corpus",
        "data_license": "CDLA-Permissive-2.0 (see LICENSE-DATA)",
        "cases": len(cases),
        "held_out_cases": len(held),
        "held_out_breaks": sum(1 for c in held if c["class"] == "break"),
        "held_out_controls": sum(1 for c in held if c["class"] == "control"),
        "held_out_seal": SEAL,
        "note": "metadata and pinned references only; no third-party file contents",
    }
    with open(OUT, "w", encoding="utf-8") as f:
        f.write(json.dumps(meta, sort_keys=True) + "\n")
        for c in cases:
            f.write(json.dumps(c, sort_keys=True) + "\n")
    print(f"wrote {OUT}: 1 meta + {len(cases)} case records "
          f"({meta['held_out_breaks']} held-out breaks, "
          f"{meta['held_out_controls']} held-out controls)")


if __name__ == "__main__":
    sys.exit(main())
