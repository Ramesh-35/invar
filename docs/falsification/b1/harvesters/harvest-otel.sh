#!/usr/bin/env bash
# B1 harvest, OpenTelemetry family: parse the semantic-conventions schema
# files (machine-readable rename ledgers, one per released version), then
# check which OLD attribute names independent consumer repositories still
# read, word-boundary with dots respected, with file:line snippets.
# Usage: bash harvest-otel.sh <work-dir> [consumer-repo-url ...]
set -euo pipefail
work="$1"; shift
consumers=("$@")
if [ "${#consumers[@]}" -eq 0 ]; then
  consumers=(
    https://github.com/jaegertracing/jaeger
    https://github.com/SigNoz/signoz
  )
fi
mkdir -p "$work"
if [ ! -d "$work/semantic-conventions/.git" ]; then
  git clone -q --depth 1 https://github.com/open-telemetry/semantic-conventions "$work/semantic-conventions"
fi
for c in "${consumers[@]}"; do
  name="$(basename "$c")"
  if [ ! -d "$work/$name/.git" ]; then
    git clone -q --depth 1 "$c" "$work/$name" || echo "WARN: could not clone $c" >&2
  fi
done
python3 - "$work" <<'EOF'
import os, re, sys
work = sys.argv[1]
sdir = os.path.join(work, "semantic-conventions", "schemas")

def parse_renames(path):
    # attribute_map blocks: "  old.name: new.name" until dedent.
    pairs, in_map, map_indent = [], False, 0
    for raw in open(path, errors="ignore"):
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip())
        line = raw.strip()
        if line == "attribute_map:":
            in_map, map_indent = True, indent
            continue
        if in_map:
            if indent <= map_indent:
                in_map = False
            else:
                m = re.match(r"([A-Za-z0-9_.]+):\s*([A-Za-z0-9_.]+)$", line)
                if m:
                    pairs.append((m.group(1), m.group(2)))
    return pairs

renames = {}
for fn in sorted(os.listdir(sdir)):
    if not re.fullmatch(r"\d+\.\d+(\.\d+)?", fn):
        continue
    for old, new in parse_renames(os.path.join(sdir, fn)):
        renames.setdefault(old, (fn, new))
print(f"# rename ledger: {len(renames)} old attribute names across schema files", file=sys.stderr)
olds = sorted(renames)
pat = re.compile(r"(?<![A-Za-z0-9_.])(" + "|".join(re.escape(o) for o in olds) + r")(?![A-Za-z0-9_.])")
exts = (".go", ".ts", ".tsx", ".jsx", ".py", ".json", ".yaml", ".yml", ".sql")
skip_dirs = ("/.git", "node_modules", "/vendor/", "/dist/", "/docs/", "/examples/")
hits = {}
consumer_dirs = [d for d in sorted(os.listdir(work))
                 if d != "semantic-conventions" and os.path.isdir(os.path.join(work, d, ".git"))]
for d in consumer_dirs:
    for dirpath, _, files in os.walk(os.path.join(work, d)):
        if any(s in dirpath + "/" for s in skip_dirs):
            continue
        for fn in files:
            if not fn.endswith(exts) or fn.endswith("_test.go"):
                continue
            path = os.path.join(dirpath, fn)
            try:
                text = open(path, errors="ignore").read()
            except Exception:
                continue
            for m in pat.finditer(text):
                if len(hits.setdefault(m.group(1), [])) < 3:
                    line = text[:m.start()].count("\n") + 1
                    snippet = text.splitlines()[line - 1].strip()[:70]
                    hits[m.group(1)].append(f"{os.path.relpath(path, work)}:{line}: {snippet}")
print("old_attribute\tnew_attribute\tschema_version\tconsumer_hits")
shown = 0
for old in olds:
    h = hits.get(old, [])
    if not h:
        continue
    fn, new = renames[old]
    shown += 1
    print(f"{old}\t{new}\t{fn}\t{' ;; '.join(h)}")
print(f"# old names still read by consumers: {shown} of {len(olds)}", file=sys.stderr)
EOF
