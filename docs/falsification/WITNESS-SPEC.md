# WITNESS r0: `invar witness` writes counterexamples as SchemaStore test files (r0, FROZEN)

Drafted 2026-11-08 for ratification. Frozen with
crates/invar-falsify/tests/witness_test.rs BEFORE implementation; freeze
record: WITNESS-SPEC.frozen. No judge ruling changes: this is a surface
over verdicts the judge already gives.

## Why

A VIOLATED verdict carries a counterexample: a document valid at the base
and rejected at the head, confirmed by two validators in the harness.
SchemaStore requires positive and negative test documents for its schemas
(`src/test/<name>/`, `src/negative_test/<name>/`, CONTRIBUTING.md) and
runs them in CI. A maintainer acts on a finished test file in minutes and
on an issue in weeks, if ever. So the counterexample should arrive as the
file their CI already expects, under their layout, with nothing to edit.

## Ruling WITNESS (r0)

`invar witness BASE HEAD --name NAME --out DIR [--positive]` judges BASE
and HEAD exactly as `invar judge BASE HEAD` does and writes every witness
that carries a counterexample as one file:

- `DIR/src/negative_test/NAME/invar-NN-<slug>.json` by default: the head
  rejects the document, so it is a negative test of the head (the break,
  documented);
- `DIR/src/test/NAME/invar-NN-<slug>.json` with `--positive`: for a pull
  request that restores what the base accepted, the same document is a
  positive test of the fixed schema.

`NN` counts from 01 in the judge's own witness order; `<slug>` is the
witness path with every run of characters outside `[A-Za-z0-9]` replaced
by one `-`, trimmed, `[]` read as `items`, at most 60 characters. The
file is the counterexample alone: pretty JSON, two-space indent, keys in
sorted order, one trailing newline (what SchemaStore's prettier produces
for JSON). Same inputs, same bytes: a second run overwrites with identical
files. Directories are created as needed.

Output, one line per file written: `wrote <path relative to DIR>: <kind>
<witness path>, valid at BASE, rejected at HEAD`; then one line per
witness without a counterexample: `no document for <kind> <witness
path>: <reason>`; then `verdict <VERDICT>: N document(s) written, M
witness(es) without one`. Exit 0 when the judge ran, whatever it found
(a generator, not a verdict: `invar judge` is the verdict); exit 3 on an
unreadable file, a NAME that is not a single path component, or an
unwritable DIR. Nothing is written on exit 3.

Not in r0: confirmation of the documents by the two validators (the
harness does that, `action/tests.py` and the counterexample check);
YAML or TOML test files; a document for witnesses that carry none.

## Fixtures (frozen before implementation)

- w_f01 one enum narrowing: exactly one file,
  `src/negative_test/n/invar-01-a.json`, bytes `{\n  "a": "b"\n}\n`, the
  wrote line, the verdict line, exit 0.
- w_f02 `--positive`: the same file under `src/test/n/`.
- w_f03 HOLDS: no file, `verdict HOLDS: 0 document(s) written, 0
  witness(es) without one`, exit 0.
- w_f04 a pattern change (a witness without a counterexample): no file,
  the `no document for` line, exit 0.
- w_f05 two breaks: `invar-01-...` and `invar-02-...` in the judge's
  order; an array element path `t[]` slugs to `t-items`.
- w_f06 guard: `--name a/b` exits 3 and writes nothing.
- w_f07 idempotence: running twice leaves identical bytes.

## Mutation operators

- W0-M1 the negative directory written under `src/test`      -> w_f01
- W0-M2 a witness without a counterexample written as `null`  -> w_f04
- W0-M3 the slug not applied (raw path in the file name)      -> w_f05

## Pre-registered prediction

No verdict moves on any corpus (the judge is untouched). On the 15
two-validator breaks in SchemaStore's history, `invar witness` writes
every counterexample the judge carries for them; the count is reported
in the Measured outcome, not predicted, because it depends on which
witnesses carry a document.
