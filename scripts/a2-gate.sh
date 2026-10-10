#!/usr/bin/env bash
# A2 determinism gate, one platform: for every corpus config, emit a receipt
# twice (byte-equal), verify it (exit 0), and record its sha256 for the
# cross-platform comparison.
set -u
invar="$1"; corpus="$2"; out="$3"
pass=0; fail=0; err=0
: > "$out"
for dir in "$corpus"/*/; do
  name=$(basename "$dir")
  [ "$name" = "_template" ] && continue
  a=$(mktemp); b=$(mktemp)
  env -u GITHUB_TOKEN "$invar" check --ci --config "$dir/invar.toml" --receipt "$a" >/dev/null 2>&1
  c1=$?
  env -u GITHUB_TOKEN "$invar" check --ci --config "$dir/invar.toml" --receipt "$b" >/dev/null 2>&1
  c2=$?
  if [ $c1 -eq 3 ] || [ $c2 -eq 3 ]; then
    # one retry for transient fetch failures
    env -u GITHUB_TOKEN "$invar" check --ci --config "$dir/invar.toml" --receipt "$a" >/dev/null 2>&1 || { echo "$name ERROR" >> "$out"; err=$((err+1)); rm -f "$a" "$b"; continue; }
    env -u GITHUB_TOKEN "$invar" check --ci --config "$dir/invar.toml" --receipt "$b" >/dev/null 2>&1
  fi
  if ! cmp -s "$a" "$b"; then echo "$name NONDETERMINISTIC" >> "$out"; fail=$((fail+1)); rm -f "$a" "$b"; continue; fi
  env -u GITHUB_TOKEN "$invar" verify-receipt "$a" --config "$dir/invar.toml" >/dev/null 2>&1
  v=$?
  if [ $v -ne 0 ]; then echo "$name VERIFY_EXIT_$v" >> "$out"; fail=$((fail+1)); rm -f "$a" "$b"; continue; fi
  echo "$name OK $(sha256sum "$a" | cut -d' ' -f1)" >> "$out"
  pass=$((pass+1)); rm -f "$a" "$b"
done
echo "gate: $pass ok, $fail failed, $err error"
