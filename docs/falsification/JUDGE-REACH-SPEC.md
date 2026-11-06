# JUDGE-REACH r0: HOLDS means every change was judged (r0, FROZEN)

Ruling REACH ratified by the founder 2026-11-05. Frozen with
crates/invar-falsify/tests/judge_reach_test.rs and two real AsyncAPI
fixtures BEFORE implementation; freeze record: JUDGE-REACH-SPEC.frozen.

## Why

Auditing asyncapi/spec-json-schemas (at eee8d22, Apache-2.0) found a false
pass in INVAR itself. AsyncAPI 2.0.0 to 2.1.0, with the version tag
neutralized, judges HOLDS (exit 0). Python jsonschema shows a whole 2.0.0
document valid before and rejected after: `messageTrait.examples.items`
gained `additionalProperties: false` and an `anyOf` of required keys. The
change is reached only through `patternProperties`, then `oneOf`, then
`$ref`. The judge's walk follows `properties` and a node's own local
`$ref` and nothing else, and the `$ref` string at the walk's last stop is
identical on both sides. So the walk saw no change and HOLDS printed
"keeps every promise" for a change it never looked at. D1's NOT EVALUATED
(`unjudged_construct`) did not fire, because no node the walk compared
differed.

On the SchemaStore corpus, 340 of the 594 pairs that judge HOLDS changed
bytes in `definitions` or `$defs`. That is an upper bound: many of those
changes are reached through a plain `$ref` and were genuinely judged. The
exact count of false passes needs this ruling's instrumentation.

## Ruling REACH (r0)

HOLDS asserts that every change between base and head was judged. The
walk records every schema node it reaches, on each side, by its JSON
Pointer in its own document, including the targets of the local `$ref`
chains it resolves. After the walk, the base and head documents are
compared node by node. A difference is **covered** when it is one of:

- a keyword the judge reads at a reached node, exactly: `type`, `enum`,
  `minimum`, `exclusiveMinimum`, `minLength`, `minItems`, `maximum`,
  `exclusiveMaximum`, `maxLength`, `maxItems`, `pattern`, `format`,
  `required`, the key set of `properties` (a property added or removed),
  `additionalProperties` and `unevaluatedProperties` when a boolean on
  both sides (open or closed), and the presence of `patternProperties`.
  Nothing else: `const`, `multipleOf`, `minProperties`, `maxProperties`,
  `uniqueItems`, `items`, `contains`, `propertyNames` and the rest are
  not judged (the counterexample checker reads some of them; the judge
  does not), so their change at a reached node is not covered;
- anything inside a node D1 compared as a whole (a node carrying
  `anyOf`, `oneOf`, `allOf` or an unresolved `$ref`): D1 already states
  a changed one;
- an annotation, which no validator asserts: `title`, `description`,
  `$comment`, `examples`, `default`, `deprecated`, `readOnly`,
  `writeOnly`, and any keyword no JSON Schema draft defines (such as
  `markdownDescription` or `x-` vendor keywords).

Annotations are recognized by position, never by name alone: under
`properties`, `patternProperties`, `definitions`, `$defs`,
`dependentSchemas` and `dependencies`, keys are names, so a property
named `title` is a schema like any other.

Every other difference is **unchecked**: unreached by the walk, or reached at a keyword the judge does not read. It is reported as the shallowest
schema node whose own non-annotation keywords differ, or that exists on
one side only, and a reported node stands for every difference beneath
it. A reached ancestor never covers a descendant the walk did not reach.

- No witnesses, no D1 stated paths, one or more unchecked schemas: NOT
  EVALUATED, reason `unchecked_change`, exit 4.
- D1 stated paths and unchecked schemas, no witnesses: NOT EVALUATED,
  reason `unjudged_construct` (it leads), exit 4; both are stated.
- Witnesses: VIOLATED, exit 1, as today; the unchecked schemas are
  stated beside the breaks, so a proven break never hides an unseen one.

Wording, text surface:

- headline: `NOT EVALUATED (unchecked_change): <head> changed in a way
  this judge does not check. Nothing there was judged a break or a pass;
  review those schemas by hand.`
- one line per unchecked schema, after the D1 lines and before the noted
  lines: `  not evaluated: the schema at <pointer> changed in a way this
  judge does not check.`

JSON surface: a new key `unchecked`, the list of JSON Pointers (RFC 6901
escaping), sorted lexicographically, always present (empty when nothing
is unchecked). SARIF and GitHub carry the same sentence, one voice.

Out of scope in r0, stated so nothing is implied: following `$ref` into
`patternProperties`, `additionalProperties` schemas, `items` or
combinator branches (that is judging more; r0 is about never claiming to
have judged what it did not); external `$ref`; proving a definition
unused (r0 states a changed unreferenced definition rather than assume
nothing reaches it); `$schema` and `$id` changes, which alter how a
validator reads the document, are differences like any other.

## Fixtures (judge_reach_test.rs)

