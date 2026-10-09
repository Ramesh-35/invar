#!/usr/bin/env bash
# Fail if any workflow or action.yml uses a third-party action by a mutable
# ref (tag or branch) instead of a full 40-character commit SHA. Local actions
# (uses: ./...) and docker:// images are out of scope. Tags can be repointed
# by whoever controls the upstream repo (tj-actions/changed-files, CVE-2025-30066).
set -euo pipefail
root="${1:-.}"
bad=0
while IFS= read -r line; do
  ref="${line#*uses:}"; ref="${ref%%#*}"; ref="$(echo "$ref" | tr -d ' "'"'")"
  case "$ref" in ./*|docker://*|'') continue ;; esac
  sha="${ref##*@}"
  if ! [[ "$sha" =~ ^[0-9a-f]{40}$ ]]; then echo "unpinned: $line"; bad=1; fi
done < <(grep -rhoE '^\s*-?\s*uses:\s*[^ ]+' "$root/.github/workflows" "$root/action.yml" 2>/dev/null || true)
[ "$bad" -eq 0 ] && echo "all third-party actions pinned to commit SHAs"
exit "$bad"
