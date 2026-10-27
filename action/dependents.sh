#!/usr/bin/env bash
# SIB-2: run `invar dependents --repo` (which consumers this change breaks)
# and record its exit code, broken count and markdown report. Advisory: this
# step never fails the job, whatever the evaluation finds.
# SIB-3: on a pull request (INVAR_BASE_REF set), a workspace-relative index
# and trust root are read from origin/<base>, never from the pull request;
# if the base cannot be read, nothing is evaluated (stated, exit 3
# recorded) and the step fails the job (B-D2 r1): a lost trust root is an
# infrastructure failure, never an advisory outcome.
set -uo pipefail

report="${RUNNER_TEMP:-/tmp}/invar-dependents.md"
rm -f "$report"

locks="$INVAR_DEPENDENTS_LOCKS"
signers="${INVAR_DEPENDENTS_SIGNERS:-}"
base="${INVAR_BASE_REF:-}"
relative() { [ -n "$1" ] && [ "${1#/}" = "$1" ]; }

if [ -n "$base" ]; then
  paths=()
  if relative "$locks"; then paths+=("$locks"); fi
  if relative "$signers"; then paths+=("$signers"); fi
  if [ "${#paths[@]}" -gt 0 ]; then
    tree="${RUNNER_TEMP:-/tmp}/invar-base"
    rm -rf "$tree"
    mkdir -p "$tree"
    if git fetch -q --no-tags --depth=1 origin "+refs/heads/$base:refs/remotes/origin/$base" 2>/dev/null &&
      git archive "origin/$base" -- "${paths[@]}" 2>/dev/null | tar -x -C "$tree"; then
      if relative "$locks"; then locks="$tree/$locks"; fi
      if relative "$signers"; then signers="$tree/$signers"; fi
    else
      msg="not evaluated: base ref $base could not be read; the pull request's own index is never used"
      echo "$msg"
      { echo "dependents_exit=3"; echo "dependents_broken=0"; } >> "$GITHUB_OUTPUT"
      if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then echo "$msg" >> "$GITHUB_STEP_SUMMARY"; fi
      exit 1
    fi
  fi
fi

args=(dependents --locks "$locks" --repo "$INVAR_DEPENDENTS_REPO" --report-md "$report")
if [ -n "$signers" ]; then
  args+=(--lock-signers "$signers")
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
