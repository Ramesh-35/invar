#!/usr/bin/env bash
# Run `invar check --ci` and record its exit code and verdict. This step never
# fails the job itself; the Gate step decides after the PR comment is handled.
set -uo pipefail

report="${RUNNER_TEMP:-/tmp}/invar-report.md"
rm -f "$report"

GITHUB_TOKEN="${INVAR_TOKEN:-}" \
GITHUB_BASE_REF="${INVAR_BASE_REF:-${GITHUB_BASE_REF:-}}" \
  "$INVAR_BIN" check --ci --config "$INVAR_CONFIG" --report-md "$report"
code=$?

case "$code" in
  0) verdict=hold ;;
  1) verdict=violated ;;
  2) verdict=ledger_break ;;
  4) verdict=skipped ;;
  *) verdict=error ;;
esac

{
  echo "exit_code=$code"
  echo "verdict=$verdict"
  if [ -f "$report" ]; then echo "report=$report"; fi
} >> "$GITHUB_OUTPUT"

if [ -f "$report" ] && [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  cat "$report" >> "$GITHUB_STEP_SUMMARY"
fi
exit 0
