# INVAR Roadmap (LOCKED 2026-10-08)

Locked by founder directive "INVAR re-grounding & research mandates"
(2026-10-08), adopting the research report "INVAR: What We Are Missing"
(October 2026) as binding design constraints. Changing the order or the
"Do not build" list needs explicit founder authorization.

Adequacy is unchanged by this document: D PARTIAL, overall NOT ESTABLISHED.

## Source status

The research report is an external input. Its citations (incident dates,
acquisitions, vendor figures) were not independently re-verified in this
repository; facts drawn from it are labelled SOURCED (external report) until
checked against the primary source. The two quantified anchors it relies on:

- Azure (Liu et al., HotOS '19): 21% of 112 high-severity software-bug
  incidents were data-format inconsistencies; about 60% of those involved
  message interfaces; 83% were producer/consumer mismatches. One company, six
  months; the authors warn it may not generalize.
- Upgrade failures (Zhang et al., SOSP '21): about two thirds of 123 upgrade
  failures came from incompatible interaction between versions.

## Claim-labelling rules (binding)

- AI-defect figures (e.g. 1.7x, "up to 2.74x"): label
  `SOURCED (vendor study, not about contracts)`. They do not support INVAR's
  thesis.
- Any outage or incident-share figure not backed by the two studies above:
  `UNVERIFIED`.
- The CUDA thesis is RETIRED (docs/architecture/strategy/CUDA-THESIS.md). Do
  not cite it. Defensibility rests on trust, supply-chain integrity,
  deterministic replay and backward-compatible lockfile and result formats.

## How the primitives are described (ratified 2026-10-07)

1. `invar init` works by pattern matching, not by parsing an AST. Only JSON
   Schema and OpenAPI-JSON components become proposed `[[invariant]]` blocks;
   YAML specs, AsyncAPI, protobuf, framework routes and Pydantic models are
   listed as inventory comments for review.
2. The static Linux binary has no runtime library dependencies and `--locked`
   holds. `invar check --ci` runs `curl` as an argv subprocess for HTTPS.
3. "Verification in under 10 seconds" is a TARGET until benchmarked (R6).

## Already done

- Witness locations: every violation names the reading line and the producer
  artifact lacking the field (commit `0bcc0e0`, test 24). Ships in
  `v0.1.0-alpha.2`.

## Execution order

| # | Phase | Status |
|---|---|---|
| R1 | Real-tier falsification corpus, head-to-head vs baselines | IN PROGRESS: 15 breaks + 3 controls of 30; held-out batch 2: INVAR 4/10 (docs/falsification/R1-CORPUS.md) |
| R2 | Consumer-only contract lockfile `invar.lock` | TARGET |
| R3 | Producer-side blast radius from consumers' lockfiles | TARGET |
| R4 | Release and Action supply-chain hardening | TARGET |
| R5 | Expand/contract states and expiring waivers | TARGET |
| R6 | Telemetry-free time-to-first-value benchmark | TARGET |
| R7 | SARIF output and a frozen, versioned result schema | TARGET |

### R1. Real-tier falsification corpus (critical path)
At least 30 real historical cross-repository breaks, each pinned (producer
commit, consumer commit, expected residue, public source). Run INVAR and the
composite baseline on every case (oasdiff or openapi-diff, `buf breaking` at
the right category, schema-registry compatibility checks, Semgrep and property
tests where they apply, one AI reviewer as a non-deterministic reference).
Publish every result, including misses.
Falsified if, on 30 or more real cases, the baseline catches 90% or more of
what INVAR catches, or INVAR misses more than 20% of true breaks.

### R2. Contract lockfile `invar.lock` (consumer-only)
Committed in the consumer repository. Each entry: producer source and ref,
BLAKE3 of the fetched surface, and the exact slice the consumer depends on.
Sorted, deterministic, zero producer setup. Written promise: every future
INVAR reads every past lockfile. Closes "`invar check` is not bitemporal"
(INIMITABILITY I): it records what was true and when it was known; it does
not explain causes.
Falsified if, across 10 or more real consumer repos, more than half cannot
locate a machine-readable producer surface without producer changes, or more
than 30% of lockfile changes are noise unrelated to the consumer's slice.

### R3. Producer-side blast radius
On a producer PR, read consumers' `invar.lock` files the token can already
read (or a producer-side consumer list) and report which consumers depend on
what the change removes. Reads lockfiles only, never consumer source;
read-only and opt-in.
Falsified if, over 50 or more flagged producer PRs, producers change course or
add a waiver in fewer than 20% of cases, or enumeration needs token scopes
organizations will not grant.

### R4. Supply-chain hardening
Immutable releases; build-provenance attestations
(`actions/attest-build-provenance`); the Action carries the expected binary
digest in its own source and verifies digest and attestation before running;
digests published in a second place; full-commit-SHA pinning is the only
documented way to use the Action; least-privilege workflow (no secrets, no
`pull_request_target`, no cache restore, no nested third-party actions);
parser hardening and fuzzing; a separate download-token input (finding from
the PR test). Advertise SLSA Build Level 2 at most.
Constraint to resolve first: per the report, attestations for private
repositories need GitHub Enterprise Cloud, and `invar-platform` is private.
Falsified if a red-team asset swap or tag move still gets the substituted
binary executed.

### R5. Expand/contract and expiring waivers
Typed residue states: `provided`, `deprecated-until <date>`, `removed`.
Waivers carry owner, reason and expiry, live in the consumer lockfile, are
recorded in the hash-chained ledger, and fail (exit 1) after expiry. A small,
closed set of compatibility levels (wire, JSON, source).
Falsified if more than 30% of waivers are renewed past expiry with the residue
unchanged.

### R6. Time-to-first-value benchmark
Public script over 20 or more public producer/consumer pairs, no telemetry.
TARGET: under 5 minutes overall, under 60 seconds where a spec already exists,
`invar init` usable without edits on 70% of pairs.
Falsified if the median exceeds 15 minutes after two improvement rounds.

### R7. SARIF and a frozen result schema
SARIF alongside the PR comment; a versioned JSON result schema with a written
backward-compatibility promise; the PR comment becomes a rendering of that
JSON.
Falsified if no third party consumes the JSON or SARIF within 12 months.

## Do not build (LOCKED)
1. A public append-only log or custom PKI. Use Sigstore's existing log for
   INVAR's own releases (R4); do not operate one.
2. AI-inferred contracts. They break determinism and replay.
3. Hosted dashboards or SaaS control planes. Local files are the defence
   against the category's shutdown failure mode.
4. Pact-style provider-state replay.
5. Runtime traffic monitoring.
Also not now: more configuration options (ship a closed set, R5); an
enterprise governance tier before R1 passes; new contract kinds before real
evidence for the existing three.
