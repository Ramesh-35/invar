#!/usr/bin/env python3
"""Derive Software Heritage anchors for every corpus pin (stdlib only).

Reads invar-corpus.jsonl and writes docs/reference/SWHID-ANCHORS.tsv:
one row per pinned reference (case id, role, origin URL, swh:1:rev
identifier, path). swh:1:rev identifiers for git commits are computed
identically to git commit SHAs, so they derive from the pins alone,
with zero network. DERIVED IS NOT ARCHIVED: a row here says what the
anchor is, not that Software Heritage holds it; archival is verified or
requested (Save Code Now) separately, and that status is recorded in
STATUS.md when done, never assumed.
"""
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REF = re.compile(r"^github:([^/@]+)/([^/@]+)@([0-9a-f]{40}):(.+)$")


def main():
    rows = []
    origins = set()
    skipped = []
    with open(os.path.join(ROOT, "invar-corpus.jsonl"), encoding="utf-8") as f:
        for line in f:
            r = json.loads(line)
            if r.get("record") != "case":
                continue
            for role in ("producer", "producer_base", "producer_code",
                         "producer_code_base", "consumer"):
                spec = r.get("invariant", {}).get(role)
                if not spec:
                    continue
                m = REF.match(spec)
                if not m:
                    skipped.append((r["id"], role, spec))
                    continue
                owner, repo, sha, path = m.groups()
                origin = f"https://github.com/{owner}/{repo}"
                origins.add(origin)
                rows.append((r["id"], role, origin, f"swh:1:rev:{sha}", path))
    rows.sort()
    out = os.path.join(ROOT, "docs/reference/SWHID-ANCHORS.tsv")
    with open(out, "w", encoding="utf-8") as f:
        f.write("# Software Heritage anchors derived from corpus pins "
                "(scripts/swhid-anchors.py). Derived, not archived: see "
                "STATUS.md for verified archival state.\n")
        f.write("case\trole\torigin\tswhid_rev\tpath\n")
        for row in rows:
            f.write("\t".join(row) + "\n")
    print(f"{len(rows)} pinned references, {len(origins)} distinct origins, "
          f"{len(skipped)} non-github refs skipped (stated)")
    for s in skipped:
        print("  skipped:", *s)


if __name__ == "__main__":
    sys.exit(main())
