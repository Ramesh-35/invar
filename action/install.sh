#!/usr/bin/env bash
# Resolve the invar binary for this runner: a release asset whose SHA-256
# checksum and SSH signature both verify (trusted key: action/release-signers
# at the action ref you pinned), otherwise a --locked source build from the
# action's own checkout. A signature that fails to verify stops the job; it
# never falls back to anything.
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
  if dl_err=$(gh release download "$tag" --repo "$INVAR_ACTION_REPO" \
       --pattern "$asset" --pattern "$asset.sha256" --dir "$dest" --clobber 2>&1); then
    if command -v sha256sum >/dev/null; then
      (cd "$dest" && sha256sum -c "$asset.sha256")
    else
      (cd "$dest" && shasum -a 256 -c "$asset.sha256")
    fi
    signers="$INVAR_ACTION_PATH/action/release-signers"
    if [ -f "$signers" ]; then
      if ! gh release download "$tag" --repo "$INVAR_ACTION_REPO" \
           --pattern "$asset.sig" --dir "$dest" --clobber >/dev/null 2>&1; then
        echo "::error::invar: release $tag has no signature ($asset.sig); refusing an unsigned binary. Pin version to a signed release (v0.1.0-alpha.7 or later) or leave version empty."
        exit 1
      fi
      if ! ssh-keygen -Y verify -f "$signers" -I invar-release -n invar-release \
           -s "$dest/$asset.sig" < "$dest/$asset" >/dev/null 2>&1; then
        echo "::error::invar: SIGNATURE CHECK FAILED for $asset ($tag). Not running it, not falling back."
        exit 1
      fi
      verified="checksum and signature verified"
    else
      verified="checksum verified (no release-signers in this action ref)"
    fi
    tar -xzf "$dest/$asset" -C "$dest"
    echo "bin=$dest/invar" >> "$GITHUB_OUTPUT"
    echo "invar: using release $tag ($target), $verified"
    exit 0
  fi
  # Say why: a token without read access to a private action repo looks
  # exactly like a missing asset otherwise (found 2026-10-07).
  echo "invar: release asset $asset for $tag not downloaded: ${dl_err##*$'\n'}"
  echo "invar: building from source instead"
fi

cargo build --release --locked -p invar-falsify --manifest-path "$INVAR_ACTION_PATH/Cargo.toml"
echo "bin=$INVAR_ACTION_PATH/target/release/invar" >> "$GITHUB_OUTPUT"
echo "invar: built from source (${INVAR_ACTION_REF:-local checkout})"
