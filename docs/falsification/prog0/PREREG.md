# PROG-0: does the program that reads the file agree with the schema change? (pre-registration)

Written 2026-11-09, before any program was installed or run. Frozen by the
BLAKE3 and SHA-256 in PREREG.frozen. Nothing below changes after the first
run; anything learned goes in RESULTS.md, labelled as found after.

## Question

When INVAR says a SchemaStore change is VIOLATED (the old schema accepts a
document the new one rejects), does the program that reads that file
agree? If programs routinely accept documents their own published schema
rejects, the "schema vs program" corpus is real and worth building. If
they almost always agree, it is thin and INVAR stays a pure inclusion judge.

## Population (fixed now)

The 67 SchemaStore changes INVAR v0.2.24..v0.2.26 (same judge) calls VIOLATED,
restricted to those whose file has one named program that reads it and
that installs on Linux from npm, PyPI, crates or GitHub releases. Fourteen:

| change | program | version rule |
|---|---|---|
| 2f74efc3f claude-code-settings | Claude Code | 2.1.220 (named in the commit) |
| 5fcc2e0ed claude-code-settings | Claude Code | latest npm release on or before 2026-05-24 |
| cfd4af801 claude-code-settings | Claude Code | 2.1.195 (named in the commit) |
| 2f74efc3f claude-code-keybindings | Claude Code | 2.1.220 |
| 9497fde91 codex-hooks | Codex CLI | latest npm release on or before 2026-06-16 |
| 714ea6b79 ruff | ruff | latest PyPI release on or before 2026-06-25 |
| b056a70c8 traefik-v3 | Traefik | latest v3 GitHub release on or before 2026-07-06 |
| 18132a01a pnpm-workspace | pnpm | latest npm release on or before 2026-11-01 |
| 2eb086dfa pnpm-workspace | pnpm | latest npm release on or before 2026-08-06 |
| e7373bd3f helmfile | helmfile | latest GitHub release on or before 2026-08-13 |
| 13e89998e cargo | cargo | the toolchain release current on 2026-09-03 |
| 6b9673c4c pyproject | pip | latest PyPI release on or before 2026-10-12 |
| a83ce6f19 package | npm | latest npm release on or before 2026-09-27 |
| 51b3fa1a0 partial-pixi | pixi | 0.81.0 (named in the commit) |

If a version cannot be obtained, the nearest later release is used and the
deviation is stated; if none can be obtained the change is excluded and
listed, never counted.

## Procedure, per change

1. Counterexample: the first witness INVAR built. Where INVAR built none,
   one is written by hand from the witness and used only if Python
   jsonschema 4.26 and Ajv 8.20 both accept it under the old schema and
   both reject it under the new.
2. Test file: the smallest file of that type the program loads, with the
   counterexample merged in. Control: the same file with the offending
   value replaced by one the new schema accepts.
3. Run the program's own loading path (the command a user would run that
   reads that file) on the control, then on the test file, offline, in a
   clean directory and HOME.
4. The control must load without error, or the change is excluded as "no
   clean control" and listed.
5. Classify the test file: REJECT (exit non-zero, or an error naming the
   key or value), WARN (loads, a warning naming it), ACCEPT (loads, no
   message). Every command, version, exit code and output is recorded.

## Measure and thresholds (fixed now)

Disagreement = the program loads the test file without error (ACCEPT or
WARN), although the new schema rejects it.

Rate = disagreements / changes with a clean control.

- Prediction (from the research report): rate >= 20%.
- Kill: rate < 10%. Then the schema-vs-program corpus is not the moat:
  no paid tier is built on it and INVAR stays a pure inclusion judge.
- 10% to 20%: inconclusive; widen the population before deciding.

## What this does not claim

A disagreement says one program at one version accepted one document its
published schema rejects. It does not say the schema change was wrong,
that the change is safe for other readers, or anything about versions not
run. The schema verdict is not changed by any result here.
