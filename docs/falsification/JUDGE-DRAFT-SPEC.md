# JUDGE-DRAFT r0: a draft migration that changes nothing is stated, not unchecked (r0, FROZEN)

Ratified by the founder 2026-11-06. Frozen with
crates/invar-falsify/tests/judge_draft_test.rs and two real SchemaStore
fixtures BEFORE implementation; freeze record: JUDGE-DRAFT-SPEC.frozen.
Extends JUDGE-REACH r0, which stays frozen as written.

## Why

JUDGE-REACH r0 made 521 SchemaStore pairs NOT EVALUATED. Attributing each
to what the judge did not check, with nothing hidden under the root
pointer, the single largest group is draft migrations: 97 pairs whose
only unchecked change is `$schema` moving from draft-04 to draft-07 and
`id` renamed `$id`. That is SchemaStore's own housekeeping, not a change
of contract, when the document uses nothing whose meaning differs
between those drafts.

## Ruling DRAFT (r0)

At the document root, a difference in `$schema`, `id` or `$id` is
covered when all of these hold:

- both `$schema` values name draft-04, draft-06 or draft-07, spelled
  `http` or `https`, with or without the trailing `#`;
- the document's identifier is unchanged: `$id` if present, else `id`,
  on each side (so `id` renamed `$id` with the same value is covered, and
  a changed or added identifier is not);
- neither document, at any schema position, uses a keyword whose meaning
  differs between those drafts: `exclusiveMinimum`, `exclusiveMaximum`
  (a boolean in draft-04, a number after), `const`, `contains`,
  `propertyNames`, `if`, `then`, `else`, `contentMediaType`,
  `contentEncoding` (ignored by draft-04, read after);
- neither uses a boolean subschema, a draft-06 construct, except where
  draft-04 already allows a boolean: `additionalProperties` and
  `additionalItems`;
- every `$ref` in either document is a local fragment (`#` or `#/...`),
  so the identifier's base URI never takes part in resolution.

A covered migration is stated, never silent, as a note:

`  noted: draft migration: $schema moves from <from> to <to>, with id renamed $id (same value); no keyword either version uses means something different between those drafts.`

The `, with id renamed $id (same value)` clause appears only when the
base carries `id` and the head carries `$id`. Draft names are written
`draft-04`, `draft-06`, `draft-07`.

Covering the root's migration covers exactly those three keywords at the
root: every other difference anywhere is judged or reported unchecked
as under JUDGE-REACH, by its own pointer, never hidden beneath the
root.

Out of scope in r0: 2019-09 and 2020-12 (their vocabularies differ more
widely); `id` or `$id` below the root (draft-04 used `id` to rebase
subschemas); `format`, which INVAR never decides in any draft (its
assertion is optional in every draft, and a change to it stays stated
undecided).

## Fixtures (judge_draft_test.rs)

- d_f01 plain migration: HOLDS, the note exactly (kills DR-M1).
- d_f02 identifier changed: exit 4, the document root named (kills DR-M2).
- d_f03 `const` present: exit 4 (kills DR-M3).
- d_f04 boolean `exclusiveMinimum` present: exit 4.
- d_f05 a boolean subschema: exit 4.
- d_f06 a non-local `$ref`: exit 4.
- d_f07 guard: `additionalProperties: false` does not block (kills DR-M4).
- d_f08 draft-07 to 2020-12: exit 4, out of scope.
- d_f09 a migration beside a real retype: VIOLATED, note stated, nothing
  unchecked.
- d_f10 a migration beside an unchecked definition change: exit 4,
  `unchecked` exactly `["/definitions/U"]`.
- d_f11 real SchemaStore bytes (webjobs-list.json, 8cfea30d1, "Migrated
  Azure Webjobs to draft-07"): HOLDS with the note.
- d_f12 `https`, no trailing `#`: the same draft.

## Gates (pre-registered)

- Full suite, clippy `-D warnings` silent, every mutant killed (DR-M1..M4
  new).
- Label-free 20/12/28/0, Iglu 5/3/0, SchemaStore replay PASS: unchanged,
  since a migration adds no witness.
- Prediction, from a Python prototype of these guards run before any
  engine code: of the 97 SchemaStore pairs whose only unchecked change is
  the migration, 85 pass every guard (10 have a non-local `$ref`, 2 a
  changed identifier, 1 `const`). The engine must convert exactly those
  85 from NOT EVALUATED to HOLDS on SchemaStore, and change no other
  pair's verdict on any corpus. A different number is a defect to explain
  before release, in the engine or in the prototype.

## Mutation operators

- DR-M1 the draft rule never applies: d_f01.
- DR-M2 identifier equality not checked: d_f02.
- DR-M3 guard keywords not checked: d_f03.
- DR-M4 every boolean read as a draft-06 subschema: d_f07.
