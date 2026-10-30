#!/usr/bin/env bash
# The admission gate for any externally produced harvest: a specimen is
# admitted only if every commit it cites exists in the repository it
# names, and the consumer file exists at the consumer commit. Checked
# against GitHub directly; a claim that "SHAs were directly observed" is
# never accepted on trust.
# Input TSV (header line required), one specimen per row:
#   id  producer_repo  producer_sha  consumer_repo  consumer_sha  consumer_path
# Exit 0 only if every row VERIFIED; 1 if any row FAILED; 3 on error.
# Unverifiable is stated (API_ERROR), never counted as verified.
# Usage: GH_TOKEN=... bash verify-specimens.sh specimens.tsv
set -uo pipefail
tsv="${1:?usage: verify-specimens.sh specimens.tsv}"
[ -f "$tsv" ] || { echo "ERROR (not a verdict): $tsv not found"; exit 3; }
auth=(); [ -n "${GH_TOKEN:-}" ] && auth=(-H "Authorization: Bearer $GH_TOKEN")
probe() { curl -s -o /dev/null -w "%{http_code}" "${auth[@]}" "https://api.github.com/repos/$1"; }
ok=0; bad=0; err=0
while IFS=$'\t' read -r id prepo psha crepo csha cpath; do
  [ "$id" = "id" ] && continue
  [ -z "$id" ] && continue
  why=""
  for pair in "$prepo/commits/$psha:producer commit" "$crepo/commits/$csha:consumer commit" "$crepo/contents/$cpath?ref=$csha:consumer file at that commit"; do
    path="${pair%%:*}"; label="${pair#*:}"
    code=$(probe "$path")
    case "$code" in
      200) ;;
      404|422) why="${why:+$why; }$label does not exist" ;;
      *) why="${why:+$why; }$label unverifiable (HTTP $code)"; err=$((err+1)) ;;
    esac
  done
  if [ -z "$why" ]; then echo "VERIFIED  $id"; ok=$((ok+1)); else echo "FAILED    $id: $why"; bad=$((bad+1)); fi
done < "$tsv"
if [ "$err" -gt 0 ]; then echo "summary: $ok verified, $bad failed ($err checks unverifiable, stated)"; else echo "summary: $ok verified, $bad failed"; fi
[ "$bad" -eq 0 ] || exit 1
