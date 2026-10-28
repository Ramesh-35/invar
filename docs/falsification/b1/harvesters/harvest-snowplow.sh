#!/usr/bin/env bash
# B1 harvest (DATA-KILLGATE-SPEC r0): sweep iglu-central for major-version
# schema bumps that REMOVED or RETYPED top-level fields, and check which of
# those fields a consumer repository references. Output: a TSV of concrete
# candidate cases for the worksheet, plus the pinned clone paths.
# Usage: bash harvest-snowplow.sh <work-dir> [consumer-repo-url ...]
# Default consumers: the snowplow dbt packages most likely to read events.
set -euo pipefail
work="$1"; shift
consumers=("$@")
if [ "${#consumers[@]}" -eq 0 ]; then
  consumers=(
    https://github.com/snowplow/dbt-snowplow-web
    https://github.com/snowplow/dbt-snowplow-ecommerce
    https://github.com/snowplow/dbt-snowplow-mobile
    https://github.com/snowplow/dbt-snowplow-unified
    https://github.com/snowplow/dbt-snowplow-media-player
    https://github.com/snowplow/dbt-snowplow-normalize
  )
fi
mkdir -p "$work"
if [ ! -d "$work/iglu-central/.git" ]; then
  git clone -q --depth 1 https://github.com/snowplow/iglu-central "$work/iglu-central"
fi
for c in "${consumers[@]}"; do
  name="$(basename "$c")"
  if [ ! -d "$work/$name/.git" ]; then
    git clone -q --depth 1 "$c" "$work/$name" || echo "WARN: could not clone $c" >&2
  fi
done
python3 - "$work" <<'EOF'
import json, os, re, sys
work = sys.argv[1]
root = os.path.join(work, "iglu-central", "schemas")
rows = []
for vendor in sorted(os.listdir(root)):
    vdir = os.path.join(root, vendor)
    if not os.path.isdir(vdir):
        continue
    for name in sorted(os.listdir(vdir)):
        jdir = os.path.join(vdir, name, "jsonschema")
        if not os.path.isdir(jdir):
            continue
        versions = {}
        for fn in os.listdir(jdir):
            m = re.fullmatch(r"(\d+)-(\d+)-(\d+)", fn)
            if not m:
                continue
            try:
                doc = json.load(open(os.path.join(jdir, fn)))
            except Exception:
                continue
            versions[tuple(map(int, m.groups()))] = doc
        majors = sorted({v[0] for v in versions})
        for a, b in zip(majors, majors[1:]):
            # Latest revision of major a vs earliest of major b.
            va = max(v for v in versions if v[0] == a)
            vb = min(v for v in versions if v[0] == b)
            pa = versions[va].get("properties", {}) or {}
            pb = versions[vb].get("properties", {}) or {}
            removed = sorted(set(pa) - set(pb))
            retyped = sorted(
                k for k in set(pa) & set(pb)
                if (pa[k] or {}).get("type") != (pb[k] or {}).get("type")
            )
            if removed or retyped:
                rows.append((vendor, name,
                             "-".join(map(str, va)), "-".join(map(str, vb)),
                             removed, retyped))
# Which consumers mention the changed fields?
consumer_dirs = [d for d in os.listdir(work)
                 if d not in ("iglu-central",) and os.path.isdir(os.path.join(work, d, ".git"))]
def mentions(field):
    # Word-boundary match (snake_case and camelCase forms), with one context
    # snippet per file so a human can verify the read in a glance. Docs and
    # test fixtures are excluded: a README mention is not a consumer.
    snake = re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", field).lower()
    pat = re.compile(r"(?<![A-Za-z0-9_])(" + re.escape(field) + "|" + re.escape(snake) + r")(?![A-Za-z0-9_])")
    hits = []
    for d in consumer_dirs:
        for dirpath, _, files in os.walk(os.path.join(work, d)):
            if "/.git" in dirpath or "integration_tests" in dirpath or "/docs" in dirpath:
                continue
            for f in files:
                if not f.endswith((".sql", ".yml", ".yaml")) or f in ("profiles.yml",):
                    continue
                path = os.path.join(dirpath, f)
                try:
                    text = open(path, errors="ignore").read()
                except Exception:
                    continue
                m = pat.search(text)
                if m:
                    line = text[:m.start()].count("\n") + 1
                    snippet = text.splitlines()[line - 1].strip()[:80]
                    hits.append(f"{d}:{os.path.relpath(path, work)}:{line}: {snippet}")
    return hits
print("vendor\tschema\tfrom\tto\tkind\tfield\tconsumer_hits")
found = 0
for vendor, name, va, vb, removed, retyped in rows:
    for kind, fields in (("removed", removed), ("retyped", retyped)):
        for field in fields:
            hits = mentions(field)
            found += bool(hits)
            print(f"{vendor}\t{name}\t{va}\t{vb}\t{kind}\t{field}\t{' ;; '.join(hits[:4]) or '-'}")
print(f"# major-bump schemas with removals or retypes: {len(rows)}; fields with consumer hits: {found}", file=sys.stderr)
EOF
