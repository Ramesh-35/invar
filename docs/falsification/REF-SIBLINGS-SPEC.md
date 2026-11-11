# REF-SIBLINGS r0: a `$ref` with sibling keywords is read two ways, and is not judged (r0, FROZEN)

Drafted 2026-11-08 for ratification. Frozen with
crates/invar-falsify/tests/ref_siblings_test.rs and one real SchemaStore
change BEFORE implementation; freeze record: REF-SIBLINGS-SPEC.frozen.
Every earlier ruling stays frozen as written.

## Why: "valid" is implementation-defined on a sixth of the registry

JSON Schema draft-04 through draft-07 say that where `$ref` appears, its
sibling keywords are ignored. Draft 2019-09 reversed that: siblings apply.
The two validators INVAR trusts read draft-07 differently, measured here
on 2026-11-08 (Python jsonschema 4.26.0, Ajv 8.20.0):

    {"properties":{"a":{"$ref":"#/definitions/s","maxLength":1}}}   draft-07
    document {"a":"xx"}:  jsonschema accepts (siblings ignored), Ajv rejects (maxLength applied)

The same for a root `$ref` beside `properties`: jsonschema ignores the
root's own `properties`, Ajv applies them. A document is therefore valid
or invalid depending on which library the reader runs. In SchemaStore's
history, 169 of the files in our corpus carry such a node under draft-07,
and 64 changes touch one. JUDGE-D5 r1 met the first consequence:
stale.json (449644d6d) is VIOLATED under Ajv's reading and HOLDS under
jsonschema's; its manifest label is `disputed`.

INVAR's judge resolved a `$ref` by merging its siblings over the target
(JUDGE-SUBTYPE r2 Amendment A, D4): Ajv's reading, chosen without saying
so. A verdict that holds under one validator and not the other is not a
verdict this judge may give.

## Ruling REF-SIBLINGS (r0)

A node is *two-way* when its document's `$schema` names draft-04, -06 or
-07, or names no draft, and the node carries `$ref` together with at
least one sibling the judge knows as a validation keyword. Not counted
as siblings: `$schema`, `$id`, `id`, `$comment`, `title`, `description`,
`default`, `examples`, `deprecated`, `readOnly`, `writeOnly`,
`definitions`, `$defs`, `$anchor`, `$vocabulary`, and any keyword the
judge does not know (the diff already ignores them). Under 2019-09 and
2020-12 no node is two-way: siblings apply by the specification, and the
judge's merge is the right reading.

- A two-way node the walk reaches as a property or an array element, on
  either side, or through a hop of its `$ref` chain, is stated and not
  judged: its path goes to `ref_siblings_paths`, its pointers are covered
  whole on both sides, and nothing beneath it is a witness or an unchecked
  pointer.
- A two-way document root is stated at the root and nothing in the
  document is walked.
- The verdict: a VIOLATED witness elsewhere still makes the change
  VIOLATED (a break is a break under both readings when the walk found
  it away from any two-way node); otherwise any two-way node makes the
  verdict NOT EVALUATED with the new reason `ref_siblings`, ahead of
  `unjudged_construct` and `unchecked_change`.
- Text: `  not evaluated: <path> carries $ref beside <kw, kw>; under
  draft-07 jsonschema ignores those keywords and Ajv applies them, so no
  single reading exists to judge.` The reason line: `NOT EVALUATED
  (ref_siblings): <contract> has a $ref with sibling keywords that
  validators read differently under its draft; nothing there was judged a
  break or a pass. Move the siblings into the referenced schema, or
  migrate to draft 2019-09 or later, where siblings apply.`
- JSON: `reason` is `ref_siblings`; `ref_siblings_paths` lists the paths
  (the document root as `""`); `unjudged_constraints` gains `$ref with
  siblings (draft-07 and earlier)`.

Deliberately not in r0: judging the change under both readings and
accepting it when they agree (r1, if the 28 HOLDS this costs on
SchemaStore are worth the machinery); any change to how 2019-09 and
2020-12 are read; annotation siblings, which are not two-way.

## Fixtures (frozen before implementation)

- rs_f01 a property `{"$ref":..., "maxLength": 1}` under draft-07, target
  unchanged, the sibling changed to 2: NOT EVALUATED, reason
  `ref_siblings`, `ref_siblings_paths` exactly `["a"]`, the text line
  naming `maxLength`, `unchecked` empty.
- rs_f02 guard: the same node under 2019-09: judged as before (the
  tightening from 2 to 1 is VIOLATED).
- rs_f03 guard: `$ref` with only `description` and `title` beside it:
  not two-way; the target's narrowing is VIOLATED as before.
- rs_f04 root `$ref` beside `properties` under draft-07: NOT EVALUATED,
  `ref_siblings_paths` exactly `[""]`, nothing else stated.
- rs_f05 a two-way node reached through a hop (`a` refers to `d`, `d`
  carries `$ref` and `type`): NOT EVALUATED at `a`.
- rs_f06 a two-way node beside a real break elsewhere: VIOLATED (the
  break stands), with `ref_siblings_paths` still listing the node.
- rs_f07 no `$schema` at all: read as draft-07, two-way.
- rs_f08 real bytes, stale.json 449644d6d: NOT EVALUATED,
  `ref_siblings_paths` exactly `[""]`, the draft-migration note still
  present.

## Mutation operators

- RS-M1 two-way nodes judged anyway (the check dropped)          -> rs_f01
- RS-M2 2019-09 treated as two-way                                 -> rs_f02
- RS-M3 annotations counted as siblings                            -> rs_f03
- RS-M4 the hop not checked                                        -> rs_f05
- RS-M5 the root not checked                                       -> rs_f04

## Pre-registered prediction

Measured with a throwaway build of this ruling, discarded before this
freeze (its diff kept as refsib-throwaway.diff outside the repository):
on SchemaStore exactly the 70 changes listed in the freeze record move,
all to NOT EVALUATED with reason `ref_siblings`: 28 from HOLDS, 41 from
NOT EVALUATED (other reasons), 1 from VIOLATED (stale.json, the disputed
one). Iglu and AsyncAPI: no move. WG gate PASS unchanged (31/45/0);
SchemaStore replay PASS; label-free VIOLATED count 39 (stale leaves it);
Iglu 5/3/0.
