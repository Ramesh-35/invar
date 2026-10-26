#!/usr/bin/env bash
# SECURITY S1 mutation gate: apply each mutant to a fresh copy of this tree, run the frozen
# S1 fixtures (tests/security_test.rs), and classify the result by the founder's kill rule:
#   CAUGHT        a fixture failed on an assertion (a behavioral kill);
#                 the failing fixtures are listed, the declared killer
#                 marked, so a kill by an undeclared fixture is visible
#   SURVIVED      every fixture passed
#   INVALID       the mutant did not build, or the harness failed, or a
#                 failure was not an assertion: NOT a kill
# --no-fail-fast is required: without it cargo stops after the first
# failing test binary; kept for parity with scripts/rs although
# only one test binary runs here.
# Usage, from the repository root: bash scripts/security/mutate.sh
set -u
root="$(cd "$(dirname "$0")/../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/mutants-security}"
TIMEOUT=""
command -v timeout >/dev/null && TIMEOUT="timeout 900"
command -v gtimeout >/dev/null && TIMEOUT="gtimeout 900"
for p in "$root"/scripts/security/mutants/S1-M*.patch; do
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
  (cd "$work/t" && $TIMEOUT cargo test --release --no-fail-fast --test security_test > "$log" 2>&1)
  rc=$?
  if ! grep -q "^test result:" "$log"; then
    echo "$id INVALID (no test result: build or harness failure, rc $rc)  $head"
    continue
  fi
  failed="$(sed -n 's/^test \(s1_f[0-9a-z_]*\) \.\.\. FAILED$/\1/p' "$log" | sed 's/^\(s1_f[0-9]*[a-z]\{0,1\}\)_.*/\1/' | sort | paste -sd, -)"
  if [ -z "$failed" ]; then
    echo "$id SURVIVED  $head"
    continue
  fi
  nonassert="$(grep -c "panicked at" "$log")"
  asserts="$(grep -cE "panicked at crates/invar-falsify/tests/security_test.rs" "$log")"
  if [ "$asserts" -lt 1 ] || [ "$asserts" -ne "$nonassert" ]; then
    echo "$id INVALID (a failure outside the fixtures' assertions)  $head  failed=$failed"
    continue
  fi
  case ",$failed," in *",$killer,"*) dk="DECLARED-KILLER-FAILED";; *) dk="DECLARED-KILLER-PASSED";; esac
  incidental="$(printf '%s' "$failed" | tr ',' '\n' | grep -vx "$killer" | paste -sd, -)"
  echo "$id CAUGHT  $dk ($killer)  incidental=[${incidental:-none}]  $head"
done
