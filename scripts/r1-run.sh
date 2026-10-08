#!/usr/bin/env bash
# R1 corpus runner. For every case under corpus-real/ (except _template), run
# `invar check --ci` on the pinned commits and compare INVAR's residue with the
# residue the public source says broke. Baseline columns are copied from
# case.toml as recorded by a human; this script never infers them.
#
# Outcome per case:
#   CAUGHT   every expected field is in INVAR's residue
#   PARTIAL  some expected fields found, some missed
#   MISSED   none found (or INVAR said HOLDS on a real break)
#   CLEAN    control case ("none" expected) and INVAR reported nothing
#   FALSE_POSITIVE  control case and INVAR reported residue
#   ERROR    INVAR could not evaluate (exit 3); never counted as a verdict
# Extra fields INVAR reports beyond the expected set are listed, not hidden.
#
# Usage: scripts/r1-run.sh [path/to/invar] > r1-results.tsv
set -uo pipefail

invar="${1:-invar}"
root="$(cd "$(dirname "$0")/.." && pwd)"
get() { sed -n "s/^$2 *= *\"\(.*\)\".*/\1/p" "$1" | head -n1; }
norm() { tr ',' '\n' | sed 's/^ *//; s/ *$//' | sed '/^$/d' | sort -u; }

printf 'case\treal\texpected\tinvar_residue\toutcome\textra\toasdiff\tbuf_breaking\tschema_registry\tsemgrep\tai_reviewer\n'
for dir in "$root"/corpus-real/*/; do
  name="$(basename "$dir")"
  [ "$name" = "_template" ] && continue
  case_file="$dir/case.toml"
  expected="$(get "$case_file" residue)"
  real="$(get "$case_file" real)"

  out="$("$invar" check --ci --config "$dir/invar.toml" 2>&1)"; code=$?
  found="$(printf '%s\n' "$out" | sed -n 's/.*\[VIOLATED\][^:]*: \[\(.*\)\].*/\1/p' | tr -d '"' | norm | paste -sd, -)"

  if [ "$code" -ge 2 ]; then
    outcome=ERROR
  elif [ "$expected" = "none" ]; then
    if [ -z "$found" ]; then outcome=CLEAN; else outcome=FALSE_POSITIVE; fi
  else
    exp_set="$(printf '%s' "$expected" | norm)"
    hit="$(comm -12 <(printf '%s\n' "$exp_set") <(printf '%s' "$found" | norm) | wc -l | tr -d ' ')"
    total="$(printf '%s\n' "$exp_set" | sed '/^$/d' | wc -l | tr -d ' ')"
    if [ "$hit" -eq "$total" ]; then outcome=CAUGHT
    elif [ "$hit" -gt 0 ]; then outcome=PARTIAL
    else outcome=MISSED; fi
  fi
  extra=""
  if [ "$expected" != "none" ] && [ -n "$found" ]; then
    extra="$(comm -13 <(printf '%s' "$expected" | norm) <(printf '%s' "$found" | norm) | paste -sd, -)"
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s' "$name" "${real:-?}" "$expected" "${found:--}" "$outcome" "${extra:--}"
  for t in oasdiff buf_breaking schema_registry semgrep ai_reviewer; do
    printf '\t%s' "$(get "$case_file" "$t")"
  done
  printf '\n'
done
