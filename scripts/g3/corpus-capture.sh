#!/usr/bin/env bash
# G3 (RECALL-ATTACK-1 phase 1): capture the full `invar check --ci` output
# and exit code of every corpus case, so a before/after comparison covers
# diagnostics byte-for-byte, not only outcome tags. Output: one file per
# case, <out>/<case>.out, whose last line is "exit <code>".
# Usage: bash scripts/g3/corpus-capture.sh <invar> <out-dir>
# The fetch must not carry a placeholder token (it would route to the
# authenticated endpoint); unset GITHUB_TOKEN/GH_TOKEN when no real one.
set -u
invar="$1"
out="$2"
root="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$out"
for dir in "$root"/corpus-real/*/; do
  name="$(basename "$dir")"
  [ "$name" = "_template" ] && continue
  "$invar" check --ci --config "$dir/invar.toml" > "$out/$name.out" 2>&1
  echo "exit $?" >> "$out/$name.out"
done
ls "$out" | wc -l
