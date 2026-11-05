# COUNTEREXAMPLE r1: five more keywords read, no new dependency (r1, FROZEN)

Founder go 2026-11-03. Amends COUNTEREXAMPLE r0 (which stays frozen as
written); where they differ, r1 governs from v0.2.9. Frozen with
crates/invar-falsify/tests/counterexample_r1_test.rs and the amended
cx_f11 BEFORE implementation; freeze record: COUNTEREXAMPLE-R1-SPEC.frozen.

## What changes

The checker reads five more assertion keywords, and the generator builds
instances that satisfy them at the base. The kinds of break (C-D1) and
every surface are unchanged.

Checker (valid, invalid, unknown, as r0):
- `minProperties`, `maxProperties`: object key count.
- `uniqueItems: true`: no two items equal (numbers by value).
- `dependentRequired`, and `dependencies` in its array form: when the
  named key is present, every listed key is present. `dependencies` with
  a schema value stays unchecked (unknown).
- `multipleOf`: the number divided by it is a whole number.

Generator (smallest instance, deterministic):
- An object with fewer keys than `minProperties` gains declared,
  not-yet-present properties, in sorted key order, with their minimal
  values, until it has enough or none are left.
- Each present key's `dependentRequired` (or array `dependencies`) keys
  that the object declares in `properties` are added with their minimal
  values, until nothing more is added. An undeclared dependency is not
  built (the checker then rejects, and nothing is printed).
- An array that must hold n items under `uniqueItems: true` takes n
  distinct values: strings `""`, `"a"`, `"aa"`, ...; integers and numbers
  0, 1, 2, ... within bounds; booleans `true`, `false`; otherwise n copies
  (the checker then rejects, and nothing is printed).
- A number with `multipleOf` m is the smallest multiple of m within its
  bounds (0 when allowed).

The self-check is unchanged: an instance is printed only when the base
says valid and the post says invalid. When the generator cannot satisfy
a base keyword, the checker says invalid, and the output says
`no candidate value was valid at the base and rejected now`.

## Fixtures (counterexample_r1_test.rs)

- k_f01 `minProperties` read: a removal from a closed root with
  `minProperties: 1`: `{"age":0}` (r0 said unknown).
- k_f02 `minProperties` padding: `id` newly required, root
  `minProperties: 2`, declared `a`, `b`, `id` (strings): `{"a":"","b":""}`
  (kills K-M1).
- k_f03 `minProperties` impossible at base: closed root declaring two
  properties with `minProperties: 3`: null, `no candidate value was valid
  at the base and rejected now` (kills K-M2).
- k_f04 `maxProperties` read: removal from a closed root with
  `maxProperties: 1`: `{"age":0}`.
- k_f05 `uniqueItems` read: `array` to `object` with `uniqueItems` at
  base: `{"f":[]}` (the pnpm shape).
- k_f06 `uniqueItems` distinct items: an array of strings with
  `uniqueItems`, `maxItems` 3 lowered to 1: `{"f":["","a"]}` (kills K-M3).
- k_f07 `uniqueItems` not built for objects: items `{"type":"object"}`,
  `maxItems` 3 lowered to 1, `uniqueItems`: two copies of `{}` are equal,
  so null, no candidate, though a pair of distinct objects exists (kills
  K-M4). Stated as a limit of r1, not as impossible.
- k_f08 `dependentRequired` completion: `a` removed from a closed root
  whose base has `dependentRequired: {"a":["b"]}`: `{"a":"","b":""}`
  (kills K-M5).
- k_f09 `dependentRequired` unbuildable: the dependency names a key the
  root does not declare: the generator leaves it out, the checker
  rejects: null, no candidate (kills K-M6).
- k_f10 array `dependencies` (draft-07 form) completion, as k_f08:
  `{"a":"","b":""}`.
- k_f11 `multipleOf` read: integer `multipleOf: 5`, `maximum` 100 to 10:
  11 is not a multiple, so the old bound: `{"f":100}` (kills K-M7).
- k_f12 `multipleOf` generator: `n` removed from a closed root, `n`
  integer `minimum: 7`, `multipleOf: 5`: `{"n":10}` (kills K-M8).
- k_f13 still unchecked: `dependencies` with a schema value: null,
  `dependencies is not checked by this judge`.

## Amendment to a frozen fixture (stated, old hash kept)

cx_f11 showed the base check refusing an unchecked root keyword with
`minProperties`, which r1 now reads. Its purpose (CX-M1 killer) is kept
with `propertyNames` at the root instead. Expected: null, reason
`propertyNames is not checked by this judge`. Old hash in
JUDGE-R3-SPEC.frozen; new in COUNTEREXAMPLE-R1-SPEC.frozen.

## Gates (pre-registered)

- label-free.py and counterexample-check.py: every printed
  counterexample passes the independent check (any failure fails the
  release).
- Confirmed commits on label-free.py, reported as measured, no
  threshold. Prediction, stated before implementation: 10 to 11 (pnpm
  18132a01a). codex-hooks and zeropress are expected to stay
  unconfirmed: past `minProperties` and `dependentRequired` they reach
  `anyOf`, `pattern` and `format`.

## Mutation operators

- K-M1 minProperties padding dropped: k_f02.
- K-M2 minProperties check always valid: k_f03.
- K-M3 uniqueItems distinct generation dropped: k_f06.
- K-M4 uniqueItems check always valid: k_f07.
- K-M5 dependentRequired completion dropped: k_f08.
- K-M6 dependentRequired check always valid: k_f09.
- K-M7 multipleOf check always valid: k_f11.
- K-M8 multipleOf generation dropped: k_f12.
