# JUDGE-SUBTYPE r2: stop accusing what was not judged (r2, FROZEN)

Rulings D1 to D3 ratified by the founder 2026-11-02. Frozen with
crates/invar-falsify/tests/judge_subtype_r2_test.rs and the SchemaStore
replay manifest BEFORE implementation; freeze record:
JUDGE-SUBTYPE-R2-SPEC.frozen. Amends JUDGE-SUBTYPE r1 (which stays frozen
as written); where they differ, r2 governs from v0.2.4.

## Why

A replay of `invar judge` (v0.2.3 logic) over every schema modified in
SchemaStore between 2026-05-01 and 2026-11-01 (479 commits, 819 schema
changes) flagged 77 commits VIOLATED. Classified by rule and spot-checked
by hand: 19 real narrowings, 4 pattern or format changes (undecided, as
stated), 54 false alarms. The false alarms came from three defects:

- D1: a field whose type moved behind `anyOf`, `oneOf`, `allOf` or `$ref`
  was reported as "retyped ... now unspecified" (566 of 602 retyped
  witnesses). Those constructs are on the stated not-judged list; an
  accusation about them breaks INVAR's own rule.
- D2: removing a property from an object that admits other keys was a
  break (208 of 222 removals). A schema is read as a validator (r1): an
  instance carrying that key still validates.
- D3: keywords were judged one at a time. `enum: [0, 1]` replaced by
  `minimum: 0` was reported tightened, though it accepts more.

## Rulings

D1 (combinators). A field whose node carries `anyOf`, `oneOf`, `allOf` or
`$ref` at either ref, and whose node changed between the refs, is not
evaluated: no witness of any kind is produced for that node or beneath it,
and its dotted path is listed in `not_evaluated_paths`. A node that is
byte-for-byte the same at both refs is not listed. Verdict:
- any witness elsewhere: VIOLATED (exit 1), with the paths listed;
- no witness and at least one such path: NOT EVALUATED, reason
  `unjudged_construct` (exit 4). A skip is never a pass;
- otherwise HOLDS.
In the GitHub Action, `unjudged_construct` passes the job with a
`::warning` annotation that names the reason: stated, never claimed as a
pass and never as a break.

