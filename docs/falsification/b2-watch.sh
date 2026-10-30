#!/usr/bin/env bash
# B2 watch instrument: counts the kill-rule signals mechanically, one row
# per run, appended to B2-WATCH-LEDGER.tsv next to this script. The kill
# rule (PROGRAM-LEDGER): fewer than 2 external independent repositories
# running `invar judge` in an active PR pipeline within 14 days of the
# v0.2.0 release kills the standalone data slice.
# What is counted, honestly:
# - stars, forks, watchers and release-asset downloads: public API, always.
# - external repositories referencing judge_contract in workflow files:
#   GitHub code search NEEDS a token; without GH_TOKEN this column is
#   NOT EVALUATED, stated, never zero. A skip is never a pass.
# Usage: bash docs/falsification/b2-watch.sh   (run from the repo root)
set -uo pipefail
ledger="$(dirname "$0")/B2-WATCH-LEDGER.tsv"
[ -f "$ledger" ] || printf 'utc\tstars\tforks\twatchers\tv020_downloads\texternal_judge_repos\tnotes\n' > "$ledger"
ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
repo_json="$(curl -s https://api.github.com/repos/Ramesh-35/invar)"
stars="$(printf '%s' "$repo_json" | python3 -c 'import json,sys;d=json.load(sys.stdin);print(d.get("stargazers_count","ERR"))')"
forks="$(printf '%s' "$repo_json" | python3 -c 'import json,sys;d=json.load(sys.stdin);print(d.get("forks_count","ERR"))')"
watchers="$(printf '%s' "$repo_json" | python3 -c 'import json,sys;d=json.load(sys.stdin);print(d.get("subscribers_count","ERR"))')"
dl="$(curl -s https://api.github.com/repos/Ramesh-35/invar/releases/tags/v0.2.0 | python3 -c 'import json,sys
d=json.load(sys.stdin)
print(sum(a.get("download_count",0) for a in d.get("assets",[])) if "assets" in d else "no_release_yet")')"
if [ -n "${GH_TOKEN:-}" ]; then
  ext="$(curl -s -H "Authorization: Bearer $GH_TOKEN" \
    "https://api.github.com/search/code?q=judge_contract+Ramesh-35%2Finvar+path%3A.github%2Fworkflows" \
    | python3 -c 'import json,sys
d=json.load(sys.stdin)
items=d.get("items",[])
repos=sorted({i["repository"]["full_name"] for i in items if not i["repository"]["full_name"].startswith("Ramesh-35/")})
joined = ",".join(repos); print(f"{len(repos)}:{joined}" if repos else "0")')"
  note="code search live"
  # A failed search (rate limit, auth, malformed reply) must never render
  # as a blank or a zero: it is stated as ERROR. A skip is never a pass.
  case "$ext" in
    ""|*[!0-9:,a-zA-Z_./-]*) ext="ERROR"; note="code search failed: column not evaluated, stated" ;;
  esac
else
  ext="NOT_EVALUATED"
  note="no GH_TOKEN: external-repo column not evaluated, stated"
fi
printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$ts" "$stars" "$forks" "$watchers" "$dl" "$ext" "$note" >> "$ledger"
tail -2 "$ledger"
