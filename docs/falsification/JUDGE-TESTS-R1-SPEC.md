# JUDGE-TESTS r1: a test document is a break only when two validators say so (r1, FROZEN)

Ratified by the founder 2026-11-07. Frozen with three fixtures added to
crates/invar-falsify/tests/judge_tests_test.rs and one real SchemaStore
change BEFORE implementation; freeze record: JUDGE-TESTS-R1-SPEC.frozen.
JUDGE-TESTS r0 stays frozen as written; r1 adds to it.

## Why

r0 judged with Python jsonschema alone. Amendment C ran Ajv 8 over every
witness file: Ajv agreed on 21 of 28, and for 4 of the 19 changes it
accepted at the head what jsonschema rejected, or could not compile the
head at all. A rejection is one validator's word until a second agrees.
The Action must say so itself, not a footnote.

## Ruling TESTS r1

`action/tests.py` keeps every rule of r0 and adds a second validator,
Ajv 8 (`ajv` 8.20.0, `ajv-draft-04` 1.0.0, `ajv-formats` 3.0.1; strict
mode off; formats not asserted), run through one Node adapter,
`action/tests-ajv.mjs`, that receives the two schemas and the parsed
documents as JSON on stdin and answers, per document and side, `valid`,
`invalid` with the first error's pointer and message, or `error` with the
reason. Classification stays in `tests.py`, one owner.

- A document is a promise only when both validators accept it at the
  base. It is a witness only when both reject it at the head.
- Any split, on either side, is `disputed`: the file and both answers are
  listed; a disputed document is never a witness and never a pass.
- A validator error on either side is `unverifiable` with its reason, as
  in r0.
- VIOLATED means two validators agree. With one validator only
  (`--validators jsonschema`, or `node` or the adapter's packages
  missing), the script cannot say VIOLATED: every rejection is
  `unconfirmed` under NOT EVALUATED, and the first line names the
  validator that ran. The Action installs the packages itself and treats
  a failed install as exit 3.
- Reasons, in order: `no_instances`, `no_base_valid_instance`,
  `disputed`, `unverifiable`, `instances_pass`; `unconfirmed` replaces
  `disputed` and `unverifiable` in single-validator mode.

Text: the first line of r0, followed in two-validator mode by nothing
new; in single-validator mode `NOT EVALUATED (unconfirmed): N
document(s) rejected by jsonschema alone; a break needs two validators.`
Witness lines carry both messages: `  rejected: <file> at <pointer>:
<jsonschema message> | ajv: <ajv message>`. Disputed lines:
`  disputed: <file>: jsonschema <answer>, ajv <answer>`. JSON gains
`validators` (the list that ran), `disputed` (`[{file, jsonschema, ajv}]`)
and `ajv_message` on each witness; `instances` gains `disputed`.

## Fixtures (judge_tests_test.rs, added)

- t_f16 real bytes, es6importsorterrc.json at f9d455e8c ("Fixed
  duplicate IDs") with its own es6importsorterrc-test.json: jsonschema
  rejects at the head, Ajv accepts: NOT EVALUATED `disputed`, exit 4, the
  file named with both answers (kills TS-M5, TS-M6).
- t_f17 `--validators jsonschema` on t_f01's documents: NOT EVALUATED
  `unconfirmed`, exit 4, never 1; the first line names jsonschema (kills
  TS-M7).
- t_f18 t_f01's documents with both validators: VIOLATED, exit 1, the
  witness carries `ajv_message`.

## Gates (pre-registered)

- 18 of 18 fixtures; full suite; mutants TS-M5..M7 killed with every
  earlier one; clippy silent; the Rust judge unchanged on every corpus.
- Prediction: on SchemaStore at v0.2.15 (819 changes), exactly 15 changes
  VIOLATED and 4 `disputed`, the sets Amendment C names; no HOLDS change
  with a witness. Every other count is measured and recorded, not
  predicted.

## Mutation operators

- TS-M5 Ajv's answer is ignored (the jsonschema answer is copied as
  Ajv's): t_f16.
- TS-M6 a split is reported as a witness: t_f16.
- TS-M7 single-validator mode exits 1 on a rejection: t_f17.

## Out of scope in r1

A third validator; resolving sibling `$ref`s; cross-checking the static
judge's counterexamples with Ajv (its own ruling).

## Measured outcome (v0.2.18 candidate, 2026-11-07)

judge_tests_test 18 of 18; full suite 814 passed; mutants 86 of 86
(TS-M5..M7 each killed by its named fixture); clippy `-D warnings` silent;
the Rust judge unchanged.

Two defects found by running the script over SchemaStore before release,
fixed without touching a fixture:

- jsonschema was retrieving remote `$ref`s over the network by default,
  against the frozen rule that nothing outside a schema document is
  resolved. Validation now runs against an offline registry; an
  unresolvable reference is `unverifiable`, on either validator. Amendment
  C's lychee verdict ("jsonschema rejects at the head") was reached after
  such a fetch; offline, lychee is unverifiable by both validators.
- Ajv compiles the whole schema before validating, so one unresolvable
  reference anywhere makes every document of that change `unverifiable`
  under Ajv, where jsonschema errors only when the reference is reached.
  Stated, not hidden: `unverifiable` rose from 12 changes to 113.

The prediction held on both counts that bear on soundness and failed on
the third: exactly the 15 named changes are VIOLATED and no HOLDS change
has a witness; `disputed` is 14 changes, not 4. The 10 unpredicted ones
(function.json, host.json and web-manifest across several commits,
webjob-publish-settings at b37ff9eed, es6importsorterrc at 1983458b5) are
documents jsonschema rejects at the base while Ajv accepts them, on
draft-04 schemas. r0 had filed those as "not promises" on one validator's
word and said nothing; Amendment C could not see them, because it ran Ajv
only on documents jsonschema had rejected at the head. r1 surfaces them,
which is the point of a second validator. Per change: tests-moves/
schemastore-r1.tsv. Remaining counts, measured: 447 `instances_pass`, 11
`no_base_valid_instance`, 219 with no test directory.
