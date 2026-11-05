# JUDGE-D9 r0: a type its enum already implies is not a break (r0, FROZEN)

Ruling D9 ratified by the founder 2026-11-03. Frozen with
crates/invar-falsify/tests/judge_d9_test.rs and four real Iglu Central
fixtures BEFORE implementation; freeze record: JUDGE-D9-SPEC.frozen.
Amends JUDGE-SUBTYPE r2 Amendment B (D5), which stays frozen as written.

## Why

The first Iglu Central sweep (snowplow/iglu-central at 7771e92, 146
consecutive version pairs) flagged 9 ADDITION bumps. In 4 of them every
finding, and in one more one of its two findings, has the same shape: a
field with an `enum` of strings and no `type` gains `type: string` (or
`string|null`). Checked with Python jsonschema, every one of the 84 old
enum values still validates against the new field schema: nothing valid
before is rejected. The counterexample engine built nothing for these
claims, because no payload exists. The rule, not the engine, was wrong.

## Ruling D9

A field with no `type` at base but an `enum` (or `const`) accepts only
those values. When the post adds a `type` (and the node is not under D1):

- If every enum (or const) value is an instance of a post type
  (`integer` values are instances of `number`; a whole number written
  1.0 is an integer), no witness; a stated note:
  `type made explicit: <path> is now <type>, which every value of its enum already is; every value valid before is still valid.`
- Otherwise VIOLATED `retyped`, as before. The base enum (or const)
  values are tried first as counterexample candidates, in order, so the
  excluded value is the instance shown.

D9 judges only what the added `type` does. A change to the enum itself
is judged by the existing enum rule (r1), in the same run.

## Fixtures (judge_d9_test.rs)

- d9_f01 enum of strings gains `type: string`: HOLDS, the note exactly
  (kills D9-M1).
- d9_f02 enum of strings gains `type: ["string","null"]`: HOLDS.
- d9_f03 `const` "x" gains `type: string`: HOLDS.
- d9_f04 enum ["1", 2] gains `type: string`: VIOLATED retyped,
  counterexample `{"f":2}` (kills D9-M1, D9-M3).
- d9_f05 enum [1.5] gains `type: integer`: VIOLATED, counterexample
  `{"f":1.5}`.
- d9_f06 enum [1, 2] gains `type: number`: HOLDS (kills D9-M2).
- d9_f07 real bytes, campaign_attribution 1-0-0 to 1-0-1: HOLDS.
- d9_f08 real bytes, yauaa_context 1-0-3 to 1-0-4: HOLDS.
- d9_f09 guard: no enum, no type, gains `type: string`: VIOLATED (D5 and
  the retype rule unchanged).
- d9_f10 guard: enum ["a","b"] gains `type: string` and loses "b":
  VIOLATED narrowed, no retyped witness.

Fixture bytes d9_f07 and d9_f08 are copied unchanged from
snowplow/iglu-central at 7771e92d6e8a43f3bc9074393869ae9bcb08af1e
(Apache License 2.0), as the B1 cases were.

## Gates (pre-registered)

- Full suite, clippy (no new warnings), every mutant killed.
- SchemaStore replay: PASS, unchanged labels.
- label-free.py and counterexample-check.py: every printed
  counterexample passes the independent check.
- Iglu Central replay (iglu-replay.py, committed with this release):
  every printed counterexample passes the independent check; ADDITION
  pairs VIOLATED go from 9 to 5 (the 3 confirmed, schedule_update, and
  client_session, whose `userId` finding is real); none of the 4
  false-alarm pairs stays VIOLATED.

## Mutation operators

- D9-M1 enum coverage check dropped (every type added to an enum field
  is a note): d9_f04.
- D9-M2 integer not an instance of number: d9_f06.
- D9-M3 enum values not offered as retype candidates: d9_f04.
