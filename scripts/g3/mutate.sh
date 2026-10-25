#!/usr/bin/env bash
# G3 Gate 3: apply each mutant to a fresh copy of this tree, run the frozen
# G3 fixtures, and classify the result by the founder's kill rule:
#   CAUGHT        a fixture failed on an assertion (a behavioral kill);
#                 the failing fixtures are listed, the declared killer
#                 marked, so a kill by an undeclared fixture is visible
#   SURVIVED      every fixture passed
#   INVALID       the mutant did not build, or the harness failed, or a
#                 failure was not an assertion: NOT a kill
# Usage, from the repository root: bash scripts/g3/mutate.sh
set -u
root="$(cd "$(dirname "$0")/../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/mutants-g3}"
TIMEOUT=""
command -v timeout >/dev/null && TIMEOUT="timeout 900"
command -v gtimeout >/dev/null && TIMEOUT="gtimeout 900"
for p in "$root"/scripts/g3/mutants/M*.patch; do
  id="$(basename "$p" .patch)"
  head="$(head -1 "$p" | sed 's/^# //')"
  killer="$(printf '%s' "$head" | sed -n 's/.*declared-killer=\([a-z0-9_]*\).*/\1/p')"
  rm -rf "$work/t"
  mkdir "$work/t"
  (cd "$root" && tar --exclude=./target --exclude=./.git -cf - .) | tar -x -C "$work/t"
  if ! (cd "$work/t" && patch -s -p1 < "$p"); then
    echo "$id INVALID (patch did not apply)  $head"
    continue
  fi
  log="$work/$id.log"
  (cd "$work/t" && $TIMEOUT cargo test --release --test g3_test > "$log" 2>&1)
  rc=$?
  if ! grep -q "^test result:" "$log"; then
    echo "$id INVALID (no test result: build or harness failure, rc $rc)  $head"
    continue
  fi
  failed="$(sed -n 's/^test \(g3_f[0-9a-z_]*\) \.\.\. FAILED$/\1/p' "$log" | sed 's/^\(g3_f[0-9]*\)_.*/\1/' | sort | paste -sd, -)"
  if [ -z "$failed" ]; then
    echo "$id SURVIVED  $head"
    continue
  fi
  nonassert="$(grep -c "panicked at" "$log")"
  asserts="$(grep -c "panicked at crates/invar-falsify/tests/g3_test.rs" "$log")"
  if [ "$asserts" -lt 1 ] || [ "$asserts" -ne "$nonassert" ]; then
    echo "$id INVALID (a failure outside the fixtures' assertions)  $head  failed=$failed"
    continue
  fi
  mark="undeclared"
  case ",$failed," in *",$killer,"*) mark="declared";; esac
  echo "$id CAUGHT by $failed (declared killer $killer: $mark)  $head"
done
