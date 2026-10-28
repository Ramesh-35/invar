#!/usr/bin/env bash
# B1 pinning (DATA-KILLGATE-SPEC r0, B1-D3): lock each candidate case's
# bytes at its pinned commits. Input: a TSV (comments and blank lines
# ignored) with tab-separated fields:
#   id  repo_url  pre_sha  post_sha  contract_path  consumer_note
# For each case: fetch exactly the two commits, extract the contract file
# at both, store under <out>/<id>/{pre,post}/, and write MANIFEST.sha256
# and case.tsv. A case whose fetch or extraction fails is recorded in
# FAILED.tsv with the reason, never silently dropped.
# Usage: bash pin-cases.sh <cases.tsv> <out-dir>
set -euo pipefail
tsv="$1"; out="$2"
mkdir -p "$out"
: > "$out/FAILED.tsv"
while IFS=$'\t' read -r id repo pre post path note; do
  case "$id" in ''|'#'*) continue;; esac
  dir="$out/$id"
  if [ -e "$dir/MANIFEST.sha256" ]; then echo "$id: already pinned"; continue; fi
  rm -rf "$dir"; mkdir -p "$dir/pre" "$dir/post" "$dir/.git-tmp"
  ok=1
  if git -C "$dir/.git-tmp" init -q 2>/dev/null &&
     git -C "$dir/.git-tmp" remote add origin "$repo" &&
     git -C "$dir/.git-tmp" fetch -q --depth 1 origin "$pre" &&
     git -C "$dir/.git-tmp" fetch -q --depth 1 origin "$post"; then
    git -C "$dir/.git-tmp" show "$pre:$path"  > "$dir/pre/$(basename "$path")"  2>/dev/null || ok=0
    git -C "$dir/.git-tmp" show "$post:$path" > "$dir/post/$(basename "$path")" 2>/dev/null || true
  else
    ok=0
  fi
  if [ "$ok" -eq 0 ]; then
    printf '%s\t%s\tfetch or pre-extraction failed\n' "$id" "$repo" >> "$out/FAILED.tsv"
    rm -rf "$dir"
    echo "$id: FAILED"
    continue
  fi
  rm -rf "$dir/.git-tmp"
  printf 'id\t%s\nrepo\t%s\npre\t%s\npost\t%s\ncontract\t%s\nconsumer\t%s\n' \
    "$id" "$repo" "$pre" "$post" "$path" "$note" > "$dir/case.tsv"
  (cd "$dir" && find . -type f ! -name MANIFEST.sha256 -exec shasum -a 256 {} + | sort -k2) > "$dir/MANIFEST.sha256"
  echo "$id: pinned ($(wc -l < "$dir/MANIFEST.sha256" | tr -d ' ') files)"
done < "$tsv"
echo "done: $(ls "$out" | grep -vc FAILED) pinned, $(wc -l < "$out/FAILED.tsv" | tr -d ' ') failed (see FAILED.tsv)"
