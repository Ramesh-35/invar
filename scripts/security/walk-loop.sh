#!/usr/bin/env bash
# SECURITY S0 probe, path 5: the dependents walk (CLI `invar dependents
# --locks` and the MCP `dependents` tool, which accepts any absolute path)
# follows symbolic links. A directory holding a link to its own parent is
# walked until the stack or the clock runs out; three links per level make
# the walk branch (the kernel bounds each path at its symlink limit, not the
# number of paths). Reports the exit code and
# wall time under a 20 s limit.
# Usage: bash scripts/security/walk-loop.sh <invar binary>
set -u
B="$1"; W="$(mktemp -d)"; trap 'rm -rf "$W"' EXIT
mkdir -p "$W/locks/a"; ln -s .. "$W/locks/a/up"; ln -s .. "$W/locks/a/up2"; ln -s . "$W/locks/a/self"
start=$(date +%s.%N)
timeout 20 "$B" dependents Order.total --locks "$W/locks" > "$W/out" 2>&1; code=$?
end=$(date +%s.%N)
printf "dependents over a self-referencing link: exit %s after %.1f s\n" "$code" "$(echo "$end - $start" | bc)"
tail -c 300 "$W/out"
