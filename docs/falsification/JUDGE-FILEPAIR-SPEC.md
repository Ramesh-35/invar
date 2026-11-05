# JUDGE-FILEPAIR r0: judge two files, no git anywhere (r0, FROZEN)

Ratified by the founder 2026-11-04. Frozen with
crates/invar-falsify/tests/judge_filepair_test.rs BEFORE implementation;
freeze record: JUDGE-FILEPAIR-SPEC.frozen.

## Why

`invar judge` needs a git repository and a base ref. That is right for
CI, where the base tree is the trust anchor (SIB-3). It is friction for
the first five minutes: someone with two versions of a schema on disk
should be able to ask the question directly. The two-file mode removes
every setup step without touching what is judged or how.

## Ruling FP (r0)

`invar judge <base.json> <head.json>` takes exactly two positional
paths: first the base (the promise), then the head (the change). No git
repository is needed or consulted.

- The engine is the repository mode's, by construction: the same judge,
  the same rulings (D1-D7, D9, J1-J3, r1, r2), the same counterexample
  build, the same verdicts and exit codes (HOLDS 0, VIOLATED 1, error 3,
  NOT EVALUATED 4). Nothing in the two-file mode may exit 2.
- The labels are the paths as given: the base path wherever the
  repository mode prints the ref ("made at a.json", "line 1 at a.json",
  "valid at a.json, rejected now", JSON "base"), the head path wherever
  it prints the contract name (JSON "contract", witness "contract", the
  github and sarif file labels).
- `--format` works unchanged. `--receipt` is refused in r0 (receipts
  record a repository's base tree, which two loose files do not have):
  `ERROR (not a verdict): --receipt needs a repository; judge two files
  without it.` and no file is written.
- The modes do not mix. Two files with `--contract` or `--contracts`:
  `ERROR (not a verdict): two files and --contract cannot be combined:
  judge the two files, or judge a contract in this repository.` Two
  files with an explicit `--base`: `ERROR (not a verdict): two files and
  --base cannot be combined: the first file is the base.` One positional
  path alone is likewise an error, never a guess at a default.
- An unreadable file is `ERROR (not a verdict): reading <path>: <cause>`
  and a file that does not parse is `ERROR (not a verdict): <path> is
  not JSON: <cause>`, both exit 3.

Out of scope in r0, stated so nothing is implied: receipts, locks,
`invar.toml`, globs over directories, and any change to the replay
scripts (they keep building temp repositories, so their history of
results stays comparable; moving them to the two-file mode is a later
slice with its own gate).

## Fixtures (judge_filepair_test.rs)

All run in temp directories that are never git repositories, except the
repository half of fp_f08.

- fp_f01 removed field in a closed object: VIOLATED, file labels in the
  verdict, witness and counterexample lines (kills FP-M1, FP-M2).
- fp_f02 identical files: HOLDS, file labels.
- fp_f03 D7 parity: `integer` to `integer|null` is a stated note.
- fp_f04 D9 parity: a type the enum already implies is a stated note.
- fp_f05 missing base file: exit 3, `reading a.json:`.
- fp_f06 head not JSON: exit 3, `b.json is not JSON:`.
- fp_f07 two files + `--contract`, and two files + `--base`: both exit 3
  with the exact sentences above.
- fp_f08 epistemic parity: the same bytes through the repository mode
  and the two-file mode give byte-equal JSON verdicts and witnesses once
  the label fields (`base`, `contract`) are removed; the case exercises
  oneOf reading and a tightened bound with its counterexample.
- fp_f09 JSON labels are the file paths; a pattern change stays
  `constraint_changed` with reason `pattern and format changes are not
  decided`, no counterexample.
- fp_f10 a change only in unjudged constructs: exit 4, NOT EVALUATED
  (kills FP-M3).
- fp_f11 `--receipt` with two files: exit 3, the exact sentence, and no
  receipt file written.

## Gates (pre-registered)

- Full suite, clippy `--workspace --all-targets -- -D warnings` silent,
  every mutant killed (FP-M1..M3 new).
- SchemaStore replay PASS, labels unchanged; Iglu replay PASS, the same
  5 ADDITION pairs, 3 confirmed, 0 failed, 0 errors. Both replays run
  unmodified: this slice must not change any replay script.
- README gains one short paragraph for the mode; no other claims change.

## Mutation operators

- FP-M1 file order swapped (the head judged as the base): fp_f01 (the
  removed field becomes an added one and would hold).
- FP-M2 base label falls back to "main": fp_f01.
- FP-M3 the base read from the head path (no change ever seen): fp_f10.
