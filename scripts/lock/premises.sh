#!/usr/bin/env bash
# LOCK-1 premises on the pre-P4 engine, through the CLI's existing
# --lock-signers path (the MCP flags do not exist yet). Usage: premises.sh <invar>
set -u
B="$1"; root=$(mktemp -d)
mk() { d="$root/$1"; mkdir -p "$d/lib" "$d/src"
  printf 'export interface Limits {\n  maxBytes: number;\n  maxLines: number;\n}\n' > "$d/lib/limits.ts"
  printf 'import type { Limits } from "../lib/limits";\nexport function f(l: Limits) {\n  return [l.maxBytes, l.maxLines];\n}\n' > "$d/src/use.ts"
  printf '[[invariant]]\nkind = "schema"\nid = "lk"\nproducer = "lib/limits.ts"\nconsumer = "src/use.ts"\n' > "$d/invar.toml"
  (cd "$d" && HOME="$d" "$B" lock >/dev/null)
  ssh-keygen -t ed25519 -N "" -q -f "$d/k"; ssh-keygen -t ed25519 -N "" -q -f "$d/other"
  printf 'release@invar %s' "$(cat "$d/k.pub")" > "$d/signers"; echo "$d"; }
run() { (cd "$1" && HOME="$1" "$B" check --no-reuse --lock-signers signers --lock-principal release@invar 2>&1 | head -1; echo "exit ${PIPESTATUS[0]}") | paste -sd' '; }
d=$(mk f04); ssh-keygen -Y sign -n invar-lock -f "$d/k" "$d/invar.lock" 2>/dev/null; echo "f04 trusted signature: $(run "$d")"
d=$(mk f05); echo "f05 no signature: $(run "$d")"
d=$(mk f06); ssh-keygen -Y sign -n invar-lock -f "$d/other" "$d/invar.lock" 2>/dev/null; echo "f06 untrusted key: $(run "$d")"
d=$(mk f07); ssh-keygen -Y sign -n invar-lock -f "$d/k" "$d/invar.lock" 2>/dev/null; grep -c "maxLines number" "$d/invar.lock" | sed 's/^/f07 lines to edit: /'; sed -i.bak 's/maxLines number/maxLines string/' "$d/invar.lock"; echo "f07 edited after signing: $(run "$d")"
d=$(mk f08); ssh-keygen -Y sign -n invar-lock -f "$d/k" "$d/invar.lock" 2>/dev/null; printf '\n' >> "$d/invar.lock"; echo "f08 newline appended: $(run "$d")"
