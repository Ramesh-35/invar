# JUDGE-TESTS r2: the Ajv side reads every draft the Python side reads (r2, FROZEN)

Drafted 2026-11-08 for ratification. Frozen with the four `t2_` fixtures
appended to crates/invar-falsify/tests/judge_tests_test.rs BEFORE
implementation; freeze record: JUDGE-TESTS-R2-SPEC.frozen. Extends
JUDGE-TESTS r1; the Rust judge is untouched.

## Why

`action/tests-ajv.mjs` (r1) builds `new Ajv(...)` for every schema except
draft-04. Ajv 8's default class knows draft-07 only: a schema whose
`$schema` names 2019-09 or 2020-12 fails to compile ("no schema with key
or ref ..."), and so does one whose `$schema` is a URI Ajv has never
seen. The Python side reads 2019-09 and 2020-12 natively and falls back
to draft-07 for an unknown URI. Every such schema was therefore
`unverifiable` on the Ajv side, and no document of it could be a witness
or a pass, whatever it did. Of the 599 SchemaStore changes with test
documents in INVAR's corpus, 43 are under 2019-09 or 2020-12 and 3 name
no draft; they are part of the 113 `unverifiable` changes r1 reported.
Measured today on draft-07 and 2019-09 the two validators also disagree
on `$ref` siblings (REF-SIBLINGS r0), which is why the harness must read
both drafts correctly before any ruling leans on it.

## Ruling TESTS (r2)

In `tests-ajv.mjs`, the class follows the schema's `$schema`:

- draft-04: `ajv-draft-04` (as in r1);
- draft-06: `Ajv` with the draft-06 meta-schema added
  (`ajv/dist/refs/json-schema-draft-06.json`);
- draft-07, no `$schema`, or a `$schema` URI that names none of these:
  `Ajv` (draft-07), with the unknown URI removed from the compiled copy so
  Ajv does not try to resolve it; this is what the Python side does
  (`validator_for(..., default=Draft7Validator)`);
- 2019-09: `Ajv2019` (`ajv/dist/2019`);
- 2020-12: `Ajv2020` (`ajv/dist/2020`).

Options stay `strict: false, allErrors: false, validateFormats: false`,
`ajv-formats` added, as r1. Classification in `tests.py` is unchanged:
witness, disputed, unconfirmed, unverifiable mean what r1 says. Pinned
versions unchanged (ajv 8.20.0, ajv-draft-04 1.0.0, ajv-formats 3.0.1):
the two newer classes ship inside ajv 8.

## Fixtures (frozen before implementation)

- t2_f01 a 2019-09 pair, one document rejected at the head: VIOLATED,
  both validators named, `not_checked` empty.
- t2_f02 a 2020-12 pair using `prefixItems` with `items: false` (read as
  the specification says only by the 2020-12 class): VIOLATED, nothing
  disputed.
- t2_f03 a draft-06 pair: VIOLATED, `not_checked` empty.
- t2_f04 an unknown `$schema` URI: read as draft-07 by both, VIOLATED.
  (Today Ajv fails to compile it; the Python side already reads it as
  draft-07, so today this is `unverifiable`.)

## Mutation operators

- TS2-M1 the draft switch dropped (every schema read as draft-07)  -> t2_f01
- TS2-M2 2020-12 read with the 2019-09 class                        -> t2_f02

## Pre-registered prediction

On the SchemaStore test-document corpus (`tests-predict-r1.py`, the 599
changes with test documents, both validators): the 113 `unverifiable`
changes of r1 fall; the fall is at most the 46 whose head is 2019-09,
2020-12 or names no draft, because those are the only schemas this
ruling changes the reading of. Which of them become `instances_pass`,
`disputed` or VIOLATED is measured, not predicted: before r2 no document
of theirs has ever been checked by Ajv. No VIOLATED or `instances_pass`
change of r1 moves. The Rust judge's corpora and gates do not move.
