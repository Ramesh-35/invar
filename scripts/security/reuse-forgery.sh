#!/usr/bin/env bash
# SECURITY S0 probe: can a forged reuse record turn a real VIOLATED into
# HOLDS? Builds a contract whose consumer reads a field the producer drops,
# then forges (1) the local verdict cache and (2) an unsigned invar.lock by
# rewriting the recorded input hashes to the broken bytes. Prints the exit
# code of a plain local `check` and of `check --ci` after each forgery.
# Before S1: local 0 (forged HOLDS), --ci 1. After S1: every line exit 1.
# The cache forger knows the record format but not the per-user key.
# Usage: bash scripts/security/reuse-forgery.sh <path to invar binary>
set -u
B="$1"; W="$(mktemp -d)"; trap 'rm -rf "$W"' EXIT; cd "$W"
printf 'properties:\n  id: {type: string}\n  legacy: {type: string}\n' > base.yaml
cp base.yaml head.yaml
printf 'def f(d):\n    return d["legacy"]\n' > client.py
printf '[[invariant]]\nkind = "schema"\nid = "s"\nproducer = "head.yaml"\nproducer_base = "base.yaml"\nconsumer = "client.py"\n' > invar.toml
rehash() { python3 - "$1" <<'PY'
import sys, re, json, hashlib
try:
    import blake3; h = lambda b: blake3.blake3(b).hexdigest()
except ImportError:
    sys.exit("needs the python blake3 module")
p = sys.argv[1]; t = open(p).read()
if p.endswith(".json"):
    d = json.loads(t)
    for r in d["records"].values():
        rec = r.get("held", r)  # format 2 keyed record, or format 1
        rec["inputs"] = [[h(open(f, "rb").read()), f] for _, f in rec["inputs"]]
    open(p, "w").write(json.dumps(d))
else:
    t = re.sub(r'held_input = "[0-9a-f]{64} ([^"]+)"', lambda m: f'held_input = "{h(open(m.group(1), "rb").read())} {m.group(1)}"', t)
    open(p, "w").write(t)
PY
}
"$B" check > /dev/null
printf 'properties:\n  id: {type: string}\n' > head.yaml
"$B" check --no-reuse > /dev/null; echo "true verdict (evaluated): exit $?"
rehash .invar/cache/verdicts.json
"$B" check > /dev/null; echo "forged cache, local check: exit $?"
"$B" check --ci > /dev/null 2>&1; echo "forged cache, check --ci: exit $?"
rm -rf .invar invar.lock; cp base.yaml head.yaml; "$B" lock > /dev/null 2>&1
printf 'properties:\n  id: {type: string}\n' > head.yaml
rehash invar.lock
"$B" check > /dev/null; echo "forged unsigned lock, local check: exit $?"
"$B" check --ci > /dev/null 2>&1; echo "forged unsigned lock, check --ci: exit $?"
