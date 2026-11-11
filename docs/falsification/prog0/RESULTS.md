# PROG-0 results (2026-11-09), against PREREG.md as frozen (blake3 5cbd99c4...)

Every program ran offline (no network namespace), `env -i` with PATH and HOME only, a fresh
HOME and a fresh working directory. Full output of every run is in runs/<change>-<case>/.
Deviations, each written before the run it affects: DEVIATIONS.md.

## Counted (10 changes with a clean control)

| change | program, version | test document (new schema rejects) | program did | class |
|---|---|---|---|---|
| b056a70c8 traefik-v3 | Traefik v3.7.6 | `experimental.otlplogs: true` | starts; loaded config shows `"otlplogs": true` (keys matched case-insensitively) | ACCEPT |
| cfd4af801 claude-code-settings | Claude Code 2.1.195 | `effortLevel: "max"` | `claude doctor` reports no invalid settings (it does report `permissions: 5`, and reports nothing for `"bogus-value"` either) | ACCEPT |
| 9497fde91 codex-hooks | Codex 0.140.0 | `"$schema": ""` at the root | `failed to parse hooks config ... unknown field $schema`; hooks file skipped | REJECT |
| 714ea6b79 ruff | ruff 0.15.20 | `extend-select = ["RUF076"]` | `Rule RUF076 was removed and cannot be selected`, exit 2 | REJECT |
| 2eb086dfa pnpm-workspace | pnpm 11.20.0 | `noproxy: ""` | loads, no message; stored as `noproxy`, `noProxy` stays undefined (accepted, not used) | ACCEPT |
| 18132a01a pnpm-workspace | pnpm 11.28.10 (frozen rule) | `include: []` | warns `"include" ... not recognized ... ignored` (same warning for the control) | WARN |
| e7373bd3f helmfile | helmfile v1.7.3 | `helmDefaults.historyMax: 0.5` | loads, no message; built state shows `historyMax: 0` (value silently truncated) | ACCEPT |
| 13e89998e cargo | cargo 1.98.0 | `crate-type = [""]` | `unknown crate type`, exit 101 | REJECT |
| 6b9673c4c pyproject | pip 26.2.1 | `build-backend = ""` (hand-built, both validators confirm) | `metadata-generation-failed`, exit 1 | REJECT |
| a83ce6f19 package | npm 11.20.0, `install` | `"version": ""` (hand-built, both validators confirm) | installs, no message | ACCEPT |

Disagreement (ACCEPT or WARN) = 6 of 10 = 60%.
Prediction (>= 20%): met. Kill (< 10%): not reached.

Reported, not counted (fixed before running):
- 18132a01a with pnpm 12.17.0 (the newer line on that date): `ERR_PNPM_INVALID_WORKSPACE_INCLUDE`, REJECT.
  With this in place of 11.28.10 the rate is 5 of 10 = 50%.
- a83ce6f19 with `npm pack --dry-run`: `Invalid package, must have name and version`, REJECT.
  The same program agrees in one command and disagrees in another.

## Excluded (listed, not counted)

- 51b3fa1a0 partial-pixi: pixi 0.81.0 not obtainable here.
- 2f74efc3f, 5fcc2e0ed claude-code-settings: no counterexample exists. See the finding below.
- 2f74efc3f claude-code-keybindings: no observable (`doctor` silent even for `{"bindings": 5}`).

## What this says, and does not

Six of ten programs loaded, without an error, a document their own published schema rejects.
In three of the six the value was not what the file said: pnpm stored `noproxy` and did not
use it, helmfile turned 0.5 into 0, Claude Code's validator does not check `effortLevel` values.
Ten changes, programs chosen because they install here, one version each: a small, selected
sample. It supports building the schema-vs-program corpus; it does not estimate a rate for the
ecosystem, and no schema verdict is changed by it.

## Finding about INVAR, found while preparing the population (not part of the prediction)

27 of INVAR's 67 SchemaStore VIOLATED verdicts have no counterexample and cannot have one:
every witness is a `pattern` or `format` change, and in each the new schema accepts at least
everything the old one did (22 SARIF `format: uri` -> `uri-reference`; 2 claude-code-settings,
openapi-overlay, openapi-arazzo, partial-inwards: names or alternatives added, a lookahead or a
prefix made optional). JUDGE-SUBTYPE r1 (JS-D3) makes every pattern or format change VIOLATED
by design, with the sentence "not decided here". Two of the 29 such VIOLATED are real
narrowings (pyproject: pattern added; linutil-tabs: `[^ ]` -> `\S`, "\t" breaks).
