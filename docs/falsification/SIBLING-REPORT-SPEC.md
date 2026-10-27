# SIB-2: deliver the producer-side evaluation where it is acted on (P5 slice 2)

Status: r0, FROZEN with its fixtures before any implementation, for
founder ruling on S2-D1 to S2-D4 (the fixtures encode the recommended
options; a different ruling yields r1 with fixtures revised before code).
Parent: SIB-1 (SIBLING-EVAL-SPEC.md r1) and the 2026-10-27 decision "the
agent needs to see 'you broke web-app's maxLines' in its context window
before it commits". Fixtures: tests/sibling_report_test.rs; hashes in
SIBLING-REPORT-SPEC.frozen.

## A. The gap

SIB-1 prints its evaluation as CLI text. An agent reading the MCP
`dependents` tool gets only the query mode ("who depends on X"), and a
pull request gets nothing: the Action always runs `invar check` first, and
`check` refuses a pure producer repository (no contracts of its own).

## B. Rules

- R-1, one evaluation. SIB-1's evaluation becomes data rendered three
  ways: the existing text (byte-identical, SIB-1's fixtures unchanged),
  JSON for MCP, and markdown for the PR. The three cannot disagree.
- R-2, MCP. The `dependents` tool without `query` evaluates, when the
  server was started with `--repo OWNER/NAME` (S2-D2); consumer locks are
  verified against `--dependents-signers <file>` (S2-D1); without it,
  nothing is evaluated (`no-signers`, stated). `locks` stays inside the
  configuration directory (SECURITY S2-5). `query` is no longer required;
  the query mode is unchanged. Result:

      {"repo", "lockfiles_scanned",
       "consumers": [
         {"lockfile", "state": "verified", "principal",
          "contracts": [{"id", "producer",
                         "broken": [{"field", "rule": "absent"|"retyped",
                                     "locked_type", "now_type"?}],
                         "provided"}
                        | {"id", "producer", "not_evaluated": {"reason", "text"}}]}
         | {"lockfile", "state": "not_evaluated", "reason", "text"}],
       "summary": {"broken", "provided", "lockfiles_not_evaluated",
                   "contracts_not_evaluated"},
       "exit_code", "note"}

  Reasons: `no-signers`, `unsigned`, `signature`, `unreadable` (lockfile);
  `producer-missing`, `unextractable` (contract). Nothing read from an
  unverified lock appears.
- R-3, PR report. `dependents --repo ... --report-md <path>` also writes a
  markdown report starting `<!-- invar-dependents -->`: a table (consumer
  lock, signed by, contract, field, change), the not-evaluated list, the
  line `Advisory: this report never fails the check.` and the field-level
  note. Every repository value is inert code (the `md_code` rule `check`
  uses: pipes escaped, backticks neutralized, mentions inside code).
  Writing it changes neither stdout nor the exit code (an unwritable path
  is exit 3, as in `check`).
- R-4, Action (S2-D4). Optional inputs `check` (default 'true'),
  `dependents_repo`, `dependents_locks`, `dependents_signers` (default '').
  A pure producer repository sets `check: 'false'`. When `dependents_repo`
  is set, action/dependents.sh runs the evaluation with `--report-md`,
  writes `dependents_exit`, `dependents_broken`, `dependents_report` to
  the step outputs and the job summary, and always exits 0. The comment is
  its own (marker `<!-- invar-dependents -->`, via comment.sh's
  `INVAR_MARKER`): created when something is broken, refreshed otherwise,
  best effort (continue-on-error). The Gate step never reads it.
- Unchanged: `check`, every verdict and exit code meaning, the CLI's
  `--format json` with `--repo` (still an error, S2-D3), SIB-1's text.

## C. Measured before code

`invar lock` accepts a contract id built to break a table
(`x|y @someone` + backtick): it reaches the lock verbatim, so the report
must neutralize it (f11). SIB-1's premises (remote producer parse,
extractor types, find-principals) hold unchanged.

## D. Fixtures (frozen with r0)

| Fixture | Asserts |
|---|---|
| s2_f01 | MCP: verified consumer, removed field as `{field, rule: absent, locked_type}`, summary, exit_code 0, note |
| s2_f02 | MCP: retyped field with `now_type` |
| s2_f03 | MCP: unsigned lock is a reason only; nothing of it in the result |
| s2_f04 | MCP: no `--dependents-signers`: `no-signers`, exit_code 4 |
| s2_f05 | MCP: no query, server without `--repo`: error |
| s2_f06 | MCP: locks outside the configuration directory still refused |
| s2_f07 | CLI text summary and MCP summary and exit code agree |
| s2_f08 | MCP query mode unchanged |
| s2_f09 | tools/list: `required: ["locks"]`, description states evaluation |
| s2_f10 | report: marker, table row, not-evaluated line, advisory line, note; stdout and exit unchanged |
| s2_f11 | report: hostile contract id inert, row keeps five cells |
| s2_f12 | action.yml: four inputs and defaults, check skippable, dependents step gated, comment marker parameter |
| s2_f13 | action/dependents.sh: exits 0, writes dependents_exit, dependents_broken, dependents_report |

Every MCP call asserts that stdout carries JSON-RPC only.

## E. Pre-registered mutation operators

| Operator | Change | Declared killer |
|---|---|---|
| S2-M1 | JSON drops `broken` entries | s2_f01 |
| S2-M2 | JSON omits `now_type` | s2_f02 |
| S2-M3 | unverified lock's contracts included in JSON | s2_f03 |
| S2-M4 | server signers ignored (evaluate without) | s2_f04 |
| S2-M5 | `locks` containment check removed in evaluation mode | s2_f06 |
| S2-M6 | JSON summary counts provided as broken | s2_f07 |
| S2-M7 | `query` still required in tools/list | s2_f09 |
| S2-M8 | report written without the marker | s2_f10 |
| S2-M9 | report cells not passed through md_code | s2_f11 |
| S2-M10 | dependents.sh propagates the evaluation's exit code | s2_f13 |

## F. Decision points

- S2-D1 (trust root for consumer locks in the server). (a) A separate
  `--dependents-signers <file>` (recommended: P4's `--lock-signers`
  verifies this repository's own lock with a fixed principal; consumer
  locks are verified by find-principals over a file of many principals.
  Two roles, two flags; the operator may pass the same file to both).
  (b) Reuse `--lock-signers`: one flag, but a pure producer would have to
  name a principal for a lock it does not have.
- S2-D2 (identity in MCP). (a) Server flag `--repo`, set by the operator
  (recommended: the agent can never redirect identity). (b) A tool
  argument.
- S2-D3 (CLI JSON). (a) Stays out, MCP only, as JSON-D3 (recommended).
  (b) Allow `--format json` with `--repo` (SIB-1's f12 would change).
- S2-D4 (Action). (a) Four optional inputs and a separate, advisory
  comment (recommended: a pure producer can use the Action at all). (b)
  No Action change; document a workflow step instead.

## G. Bounds

No verdict or exit code meaning changes; `check` unchanged unless the
Action skips it by input; SIB-1 text byte-identical; zero network beyond
the existing comment step; zero new dependencies; frozen spec and fixtures
before code; mutation operators with declared killers; native
verification.
