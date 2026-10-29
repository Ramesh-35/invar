#!/usr/bin/env bash
# JUDGE-1: run `invar judge` on a declared data contract against the base
# ref. The PRE contract is read from origin/<base>, never from the pull
# request (the SIB-3 trust discipline; the engine enforces it, this step
# only fetches the ref). Exit mapping, stated:
#   HOLDS (0)                        -> pass
#   VIOLATED (1)                     -> fail, unless INVAR_FAIL_ON_VIOLATION=false
#   NOT EVALUATED new_contract (4)   -> pass: nothing was promised before
#   NOT EVALUATED unparsable (4)     -> fail: this change made the contract unreadable
#   NOT EVALUATED unreadable_base(4) -> fail (B-D2 r1): a lost baseline is an
#                                       infrastructure failure, never a pass
set -uo pipefail

base="${INVAR_BASE_REF:-}"
ref="origin/main"
if [ -n "$base" ]; then
  git fetch -q --no-tags --depth=1 origin "+refs/heads/$base:refs/remotes/origin/$base" 2>/dev/null || true
  ref="origin/$base"
fi

receipt="${RUNNER_TEMP:-/tmp}/invar-judge-receipt.json"
text="$("$INVAR_BIN" judge --contract "$INVAR_JUDGE_CONTRACT" --base "$ref" --receipt "$receipt" 2>&1)"
code=$?
printf '%s\n' "$text"

json="$("$INVAR_BIN" judge --contract "$INVAR_JUDGE_CONTRACT" --base "$ref" --format json 2>/dev/null)"
verdict="$(printf '%s\n' "$json" | sed -n 's/^  "verdict": "\(.*\)",*$/\1/p' | head -n1)"
reason="$(printf '%s\n' "$json" | sed -n 's/^  "reason": "\(.*\)",*$/\1/p' | head -n1)"

{
  echo "judge_exit=$code"
  echo "judge_verdict=${verdict:-error}"
  echo "judge_reason=${reason:-}"
  if [ -f "$receipt" ]; then echo "judge_receipt=$receipt"; fi
} >> "$GITHUB_OUTPUT"

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  {
    echo '## invar judge'
    echo
    echo '```'
    printf '%s\n' "$text"
    echo '```'
    # The seal: one quiet line from the real receipt bytes. It states the
    # receipt's own BLAKE3 and the engine version, nothing it cannot prove.
    if [ -f "$receipt" ]; then
      # Pure-shell extraction (no sed): the receipt footer is the last
      # line, {"receipt_blake3":"<hex>"}; portable across BSD and GNU.
      footer="$(tail -n1 "$receipt")"
      rb="${footer#*\"receipt_blake3\":\"}"
      rb="$(printf '%.16s' "$rb")"
      case "$rb" in *[!0-9a-f]*) rb="";; esac
      ver="$("$INVAR_BIN" --version 2>/dev/null | head -n1)"
      if [ -n "$rb" ]; then
        echo
        echo "<sub>sealed by ${ver:-invar} · receipt blake3 \`$rb…\` · a skip is never a pass</sub>"
      fi
    fi
  } >> "$GITHUB_STEP_SUMMARY"
fi

case "$code" in
  0) exit 0 ;;
  1) if [ "${INVAR_FAIL_ON_VIOLATION:-true}" = "false" ]; then exit 0; else exit 1; fi ;;
  4) case "$reason" in
       new_contract) exit 0 ;;
       *) exit 1 ;;
     esac ;;
  *) exit 1 ;;
esac
