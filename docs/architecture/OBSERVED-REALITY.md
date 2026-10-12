# Observed-reality locking: `invar lock --sample` (spec, frozen 2026-10-11)

Ratified direction; implementation is the next engine increment. RATIFY
markers are the open points.

## Problem, stated from the measured record

Every detection rule to date inferred the producer's OUTPUT from the
producer's TEXT, and the misses that recur (46% held out) are exactly the
cases where the text still contains the old field while the live output
changed: default flips, version switches, envelope moves, legacy code paths.
The inference axis is exhausted (rule expansion frozen 2026-10-10). The
inversion: let the consumer lock the shape of a payload they actually
observed. Reality does not keep legacy code paths.

## What it does

`invar lock --sample <file.json> --contract <id>` reads ONE real payload the
consumer captured (a saved API response, a fixture from a recorded call) and
records, in invar.lock under that contract:

- `[[contract.sample]]`: the sample's BLAKE3, the date, and the derived
  field surface: every key path with its JSON type (the existing
  spec_fields/key-path machinery; nothing new is parsed).
- Field types are locked at the shape level (name, type, nullability,
  array-element shape), NOT byte level: payload VALUES change on every call
  and must not drift the lock. Determinism is over the derived shape.

`invar check` then gains one comparison per sampled contract: derive the
shape of the producer's CURRENT declared output as today, AND, when the
consumer supplies a fresh sample (`invar check --sample <file.json>`),
compare the fresh sample's shape against the locked sample's shape. A field
present in the locked shape and absent or retyped in the fresh sample is
residue with `observed at sample(<date>)` provenance, subject to the same
waivers. No fresh sample, no new claim: the locked sample alone never
upgrades a static verdict, it only records what was accepted.

## What it refuses to do (the honesty lines)

- A sample is evidence about ONE response at ONE moment. The lock records
  its date and hash; the report says "observed", never "guaranteed".
  Optional fields absent from the sample are simply not in the locked
  shape, and the docs say that plainly.
- No network capture, no proxy, no recording mode: the consumer brings the
  file. INVAR stays a checker.
- PII: samples are hashed into the lock but their CONTENT is never copied
  into lock, receipt, or report; only key paths and types. The sample file
  itself stays wherever the user keeps it. RATIFY: with --sample, warn
  once that sample files may contain real data and belong in .gitignore
  unless scrubbed.
- Staleness: a locked sample older than the waiver cap (90 days) is
  reported as stale context in the report (not an error). RATIFY.

## Measure (before any claim)

Tuning check on the existing corpus: for the recurring miss cases
(r1-059, r1-075, r1-099, r1-100, r1-113), construct the real base and head
payloads from the public evidence where recoverable, and measure whether
sample-vs-sample comparison flags them. Report per case; cases where the
payload is not recoverable are stated as such. Held-out discipline applies
to any claim beyond those five: next real batch, fresh samples.

## Non-goals

No JSON Schema generation, no OpenAPI emission, no sample synthesis, no
multi-sample statistics, no new dependencies.

## Effectiveness gate, measured 2026-10-11 (engine v0.1.0-alpha.18)

Protocol as frozen: real payloads only, pinned sources, three base
observations where honestly obtainable, one head payload, verdicts
recorded as they fired. Payload files are not committed (third-party
content, one pair is 1.7 MB); every source is pinned below.

| Case | Base payloads (3 observations) | Head payload | Verdict | Honest reading |
|---|---|---|---|---|
| r1-059 deno.lock v3->v4 | denoland/fresh deno.lock @ 97e2b54b, becd440d, a79e7972 (v3, real) | @ de8f1b0c (v4, real) | VIOLATED, exit 1 | CAUGHT: `packages` (and `packages.specifiers` beneath it) witnessed 3/3, gone at head. Buried in 1,232 residue items of version-key churn (see finding). |
| r1-075 npm lockfile v3 drops `dependencies` | axios package-lock.json @ 4e9b33d6, b2cb45d5, 39ec2064 (v2, real) | @ 29f75425 (v3, real) | VIOLATED, exit 1 | CAUGHT: `dependencies` and its whole tree witnessed 3/3, gone at head. Buried in 23,240 residue items. |
| r1-099 Suricata EVE DNS v2->v3 | the three documented v2 records in the producer's own docs @ 9ecc3573 (query, detailed, grouped) | documented v3 detailed record @ master (post-pin; the head-pin predates its own docs) | NOT FULLY EVALUATED, exit 4 | INCONCLUSIVE: the doc examples are abridged and heterogeneous, so `rrname` showed as optional (1 of 3) and absence was stated, not accused. Stated, not silent; but no catch claim. Full production records would be needed for a verdict. |
| r1-100 Kibana status state->level | no real response fixture found at the pinned commits (route test paths 404; docs examples absent) | - | NOT RUN | Recorded as not honestly reconstructable from public fixtures, per the gate's own rule. |
| r1-113 Cargo.lock v3->v4 | - | - | NOT RUN | The payload is TOML; sample ingestion is JSON-only by frozen spec. Out of mechanism scope, stated. |

