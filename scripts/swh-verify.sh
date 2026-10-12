#!/bin/bash
# Re-verify Software Heritage archival of every origin in
# docs/reference/SWH-ARCHIVAL.tsv. Exit 0: all archived. Exit 1: at
# least one origin absent (named on stdout). Exit 3: a check could not
# be completed (stated, never silently dropped). HTTP 429 is retried
# with backoff (4 attempts); only a check that still cannot complete
# counts as a failure.
#
# Pacing: SWH's anonymous API budget is far below 178 quick requests
# (run 38232152147 burned 46 minutes in retries and failed), so without
# a token the sweep deliberately walks at one origin per 31s, about 95
# minutes total, which a weekly cron affords. Set SWH_TOKEN in the
# environment (a GitHub Actions secret, never argv) to use
# authenticated limits and the fast pace.
set -u
AUTH=()
PACE=31
if [ -n "${SWH_TOKEN:-}" ]; then
  AUTH=(-H "Authorization: Bearer $SWH_TOKEN")
  PACE=0.5
fi
TSV="$(dirname "$0")/../docs/reference/SWH-ARCHIVAL.tsv"
[ -f "$TSV" ] || { echo "ERROR: $TSV not found"; exit 3; }
absent=0; errors=0; checked=0
while IFS=$'\t' read -r origin _status _note; do
  case "$origin" in \#*|origin|"") continue;; esac
  code=000
  for attempt in 1 2 3 4; do
    code=$(curl -s --max-time 40 -o /dev/null -w "%{http_code}" "${AUTH[@]}" \
      "https://archive.softwareheritage.org/api/1/origin/$origin/get/")
    case "$code" in
      200|404) break;;
      429) sleep $((3 * attempt));;
      *) sleep 1;;
    esac
  done
  checked=$((checked+1))
  if [ "$code" = "404" ]; then
    echo "ABSENT: $origin"; absent=$((absent+1))
  elif [ "$code" != "200" ]; then
    echo "CHECK FAILED ($code): $origin"; errors=$((errors+1))
  fi
  sleep "$PACE"
done < "$TSV"
echo "checked: $checked, absent: $absent, check failures: $errors"
[ "$errors" -gt 0 ] && exit 3
[ "$absent" -gt 0 ] && exit 1
exit 0
