#!/usr/bin/env bash
# JUDGE-CONTRACTS r0: judge every declared contract this change touches,
# in one pass. INVAR_JUDGE_CONTRACTS holds one or more git pathspec globs,
# separated by whitespace or newlines. The base tree is read from
# origin/<base>, never from the pull request (SIB-3). Exit mapping, stated:
#   0  every changed contract HOLDS (or is new)          -> pass
#   1  at least one VIOLATED                             -> fail, unless INVAR_FAIL_ON_VIOLATION=false
#   4  NOT EVALUATED: unjudged_construct or
#      unchecked_change only (JUDGE-REACH r0)            -> pass, each one a ::warning
#      NOT EVALUATED: unparsable, unreadable_base or
#      contract_deleted on any contract                  -> fail: never a pass
#   3  error, including a glob that matches nothing      -> fail
set -uo pipefail

base="${INVAR_BASE_REF:-}"
ref="origin/main"
if [ -n "$base" ]; then
  git fetch -q --no-tags --depth=1 origin "+refs/heads/$base:refs/remotes/origin/$base" 2>/dev/null || true
  ref="origin/$base"
fi

set -f # globs go to git verbatim, never expanded by this shell
args=()
for g in $INVAR_JUDGE_CONTRACTS; do args+=(--contracts "$g"); done
if [ "${#args[@]}" -eq 0 ]; then
  echo "::error title=invar.judge::judge_contracts is set but holds no glob"
  exit 1
fi

text="$("$INVAR_BIN" judge "${args[@]}" --base "$ref" 2>&1)"
code=$?
printf '%s\n' "$text"

json="$("$INVAR_BIN" judge "${args[@]}" --base "$ref" --format json 2>/dev/null)"
reasons="$(printf '%s\n' "$json" | grep -o '"reason": "[a-z_]*"' | cut -d'"' -f4 | sort -u | tr '\n' ' ')"

if [ "$code" -ne 0 ]; then
  "$INVAR_BIN" judge "${args[@]}" --base "$ref" --format github 2>/dev/null || true
fi

summary="$(printf '%s\n' "$text" | grep '^judged ' | tail -n1)"
{
  echo "judge_contracts_exit=$code"
  echo "judge_contracts_summary=${summary:-error}"
} >> "$GITHUB_OUTPUT"

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  {
    echo '## invar judge (contracts)'
    echo
    echo '```'
    printf '%s\n' "$text"
    echo '```'
  } >> "$GITHUB_STEP_SUMMARY"
fi

case "$code" in
  0) exit 0 ;;
  1) if [ "${INVAR_FAIL_ON_VIOLATION:-true}" = "false" ]; then exit 0; else exit 1; fi ;;
  4) case " $reasons " in
       *" unparsable "*|*" unreadable_base "*|*" contract_deleted "*) exit 1 ;;
       *) exit 0 ;;
     esac ;;
  *) exit 1 ;;
esac
