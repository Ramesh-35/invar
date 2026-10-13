# invar backtest dogfood: fastapi/fastapi, 2026-10-12

Directive: founder-approved container dogfood before landing alpha.27.
Environment: cloud container, release build of alpha.27 (48/48 tests).
All numbers below were measured on this machine this day; they are not
promises about other machines.

## Setup, fully disclosed

- Clone: scratchpad audit10/fastapi (from the 2026-10-12 init audit),
  shallow; deepened via `git fetch --depth=300`, then 3000, then 6000
  (6160 commits reachable). Shallow boundaries hide file history: at
  depth 300 only 1 commit touched the producer file, at 6000, 21 did.
  A backtest on a shallow clone honestly replays only what the clone
  holds; the boundary commit shows as first-present.
- Contract: ENGINEER-AUTHORED, not an init proposal (init proposes zero
  contracts inside a single repository, as designed and as the audit
  recorded). Producer fastapi/openapi/models.py, consumer
  fastapi/openapi/utils.py, kind schema. `invar lock` bound 84 fields.

## Measurements

- `invar backtest` (default depth 50; 9 commits touch the file in that
  window): 72 ms and 83 ms wall across two runs; outputs byte-identical
  (cmp); exit 0.
- `invar backtest --depth 300` (21 commits, the file's full history
  2021-07-03 to 2026): 144 ms wall; exit 0.
- Rough per-replayed-commit cost at depth 300: ~7 ms (git show + a
  ~1100-line Python extraction + 84 presence checks).

## Findings

- Zero absence windows across 84 bindings and 21 commits: this
  producer's surface grew monotonically; no locked field was ever
  removed and restored in the replayed history.
- The introduction dates are real, independently checkable archaeology:
  OpenAPI.webhooks and Info.summary first present at 7dad5a820bfe
  (2023-06-30, the OpenAPI 3.1 migration); Schema.default,
  Schema.deprecated, Schema.description, Schema.example and peers first
  present at 97fa743ecb5a (2021-07-29); Schema.examples at 7dad5a820bfe.
- Semantics held on real data: absence before a field's first
  appearance was reported as a leading `absent` event and correctly NOT
  counted as an absence window.

## What this dogfood does not show

A drop-and-restore outage in the wild (this file has none in its
replayed history), behavior on renamed files (renames are not
followed), or timings on user hardware. The synthetic drop-restore case
is covered by test 48; a wild outage timeline will come from whichever
real repository first shows one.
