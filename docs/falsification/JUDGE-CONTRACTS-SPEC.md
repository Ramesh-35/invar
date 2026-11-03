# JUDGE-CONTRACTS r0: judge every contract a change touches, in one pass (r0, FROZEN)

Founder go 2026-11-02, with D6 (epistemic voice) ratified into the same
build. Frozen with crates/invar-falsify/tests/judge_contracts_test.rs
BEFORE implementation; freeze record: JUDGE-CONTRACTS-SPEC.frozen.

Why: the judge took one contract per run. SchemaStore keeps over 1,000
schemas and a pull request touches whichever it touches; a one-file
input cannot follow that. A glob that matches nothing must never read as
a pass, and an unchanged contract must cost nothing.

## Surface

    invar judge --contracts '<glob>' [--contracts '<glob>' ...] --base <ref> [--format text|json|sarif|github]

`--contract <path>` keeps working exactly as before (plus D6). The two
cannot be combined. `--receipt` with `--contracts` is an error in r0
(exit 3, stated): one receipt per contract is a later decision.
Action input: `judge_contracts: '<glob>'`.

## Decisions taken in this spec (overrule before implementation if wrong)

- M-D1 Matching uses git pathspec glob semantics (`*` within a path
  segment, `**` across segments), no new dependency.
- M-D2 Selected: files matching a glob that differ between the base tree
  and the working tree, including added, deleted and untracked files not
  ignored by git. Files identical at both are not judged; their count is
  stated.
- M-D3 A contract deleted by the change: NOT EVALUATED, reason
  `contract_deleted`. Deleting a contract withdraws everything it
  promised; INVAR states it and the Action fails the job, as it does for
  `unparsable`.
- M-D4 A glob that matches no file at the base ref or in the working tree
  is an error (exit 3): a misconfigured glob must not read as a pass.
  Matching files with none changed: exit 0, "0 changed" stated.
- M-D5 Aggregate exit: 1 if any contract is VIOLATED, else 4 if any is
  NOT EVALUATED, else 0. Every contract's own verdict is reported.
- M-D6 Order: contracts sorted by path; output is byte-stable.

## Outputs

- text: each judged contract's block exactly as `--contract` prints it,
  in path order, then one summary line:
  `judged N changed contracts under <globs> at <base>: a VIOLATED, b HOLDS, c NOT EVALUATED; M unchanged, not judged.`
- json: `{"base", "globs", "contracts": [<the --contract JSON body per
  contract>], "summary": {"violated", "holds", "not_evaluated",
  "unchanged"}}`.
- github: every contract's lines, each with its own `file=`.
- sarif: one run; results from every contract, each located in its own
  file; one notification per NOT EVALUATED contract;
  `run.properties.contracts` lists each path with its verdict.

## D6 (epistemic voice, ratified 2026-11-02)

INVAR does not know who reads a contract: a pipeline parsing payloads or
an editor validating a config file. A sentence states the structural
change and what it does to validity, never a guess about a reader. Kind,
path, types and line are unchanged; the removed sentence gains the
structural reason it is a break (D2: the object admits no other keys);
every consequence is restated:

- removed: `removed: <path> was <type> (line <n> at <base>), from an object that admits no other keys. Instances that carry it, valid before, are now rejected; keep the field, or version the change.`
- retyped: `retyped: <path> was <a>, is now <b>. Values of one type but not the other change between valid and invalid; restore the type, or version the change.`
- narrowed (values lost): `narrowed: <path> no longer accepts <values>. Instances using them, valid before, are now rejected; keep the values, or version the change.`
- narrowed (enum added): `narrowed: <path> now accepts only <set>. Values outside that set, valid before, are now rejected; drop the enum, or version the change.`
- escalated: `escalated: <path> is now required. Instances that omit it, valid before, are now rejected; keep it optional, or version the change.`
- tightened and changed: unchanged (already structural).

## Fixtures (judge_contracts_test.rs)

- jc_f01 three schemas: one widened, one narrowed, one untouched:
  exit 1; exactly two judged (widened HOLDS, narrowed VIOLATED); one
  unchanged stated (kills JC-M1).
- jc_f02 `--format github`: every `::error` carries the narrowed file;
  none for the widened or the untouched one.
- jc_f03 an added schema: `new_contract`, exit 0 when it is the only
  change.
- jc_f04 a deleted schema: NOT EVALUATED `contract_deleted`, exit 4.
- jc_f05 a glob matching nothing: exit 3 (kills JC-M3).
- jc_f06 matching schemas, none changed: exit 0, "0 changed" stated.
- jc_f07 an untracked new schema is judged (kills JC-M4).
- jc_f08 one VIOLATED and one unparsable: exit 1, both stated (kills
  JC-M2).
- jc_f09 SARIF: one run, results located in two different files.
- jc_f10 D6: the removed and retyped sentences are exactly as above; no
  output says "Consumers", "Senders", "Payloads" or "misread".
- jc_f11 `--contract` and `--contracts` combined: exit 3.

## Mutation operators

- JC-M1 unchanged-file filter dropped: jc_f01.
- JC-M2 aggregate takes the last verdict, not the worst: jc_f08.
- JC-M3 a glob matching nothing passes: jc_f05.
- JC-M4 untracked files ignored: jc_f07.

## Amendment A (stated during implementation, 2026-11-02; fixtures unchanged)

The frozen text contradicted the frozen fixtures, and the fixtures are
right. M-D5 said any NOT EVALUATED contract gives exit 4; jc_f03 and
jc_f07 (frozen with it) expect exit 0 when the only change is a new
contract. A new contract promised nothing before, which is why the
Action already passes `new_contract` for a single contract. Amended:

- M-D5: exit 1 if any contract is VIOLATED; else 4 if any is NOT
  EVALUATED for a reason other than `new_contract`; else 0. A new
  contract is still reported as NOT EVALUATED (`new_contract`), and
  counted.
- Summary line: `judged N changed contracts under <globs> at <base>: a VIOLATED, b HOLDS, c NOT EVALUATED (d new); M unchanged, not judged.`
  JSON `summary` adds `new`.
- Implementation note: `git ls-tree` does not accept glob pathspecs, so
  the base tree is listed as a diff from the empty tree, through the same
  matcher as every other step (M-D1 unchanged).

Mutation operators added (each killed by an existing frozen fixture):
- JC-M5 a new contract moves the exit code: jc_f03.
- JC-M6 a deleted contract passes: jc_f04.
- JC-M7 D6 reverted (the removed sentence guesses at readers): jc_f10.

The test file's hash is unchanged; the spec's new hashes are appended to
JUDGE-CONTRACTS-SPEC.frozen, the original lines kept.
