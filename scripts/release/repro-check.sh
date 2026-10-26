#!/usr/bin/env bash
# SECURITY-SPEC S3-4: the release binary is reproducible. Builds the
# committed tree twice, from two different checkout paths into two clean
# target directories, and compares the binaries and their packed archives
# byte for byte. Exit 0 when identical, 1 when they differ.
# Usage (from the repository root): scripts/release/repro-check.sh [scratch dir]
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
scratch="${1:-$(mktemp -d)}"; mkdir -p "$scratch"; scratch="$(cd "$scratch" && pwd)"
sum() { (sha256sum "$1" 2>/dev/null || shasum -a 256 "$1") | cut -d' ' -f1; }
for n in first second-path; do
  rm -rf "$scratch/$n"; mkdir -p "$scratch/$n/src"
  git -C "$root" archive HEAD | tar -x -C "$scratch/$n/src"
  (cd "$scratch/$n/src" && CARGO_TARGET_DIR="$scratch/$n/target" cargo build --release --locked -p invar-falsify --quiet)
  python3 "$root/scripts/release/pack.py" "$scratch/$n/target/release/invar" "$scratch/$n/invar.tar.gz"
done
a="$(sum "$scratch/first/target/release/invar")"; b="$(sum "$scratch/second-path/target/release/invar")"
ta="$(sum "$scratch/first/invar.tar.gz")"; tb="$(sum "$scratch/second-path/invar.tar.gz")"
printf 'binary   %s %s\narchive  %s %s\n' "$a" "$b" "$ta" "$tb"
if [ "$a" = "$b" ] && [ "$ta" = "$tb" ]; then echo reproducible; exit 0; fi
echo "NOT reproducible"; exit 1
