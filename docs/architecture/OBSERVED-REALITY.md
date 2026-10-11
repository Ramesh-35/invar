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
