#!/usr/bin/env bash
# Final pass/fail. An evaluation ERROR is never reported as a contract verdict,
# and errors fail the job even in report-only mode.
set -uo pipefail

case "${INVAR_EXIT:-}" in
  0)
    echo "invar: all contracts hold"
    exit 0 ;;
  1)
    if [ "${INVAR_FAIL_ON_VIOLATION:-true}" = "true" ]; then
      echo "::error title=INVAR contract violated::At least one cross-repository contract is VIOLATED. See the report."
      exit 1
    fi
    echo "::warning title=INVAR contract violated::Report-only mode (fail_on_violation=false)."
    exit 0 ;;
  2)
    echo "::error title=INVAR ledger integrity break::The bi-temporal ledger failed verification."
    exit 2 ;;
  4)
    echo "::error title=INVAR not fully evaluated::Some contracts were SKIPPED (language not supported). A skip is not a pass."
    exit 4 ;;
  *)
    echo "::error title=INVAR could not evaluate::ERROR, not a verdict (exit ${INVAR_EXIT:-unset}). See the 'Run invar check --ci' step log."
    exit 3 ;;
esac
