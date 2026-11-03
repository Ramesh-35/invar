# JUDGE-WIDEN r0: a type change that only widens is not a break (r0, FROZEN)

Ruling D7 ratified by the founder 2026-11-03. Frozen with
crates/invar-falsify/tests/judge_widen_test.rs and the amended fixtures
below BEFORE implementation; freeze record: JUDGE-WIDEN-SPEC.frozen.
Amends JUDGE-SUBTYPE r2 ("Unchanged: the retype rule, any change of
declared type, both directions, stays VIOLATED: consumers parse values"),
which stays frozen as written; where they differ, this governs from
v0.2.6.

## Why

The retype rule was the last rule justified by a guess about readers
("consumers parse values"). D6 (2026-11-02) retired that voice; a schema
is read as a validator (JUDGE-SPEC r1). Found while writing the
counterexample spec: for `boolean` to `boolean|null` there is no instance
valid at base and rejected now, so no counterexample can exist, so it is
not a break. Measured:

- Demo PR Ramesh-35/invar-demo-api#4 (the real Snowplow b1-001 change):
  4 of its 10 annotations are widenings (`loop`, `muted`, `playbackRate`,
  `volume`, each `T` to `T|null`).
- B1 case b1-002: its only witness, `deviceMemory` `integer|null` to
  `number|null`, is a widening (every integer is a number).
- SchemaStore replay: the 20 real commits carry 4 retype witnesses, none a
  widening (helmfile `number` to `integer`, pnpm `array` to `object` are
  narrowings). No replay verdict changes.

## Ruling D7

Let A be the types a field declares at base and B at post (a `type`
string or array; `integer` is contained in `number`). The node is
compared only when D1 does not apply (no combinator at either ref).

- Widening: B declares no `type` at all, or every type in A is in B (or
  is `integer` with `number` in B). A stated note, never a witness:
  `retyped (widened): <path> was <a>, is now <b>; every value valid before is still valid.`
- Otherwise (some type in A is dropped): VIOLATED, kind `retyped`:
  `retyped: <path> was <a>, is now <b>. Values outside <b>, valid before, are now rejected; restore the type, or version the change.`
- A field with no `type` at base keeps r2 behaviour (D5, or VIOLATED).

Every other rule, exit code and surface is unchanged.

## Fixtures (judge_widen_test.rs)

- w_f01 `boolean` to `boolean|null`: HOLDS, exit 0, the note exactly as
  above (kills W-M4).
- w_f02 `integer` to `number`: HOLDS, note (kills W-M2).
- w_f03 `number` to `integer`: VIOLATED, the retyped sentence exactly.
- w_f04 `boolean|null` to `boolean`: VIOLATED.
- w_f05 `array` to `object`: VIOLATED.
- w_f06 `string` to no `type` (no combinator): HOLDS, note ("is now
  unspecified").
- w_f07 real b1-002 bytes: HOLDS, exit 0, the note names `deviceMemory`
  (kills W-M1).
- w_f08 real b1-001 bytes: `breaks 6 promises`; four widened notes
  (`loop`, `muted`, `playbackRate`, `volume`); no `retyped:` line.
- w_f09 `--format github` on b1-002: no `::error` line.
- w_f10 `integer|string` to `number`: VIOLATED (`string` dropped; kills
  W-M3).

## Amendments to earlier frozen fixtures (stated, old hashes kept)

Seven fixtures used b1-002 (now a widening) as their example of a retype.
Their purpose is the retype mechanics (detection, JSON fields, SARIF
result, GitHub line, run properties, D6 sentence), not the direction, so
each now judges the same real bytes in the narrowing direction, pre and
post swapped: `deviceMemory` `number|null` to `integer|null` (1.5 no
longer validates). Two fixtures asserted counts D7 changes.

- judge_test.rs: jf_f02 and json_format_carries_witnesses_and_unjudged_constraints
  swapped; the json test asserts pre `number`, post `integer`.
- judge_sarif_test.rs: sf_f01, sf_f02, sf_f07, sf_f08 (first half)
  swapped, sf_f08 reads the post line from the swapped post file. sf_f03
  (SF-M3 killer, located at the post line) used b1-001's `volume` retype,
  now a widening; it uses b1-001's `playbackRate` tightening instead
  (line 56 at base, 90 at post).
- judge_subtype_r2_test.rs: r2_f09 asserts b1-001 `breaks 6 promises`
  and b1-002 HOLDS.
- judge_contracts_test.rs: jc_f10 (second half) swapped, with the new
  retyped sentence.

Old hashes stay in JUDGE-SPEC.frozen, SARIF-SPEC.frozen,
JUDGE-SUBTYPE-R2-SPEC.frozen and JUDGE-CONTRACTS-SPEC.frozen; new hashes
are in JUDGE-WIDEN-SPEC.frozen.

## Mutation operators

- W-M1 widening check dropped (every retype VIOLATED): w_f07.
- W-M2 `integer` not contained in `number`: w_f02.
- W-M3 any shared type counts as widening: w_f10.
- W-M4 widened note dropped (silent HOLDS): w_f01.
