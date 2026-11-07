# JUDGE-TESTS r0: a document the base accepted and the head rejects is a break, whoever wrote it (r0, FROZEN)

Ratified by the founder 2026-11-07. Frozen with
crates/invar-falsify/tests/judge_tests_test.rs and two real SchemaStore
changes with their own test files BEFORE implementation; freeze record:
JUDGE-TESTS-SPEC.frozen. Extends nothing in the Rust judge, which stays
as released in v0.2.15.

## Why

After JUDGE-ITEMS r0, 597 of SchemaStore's 819 changes in six months are
NOT EVALUATED: the static judge does not reach them. SchemaStore keeps
positive test documents beside every schema. Measured before this spec
(scratch prototype, Python jsonschema, format not asserted): of those
597, 11 have a base-valid test document the head rejects; 418 have every
base-valid test passing at the head; 9 are unverifiable there; 135 have
no tests and 24 no base-valid test. Of the 174 HOLDS, none is contradicted
and none unverifiable. Of the 48 VIOLATED, 8 are corroborated.

A test document is a promise the schema's own maintainers wrote down.
Validating it on both sides needs a whole JSON Schema validator; the Rust
crate has none and takes no new dependency, so this check lives where a
mature validator already serves INVAR's independent checks: the Python
harness, as one file.

## Ruling TESTS (r0)

`action/tests.py BASE HEAD DIR [--format text|json|github]` reads every
file in DIR, in name order, as an instance document:

- `.json` by the standard library; `.toml` by `tomllib`; `.yaml` and
  `.yml` by PyYAML. A file of any other extension, a file that does not
  parse, or a YAML file with no reader installed is `unreadable`, named
  with its reason.
- An instance BASE rejects is not a promise: counted, never judged.
- An instance BASE accepts and HEAD rejects is a witness: the file, the
  head's first error as a JSON Pointer, and the validator's message.
- An instance BASE accepts and HEAD accepts passes.
- An instance whose validation raises on either side (a regex feature
  Python's `re` lacks, a `$ref` that does not resolve) is `unverifiable`,
  named with the exception's reason. No reference outside the schema
  document is resolved in r0: 9 of SchemaStore's 597 unchecked changes
  refer to sibling files, and they stay unverifiable, stated.
- Validation uses `jsonschema.validators.validator_for` on the schema's
  `$schema` (Draft 7 when absent); `format` is never asserted, as in the
  judge.

The verdict is one of two, never HOLDS: a corpus of documents is not
exhaustive, so passing instances prove nothing about the rest.

- VIOLATED, exit 1, when any witness exists; every witness is listed.
- NOT EVALUATED, exit 4, otherwise, with one reason: `no_instances` (no
  readable file), `no_base_valid_instance`, `unverifiable` (some
  base-valid instance could not be checked at the head), or
  `instances_pass`.
- Exit 3 when BASE or HEAD cannot be read or parsed, or `jsonschema` is
  not importable.

Text, first line: `VIOLATED: N document(s) valid at BASE are rejected by
HEAD.` or `NOT EVALUATED (<reason>): <sentence>.`; then one line per
witness `  rejected: <file> at <pointer>: <message>`; then one line per
unreadable or unverifiable file `  not checked: <file>: <reason>`; then
`  instances: <total> read, <n> not promises, <n> pass, <n> not checked`.
JSON: `{"verdict","reason","base","head","witnesses":[{"file","pointer",
"message"}],"not_checked":[{"file","reason"}],"instances":{"read",
"not_promises","pass","not_checked"}}`, keys sorted, witnesses in file
order. GitHub: one `::error file=<HEAD>::` line per witness, one
`::warning::` line naming the not-checked files when any.

In the Action, the input `judge_tests` names the instance directory for
a contract as a pattern with `{name}` for the contract's file stem (for
SchemaStore, `src/test/{name}`), read from the base ref. For each
contract `judge_contracts` judged, the Action runs this check; VIOLATED
annotates each witness and fails the job (unless `fail_on_violation` is
false); `unverifiable` is one warning; `instances_pass`, `no_instances`
and `no_base_valid_instance` are one summary line each, no annotation.
The static verdict is never changed by this check; the two stand side by
side in the summary.

## Fixtures (judge_tests_test.rs)

Each runs `python3 action/tests.py` from the crate's test binary.

- t_f01 one instance valid at the base, rejected at the head: VIOLATED,
  exit 1, the file, pointer and message (kills TS-M1).
- t_f02 every instance passes: NOT EVALUATED `instances_pass`, exit 4,
  never 0 (kills TS-M2).
- t_f03 an instance the base rejects, which the head also rejects: not a
  promise, no witness (kills TS-M3).
- t_f04 an empty directory: `no_instances`.
- t_f05 only base-invalid instances: `no_base_valid_instance`.
- t_f06 a passing instance beside an unparsable file: `unverifiable`, the
  file named with its reason (kills TS-M4).
- t_f07 a TOML instance is read and judged.
- t_f08 a YAML instance is read and judged.
- t_f09 a `$ref` to a sibling file: `unverifiable`, the reason names the
  reference.
