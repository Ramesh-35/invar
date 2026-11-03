# COUNTEREXAMPLE r0: every break comes with an instance you can check (r0, FROZEN)

Ruling C-D1 ratified by the founder 2026-11-03. Frozen with
crates/invar-falsify/tests/counterexample_test.rs and
docs/falsification/counterexample-check.py BEFORE implementation; freeze
record: COUNTEREXAMPLE-SPEC.frozen.

Why: a VIOLATED witness is a claim. A counterexample is an instance that
the base schema accepts and the changed schema rejects, so a maintainer
can check the claim with a validator they already trust, without trusting
INVAR. Writing this spec found D7 (JUDGE-WIDEN r0): a claimed break with
no possible counterexample is not a break.

## What r0 builds (C-D1)

For each VIOLATED witness of kind `removed`, `retyped`, `narrowed`,
`tightened` or `escalated`, the judge tries to build one JSON instance of
the whole contract (not a fragment): valid at the base ref, rejected at
the post ref, at the witness's path. No solver, no new dependency.

Construction, deterministic:

- The instance is the smallest the base schema allows: an object carries
  only its `required` properties (plus what the witness needs on its
  path); an array carries `minItems` items. Values come from the schema,
  in this order: `const`; the first `enum` value; else by the first
  declared type: `null`; `true`; integer and number: `0`, moved into
  `minimum`..`maximum` (the bound itself, or one past an exclusive bound);
  string: `"a"` repeated `minLength` times (empty when none); array and
  object as above. A field with no `type` and no other keyword gets `null`.
- At the witness path, the value that shows the break:
  - removed: the parent carries the removed property, with its base value.
  - retyped: the first of `null`, `true`, `0`, `0.5`, `""`, `[]`, `{}`
    that the base accepts at that path and the post rejects.
  - narrowed (values lost): the first lost enum value, in base order.
  - narrowed (enum added): the base value of the field, if the new enum
    does not hold it.
  - tightened: the value just past the new bound: `maximum` lowered to M
    gives M+1 (integer) or M+0.5 (number); `minimum` raised to m gives m-1
    or m-0.5; an exclusive bound gives the bound itself; `maxLength` n
    gives n+1 characters `"a"`; `minLength` n gives n-1; `maxItems` n
    gives n+1 base items; `minItems` n gives n-1. If the base rejects that
    value, the old inclusive bound itself is tried once.
  - escalated: the parent without the newly required property.

Self-check, before anything is printed: a checker for the keywords the
judge reads (`type`, `properties`, `required`, `additionalProperties`
`false` or a schema, `enum`, `const`, `minimum`, `maximum`,
`exclusiveMinimum`, `exclusiveMaximum`, `minLength`, `maxLength`,
`minItems`, `maxItems`, `items` as one schema, local `$ref`) returns
valid, invalid, or unknown. Keywords that do not assert (annotations
such as `title`, `description`, `default`, `examples`, `$schema`, `$id`,
`$comment`, and keywords outside the JSON Schema vocabulary, which
validators ignore, such as Iglu's `self`) are ignored. Any other
assertion keyword (for example `pattern`, `format`, `multipleOf`,
`uniqueItems`, `minProperties`, `patternProperties`, `propertyNames`,
`dependencies`, `not`, `if`, combinators, `items` as an array, a
non-local `$ref`) is unchecked. An unchecked keyword, met on a node
the instance reaches, makes the answer unknown, except that invalid stays
invalid when a checked keyword fails on a path of plain conjunction (no
`not`, `anyOf`, `oneOf`, `allOf`, `if` above it). The instance is printed
only when the base says valid and the post says invalid. Otherwise:
`no counterexample built: <reason>.` Never a guess.

Reasons, exact:
- `<keyword> is not checked by this judge` (first unchecked keyword met)
- `pattern and format changes are not decided` (kind `constraint_changed`)
- `the path could not be followed`
- `no candidate value was valid at the base and rejected now`
- `the instance would be larger than 512 bytes`

## Surfaces

- JSON (and the receipt's `result`): each witness gains
  `counterexample` (the instance, or null) and `counterexample_reason`
  (null, or the reason). Keys sorted, as everywhere.
- Text: under each witness line, one line:
  `    counterexample (valid at <base>, rejected now): <compact JSON>` or
  `    no counterexample built: <reason>.`
- GitHub: each `::error` message appends `%0A` and that same line,
  without indent.
- SARIF: each result's `properties.counterexample` (instance or null);
  the message text stays the witness sentence (sf_f02 unchanged).

## Independent check (counterexample-check.py)

A frozen script runs the judge over the SchemaStore replay corpus's real
commits (MANIFEST.tsv, label `real`), and checks every printed
counterexample with Python `jsonschema` (the draft each schema declares):
valid against the base file, invalid against the post file. Pre-registered
acceptance, before implementation:

- every printed counterexample passes the independent check (any failure
  fails the release);
- coverage (witnesses with a counterexample, of all VIOLATED witnesses on
  those commits) and the reasons for the rest are reported as measured;
  no threshold is set, none is hidden.

The expected values in the Rust fixtures below were checked with the
same library at freeze (recorded in the freeze record).

## Fixtures (counterexample_test.rs)

- cx_f01 removed from a closed object: `{"age":0}`; text line exact.
- cx_f02 retyped `number` to `integer`: `{"f":0.5}`; `boolean|null` to
  `boolean`: `{"f":null}`.
- cx_f03 narrowed, enum `["a","b","c"]` to `["a","b"]`: `{"f":"c"}`
  (kills CX-M3).
- cx_f04 tightened: integer `maximum` 100 to 10: `{"f":11}`; `minimum`
  0 to 5: `{"f":4}`; `minimum` 0 to 0.5 on an integer (m-1 = -0.5 is
  rejected at base, so the old bound is tried): `{"f":0}`; `maxLength` 10 to 3: `{"f":"aaaa"}`; `minItems` 0 to
  2 over strings: `{"f":[""]}` (kills CX-M4).
- cx_f05 escalated: `{}`.
- cx_f06 nested and required: `payload` and `user` required at base,
  `zipcode` removed from closed `user`:
  `{"payload":{"user":{"zipcode":""}}}` (kills CX-M5).
- cx_f07 a pattern change: counterexample null, reason `pattern and
  format changes are not decided`.
- cx_f08 a required sibling carrying `pattern`: null, reason `pattern is
  not checked by this judge`. No guess.
- cx_f09 real b1-001: all 6 witnesses carry a counterexample; two runs
  byte-identical.
- cx_f10 JSON surface: both keys present on every witness; GitHub
  annotation carries the line.
- cx_f11 self-check at base: root `minProperties` (unchecked): null,
  reason `minProperties is not checked by this judge` (kills CX-M1).
- cx_f12 self-check at post: the judge flags a removal from a closed
  object whose post adds `patternProperties` that admit the key; the
  checker cannot confirm the rejection: null, reason `patternProperties
  is not checked by this judge` (kills CX-M2). The judge's claim is not
  confirmed, and the output says so.

## Mutation operators

- CX-M1 base check skipped: cx_f11.
- CX-M2 post check skipped: cx_f12.
- CX-M3 the kept enum value used instead of the lost one: cx_f03.
- CX-M4 tightened value on the bound instead of past it: cx_f04.
- CX-M5 required properties not built (path only): cx_f06.
