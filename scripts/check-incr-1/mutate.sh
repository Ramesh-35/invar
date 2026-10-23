#!/usr/bin/env bash
# CHECK-INCR-1 Gate 3: apply each mutant patch to a clean copy of this tree,
# run the CHECK-INCR-1 fixtures, and record CAUGHT (the fixtures fail) or
# SURVIVED (they pass). Gate 3 passes when every non-equivalent mutant is
# CAUGHT. Usage, from the repository root:
#   bash scripts/check-incr-1/mutate.sh
set -u
root="$(cd "$(dirname "$0")/../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
git -C "$root" archive HEAD | tar -x -C "$work"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/mutants-incr1}"
TIMEOUT=""
command -v timeout >/dev/null && TIMEOUT="timeout 600"
command -v gtimeout >/dev/null && TIMEOUT="gtimeout 600"
caught=0
survived=0
for p in "$root"/scripts/check-incr-1/mutants/E*.patch; do
  id="$(basename "$p" .patch)"
  what="$(head -1 "$p" | sed 's/^# //')"
  if ! (cd "$work" && git apply "$p"); then
    echo "$id NOT APPLIED  $what"
    exit 3
  fi
  if (cd "$work" && $TIMEOUT cargo test --release --test check_incr1_test >/dev/null 2>&1); then
    echo "$id SURVIVED     $what"
    survived=$((survived + 1))
  else
    echo "$id CAUGHT       $what"
    caught=$((caught + 1))
  fi
  (cd "$work" && git apply -R "$p")
done
echo "caught $caught, survived $survived"
[ "$survived" -eq 0 ]
