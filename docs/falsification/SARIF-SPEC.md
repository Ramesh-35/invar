# JUDGE-SARIF r0: the verdict inside the pull-request diff (r0, FROZEN)

Rulings S-D1..S-D4 ratified 2026-10-30. Frozen with
crates/invar-falsify/tests/judge_sarif_test.rs BEFORE implementation;
freeze record: SARIF-SPEC.frozen. Backlog 7b, licensed by Brief 5: when
platform LLM reviewers become the default, the deterministic verdict
they cite must arrive in formats they and every CI system already read.
No new verdict logic: every result is an existing judge witness.

## Surfaces

1. `invar judge --format sarif`: a SARIF 2.1.0 log on stdout, for code
   scanning upload, GitLab, IDEs and agents.
2. `invar judge --format github`: GitHub workflow commands, one line per
   witness, which the Action prints so each break is annotated on the
   pull request's Files changed tab with no code scanning setup, on
   public and private repositories alike.
Exit codes unchanged for every format.

## Mapping (SARIF)

- `$schema` https://json.schemastore.org/sarif-2.1.0.json, `version`
  2.1.0, exactly one run.
- `tool.driver`: name `invar`, the engine version, informationUri the
  repository URL, and the six rules `invar.judge.removed`,
  `invar.judge.retyped`, `invar.judge.narrowed`, `invar.judge.tightened`,
  `invar.judge.escalated`, `invar.judge.constraint_changed` (S-D3),
  each with a one-line description.
- One result per witness: `ruleId`, `level` error (S-D4), `message.text`
  equal to the exact sentence the text format prints for that witness
  (one voice, every encoding), one location: the contract file and a
  line.
- Line (S-D1): the witness's line in the POST file; for a removed field,
  which has no post line, the line of its parent's key in the post file
  (the root `properties` key for a top-level field), so the annotation
  sits inside the reviewer's diff where the field used to live; line 1
  only when no anchor is found. The message keeps the base-ref line.
- `run.properties`: `unjudged_constraints`, `verdict`, `base`.
- NOT EVALUATED (S-D2): zero results, `invocations[0].executionSuccessful`
  false, one `toolExecutionNotifications` entry with level warning,
  `descriptor.id` the reason code and the plain sentence; exit 4. An
  empty SARIF log must never read as clean.
- Byte-stable: no timestamps, no absolute paths, sorted keys and
  deterministic order; two runs on the same inputs are identical.

## Mapping (GitHub annotations)

- VIOLATED: per witness, `::error file=<contract>,line=<line>,title=invar.judge.<kind>::<sentence>`
  with the same line rule as SARIF.
- NOT EVALUATED: one `::warning file=<contract>,title=invar.judge.not_evaluated::<sentence>` (S-D4).
- Messages escape `%`, CR and LF as GitHub requires.

## Meaning of the levels

`error` means a witnessed violation of the declared contract, located
and reproducible from the receipt. `warning` means the judge could not
evaluate, with the reason stated. Neither claims more than that.

## Fixtures (judge_sarif_test.rs)

- sf_f01 SARIF 2.1.0 shape: version, $schema, one run, driver `invar`,
  the six rule ids.
- sf_f02 real b1-002: one result, ruleId invar.judge.retyped, message
  equal to the text sentence.
- sf_f03 real b1-001: the `volume` retype is located at its POST line
  (115), not its base line (62) (kills SF-M3).
- sf_f04 removed fields: b1-001 `isLive` anchors to the post root
  `properties` line; a nested removal anchors to its parent's key line.
- sf_f05 unparsable post: zero results, executionSuccessful false,
  notification id unparsable at level warning, exit 4 (kills SF-M2).
- sf_f06 two runs byte-identical.
- sf_f07 unjudged_constraints in run properties.
- sf_f08 `--format github`: an ::error line with file, line, title and
  the sentence; a NOT EVALUATED run gives one ::warning line.

## Mutation operators

- SF-M1 results dropped: killed by sf_f02.
- SF-M2 NOT EVALUATED emitted as a successful run: killed by sf_f05.
- SF-M3 base-file line used where the post-file line belongs: sf_f03.
