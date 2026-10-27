#!/usr/bin/env bash
# SIB-1 premises, measured on the pre-P5 engine. Usage: premises.sh <invar>
set -u
bin="$1"; w="$(mktemp -d)"; export HOME="$w"
mkdir -p "$w/c/lib" "$w/c/src" "$w/locks/consumer-a" "$w/r/lib" "$w/r/src"
printf 'export interface Limits {\n  maxBytes: number;\n  maxLines: number;\n}\n' > "$w/c/lib/limits.ts"
printf 'import type { Limits } from "../lib/limits";\nexport function f(l: Limits) {\n  return [l.maxBytes, l.maxLines];\n}\n' > "$w/c/src/use.ts"
printf '[[invariant]]\nkind = "schema"\nid = "lk"\nproducer = "lib/limits.ts"\nconsumer = "src/use.ts"\n' > "$w/c/invar.toml"
(cd "$w/c" && "$bin" lock >/dev/null)
sed 's#producer = "lib/limits.ts"#producer = "github:acme/api@0123456789abcdef0123456789abcdef01234567:lib/limits.ts"#' "$w/c/invar.lock" > "$w/locks/consumer-a/invar.lock"
echo "P1 a lock rewritten to a remote producer parses and is found by the existing query mode:"
"$bin" dependents Limits.maxLines --locks "$w/locks"; echo "exit $?"
echo "P2 the TS extractor types a retyped field as string (the same extractor P5 reuses):"
cp "$w/c/src/use.ts" "$w/r/src/"; cp "$w/c/invar.toml" "$w/r/"
printf 'export interface Limits {\n  maxBytes: number;\n  maxLines: string;\n}\n' > "$w/r/lib/limits.ts"
(cd "$w/r" && "$bin" lock >/dev/null && grep 'Limits.maxLines' invar.lock)
echo "P3 one signers file names the signer of a lock (find-principals), and verify accepts it:"
ssh-keygen -t ed25519 -N "" -q -f "$w/ka"; ssh-keygen -t ed25519 -N "" -q -f "$w/kb"
printf 'consumer-a@acme %s\nconsumer-b@acme %s\n' "$(cat "$w/ka.pub")" "$(cat "$w/kb.pub")" > "$w/signers"
ssh-keygen -Y sign -n invar-lock -f "$w/ka" "$w/locks/consumer-a/invar.lock" 2>/dev/null
p="$(ssh-keygen -Y find-principals -s "$w/locks/consumer-a/invar.lock.sig" -f "$w/signers")"; echo "principal: $p"
ssh-keygen -Y verify -f "$w/signers" -I "$p" -n invar-lock -s "$w/locks/consumer-a/invar.lock.sig" < "$w/locks/consumer-a/invar.lock" >/dev/null 2>&1; echo "verify exit $?"
echo "P4 an untrusted signer is named by no principal:"
ssh-keygen -t ed25519 -N "" -q -f "$w/kx"
cp "$w/locks/consumer-a/invar.lock" "$w/x.lock"; ssh-keygen -Y sign -n invar-lock -f "$w/kx" "$w/x.lock" 2>/dev/null
ssh-keygen -Y find-principals -s "$w/x.lock.sig" -f "$w/signers" >/dev/null 2>&1; echo "find-principals exit $?"
echo "P5 the current CLI has no evaluation mode (query required):"
"$bin" dependents --locks "$w/locks" --repo acme/api 2>&1 | head -2; echo "exit ${PIPESTATUS[0]}"
rm -rf "$w"
