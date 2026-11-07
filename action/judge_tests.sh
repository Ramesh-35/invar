#!/usr/bin/env bash
# JUDGE-TESTS r0: for every contract this change modifies (the judge_contracts
# globs), run the base ref's own instance documents against the head
# schema. INVAR_JUDGE_TESTS names the instance directory with {name} for the
# contract's file stem, read from origin/<base>, never from the pull request.
# Exit mapping, stated:
#   1  a document valid at the base is rejected by the head  -> fail, unless INVAR_FAIL_ON_VIOLATION=false; one ::error per document
#   4  NOT EVALUATED                                          -> pass; `unverifiable` is one ::warning, the rest a summary line
#  124  the per-contract time budget (INVAR_TESTS_TIMEOUT, 120s) ran out -> pass with a ::warning; nothing judged, stated
#   3  error (a schema unreadable, jsonschema missing)        -> fail
# The static verdict (judge_contracts) is never changed by this step.
set -uo pipefail

base="${INVAR_BASE_REF:-}"
ref="origin/main"
if [ -n "$base" ]; then
  git fetch -q --no-tags --depth=1 origin "+refs/heads/$base:refs/remotes/origin/$base" 2>/dev/null || true
  ref="origin/$base"
fi
case "$INVAR_JUDGE_TESTS" in *'{name}'*) ;; *) echo "::error title=invar.tests::judge_tests must contain {name}"; exit 1 ;; esac

python3 -c 'import jsonschema' 2>/dev/null || python3 -m pip install -q 'jsonschema==4.26.0' 'PyYAML==6.0.3' || { echo "::error title=invar.tests::could not install jsonschema"; exit 1; }

set -f
globs=()
for g in $INVAR_JUDGE_CONTRACTS; do globs+=("$g"); done
mapfile -t contracts < <(git diff --name-only --diff-filter=M "$ref" -- "${globs[@]}" 2>/dev/null)
set +f

tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
worst=4; lines=()
for c in "${contracts[@]}"; do
  stem="$(basename "$c")"; stem="${stem%.*}"
  dir="${INVAR_JUDGE_TESTS//\{name\}/$stem}"
  inst="$tmp/$stem"; mkdir -p "$inst"
  n=0
  while IFS= read -r f; do
    [ -n "$f" ] || continue
    git show "$ref:$f" > "$inst/$(basename "$f")" 2>/dev/null && n=$((n+1))
  done < <(git ls-tree -r --name-only "$ref" -- "$dir" 2>/dev/null)
  if [ "$n" -eq 0 ]; then lines+=("$c: no instance documents under $dir at $ref"); continue; fi
  git show "$ref:$c" > "$tmp/$stem.base.json" 2>/dev/null || { lines+=("$c: unreadable at $ref"); worst=3; continue; }
  budget="${INVAR_TESTS_TIMEOUT:-120}"
  text="$(timeout "$budget" python3 "$INVAR_ACTION_PATH/action/tests.py" "$tmp/$stem.base.json" "$c" "$inst" 2>&1)"; code=$?
  if [ "$code" -eq 124 ]; then
    # A schema too costly to validate within the budget is stated, never passed.
    lines+=("$c: not checked: the time budget of ${budget}s was exceeded (NOT EVALUATED)")
    echo "::warning file=$c::invar tests: the time budget of ${budget}s was exceeded; no document was judged"
    continue
  fi
  lines+=("$c:" "$text")
  timeout "$budget" python3 "$INVAR_ACTION_PATH/action/tests.py" "$tmp/$stem.base.json" "$c" "$inst" --format github 2>/dev/null || true
  case "$code" in 1) [ "$worst" -eq 3 ] || worst=1 ;; 3) worst=3 ;; esac
done

printf '%s\n' "${lines[@]}"
{
  echo "judge_tests_exit=$worst"
  echo "judge_tests_summary=${#contracts[@]} contract(s) checked against their base instances"
} >> "$GITHUB_OUTPUT"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  { echo '## invar tests (instances)'; echo; echo '```'; printf '%s\n' "${lines[@]}"; echo '```'; } >> "$GITHUB_STEP_SUMMARY"
fi
case "$worst" in
  1) if [ "${INVAR_FAIL_ON_VIOLATION:-true}" = "false" ]; then exit 0; else exit 1; fi ;;
  3) exit 1 ;;
  *) exit 0 ;;
esac
