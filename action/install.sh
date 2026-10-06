#!/usr/bin/env bash
# Resolve the invar binary for this runner: a checksum-verified release asset
# when one matches the requested tag, otherwise a --locked source build from
# the action's own checkout (always available on GitHub-hosted runners).
set -euo pipefail

case "${RUNNER_OS:-}-${RUNNER_ARCH:-}" in
  Linux-X64)   target=x86_64-unknown-linux-musl ;;
  macOS-X64)   target=x86_64-apple-darwin ;;
  macOS-ARM64) target=aarch64-apple-darwin ;;
  *)           target="" ;;
esac

tag="${INVAR_VERSION:-}"
if [ -z "$tag" ] && [[ "${INVAR_ACTION_REF:-}" == v* ]]; then tag="$INVAR_ACTION_REF"; fi

dest="${RUNNER_TEMP:-/tmp}/invar-bin"
mkdir -p "$dest"

if [ -n "$tag" ] && [ -n "$target" ] && [ -n "${INVAR_ACTION_REPO:-}" ] && command -v gh >/dev/null; then
  asset="invar-${tag}-${target}.tar.gz"
  if gh release download "$tag" --repo "$INVAR_ACTION_REPO" \
       --pattern "$asset" --pattern "$asset.sha256" --dir "$dest" --clobber 2>/dev/null; then
    if command -v sha256sum >/dev/null; then
      (cd "$dest" && sha256sum -c "$asset.sha256")
    else
      (cd "$dest" && shasum -a 256 -c "$asset.sha256")
    fi
    tar -xzf "$dest/$asset" -C "$dest"
    echo "bin=$dest/invar" >> "$GITHUB_OUTPUT"
    echo "invar: using release $tag ($target), checksum verified"
    exit 0
  fi
  echo "invar: no release asset $asset for $tag; building from source"
fi

cargo build --release --locked -p invar-falsify --manifest-path "$INVAR_ACTION_PATH/Cargo.toml"
echo "bin=$INVAR_ACTION_PATH/target/release/invar" >> "$GITHUB_OUTPUT"
echo "invar: built from source (${INVAR_ACTION_REF:-local checkout})"
