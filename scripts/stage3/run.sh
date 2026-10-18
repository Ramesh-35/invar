#!/usr/bin/env bash
# STAGE3-EVAL-001 phase 1 runner.
# Usage: INVAR_BIN=/path/to/release/invar ./run.sh <manifest> <outdir>
# Manifest line: <name> <git-url-or-path> <commit-sha>
# Blank lines and lines starting with # are ignored.
# Per repository: two fresh clones at the pinned SHA, invar init in each,
# raw stdout/stderr/exit/seconds recorded, byte comparison with zero
# normalization, one ledger row. The second clone is removed after the
# comparison; the first stays for phase 2.
set -u
MANIFEST=${1:?usage: run.sh <manifest> <outdir>}
OUT=${2:?usage: run.sh <manifest> <outdir>}
: "${INVAR_BIN:?set INVAR_BIN to the frozen release invar binary}"
h() { (shasum -a 256 "$1" 2>/dev/null || sha256sum "$1") | cut -c1-12; }
mkdir -p "$OUT"
BINHASH=$(h "$INVAR_BIN")
LEDGER="$OUT/ledger.tsv"
printf 'repo\tbinary\texit1\texit2\tdeterministic\tsec1\tsec2\tsummary\n' > "$LEDGER"
grep -v '^[[:space:]]*#' "$MANIFEST" | grep -v '^[[:space:]]*$' | \
while read -r name url sha; do
  for run in 1 2; do
    dir="$OUT/$name.run$run"
    rm -rf "$dir"
    git clone --quiet "$url" "$dir" || { echo "CLONE-FAILED $name" >> "$OUT/$name.findings"; continue 2; }
    git -C "$dir" checkout --quiet "$sha" || { echo "PIN-FAILED $name $sha" >> "$OUT/$name.findings"; continue 2; }
    start=$(date +%s)
    rc=0
    (cd "$dir" && "$INVAR_BIN" init > "$OUT/$name.init$run.out" 2> "$OUT/$name.init$run.err") || rc=$?
    end=$(date +%s)
    echo "$rc" > "$OUT/$name.init$run.code"
    echo "$((end - start))" > "$OUT/$name.init$run.sec"
  done
  det=yes
  cmp -s "$OUT/$name.init1.out" "$OUT/$name.init2.out" || det=no
  t1="$OUT/$name.run1/invar.toml"
  t2="$OUT/$name.run2/invar.toml"
  if [ -f "$t1" ] || [ -f "$t2" ]; then
    { [ -f "$t1" ] && [ -f "$t2" ] && cmp -s "$t1" "$t2"; } || det=no
  fi
  rm -rf "$OUT/$name.run2"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$name" "$BINHASH" \
    "$(cat "$OUT/$name.init1.code")" "$(cat "$OUT/$name.init2.code")" \
    "$det" \
    "$(cat "$OUT/$name.init1.sec")" "$(cat "$OUT/$name.init2.sec")" \
    "$(tail -n 1 "$OUT/$name.init1.out")" >> "$LEDGER"
done
echo "ledger: $LEDGER"
