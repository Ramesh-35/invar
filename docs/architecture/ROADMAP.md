# INVAR Roadmap: four additions (ratified 2026-10-07)

Each addition is judged by M1: does it help INVAR out-detect a fairly
strengthened rival on real incidents? None of them changes the adequacy
matrix by itself (D PARTIAL, overall NOT ESTABLISHED). Status vocabulary as
in [INIMITABILITY.md](INIMITABILITY.md): HOLDS, PARTIAL, TARGET.

## Ratified corrections to how the primitives are described

1. `invar init` works by pattern matching, not by parsing an AST. Only JSON
   Schema and OpenAPI-JSON components become proposed `[[invariant]]` blocks;
   YAML specs, AsyncAPI, protobuf, framework routes and Pydantic models are
   listed as inventory comments for a human to review.
2. The static Linux binary has no runtime library dependencies and `--locked`
   holds. `invar check --ci` runs `curl` as an argv subprocess (never a shell)
   for HTTPS fetches.
3. "Verification in under 10 seconds" is a TARGET until it is benchmarked and
   recorded.

## Sequence

| # | Addition | Status |
|---|---|---|
| 1 | Witness locations | HOLDS in code, unreleased (test 24; not yet run on a runner) |
| 2 | Contract lockfile `invar.lock` | TARGET |
| 3 | Blast radius on the producer's PR | TARGET |
| 4 | Expiring waivers | TARGET |

### 1. Witness locations
Every violation names where it comes from:
`amount_cents` read at `client.js:6` · not in `schema.json /properties`.
Schema and event contracts list each consumer line that reads the missing
field (first three, then "+n more") and the producer artifact (with its JSON
pointer) that lacks it. Lock-order violations list the trace lines of the two
steps. Witnesses explain residue; they never change it (REAL-INCIDENT-001's
residue and probe hash are asserted unchanged in test 24). All locations are
escaped inline code in the PR report.

### 2. Contract lockfile (`invar.lock`)
Pin every artifact a contract depends on by BLAKE3, recording when INVAR first
saw each hash (`t_r`) and, where known, the commit time of the artifact
(`t_v`). Detects drift before it breaks a field, and with a small receipt
(input hashes, binary version, verdict) lets `invar verify` replay any past
check. This closes the gap "`invar check` is not bitemporal"
(INIMITABILITY.md I). It does not by itself make INVAR a causality engine:
it records what was true and when it was known, not why.

### 3. Blast radius on the producer's PR
A producer's `invar.toml` lists its consumers as `github:` references (already
supported by the resolver). On the producer's PR, report which consumers read
a field the change removes, before merge. The cross-repository moat claim
stays out of user-facing text until it beats a rival on real incidents (M1).

### 4. Expiring waivers
```toml
[[waiver]]
residue = "amount_cents"
reason  = "migration to micro-units, ticket ORD-142"
expires = "2026-11-30"
```
A waived residue stays visible as WAIVED until its date and fails (exit 1)
after it. An expired or malformed waiver never silences a violation.

## Exclusions
- No AI-inferred contracts: non-deterministic output cannot be replayed.
- No SaaS or dashboard layer: value stays in the binary, CLI and PR action.
- No new contract kinds until schema, event and lock-order have real-incident
  evidence.
