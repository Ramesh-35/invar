#!/bin/bash
# Re-verify Software Heritage archival of every origin in
# docs/reference/SWH-ARCHIVAL.tsv. Exit 0: all archived. Exit 1: at
# least one origin absent (named on stdout). Exit 3: a check could not
# be completed (stated, never silently dropped).
set -u
TSV="$(dirname "$0")/../docs/reference/SWH-ARCHIVAL.tsv"
[ -f "$TSV" ] || { echo "ERROR: $TSV not found"; exit 3; }
absent=0; errors=0
while IFS=$'\t' read -r origin _status _note; do
  case "$origin" in \#*|origin|"") continue;; esac
  code=$(curl -s --max-time 40 -o /dev/null -w "%{http_code}" \
    "https://archive.softwareheritage.org/api/1/origin/$origin/get/")
  if [ "$code" = "200" ]; then
    :
  elif [ "$code" = "404" ]; then
    echo "ABSENT: $origin"; absent=$((absent+1))
  else
    echo "CHECK FAILED ($code): $origin"; errors=$((errors+1))
  fi
done < "$TSV"
echo "absent: $absent, check failures: $errors"
[ "$errors" -gt 0 ] && exit 3
[ "$absent" -gt 0 ] && exit 1
exit 0