D2 (open objects). A removal is VIOLATED only when the object it was
removed from is closed at the post ref: `additionalProperties: false` or
`unevaluatedProperties: false`. From any other object it is recorded in
`notes` ("removed from an object that admits other keys; instances
carrying it still validate") and does not change the verdict.

D3 (enum to bounds). When a property had an `enum` at the base ref and has
none at the post ref, a bound added or tightened at the post ref is
VIOLATED only if some old enum value violates the new bound. Otherwise
the change is a widening and produces no witness.

Unchanged: the retype rule (any change of declared type, both directions,
stays VIOLATED: consumers parse values), enum narrowing, `required`
escalation, pattern and format (flagged, never claimed proven), exit
codes, receipts, SARIF and GitHub surfaces (plus the additions above).

## Surfaces

- JSON (and the receipt's `result`): adds `not_evaluated_paths` (sorted)
  and `notes` (sorted). Every other field is unchanged.
- Text: one line per not-evaluated path and per note, after the witness
  lines and before the `judged:` line. Outputs with neither are byte
  identical to v0.2.3 (both B1 fixtures are closed objects without
  combinators).
- SARIF: `run.properties` adds both lists; the `unjudged_construct`
  notification uses the existing NOT EVALUATED path.

## Fixtures (judge_subtype_r2_test.rs)

- r2_f01 D1: a field moved into `anyOf`, nothing else changed: exit 4,
  reason `unjudged_construct`, path listed, no "retyped" (kills R2-M1,
  R2-M4).
- r2_f02 D1: an unchanged `$ref` node beside a real closed removal:
  VIOLATED, the `$ref` path not listed.
- r2_f03 D1: a changed `allOf` field plus a closed removal: VIOLATED, both
  stated.
- r2_f04 D2: removal from an open root object: HOLDS, note present
  (kills R2-M2).
- r2_f05 D2: removal from an `additionalProperties: false` object:
  VIOLATED.
- r2_f06 D2: nested open object removal noted; nested closed removal
  VIOLATED.
- r2_f07 D3: the real bun.lock change, `enum: [0, 1]` to `minimum: 0`:
  HOLDS (kills R2-M3).
- r2_f08 D3: `enum: [0, 1, 5]` to `maximum: 3`: VIOLATED tightened.
- r2_f09 B1 unchanged: b1-001 still breaks 10 promises, b1-002 one.
- r2_f10 `--format github` on `unjudged_construct`: one `::warning`
  naming the reason.

## Regression corpus (docs/falsification/schemastore-replay/)

MANIFEST.tsv pins SchemaStore at 1f9ebc8379f842319663f776f9767fdf6ca5e4a5
and lists all 819 schema changes with their v0.2.3 verdict, plus the 77
flagged commits labelled real, undecided or false_alarm. The labels come
from structural rules, spot-checked by hand; they are stated as such.
replay.sh re-runs the judge over the pinned history. Acceptance:

- all 19 real commits stay VIOLATED (any miss fails the build);
- no commit that HOLDS under v0.2.3 becomes VIOLATED;
- pre-registered before implementation: at most 10 of the 54 false-alarm
  commits remain VIOLATED. Whatever remains is listed with its reason,
  never hidden.

## Amendments to earlier frozen fixtures (stated, old hashes kept)

Two fixtures removed a field from an open nested object and expected
VIOLATED, which D2 reverses. Their purpose is the dotted path (jf_f07,
killer of JM-M5) and the parent-line anchor (sf_f04), not openness, so
the nested `user` object is made closed (`additionalProperties: false`)
in judge_test.rs (NESTED_PRE, NESTED_POST, ADDED_POST) and
judge_sarif_test.rs (NESTED_PRE, NESTED_POST). Nothing else changes. The
new hashes are in JUDGE-SUBTYPE-R2-SPEC.frozen; the old ones stay in
JUDGE-SPEC.frozen and SARIF-SPEC.frozen.

## Mutation operators

- R2-M1 combinator check dropped: killed by r2_f01.
- R2-M2 closed-object check ignored (every removal VIOLATED): r2_f04.
- R2-M3 old enum values not checked against the new bound: r2_f07.
- R2-M4 `unjudged_construct` reported as HOLDS: r2_f01.

## Amendment A (D4, ratified 2026-11-02, before r2 shipped)

Why: the first r2 build replayed at real 15 of 19, false alarms 6 of 54
(FAIL as frozen). Resolving the four misses by hand showed one mislabel
and three real narrowings hidden behind a reference:

- e7373bd3f (helmfile `missingFileHandler`): identical once its `$ref`
  is resolved. Relabelled false_alarm. Real narrowings: 18, false
  alarms: 55.
- 929201f41 (clasp `rootDir`, `$ref` to an empty schema, then a
  non-empty string) and 6b9673c4c (pyproject, `$ref` to an array, then
  `minItems: 1`): real, hidden behind a local reference.
- 585b21f01 (sil-kit, `anyOf[integer, string]` to `enum[1, "1"]`): real,
  hidden behind `anyOf`.

D4 (local references). Before a field is compared, a `$ref` that points
inside the same document (`#`, `#/definitions/...`, `#/$defs/...`, any
JSON pointer starting `#/`) is resolved against that ref's own document;
keywords beside the `$ref` override the target's. Resolution follows at
most 8 references along one chain and stops at a cycle; a reference it
cannot resolve (external, missing, cyclic, too deep) stays a `$ref`, so
D1 applies: not evaluated, stated. `anyOf`, `oneOf`, `allOf` are
unchanged: not evaluated, stated.

Fixtures added (frozen with this amendment):
- r2_f11 a field moved inline to a local `$ref` with the same content:
  HOLDS (kills R2-M5).
- r2_f12 a `$defs` target tightened (`minLength` added): VIOLATED
  tightened at the field's path (kills R2-M5).
- r2_f13 a self-referencing `$ref` that changed: NOT EVALUATED
  `unjudged_construct`, and the judge returns (no hang).
- r2_f14 an external reference (`other.json#/x`): NOT EVALUATED.

Mutation operator added: R2-M5 local reference resolution disabled.

Acceptance, amended (replay.sh):
- real narrowings 18: at least 17 VIOLATED; any that is not VIOLATED
  must be NOT EVALUATED with its field named (never HOLDS);
- false alarms 55: at most 6 still VIOLATED;
- a commit that HOLDS under v0.2.3 and is VIOLATED under r2 is listed
  for review: resolving references may expose a narrowing that v0.2.3
  could not see; each one is classified by hand before release.

## Amendment B (D5 and corrected labels, ratified 2026-11-02, before r2 shipped)

Why: the D4 build replayed at real 16 of 18, false alarms 7 of 55, and 5
HOLDS commits newly VIOLATED (FAIL as frozen). Every one was resolved by
hand against the referenced schemas; two of the earlier labels were
wrong, and the bar is re-frozen on the evidence:

- e7373bd3f (helmfile) is real after all: `helmDefaults.historyMax` went
  `number` to `integer` (1.5 no longer validates). Amendment A checked
  the wrong field. Relabelled real.
- 929201f41 (clasp `rootDir`): the base is `base-04.json#/definitions/path`,
  an external file; the earlier hand resolution ignored the file part.
  Whether it narrowed cannot be decided from this file. Relabelled
  stated: NOT EVALUATED is the correct verdict.
- 585b21f01 (sil-kit): narrowing behind `anyOf`. Labelled stated.
- b056a70c8 (Traefik `otlplogs` renamed `otlpLogs`), 51b3fa1a0 (pixi),
  8ae3e0c8c (zeropress): fields removed from closed objects that sit
  behind a local `$ref`; real breaks v0.2.3 could not see. Relabelled
  real.
- The 6 remaining false alarms and 2 newly flagged commits (449644d6d,
  4dd722547) are one class: a field without `type` gains the type its
  own keywords already implied. D5 governs them.

D5 (type made explicit). When a field has no `type` at the base ref and
gains one at the post ref, and every post type is implied by the field's
own keywords at the base ref, the change is a stated note ("type made
explicit"), not a break. Implied: `properties`, `additionalProperties`,
`patternProperties`, `required`, `propertyNames`, `minProperties`,
`maxProperties` imply object; `items`, `prefixItems`, `contains`,
`minItems`, `maxItems`, `uniqueItems` imply array; `minLength`,
`maxLength`, `pattern`, `format` imply string; `minimum`, `maximum`,
`exclusiveMinimum`, `exclusiveMaximum`, `multipleOf` imply number (and
integer). A post type outside the implied set is still VIOLATED
(retyped). The note states what is strictly true: values of other types,
valid before, are now rejected.

Fixtures added (frozen with this amendment):
- r2_f15 `properties` without type gains `type: object`: HOLDS, note
  (kills R2-M6).
- r2_f16 `properties` without type gains `type: string`: VIOLATED retyped.
- r2_f17 a reference cycle through `properties` (the jsone.json crash,
  found by the replay after Amendment A froze): the judge returns.

Mutation operator added: R2-M6 implied-type check dropped.

Acceptance, re-frozen (replay.sh), labels per MANIFEST.tsv:
- real (20): every one VIOLATED;
- stated (2): every one NOT EVALUATED, never HOLDS, never VIOLATED;
- false_alarm (54): none VIOLATED;
- holds: none newly VIOLATED (a new one fails the replay and must be
  classified by hand before release).