Static-engine contrast: all five cases are silent HOLDS on every engine
from alpha.8 through alpha.18 (five held-out batches of record).

### The finding that gates production use

The mechanism catches the recurring default-switch class (2 clear catches
where five generations of text rules saw nothing), BUT: lockfile-style
payloads are MAPS whose keys are data (`packages.jsr.@std/assert@0.213.1`),
not schema. Path-level witnesses treat every version-suffixed key as a
field, so an ordinary dependency bump produces thousands of false residue
items; the real signal was present but buried (1,232 and 23,240 items).
By our own E1 standard (zero false alarms on clean changes), this
disqualifies the mechanism for production claims on map-keyed payloads.
Verdict: NO effectiveness claim yet. Next spec (W2, to be frozen before
any further measurement): map-key abstraction, collapsing high-cardinality
key sets into wildcard paths so `packages.jsr.*` is one witness, not
hundreds. The two catches here are recorded as mechanism validation on
reconstructed payloads, tuning-rung evidence, nothing more.

### Gate re-run under W2 map-key abstraction, measured 2026-10-11 (engine v0.1.0-alpha.19)

Same pinned payloads, same protocol, engine now carrying W2 (map
detection, wildcard paths, ancestor dominance) and the disclosed W2.1
tree-level collapse (docs/architecture/W2-MAP-KEY-ABSTRACTION.md,
amendment section). Sequence stated plainly: frozen W2 was measured
first and FAILED the npm case (230 findings against the at-most-20
bar); W2.1 was then specified, implemented, and re-measured. Both
measurements are on the record in the spec.

| Case | Verdict | Findings | Was (alpha.18) | Gate bar | Result |
|---|---|---|---|---|---|
| r1-059 deno.lock v3->v4 | VIOLATED, exit 1 | 1 (`packages`, witnessed 3/3, gone) | 1,232 | VIOLATED and <=20 | PASS |
| r1-075 npm v2->v3 | VIOLATED, exit 1 | 2 (`dependencies` witnessed 3/3 gone; plus one dot-key artifact path, disclosed in spec) | 23,240 | VIOLATED and <=20 | PASS |
| r1-099 Suricata DNS | NOT FULLY EVALUATED, exit 4 | 3 stated optional-field lines | same | unchanged, stated not silent | PASS (regression) |

The ratified metric (both catches remain VIOLATED, residue reduced from
thousands to near-unit counts) is met: 1,232 -> 1 and 23,240 -> 2. The
disqualifying finding of the alpha.18 measurement is closed. What this
does and does not license, unchanged: two catches on reconstructed
real payloads are mechanism validation, not an effectiveness claim;
r1-099 remains inconclusive, r1-100 not reconstructable, r1-113 out of
mechanism scope. The held-out record stands at 38 of 70.

## TOML payload ingestion (addendum, 2026-10-12, v0.1.0-alpha.23)

`invar lock --sample` and `invar check --sample` now ingest `.toml`
payloads through a stated TOML SUBSET parsed by the engine itself
(zero new dependencies): tables, arrays of tables, string/integer/
boolean scalars, and string arrays including multiline. Anything
outside the subset is an ERROR naming the line, never a silent
partial parse. The extension decides the format; shapes derive through
the same pipeline as JSON (W2/W2.1 apply unchanged).

Measured on landing, gate protocol, real pinned payloads
(BurntSushi/ripgrep Cargo.lock: bases 3b7fd442, 6dfaec03, 53279db4,
all version 3; head 81bed786, version 4): parse clean, 9 paths
witnessed across 3 observations, check result HOLDS, exit 0. Stated
plainly: the v3->v4 transition is shape-identical and its selector is
a value, so shape witnesses do not see it. The r1-113 class thereby
moves from out-of-mechanism-scope to measured-in-scope-and-missed.
The proposed remedy is W3 (docs/architecture/W3-VALUE-WITNESS.md,
DRAFT, awaiting ratification; not built).
