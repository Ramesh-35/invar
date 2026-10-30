# JUDGE-SARIF r0: the verdict inside the pull-request diff (r0 DRAFT, NOT FROZEN)

Backlog 7b, licensed by Brief 5: when platform LLM reviewers become the
default, the deterministic verdict they cite must arrive in the formats
they and every CI system already read. Two surfaces, one source of truth:
the judge's existing witnesses. No new verdict logic.

## Surfaces

1. `invar judge --format sarif`: a SARIF 2.1.0 log on stdout, for code
   scanning upload, GitLab, IDEs and agents. Exit codes unchanged.
2. The Action's judge step emits one GitHub workflow annotation per
   witness (`::error file=...,line=...,title=...::message`), visible on
   the pull request's Files changed tab with no code scanning setup,
   on public and private repositories alike.

## Mapping

- One run; `tool.driver` = name `invar`, the engine version,
  `informationUri` the repository URL; `rules` = the six witness kinds,
  rule ids `invar.judge.<kind>` (removed, retyped, narrowed, tightened,
  escalated, constraint_changed), each with a one-line description.
- One result per witness: `ruleId`, `level: error`, `message.text` = the
  exact sentence the text format prints (one voice, two encodings),
  `locations[0]` = the contract file and a line (S-D1).
- `run.properties.unjudged_constraints` = the stated unjudged list.
- NOT EVALUATED: zero results, `invocations[0].executionSuccessful`
  false, and a `toolExecutionNotifications` entry carrying the reason
  code and sentence; exit 4. An empty run must never read as clean (S-D2).
- Byte-stable output: no timestamps, no absolute paths, deterministic
  ordering, so two runs on the same inputs are identical (S-D3).

## Decision points for ratification

- S-D1 location of a REMOVED field: it has no line in the post file.
  Recommended: annotate the line of the parent object's `properties`
  key in the post file (where the field used to live), with the base-ref
  line stated in the message; fall back to line 1 when no parent line
  is found. Alternative: file-level only.
- S-D2 NOT EVALUATED shape: recommended as above (notification, zero
  results, executionSuccessful false). Alternative: a single warning
  result, which risks being filtered out by severity.
- S-D3 rule id form: recommended `invar.judge.<kind>`. Alternative:
  numbered ids (INV-J001), which are less readable for agents and humans.
- S-D4 annotation level: recommended `error` for VIOLATED, `warning` for
  NOT EVALUATED (stated, not hidden). Alternative: `error` for both.

## Fixtures (to freeze on ratification)

- sf_f01 the log is SARIF 2.1.0: `version`, `$schema`, one run, driver
  name `invar`, the six rules present.
- sf_f02 one result per witness on the real b1-002 bytes; ruleId
  `invar.judge.retyped`; message equals the text sentence.
- sf_f03 a retyped field's region.startLine is its POST-file line.
- sf_f04 a removed field's location follows S-D1.
- sf_f05 an unparsable post: zero results, notification with
  `unparsable`, executionSuccessful false, exit 4.
- sf_f06 two runs byte-identical.
- sf_f07 unjudged_constraints present in run properties.

## Mutation operators

- SF-M1 results dropped (empty array): killed by sf_f02.
- SF-M2 NOT EVALUATED emitted as a clean run, no notification: sf_f05.
- SF-M3 pre-file line used where the post-file line belongs: sf_f03.