- r_f01 change behind patternProperties, oneOf and $ref: exit 4, headline,
  `/definitions/T` named (kills RE-M4).
- r_f02 guard: a plain local `$ref` retype stays VIOLATED with
  `{"x":""}`; `unchecked` is empty.
- r_f03 guard: a reached definition loosened (maxLength 5 to 10) holds.
- r_f04 a definition nothing references changed: exit 4, named (kills
  RE-M1).
- r_f05 annotation-only changes (description, title, examples, default,
  $comment) hold (kills RE-M2).
- r_f06 unknown keyword changes (markdownDescription, x-vendor) hold.
- r_f07 `additionalProperties` schema changed: exit 4,
  `/additionalProperties` named.
- r_f08 a property named `title` changed type in an unreached definition:
  exit 4, `/definitions/U/properties/title` named (kills RE-M3).
- r_f09 `$defs` read like `definitions`.
- r_f10 JSON `unchecked` sorted, an ancestor standing for its
  descendants.
- r_f11 precedence: `unjudged_construct` leads, both lines stated.
- r_f12 real AsyncAPI 2.0.0 to 2.1.0: VIOLATED (the version tag), exit 1,
  and `/definitions/messageTrait` stated beside it.
- r_f13 `const` changed at a reached node: exit 4, `/properties/a`
  named (kills RE-M5).

## Gates (pre-registered)

- Full suite, clippy `-D warnings` silent, every mutant killed (RE-M1..M5
  new).
- Label-free gate: VIOLATED commits and counterexamples unchanged (20,
  12 confirmed, 28 printed, 0 failed): this ruling adds no witness.
- Iglu replay: ADDITION pairs VIOLATED unchanged at 5, 3 confirmed,
  0 failed.
- The number of pairs that move from HOLDS to NOT EVALUATED on the
  SchemaStore corpus and on Iglu is measured and recorded, not
  predicted; the SchemaStore replay labels are amended with that measured
  list, each row's reason stated.
- AsyncAPI, tag neutralized, 2.0.0 to 2.1.0: HOLDS becomes NOT EVALUATED.

## Mutation operators

- RE-M1 coverage check dropped (unreached changes ignored): r_f04.
- RE-M2 annotation filter dropped (every byte counts): r_f05.
- RE-M3 annotation filter blind to position (a name read as a keyword):
  r_f08.
- RE-M4 a reached ancestor covers all its descendants: r_f01.
- RE-M5 `const` counted as a keyword the judge reads: r_f13.

## Amendment A (2026-11-05, before any implementation)

Probing every keyword on the covered list against Python jsonschema,
before writing engine code, found the list overclaimed in four places.
Each was HOLDS on v0.2.12 with a document valid before and rejected
after:

1. an open object gains a typed property (`{"k":1}` breaks);
2. an object is closed (`additionalProperties` true or absent to false);
3. a closed object drops a `patternProperties` entry;
4. the root schema's own `type` or bounds change (`constraints()` runs
   on property nodes only, never on the root).

The founder ratified the strict model (2026-11-05): HOLDS means every
document valid at the base is valid at the head, the standard every
counterexample already proves. The covered list is amended:

- the key set of `properties`: a **removed** property is covered (D2
  judges it); an **added** property is covered only when the base node
  was closed (`additionalProperties` or `unevaluatedProperties` false,
  and no `patternProperties`), so no base document could carry that key;
- `additionalProperties` and `unevaluatedProperties`: covered only when
  the change opens the object (false to true or absent); every other
  change to them is unchecked;
- `patternProperties`: no change to it is covered;
- the root node: only `required` and the `properties` key set (with the
  rule above) are covered; its own `type`, `enum`, bounds, `pattern` and
  `format` are not judged, so their change is unchecked.

The root pointer is rendered `the document root` in text
(`  not evaluated: the document root changed in a way this judge does not
check.`) and `""` in JSON (RFC 6901).

Judging these changes with counterexamples (a later slice) is judging
more; r0 only stops claiming to have judged them.

Amendment A fixtures, appended to judge_reach_test.rs:

- r_f14 open object gains a typed property: exit 4, `/properties/k`
  named (kills RE-M6).
- r_f15 object closed at the root: exit 4, the document root named
  (kills RE-M7).
- r_f16 guard: a closed object gains a property: HOLDS.
- r_f17 closed object drops a `patternProperties` entry: exit 4,
  `/patternProperties/^x` named.
- r_f18 guard: an object opened (false to true): HOLDS.
- r_f19 root `type` object to array: exit 4, the document root named
  (kills RE-M8).

Amendment A mutation operators:

- RE-M6 an added property always covered: r_f14.
- RE-M7 any `additionalProperties` change covered: r_f15.
- RE-M8 the root judged like a property node: r_f19.

Expected consequence, stated in advance: existing fixtures elsewhere in
the suite that assert HOLDS for one of these four changes encoded the
false pass. Each one that fails under the implementation is listed in
the implementation commit with the case, never silently rewritten.
