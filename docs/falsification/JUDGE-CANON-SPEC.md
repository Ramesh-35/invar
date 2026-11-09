# JUDGE-CANON r0: equivalent after a meaning-preserving rewrite is covered, and stated (r0, FROZEN)

Ratified by the founder 2026-11-07. Frozen with
crates/invar-falsify/tests/judge_canon_test.rs and two real SchemaStore
changes BEFORE implementation; freeze record: JUDGE-CANON-SPEC.frozen.
Extends JUDGE-REACH r0; every earlier ruling stays frozen as written.

## Why

After JUDGE-TESTS, 597 of SchemaStore's 819 changes are NOT EVALUATED.
Measured on their 3,024 unchecked and unjudged pointers, 999 are one
spelling change: a node with a type list became the `anyOf` of its
types (`{"type":["string","null"]}` to
`{"anyOf":[{"type":"string"},{"type":"null"}]}`), most often in a draft-07
migration. The two spellings admit exactly the same documents. The judge
reported them unchecked or unjudged because it compares spellings.

## Ruling CANON (r0)

A normal form, applied recursively to a schema node with its annotations
set aside:

- a node whose `type` is a list of two or more names, and which carries
  no `anyOf` of its own, becomes the `anyOf` of one node per name, in
  order, each holding `"type"` and the node's keywords that apply to that
  type only: string `minLength`, `maxLength`, `pattern`, `format`,
  `contentMediaType`, `contentEncoding`; number and integer `minimum`,
  `maximum`, `exclusiveMinimum`, `exclusiveMaximum`, `multipleOf`; array
  `items`, `minItems`, `maxItems`, `uniqueItems`, `additionalItems`,
  `contains`, `prefixItems`; object `properties`, `required`,
  `additionalProperties`, `patternProperties`, `minProperties`,
  `maxProperties`, `dependencies`, `dependentRequired`,
  `dependentSchemas`, `propertyNames`, `unevaluatedProperties`; every
  other keyword stays beside the `anyOf`, shared by all branches. The
  specification already applies each of those keywords only to instances
  of its type, so the rewrite changes nothing a validator decides.

Where the judge would report a node unchecked (JUDGE-REACH) or state it
unjudged (D1), it first compares the two nodes in normal form. Equal means
the node is covered, with one note per node, at the shallowest such
pointer:

`  noted: equivalent after rewriting type lists as anyOf at <pointer>`

(the root reads "the document root"). Nothing else at that node differs,
so nothing is hidden by the note.

CANON judges equality, never inclusion. Not covered, deliberately: a head
whose branches carry a shared keyword inside each branch instead of
beside the `anyOf`; a single-name type list; `oneOf` or `allOf` in place
of `anyOf`; any further change inside a branch. Those stay NOT EVALUATED
exactly as before.

## Fixtures (judge_canon_test.rs)

- c_f01 `type: ["string","null"]` to the two-branch `anyOf` at a property:
  HOLDS, the note at `/properties/a`, `unchecked` empty (kills CN-M1).
- c_f02 `type: ["array","string"]` with `items`, to branches where `items`
  follows `array`: HOLDS (kills CN-M2).
- c_f03 a shared `enum` beside the `anyOf`: HOLDS.
- c_f04 guard: the same `enum` inside one branch only: NOT EVALUATED
  `unjudged_construct`, no note.
- c_f05 guard: a branch that also gains `minLength`: NOT EVALUATED
  `unjudged_construct`, no note.
- c_f06 titles and descriptions differ between the two spellings: HOLDS
  (kills CN-M3).
- c_f07 under `definitions`, reached by `finish` and not by the walk:
  HOLDS, the note at `/definitions/D`.
- c_f08 guard: `oneOf` in place of `anyOf`: NOT EVALUATED.
- c_f09 real bytes, plagiarize.json at 7de30f637 ("Migrated Plagiarize
  config file schemas to draft-07"): HOLDS, the draft note and two CANON
  notes.
- c_f10 real bytes, esquio.json at a0b8327b0 (the same migration, nested
  type lists under D1): HOLDS.

## Gates (pre-registered)

- Full suite, clippy `-D warnings` silent, every mutant killed (CN-M1..M3
  new).
- Label-free 26/14/50/0, Iglu 5/3/0, SchemaStore replay PASS: unchanged, a
  covered equivalence adds no witness.
- Prediction, from the Python prototype run before this file: on
  SchemaStore at v0.2.15, exactly 68 changes move from NOT EVALUATED to
  HOLDS (35 from `unchecked_change`, 33 from `unjudged_construct`), the
  list in the freeze record; 19 more keep NOT EVALUATED with a shorter
  list; no other verdict changes on any corpus. A different set is a
  defect to explain before release, in the engine or in the prototype.

## Mutation operators

- CN-M1 the normal form is never applied: c_f01.
- CN-M2 a typed keyword is shared instead of following its type: c_f02.
- CN-M3 annotations are compared instead of set aside: c_f06.
