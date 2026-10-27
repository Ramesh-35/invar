#!/usr/bin/env bash
# SIB-2: run `invar dependents --repo` (which consumers this change breaks)
# and record its exit code, broken count and markdown report. Advisory: this
# step never fails the job, whatever the evaluation finds.
set -uo pipefail

report="${RUNNER_TEMP:-/tmp}/invar-dependents.md"
rm -f "$report"

args=(dependents --locks "$INVAR_DEPENDENTS_LOCKS" --repo "$INVAR_DEPENDENTS_REPO" --report-md "$report")
if [ -n "${INVAR_DEPENDENTS_SIGNERS:-}" ]; then
  args+=(--lock-signers "$INVAR_DEPENDENTS_SIGNERS")
fi
out="$("$INVAR_BIN" "${args[@]}" 2>&1)"
code=$?
printf '%s\n' "$out"
broken="$(printf '%s\n' "$out" | sed -n 's/^summary: \([0-9][0-9]*\) broken.*/\1/p' | tail -n1)"

{
  echo "dependents_exit=$code"
  echo "dependents_broken=${broken:-0}"
  if [ -f "$report" ]; then echo "dependents_report=$report"; fi
} >> "$GITHUB_OUTPUT"

if [ -f "$report" ] && [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  cat "$report" >> "$GITHUB_STEP_SUMMARY"
fi
exit 0
