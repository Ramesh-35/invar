# Multi-sample presence witnesses (spec, frozen 2026-10-11)

Supersedes the single-sample comparison semantics of OBSERVED-REALITY.md
(the shape derivation, PII warnings, value-never-copied rule, and staleness
ruling all stand unchanged). Motivation, from the adjudicated research
(B3/B5): a single sample cannot distinguish an optional field from a
removed one, so yesterday's shipped comparison would flag an optional field
that happened to be present at lock time. With zero external users, the fix
ships before the flaw meets a user.

## The witness model

Each locked sample contract accumulates, per key path:

- `present`: observations where the path appeared (any type, null included)
- `absent`: observations where the path's parent appeared but the path did
  not (for top-level paths: the root object lacked the key)
- `null`: observations where the path appeared as null
- `types`: the set of observed JSON types, sorted, pipe-joined
- plus per contract: `observations` (N), and the observation window as
  first and last `taken` dates

`invar lock --sample <file>` (repeatable in one invocation, and cumulative
across invocations) merges each payload into the counters. Values are never
copied; only the most recent sample file's BLAKE3 is retained (the
individual sample bytes remain the consumer's own archive, stated in the
output). Deviation from the ratifying dispatch, disclosed: the window is
recorded as dates (YYYY-MM-DD), consistent with every other valid-time
field in the lock, not RFC 3339 timestamps.

## Verdict semantics (the founder's threshold, frozen)

For a fresh payload (`invar check --sample <file>`), per locked path:

- VIOLATED (residue, waivers apply) if and only if the path is absent from
  the fresh payload AND `absent == 0` AND `observations >= 3`
  (always present in at least three observations, now gone), or the fresh
  type is outside `types` under the same two conditions.
- `[not-evaluated:optional-field-absent]` when the path is absent but
  `absent > 0`: the field is known optional; its absence is stated, never
  a violation and never a silent pass.
- `[not-evaluated:witness-count-low]` when the path is absent or retyped
  but `observations < 3`: too few witnesses to accuse; stated.
- Additions in the fresh payload are never drift.

Staleness is unchanged (ratified): a window whose last date is older than
90 days is `[not-evaluated:stale-sample]`, exit 4.

## Lock format

Witness counters make a lock format "3": format 2's `[[sample]]` with
`observations`, `window = "FIRST LAST"`, and `field = "path types
present/absent/null"` lines. Rules:

- The engine reads formats 1, 2 and 3, forever. A format 2 lock (alpha.16)
  is read as witnesses with N=1 (present 1/1), which is exactly what one
  sample is.
- The engine writes the lowest format that can express the content: no
  samples, format 1; any samples, format 3. (Format 2 becomes read-only
  legacy after alpha.17.)
- Why a bump instead of extending format 2 in place, stated: an alpha.16
  engine reading counter-bearing sample lines it does not recognize would
  see an empty locked shape and silently HOLD; a format it refuses fails
  closed instead. Fail closed beats fail silent, every time.

## Measure (unchanged gate, still owed)

The observed-reality effectiveness gate stands: reconstruct the five
recurring-miss payloads (r1-059, r1-075, r1-099, r1-100, r1-113) from
public evidence, lock base-era samples, check head-era samples, record per
case which verdict fires; payloads that cannot be honestly reconstructed
are recorded as such. No effectiveness claim before that table exists.

## Non-goals

No value storage, no sample synthesis, no statistics beyond the counters
above (no rates, no confidence intervals: counts are stated and the reader
judges), no automatic sampling, no network.
