# COUNTEREXAMPLE r2: the checker evaluates allOf, anyOf, oneOf and not (r2, FROZEN)

Founder ratification in principle 2026-11-03. Amends COUNTEREXAMPLE r1
(which stays frozen as written); where they differ, r2 governs. Frozen
with crates/invar-falsify/tests/counterexample_r2_test.rs BEFORE
implementation; freeze record: COUNTEREXAMPLE-R2-SPEC.frozen.

Why: after r1, codex-hooks (9497fde91) stops at `anyOf`, and 7 printed
findings on the replay are blocked by a combinator. Combinators are pure
logic over checks the checker already makes; no solver, no dependency.

## Checker (valid, invalid, unknown, as before)

- `allOf`: the conjunction of its members (one invalid decides; else the
  first unknown; else valid).
- `anyOf`: valid if any member is valid; invalid if every member is
  invalid; otherwise unknown.
- `oneOf`: invalid if two or more members are valid; valid if exactly one
  is valid and every other member is invalid; invalid if every member is
  invalid; otherwise unknown.
- `not`: valid becomes invalid, invalid becomes valid, unknown stays
  unknown.
- An unknown carries the keyword that made it so (the innermost one met
  first, in key order), never the name of the combinator around it.
- `if`, `then`, `else` stay unchecked (unknown).

Soundness is unchanged: a node's answer is the conjunction of its
keywords, each of which is now definite or unknown; definite answers come
only from keywords the checker reads.

## Generator

- A node with `anyOf` or `oneOf` and no `type` of its own takes the
  minimal value of its first member that the checker says is valid for
  the node; if none is, the first member's minimal value (the checker
  then decides).
- An object built from a schema also carries the `required` names of each
  direct `allOf` member, a local `$ref` member resolved (members are
  conjunctive, as JUDGE-R3 J3).
- `not` is never solved: the generator ignores it, and the checker
  rejects whatever it excludes.

## Fixtures (counterexample_r2_test.rs)

- c_f01 allOf required built: removal of `age` from a closed root whose
  base `allOf` requires `name`: `{"age":0,"name":""}` (kills C-M4).
- c_f02 anyOf read: a required `id` whose schema is `anyOf` [string,
  integer]: `{"age":0,"id":""}` (kills C-M1).
- c_f03 anyOf first valid member: `id` `anyOf` [{string, minLength 2},
  {integer}]: `{"age":0,"id":"aa"}`.
- c_f04 oneOf counted: `id` `oneOf` [string, string maxLength 5]: `""` is
  valid in both, so invalid at base: null, `no candidate value was valid
  at the base and rejected now` (kills C-M2).
- c_f05 not read: `id` `not` {type string}, no type: `null` is not a
  string: `{"age":0,"id":null}` (kills C-M3).
- c_f06 not unsolved: `id` {type string, not {maxLength 5}}: `""` is
  excluded: null, no candidate.
- c_f07 unknown names the inner keyword: `id` `anyOf` [{string, pattern
  "^a"}, {string, pattern "^b"}]: no member can be checked: null, `pattern
  is not checked by this judge` (kills C-M5).
- c_f08 oneOf read at both refs: root `oneOf` [{required f}, {required
  g}] at base and post, `f` integer `maximum` 100 lowered to 10: `{"f":11}`
  (r1 said unknown).
- c_f09 if/then/else still unchecked: null, `if is not checked by this
  judge`.
- c_f10 allOf member by local `$ref`: resolved and read.

## Gates (pre-registered)

- label-free.py and counterexample-check.py: every printed
  counterexample passes the independent check (any failure fails the
  release).
- Confirmed commits on label-free.py reported as measured, no threshold.
  Prediction, stated before implementation: 11 to 12 (codex-hooks
  9497fde91), if nothing past `anyOf` on its path is unchecked.

## Mutation operators

- C-M1 anyOf read as unknown: c_f02.
- C-M2 oneOf counted as anyOf: c_f04.
- C-M3 not not inverted: c_f05.
- C-M4 allOf required not built: c_f01.
- C-M5 unknown reason replaced by the combinator's name: c_f07.
