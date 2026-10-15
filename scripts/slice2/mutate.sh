#!/bin/sh
# INIT-DISCOVERY slice 2 mutation check (acceptance gate 3).
#
# For each patch in scripts/slice2/mutations/, apply it to a fresh copy of the
# crates, confirm the mutated code BUILDS, then run the fixture test(s) named
# on its "# catch:" line. A mutation is CAUGHT only when the build succeeds
# and every named test fails. A control run first confirms the same tests pass
# on the unmutated code, so a failure is attributable to the mutation.
#
# BUILD-FAILED and APPLY-FAILED mutants are NOT kills: they fail the gate.
# Usage, from the repository root:  sh scripts/slice2/mutate.sh
# Exit 0 only if the control passes and every mutation is caught.
# This is a mutation check, not formal verification.
set -u
ROOT=$(pwd)
WORK=$(mktemp -d)
TGT="${MUTATE_TARGET:-$WORK/target}"
status=0

copy() {
  rm -rf "$WORK/t"
  mkdir -p "$WORK/t"
  cp -R "$ROOT/Cargo.toml" "$ROOT/Cargo.lock" "$ROOT/crates" "$WORK/t/"
}

run_test() { # $1 = test name; returns cargo's status
  (cd "$WORK/t" && CARGO_TARGET_DIR="$TGT" cargo test --release -q \
    --test init_slice2_test "$1" -- --exact >/dev/null 2>&1)
}

all_tests=$(sed -n 's/^# catch: //p' "$ROOT"/scripts/slice2/mutations/*.patch | tr ' ' '\n' | sort -u)

copy
for t in $all_tests; do
  if run_test "$t"; then echo "CONTROL pass $t"; else echo "CONTROL FAIL $t"; status=1; fi
done

for p in "$ROOT"/scripts/slice2/mutations/*.patch; do
  name=$(basename "$p" .patch)
  tests=$(sed -n 's/^# catch: //p' "$p")
  copy
  if ! (cd "$WORK/t" && patch -s -p1 < "$p"); then
    echo "APPLY-FAILED $name"; status=1; continue
  fi
  if ! (cd "$WORK/t" && CARGO_TARGET_DIR="$TGT" cargo test --release -q --no-run >/dev/null 2>&1); then
    echo "BUILD-FAILED $name (does not count as caught)"; status=1; continue
  fi
  caught=1
  for t in $tests; do
    if run_test "$t"; then echo "SURVIVED $name $t"; caught=0; else echo "caught   $name $t"; fi
  done
  if [ "$caught" = 1 ]; then echo "CAUGHT   $name"; else echo "SURVIVED $name"; status=1; fi
done
rm -rf "$WORK"
exit $status
