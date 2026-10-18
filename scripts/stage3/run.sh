#!/usr/bin/env bash
# STAGE3-EVAL-001 phase 1 runner, instrument r2.
# Usage: INVAR_BIN=/path/to/release/invar ./run.sh <manifest> <outdir>
# Manifest line: <name> <git-url-or-path> <commit-sha>
# Blank lines and lines starting with # are ignored.
#
# r2 changes, made and committed BEFORE the manifest exists:
#   r2.1 OUT, MANIFEST and INVAR_BIN are made absolute up front. r1 broke
#        with a relative outdir because init runs after cd into the clone,
#        so its redirections resolved against the wrong directory.
#   r2.2 pinned fetch at --depth 200 instead of a full clone: enough for
#        the ratified 100-commit backtest (S-D7) with margin, and phase 1
#        stays runnable within a bounded disk allowance.
#   r2.3 clone trees are removed after the comparison; run1's invar.toml
#        is kept as <name>.invar.toml for adjudication. Phase 2 re-clones
#        at the same pinned SHA, which is the point of pinning.
#   r2.4 a repo whose fetch or pin fails gets a ledger row saying so and
#        a findings file, never a tail error over a missing file.
set -u
MANIFEST=${1:?usage: run.sh <manifest> <outdir>}
OUT=${2:?usage: run.sh <manifest> <outdir>}
: "${INVAR_BIN:?set INVAR_BIN to the frozen release invar binary}"
mkdir -p "$OUT"
OUT=$(cd "$OUT" && pwd)
MANIFEST=$(cd "$(dirname "$MANIFEST")" && pwd)/$(basename "$MANIFEST")
case "$INVAR_BIN" in
  /*) ;;
  *) INVAR_BIN=$(cd "$(dirname "$INVAR_BIN")" && pwd)/$(basename "$INVAR_BIN") ;;
esac
h() { (shasum -a 256 "$1" 2>/dev/null || sha256sum "$1") | cut -c1-12; }
BINHASH=$(h "$INVAR_BIN")
LEDGER="$OUT/ledger.tsv"
printf 'repo\tbinary\texit1\texit2\tdeterministic\tsec1\tsec2\tsummary\n' > "$LEDGER"
grep -v '^[[:space:]]*#' "$MANIFEST" | grep -v '^[[:space:]]*$' | \
while read -r name url sha; do
  failed=""
  for run in 1 2; do
    dir="$OUT/$name.run$run"
    rm -rf "$dir"
    mkdir -p "$dir"
    git -C "$dir" init --quiet
    git -C "$dir" remote add origin "$url"
    if ! git -C "$dir" fetch --quiet --depth 200 origin "$sha"; then
      failed="FETCH-FAILED"
      echo "FETCH-FAILED run$run $sha" >> "$OUT/$name.findings"
      break
    fi
    if ! git -C "$dir" checkout --quiet FETCH_HEAD; then
      failed="PIN-FAILED"
      echo "PIN-FAILED run$run $sha" >> "$OUT/$name.findings"
      break
    fi
    start=$(date +%s)
    rc=0
    (cd "$dir" && "$INVAR_BIN" init > "$OUT/$name.init$run.out" 2> "$OUT/$name.init$run.err") || rc=$?
    end=$(date +%s)
    echo "$rc" > "$OUT/$name.init$run.code"
    echo "$((end - start))" > "$OUT/$name.init$run.sec"
  done
  if [ -n "$failed" ]; then
    rm -rf "$OUT/$name.run1" "$OUT/$name.run2"
    printf '%s\t%s\t-\t-\t-\t-\t-\t%s\n' "$name" "$BINHASH" "$failed" >> "$LEDGER"
    continue
  fi
  det=yes
  cmp -s "$OUT/$name.init1.out" "$OUT/$name.init2.out" || det=no
  t1="$OUT/$name.run1/invar.toml"
  t2="$OUT/$name.run2/invar.toml"
  if [ -f "$t1" ] || [ -f "$t2" ]; then
    { [ -f "$t1" ] && [ -f "$t2" ] && cmp -s "$t1" "$t2"; } || det=no
  fi
  [ -f "$t1" ] && cp "$t1" "$OUT/$name.invar.toml"
  rm -rf "$OUT/$name.run1" "$OUT/$name.run2"
  summary=""
  [ -s "$OUT/$name.init1.out" ] && summary=$(tail -n 1 "$OUT/$name.init1.out")
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$name" "$BINHASH" \
    "$(cat "$OUT/$name.init1.code")" "$(cat "$OUT/$name.init2.code")" \
    "$det" \
    "$(cat "$OUT/$name.init1.sec")" "$(cat "$OUT/$name.init2.sec")" \
    "$summary" >> "$LEDGER"
done
echo "ledger: $LEDGER"
