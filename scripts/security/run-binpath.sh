#!/usr/bin/env bash
# SECURITY-SPEC S3 measurement (SEC-D6, path 6): does `invar run` or
# `invar report` execute the program a repository's composite.toml names in
# bin_path? A copy of the corpus manifest and rules is evaluated with a
# composite.toml whose [semgrep] bin_path is a script that leaves a marker.
# Usage: scripts/security/run-binpath.sh <invar binary> [scratch dir]
set -u
invar="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"; here="$(cd "$(dirname "$0")/../.." && pwd)"
root="${2:-$(mktemp -d)}"; mkdir -p "$root"; root="$(cd "$root" && pwd)"
for cmd in run report; do
  d="$root/$cmd"; rm -rf "$d"; mkdir -p "$d"
  cp -R "$here/corpus" "$here/rules" "$here/corpus_manifest.json" "$d/"
  printf '#!/bin/sh\ntouch "%s/EXECUTED"\necho "{}"\n' "$d" > "$d/evil.sh"; chmod +x "$d/evil.sh"
  printf '[semgrep]\nenabled = true\nbin_path = "./evil.sh"\nrules = "./rules/semgrep/falsification_rules.yml"\n' > "$d/composite.toml"
  args=("$cmd" --manifest corpus_manifest.json --composite composite.toml)
  [ "$cmd" = run ] && args+=(--output out)
  (cd "$d" && "$invar" "${args[@]}" >/dev/null 2>&1); code=$?
  if [ -e "$d/EXECUTED" ]; then r=EXECUTED; else r=no; fi
  printf 'invar %s\texit %s\tbin_path from composite.toml\t%s\n' "$cmd" "$code" "$r"
done
