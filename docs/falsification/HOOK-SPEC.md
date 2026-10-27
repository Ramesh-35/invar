# HOOK-1: the check inside the agent's edit loop (invar hook claude-code)

Status: r0, FROZEN with its fixtures before any implementation, for
founder ruling on H-D1 to H-D4 (the fixtures encode the recommended
options). Parent: the 2026-10-27 research rulings (H1: in-loop validation;
H6: a silent HOLDS must never read as safe) and the agent-hook
authorization. Fixtures: tests/hook_test.rs; hashes in HOOK-SPEC.frozen.

## A. The gap

An agent editing this repository learns about a broken contract at commit
time at the earliest (CI, the PR comment) or must call the MCP server
itself. Claude Code's PostToolUse hooks run a command after every edit and
feed stderr back into the model's context on exit code 2; INVAR's measured
check latency (reuse 4.5 to 13.3 ms, full evaluation 11 to 56 ms,
sandbox) fits inside an edit loop. Nothing wires the two together.

## B. Rules

- H-1, the hook is the binary (H-D1). `invar hook claude-code` reads one
  PostToolUse event (JSON) from stdin: no wrapper script, no generated
  file, nothing to keep in sync. `--settings` prints the one JSON entry to
  add (matcher `Write|Edit|MultiEdit|NotebookEdit`, command
  `invar hook claude-code`) and installs nothing.
- H-2, when it runs. Only for the matched editing tools; any other
  tool_name exits 0 silently. The repository is the event's `cwd` (the
  process cwd when absent); a cwd without invar.toml exits 0 silently, so
  the hook is installable globally. Unknown JSON fields are ignored.
- H-3, what the agent hears (H-D2). The evaluation is the engine's own
  (`check`'s path, reuse included). VIOLATED: the violated contracts'
  witness text on stderr, exit 2, the one code PostToolUse feeds to the
  model; stdout stays empty. HOLDS: exit 0, silent. NOT EVALUATED (exit
  4): exit 0, silent; stated coverage belongs to the MCP `coverage` tool,
  not to every edit, and a hook that talks constantly gets uninstalled.
  Exit 2 never claims safety; it only ever carries a break.
- H-4, errors. Malformed stdin, an unreadable configuration, an engine
  error: `ERROR (not a verdict): ...` on stderr, exit 3, which PostToolUse
  surfaces as a non-blocking error to the user and never feeds to the
  model, so a broken setup cannot thrash the loop. `invar hook <unknown>`
  is the same error.
- H-5, scope (H-D3, H-D4). The full check runs on every matched edit (the
  measured reuse makes changed-file scoping unnecessary complexity);
  PostToolUse only in this slice (no Stop hook). Read-only toward the
  repository: the hook writes nothing but the engine's own verdict cache.
- Unchanged: `invar check`, every exit code meaning (2 exists only inside
  the hook subcommand, whose contract is Claude Code's, not INVAR's),
  MCP, the Action.

## C. Measured before code

On this tree: a second `invar check` in the fixture repository reuses its
HOLDS (CHECK-INCR-1, `unchanged since lock`); the fixture repository's
broken form is VIOLATED with the witness naming lk, Limits.maxLines and
src/use.ts (the text the hook forwards); `invar hook` is parsed today as
an unknown subcommand (exit 2 from the argument parser), so every fixture
fails at its first assertion on the current engine.

## D. Fixtures (frozen with r0)

| Fixture | Asserts |
|---|---|
| hk_f01 | holding repository: exit 0, both streams empty |
| hk_f02 | violation: exit 2, stderr names lk, Limits.maxLines, src/use.ts; stdout empty |
| hk_f03 | a Read on a violated repository runs nothing: exit 0, silent |
| hk_f04 | no invar.toml: exit 0, silent (installable everywhere) |
| hk_f05 | malformed stdin: exit 3, `ERROR (not a verdict)` |
| hk_f06 | broken invar.toml: exit 3, never exit 2 |
| hk_f07 | `--settings`: valid JSON, the matcher and command exactly |
| hk_f08 | no lock (check exit 4): exit 0, silent |
| hk_f09 | the event's cwd wins over the process's |
| hk_f10 | `invar hook cursor`: exit 3, `unknown agent` |
| hk_f11 (r1) | unparsable invar.toml (`[[[`): exit 3, never exit 2 |

## E. Pre-registered mutation operators

| Operator | Change | Declared killer |
|---|---|---|
| HK-M1 | a violation exits 0 | hk_f02 |
| HK-M2 | exit 2 with empty stderr | hk_f02 |
| HK-M3 | every tool_name runs the check | hk_f03 |
| HK-M4 | a missing invar.toml is an error | hk_f04 |
| HK-M5 | malformed stdin exits 2 | hk_f05 |
| HK-M6 | a configuration parse error exits 2 | hk_f11 (r1; declared hk_f06 in r0 and SURVIVED, see F2) |
| HK-M7 | the settings entry loses its matcher | hk_f07 |
| HK-M8 | check's exit 4 maps to exit 2 | hk_f08 |
| HK-M9 | the event's cwd is ignored | hk_f09 |
| HK-M10 (r1) | an evaluation error exits 2 | hk_f06 |

## F2. Amendment r1 (a survivor, repaired before landing)

In the r0 run HK-M6 SURVIVED: it mutates the configuration PARSE error
path, but hk_f06's invar.toml (`kind` only) parses and fails in
evaluation (`schema invariant needs producer`), so the mutated path was
never exercised. Measured on the implementation: an unparsable
configuration (`[[[`) errors in parse_sections, the missing-keys one in
evaluate_with. r1 adds hk_f11 for the parse path (HK-M6's killer) and
pre-registers HK-M10 for the evaluation path (killed by hk_f06). The r0
run's ledger is kept in docs/evidence/hook/mutate-hook.log.

## F. Decision points

- H-D1 (shape). The binary is the hook, stdin to stderr (recommended: zero
  generated artifacts). Or `invar hook` writes a wrapper script.
- H-D2 (what feeds back). Violations only (recommended: H6 says silence
  must never claim safety, and the fix is that exit 2 never asserts
  anything but a break; a chatty hook gets uninstalled). Or also feed NOT
  EVALUATED on every edit.
- H-D3 (scoping). Full check per edit, measured reuse carrying the cost
  (recommended: simpler, and the reuse path is already evidence-backed).
  Or changed-file scoping.
- H-D4 (events). PostToolUse only (recommended). Or also a Stop hook.

## G. Bounds

No verdict, exit code meaning or CLI behavior changes outside the new
subcommand; zero new dependencies; zero network; writes nothing but the
engine's existing verdict cache; frozen spec and fixtures before code;
mutation operators with declared killers; native verification. Not
claimed: other agents' hook formats (the subcommand name leaves room);
latency on other machines; that an agent acts on the feedback (that is
the two-week test from the research brief, measured separately).
