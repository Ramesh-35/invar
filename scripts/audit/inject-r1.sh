#!/usr/bin/env bash
# AUDIT defect injection (r1 harness; scripts/audit/inject.sh is r0, preserved as run) (AUDIT-SPEC section F): apply a DI patch to a fresh
# copy of this tree, build, and run the corpus capture with that binary.
# Gate: every corpus case that is VIOLATED on the clean engine must exit 3
# with "self-audit failed", and no case may print [VIOLATED]. Also reports
# the full fixture suite's failing tests under the defect.
# Usage, from the repository root: bash scripts/audit/inject-r1.sh <DI-n> <clean corpus capture dir>
set -u
root="$(cd "$(dirname "$0")/../.." && pwd)"
id="$1"; clean="$2"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/inject-audit}"
(cd "$root" && tar --exclude=./target --exclude=./.git -cf - .) | tar -x -C "$work"
(cd "$work" && patch -s -p1 < "scripts/audit/mutants-r1/$id.patch") || { echo "$id INVALID (patch did not apply)"; exit 1; }
(cd "$work" && cargo build --release -q 2>/dev/null) || { echo "$id INVALID (build failed)"; exit 1; }
bash "$root/scripts/g3/corpus-capture.sh" "$CARGO_TARGET_DIR/release/invar" "$work/cap" > /dev/null
total=0; trapped=0; leaked=0
for f in "$clean"/*.out; do
  n="$(basename "$f")"
  grep -q "\[VIOLATED\]" "$f" || continue
  total=$((total+1))
  if tail -1 "$work/cap/$n" | grep -q "^exit 3$" && grep -q "self-audit failed" "$work/cap/$n"; then trapped=$((trapped+1)); else echo "  NOT TRAPPED: $n"; fi
done
leaked="$(grep -l "\[VIOLATED\]" "$work"/cap/*.out 2>/dev/null | wc -l | tr -d ' ')"
(cd "$work" && cargo test --release --no-fail-fast > "$work/suite.log" 2>&1)
failing="$(grep -c "\.\.\. FAILED$" "$work/suite.log")"
if [ "$trapped" -eq "$total" ] && [ "$leaked" -eq 0 ]; then verdict=PASS; else verdict=FAIL; fi
echo "$id $verdict: corpus convictions trapped as exit-3 self-audit failures: $trapped of $total; cases still printing [VIOLATED]: $leaked; fixture tests failing under the defect: $failing"