- t_f10 `format` is not asserted: an instance the head's `format: email`
  would reject still passes.
- t_f11 two rejected instances: both listed, in file order, in text and
  JSON.
- t_f12 `--format github`: one `::error` per witness.
- t_f13 an unparsable head schema: exit 3.
- t_f14 real bytes: github-action.json at d9d98e698 ("Removed
  github-action parallelization keywords") with its own
  parallel-steps.json: VIOLATED at `/runs`.
- t_f15 real bytes: ryl.json at dd22b5c09 ("update schema to v0.17.0")
  with its own 01-valid.toml: VIOLATED at the root, the message naming
  `yaml-files`.

## Gates (pre-registered)

- Full suite, clippy `-D warnings` silent, every mutant killed (TS-M1..M4
  new, applied to action/tests.py).
- The Rust judge's verdicts are unchanged on every corpus: this check
  adds nothing to the crate.
- Prediction, from the scratch prototype run before this file: on
  SchemaStore at v0.2.15 (819 changes, instances from each base commit),
  exactly 19 changes have a witness: the 11 named in the freeze record
  from NOT EVALUATED and these 8 from VIOLATED: pnpm-workspace 18132a01a,
  claude-code-keybindings 2f74efc3f, crucible-code-schema 582d198b3 and
  64a83f4a0, codex-hooks 9497fde91, traefik-v3 b056a70c8,
  claude-code-settings cfd4af801, ttheme-palette ec65ccfaf. No HOLDS
  change has a witness. A different set is a defect to explain before
  release, in the script or in the prototype.

## Mutation operators

- TS-M1 a head rejection is never recorded: t_f01.
- TS-M2 passing instances exit 0: t_f02.
- TS-M3 base validity is not checked: t_f03.
- TS-M4 an unreadable file is counted as passing: t_f06.

## Amendment A (after measurement, fixtures unchanged): three harness facts

Running the script over SchemaStore before release found three things the
ruling did not say, each now stated:

- The schema is checked against its metaschema without format assertion.
  A `pattern` Python's `re` cannot compile (`(?<name>...)`, `\p{...}`) is a
  valid schema; each document it guards is `unverifiable`, never the
  whole change an error. The first run turned global.json, circleciconfig
  and appsettings into exit 3 and missed global.json's break.
- One validation error is enough on either side: a document is a promise
  or not, a break or not. Enumerating every error timed out on large
  schemas where the prototype, which stopped at the first, did not. The
  witness carries the first error in the validator's own order.
- The Action gives each contract a time budget (`tests_timeout`, 120
  seconds). Over budget is NOT EVALUATED with a warning, never a pass.
  azure-iot-edge-deployment-template 2.0 and 4.0 at 03ace7195 take about
  350 seconds a side in Python jsonschema, every document valid on both.

## Measured outcome (v0.2.16 candidate, 2026-11-07)

judge_tests_test 15 of 15; full suite 811 passed; mutants 83 of 83
(TS-M1..M4 each killed by its named fixture); clippy `-D warnings` silent;
the Rust judge unchanged, so every corpus verdict is unchanged.

The prediction holds after Amendment A: on SchemaStore at v0.2.15,
exactly the 19 named changes carry a witness, 11 from NOT EVALUATED and 8
from VIOLATED, and no HOLDS change does. The rest: 551 `instances_pass`,
16 `no_base_valid_instance`, 12 `unverifiable`, 219 with no test
directory, 2 over the time budget. Witness files per change are in
tests-moves/schemastore.tsv.

## Amendment B (after measurement): ten replay labels corrected

The replay corpus labels commits by what the static judge should say.
Ten commits now carry a break the static judge cannot reach, proven by
the commit's own test documents: eight labelled holds (specmatic, global,
pull-request-labeler, webjob-publish-settings, vtesttree-v2.4.0,
github-action, ryl, es6importsorterrc), one stated (sil-kit), and one
false_alarm (lychee, whose own lychee.toml the head rejects: the hand
label had excused a real break). They are relabelled `real_by_tests`,
with its own gate line: such a commit is never HOLDS on every row. With
it, SchemaStore replay: PASS (real VIOLATED 21 of 21, real by tests 10 of
10, false alarms VIOLATED 0 of 55, holds newly VIOLATED 0 of 386).

## Amendment C (after release, v0.2.17): one validator's word is one validator's word

Every witness file of the 19 changes was run through a second validator,
Ajv 8 (SchemaStore's own CI validator; strict off, formats not asserted).
Ajv agrees on 21 of 28 files, covering 15 changes: 7 beyond the static
judge's reach and the 8 it already judged VIOLATED. For webjob-publish-
settings (8cfea30d1), vtesttree-v2.4.0 (af686ddb1) and es6importsorterrc
(f9d455e8c) Ajv accepts at the head the document jsonschema rejects; for
lychee (fc3956739) Ajv cannot compile the head, which `$ref`s a remote
URL. Those four are `disputed`, counted and never asserted; `real_by_tests`
now means two validators agree. The v0.2.16 README's "11 breaks" was the
single-validator count and is corrected. The Action's check still uses
jsonschema alone and says so; a second validator in the Action is r1
work, not assumed.
