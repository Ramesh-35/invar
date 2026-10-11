#!/usr/bin/env python3
"""Standalone corpus scorer: zero dependencies, zero INVAR engine.

Scores ANY tool's results against the corpus export, so external
parties can verify or challenge the record without trusting this
repository's binaries.

Usage:
  python3 scripts/score-corpus.py --corpus invar-corpus.jsonl --results results.json
  python3 scripts/score-corpus.py ... --held-out-only

results.json maps case id to that tool's finding on the case's pinned
producer/consumer pair:
  { "r1-059": {"verdict": "violated", "residue": ["packages"]},
    "r1-047": {"verdict": "holds",    "residue": []},
    "r1-099": {"verdict": "not_evaluated"},
    "r1-113": {"verdict": "error"} }

Outcome vocabulary (identical to the committed scoring TSVs):
  break cases:  CAUGHT (every expected field in residue), PARTIAL (some),
                MISSED (none, or verdict holds), UNEVALUATED, ERROR
  controls:     CLEAN (holds), FALSE_POSITIVE (violated), UNEVALUATED, ERROR
  not in results: NOT_RUN
A residue entry matches an expected field by exact name or as the final
`.field` of a `Type.field` form. Extra residue on breaks is counted and
reported, never hidden: noise is part of the score.

The summary is printed in count form only (never percentages from small
n). Exit 0 on success, 3 on input errors; the scorer renders outcomes,
never a pass/fail gate.
"""
import argparse
import json
import sys


def field_matches(expected, item):
    return item == expected or item.rsplit(".", 1)[-1] == expected


def score(case, result):
    cls = case["class"]
    if result is None:
        return "NOT_RUN", 0
    verdict = result.get("verdict", "")
    residue = result.get("residue", []) or []
    if verdict == "error":
        return "ERROR", 0
    if verdict == "not_evaluated":
        return "UNEVALUATED", 0
    if cls == "control":
        if verdict == "violated":
            return "FALSE_POSITIVE", len(residue)
        if verdict == "holds":
            return "CLEAN", 0
        return "ERROR", 0
    expected = [f for f in case.get("expected", {}).get("residue", "").split(",") if f and f != "-"]
    hit = [e for e in expected if any(field_matches(e, r) for r in residue)]
    extra = len([r for r in residue if not any(field_matches(e, r) for e in expected)])
    if verdict == "holds" or not residue:
        return "MISSED", 0
    if len(hit) == len(expected) and expected:
        return "CAUGHT", extra
    if hit:
        return "PARTIAL", extra
    return "MISSED", extra


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", required=True)
    ap.add_argument("--results", required=True)
    ap.add_argument("--held-out-only", action="store_true")
    args = ap.parse_args()
    try:
        with open(args.corpus, encoding="utf-8") as f:
            records = [json.loads(line) for line in f if line.strip()]
        with open(args.results, encoding="utf-8") as f:
            results = json.load(f)
    except (OSError, json.JSONDecodeError) as e:
        print(f"ERROR (not a score): {e}", file=sys.stderr)
        return 3
    cases = [r for r in records if r.get("record") == "case"]
    if args.held_out_only:
        cases = [c for c in cases if c["held_out"]]
    known = {c["id"] for c in cases}
    for rid in results:
        if rid not in known:
            print(f"ERROR (not a score): results name unknown case {rid!r}", file=sys.stderr)
            return 3
    counts, noise = {}, 0
    print("case\tclass\theld_out\toutcome\textra_residue")
    for c in cases:
        outcome, extra = score(c, results.get(c["id"]))
        counts[outcome] = counts.get(outcome, 0) + 1
        noise += extra
        print(f"{c['id']}\t{c['class']}\t{str(c['held_out']).lower()}\t{outcome}\t{extra}")
    breaks = sum(1 for c in cases if c["class"] == "break")
    controls = len(cases) - breaks
    print(f"\nsummary (count form): {len(cases)} case(s) = {breaks} break(s) + {controls} control(s)")
    for k in sorted(counts):
        print(f"  {k}: {counts[k]}")
    print(f"  extra residue items across all cases (noise): {noise}")
    caught = counts.get("CAUGHT", 0)
    print(f"  breaks caught: {caught} of {breaks}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
