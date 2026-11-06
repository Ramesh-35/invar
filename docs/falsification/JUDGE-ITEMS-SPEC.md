# JUDGE-ITEMS r0: an array's element schema is judged like a property (r0, FROZEN)

Ratified by the founder 2026-11-06. Frozen with
crates/invar-falsify/tests/judge_items_test.rs and two real SchemaStore
pairs BEFORE implementation; freeze record: JUDGE-ITEMS-SPEC.frozen.
Extends JUDGE-REACH r0 and JUDGE-DRAFT r0, which stay frozen as written.

## Why

After JUDGE-DRAFT r0, 436 SchemaStore pairs are NOT EVALUATED
(unchecked_change). 83 of them carry an unchecked change inside `items`:
the judge walks `properties` and never enters an array's element schema,
so a tightened element is neither accused nor passed. The property-node
rules already decide every keyword those changes use; only the walk
stops short.

## Ruling ITEMS (r0)

At a judged property node (one the walk compares keyword by keyword,
not a D1 node compared whole), when on each side `items` is a schema
object or absent, neither side carries `prefixItems`, and neither side's
`type` excludes `array`, the element schema is judged as a child of the
node:

- its path is the node's path followed by `[]` (`tags[]`, and below it
  `tags[].name`); its JSON Pointer is the node's `/items` (local `$ref`
  chains resolved as for any property);
- it is judged by exactly the property-node rules: type, enum, the eight
  bounds, pattern, format, required, properties, local `$ref`, D1 for a
  combinator node, and recursion into its own properties and `items`;
- an absent `items` reads as `{}`: adding `items` is judged as a type
  given to elements that had none, and removing it widens (stated, as
  `retyped (widened)`);
- the node's own `items` keyword is covered when the rule applies.

Out of scope in r0, still NOT EVALUATED by pointer: `items` as an array
(tuple form), `prefixItems`, `additionalItems`, a boolean `items`, `items`
at the document root, and `items` on a node whose `type` excludes
`array`.

A counterexample for an element witness is the base's minimal document
with the element placed in a one-element array at that path; it is
printed only when the base accepts it and the head rejects it, as for
every counterexample. A path segment ending in `[]` means `items` only
when no property bears that literal name.

The summary's "not judged" list, text and JSON (`unjudged_constraints`),
names `tuple items` in place of `items`. Amendment: the JUDGE-SUBTYPE
fixture that asserts the exact list is updated to that name, nothing
else in it.

## Amendment A (before freeze): the prediction corrected

The ratified draft predicted 8 SchemaStore pairs from a prototype that
compared keywords but did not follow the walk. Rebuilt to walk exactly
as the engine does (site, then `$ref` targets, the owner of each keyword,
D1 nodes whole), the prototype finds that tsconfig.json and jsconfig.json
reach `references[]` only through root `allOf` and `anyOf`, which the
walk never enters: they stay NOT EVALUATED. cargo.json reaches
`crate-type[]` through `lib`'s `$ref`. Real-bytes fixture i_f12 is
therefore cargo.json, not tsconfig.json. The rule above is unchanged.

## Fixtures (judge_items_test.rs)

- i_f01 element gains `minLength: 1`: VIOLATED, counterexample `{"r":[""]}`.
- i_f02 element enum widened: HOLDS, nothing unchecked.
- i_f03 element enum added: VIOLATED, counterexample an array of a
  string the enum excludes (kills IT-M1).
- i_f04 `items` added to an array that had none: VIOLATED (kills IT-M2).
- i_f05 `items` removed: HOLDS, stated widened.
- i_f06 element property retyped: path `r[].a`, counterexample
  `{"r":[{"a":...}]}` (kills IT-M4).
- i_f07 element newly requires a field: escalated at `r[].b`.
- i_f08 tuple `items` changed: exit 4, `unchecked` names
  `/properties/r/items/0` (kills IT-M3).
- i_f09 a node under `anyOf` whose `items` changed: D1, exit 4
  unjudged_construct, unchanged from v0.2.14.
- i_f10 element through a local `$ref`: judged at the target.
- i_f11 a property literally named `r[]` beside `r`: the witness path
  resolves to the literal property.
- i_f12 real bytes, cargo.json at 13e89998e ("Add missing enum for
  crate-type"): VIOLATED at `lib.crate-type[]` with a counterexample.
- i_f13 real bytes, vsext.json at 3227450c3 ("Migrated vsext.json to
  draft-07"): HOLDS, the draft note and the element's type made explicit.

## Gates (pre-registered)

- Full suite, clippy `-D warnings` silent, every mutant killed (IT-M1..M4
  new).
- Label-free 20/12/28/0, Iglu 5/3/0, SchemaStore replay PASS: unchanged.
- Prediction, from the walk-faithful prototype run before any engine code:
  - SchemaStore: exactly 12 pairs leave unchecked_change. VIOLATED: cargo
    (counterexample), linutil-tabs (a pattern change, stated undecided).
    HOLDS: hugo, partial-black, crowdin, vsext, label-commenter-config.
    NOT EVALUATED unjudged_construct (an element under a combinator, now
    reached and stated): web-manifest-app-info at 32af9a900, sourcemap-v3,
    jovo-language-model, minecraft-biome, minecraft-template-pool.
  - SchemaStore: 11 further pairs keep their verdict with a shorter
    `unchecked` list; their reason may move from unchecked_change to
    unjudged_construct where a reached element is a combinator.
  - Iglu: exactly 4 pairs move NOT EVALUATED to HOLDS (mandrill
    message_clicked and message_opened, iglu resolver-config,
    snowplow bot_detection). AsyncAPI: none.
  - No pair moves from HOLDS, and no VIOLATED pair changes verdict.
  A different set is a defect to explain before release, in the engine or
  in the prototype.

## Mutation operators

- IT-M1 the element is never judged: i_f03.
- IT-M2 an absent `items` is not read as `{}`: i_f04.
- IT-M3 a tuple `items` judged as one schema: i_f08.
- IT-M4 the counterexample places the element without its array: i_f06.
