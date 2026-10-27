#!/usr/bin/env bash
# Create or update INVAR's PR comment, found by its hidden marker. On a
# violation it creates or updates; once contracts hold again it only refreshes
# an existing comment, so a stale VIOLATED comment never outlives the fix. The
# body is the Rust-rendered report passed as a file, never interpolated into a
# script. Best effort: the step is continue-on-error, so a read-only fork token
# cannot change the verdict.
set -euo pipefail

if [ ! -f "${INVAR_REPORT:-}" ]; then echo "invar: no report to post"; exit 0; fi
marker="${INVAR_MARKER:-<!-- invar-check -->}"

existing=$(gh api "repos/$INVAR_REPO/issues/$INVAR_PR/comments" --paginate \
  --jq ".[] | select(.body | startswith(\"$marker\")) | .id" | head -n1)

if [ -n "$existing" ]; then
  gh api --method PATCH "repos/$INVAR_REPO/issues/comments/$existing" -F "body=@$INVAR_REPORT" >/dev/null
  echo "invar: updated PR comment $existing"
elif [ "${INVAR_EXIT:-}" = "1" ]; then
  gh api --method POST "repos/$INVAR_REPO/issues/$INVAR_PR/comments" -F "body=@$INVAR_REPORT" >/dev/null
  echo "invar: posted PR comment"
else
  echo "invar: contracts hold and no earlier comment exists; nothing to post"
fi
