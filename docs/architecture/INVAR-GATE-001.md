# INVAR-GATE-001: the execution gate (`invar gate -- <command>`) and the diagnostic link

Status: RATIFIED (rulings G-D1 to G-D7, 2026-10-18 17:08) and IMPLEMENTED
under the R-L-2 Option B amendment (ruling of 2026-10-18 17:20, section K).
Parent: Charter A of 2026-10-18. Prepared to the house standard: honest
bounds, numbered requirements, countermodels, fixtures with declared outcomes
before code, mutation set, gates 0 to 5. Baseline: the clean tree at 690d606.

## 0. What the gate is, and the honest bounds

`invar gate -- <command>` makes the verification gate literal: check before,
run the command only if the world holds, check again after, and state whether
the watched state drifted while the command ran. One verb, composable in any
script or agent loop, std only.

The diagnostic link closes the loop opened by INVAR-EXPLAIN-001: any run that
printed an ABSTAIN line ends with exactly one pointer line naming
`invar explain`, so no human or agent ever meets a code without its exit.

Bounds, stated before any code and not negotiable by wording:

B1. The gate brackets, it does not serialize. A non-cooperative writer can
still change the world between the post-check and whatever happens next, and
can race the command itself. Drift is DETECTED and STATED, never prevented.
No TOCTOU-elimination claim is made or implied anywhere.
B2. The gate is a wrapper, not a verdict command. When the wrapped command
runs, the gate's exit code belongs to the rule in R-G-5, not to the frozen
five-code ISA; only a refusal (command never ran) carries an ISA code. This
is printed, not hidden.
B3. The pre-check and post-check read the working tree exactly as
`invar check` does; they see what any check sees, nothing more.
B4. The gate adds two check runs on the order of milliseconds; the wrapped
command dominates the cost.

## A. Frozen scope

A1. Zero new dependencies; std::process only. The argv after `--` is executed
directly, never through a shell: no interpretation, no injection surface.
A2. The wrapped command's stdin, stdout and stderr are inherited untouched.
Every gate line goes to STDERR with the prefix `GATE `, so piping the wrapped
command's stdout stays byte-clean.
A3. No new registry code in v1. Gate lines are statements; the verdict codes
in play are the check's own, already in the explain registry (G-D3).
A4. ISA invariance for everything that is not the gate: plain `check`,
`init`, `lock`, `explain` outputs change by nothing except the single pointer
line of section C, exactly where section C says.

## B. Numbered requirements: the gate

R-G-1 Invocation. `invar gate [--config <path>] [--proof] -- <command>
[args...]`. Everything after `--` is the command, executed verbatim. A
missing `--` or an empty command is exit 3 with the usage stated.

R-G-2 Pre-check. The gate evaluates the manifest exactly as `invar check`
(with `--proof` when the flag is given). Exit nonzero refuses: the command is
NOT run, the gate prints `GATE REFUSED (pre-check exit N): command not run`
and exits with the pre-check's own code. A refused gate has run nothing and
changed nothing.

R-G-3 Watch set. Before running the command the gate records the blake3
digest of: invar.toml, invar.lock (if present), every locked contract's
producer file, and, with `--proof`, every bound proof file. The digests stay
in memory; nothing is written.

R-G-4 Execute and re-check. The command runs with inherited stdio. After it
exits the gate re-checks, and for every watch-set file whose digest changed
it prints `GATE DRIFT: <file> changed while the command ran`. Drift lines are
statements; the post-check's verdict is what judges the new state.

R-G-5 Exit rule (ruling G-D1). One rule, no flags:
- pre-check nonzero: the gate exits with the pre-check code (command not run);
- the command exits nonzero: the gate exits with the COMMAND's code (the
  command's own failure is the primary signal, forwarded untouched);
- the command exits 0: the gate exits with the POST-CHECK's code, so a
  command that reports success while breaking a locked contract can never
  look green.
The gate prints which of the three cases decided the exit.

R-G-6 Transparency. The wrapped command's stdout is byte-identical to running
it bare (fixture G-F6). The gate never buffers, edits or reorders it.

R-G-7 Determinism. Given the same tree, manifest and command behavior, the
gate's own stderr lines are byte-identical across runs.

R-G-8 Budget. No new dependency, no network, no clock beyond what check
itself uses; the gate's own overhead is two checks plus hashing the watch set.

## C. Numbered requirements: the diagnostic link

R-L-1 One pointer line. Any `invar init` or `invar check --proof` run whose
output contains at least one `ABSTAIN <CODE>` line ends with exactly one
trailing line: `codes: run \`invar explain <CODE>\` for any ABSTAIN code
above`. Zero ABSTAIN lines, zero pointer. Never more than one per run.

R-L-2 Placement (AMENDED, Option B, ruling of 2026-10-18; the original "final
line everywhere" wording is void). In `invar check --proof` the pointer is
the final line, where no terminator convention exists. In `invar init` the
pointer sits immediately BEFORE the `# SUMMARY` line, the penultimate line,
because the frozen slice 1 suite proved at Gate 2 that SUMMARY-last is a
machine-read terminator of init output: 17 of its 40 fixtures parse the last
line as the summary. A machine-first boundary is never displaced by a
human-convenience line. Zero abstentions still emit zero pointers, and no
existing line moves or changes in either command.

R-L-3 Invariance. Every existing fixture suite stays green. Runs with no
abstentions are byte-identical to today. The REV1 metamorphic suite still
passes (both sides of each comparison carry the same line or neither does).

## D. Countermodels (defined before fixtures and code)

CM-G-1 pre-check violated but the command runs anyway. Declared: refused,
command not run, no side effect exists.
CM-G-2 the command fails and the gate reports 0 because the post-check holds.
Declared: the command's code is forwarded.
CM-G-3 the command exits 0 after breaking a locked contract. Declared: the
gate exits with the post-check's violation code.
CM-G-4 gate lines printed to stdout, polluting a pipe. Declared: stderr only.
CM-G-5 the command string interpreted by a shell (quoting, $, backticks).
Declared: argv passed verbatim, one argument stays one argument.
CM-G-6 drift in a watched file left unstated. Declared: a GATE DRIFT line
names each changed file.
CM-L-1 a pointer line on a run with no abstentions. Declared: none printed.
CM-L-2 one pointer per abstention (spam). Declared: exactly one per run.
CM-L-3 the pointer naming a command that does not exist. Declared: it names
`invar explain` verbatim, and the fixture greps the exact text.

## E. Fixtures (declared outcomes, before any code)

G (gate): G-F1 holding repo, wrapped command exits 0: gate exits 0, stderr
shows pre-check held, command ran, post-check held. G-F2 violated repo: the
wrapped command would create a file; the gate refuses with the pre-check's
code, the file does not exist, stderr has `GATE REFUSED`. G-F3 holding repo,
wrapped command exits 3 (the binary's own `explain NOT-A-CODE`): gate exits
3. G-F4 holding repo, wrapped command (a shell invoked BY THE FIXTURE, not by
the gate) renames a locked producer field and exits 0: gate exits 1, stderr
has `GATE DRIFT` naming the producer file and the post-check violation.
G-F5 `--proof` pass-through: a consumer repo whose proof import holds gates a
no-op command at exit 0; with the proof tampered, the gate refuses at exit 3.
G-F6 stdout transparency: the wrapped command's stdout captured through the
gate is byte-identical to running it bare. G-F7 argv fidelity: an argument
containing spaces reaches the command as one argument. G-F8 missing `--` or
empty command: exit 3, nothing run. G-N1 no invar.toml: refusal with check's
own exit 3 behavior. G-M1 gate stderr deterministic across two runs. G-M2
src/gate.rs exists and contains no network tokens; the dependency list stays
frozen (reuses the proof arc's assertions).

L (link): L-F1 init on a repo with two abstentions: exactly one pointer line,
the penultimate line, with `# SUMMARY` last (per the amended R-L-2); on a
repo with none: absent. L-F2 check --proof with an ABSTAIN:
one pointer; with a holding import: none. L-F3 two runs byte-identical.
L-F4 the pointer contains `invar explain` verbatim. L-N1 all existing suites
green unchanged (the standing suites are the fixture).

## F. Mutation set (CAUGHT / SURVIVED / BUILD-FAILED; the last two are not kills)

1 pre-check skipped (command always runs): G-F2. 2 refusal exits 0: G-F2.
3 post-check skipped: G-F4. 4 command exit swallowed (gate always exits the
post-check code): G-F3. 5 post-check code overrides a failing command: G-F3.
6 drift lines silenced: G-F4. 7 gate lines sent to stdout: G-F6. 8 argv
joined and passed through a shell: G-F7. 9 pointer line removed: L-F1.
10 pointer printed with zero abstentions: L-F1. 11 pointer printed once per
abstention: L-F1. 12 pointer names the wrong command: L-F4. Each must die by
a named assertion failure.

## G. Gates 0 to 5

Gate 0 hash ledger at the clean tree 690d606 (the GATE0-PROOF-LEDGER set plus
proof.rs, explain.rs and the landed tests as pre-change baselines). Gate 1
fixtures first, failing against the unchanged engine for declared reasons
(the gate subcommand missing; the pointer line absent). Gate 2 implementation
to allowed paths. Gate 3 mutation harness all CAUGHT with control green.
Gate 4 full suite green, zero warnings, byte-identity per R-L-3, the
completions guard satisfied. Gate 5 evidence ledger here, STATUS.md entry.

## H. Path classes

Class 1 (untouched, hash-verified): the standing protected set, lock.rs,
check.rs, config.rs, proof.rs, explain.rs, valuepairs.rs, strings.rs,
codepairs.rs. Class 2: tests/gate_test.rs, scripts/gate/. Class 3
(REQUESTED, not assumed): main.rs (subcommand dispatch and the check --proof
pointer line), init.rs (the init pointer line only), the three completion
scripts (the standing guard test demands every subcommand). Class 4:
src/gate.rs (new, self-contained), lib.rs (registration).

## I. Decision points for ruling

G-D1 The exit rule of R-G-5 (refusal: pre-check code; failing command: its
own code; clean command: post-check code), with no flags in v1.
G-D2 Gate lines on stderr with the `GATE ` prefix; the wrapped command's
stdio inherited untouched.
G-D3 No new registry code in v1; gate lines are statements and the codes in
play are the check's own. A future GATE-* code set would enter the explain
registry and the drift test together.
G-D4 The watch set of R-G-3 (manifest, lock, locked producers, bound proofs).
G-D5 No shell, ever: argv after `--` is executed verbatim.
G-D6 The pointer line text and the exactly-once rule of R-L-1.
G-D7 `--proof` pass-through on gate.

## K. Rulings of 2026-10-18 (recorded)

17:08: G-D1 to G-D7 APPROVED as drafted; path classes granted (main.rs,
init.rs, the three completion scripts as Class 3; src/gate.rs and lib.rs as
Class 4); implementation GO.

17:20, the R-L-2 Option B amendment: at Gate 2 the ratified R-L-2 (pointer as
the final line) collided with stop condition J, because 17 of 40 frozen slice
1 fixtures machine-read init's last line as the SUMMARY. Option A (amend the
frozen suite) was REJECTED; Option B was GRANTED: the SUMMARY line is an
immutable machine-readable terminator of init output, and the pointer moves
to the penultimate position there, staying final in check --proof. Ruled
principle, recorded verbatim in substance: deterministic, machine-first
interfaces are never corrupted by human convenience formatting.

## J. Stop conditions

Stop and report on: any new dependency or network capability; any shell
interpretation of the wrapped command; any change to a plain command's
verdicts, codes or bytes beyond the single declared pointer line; any
existing fixture failing for any other reason; a Class 1 hash change; any
wording anywhere that claims the gate prevents rather than detects drift.

## M. Implementation Evidence Ledger (appended at Gate 5; sections 0 to K are unchanged)

Status: IMPLEMENTED against the ratified rulings and the R-L-2 Option B
amendment. What is proven is what the fixtures and mutations state about this
engine on synthetic repositories; the gate detects drift and never prevents
it (B1), and no claim beyond that exists.

### M1. Gate results

| Gate | Result |
|---|---|
| 0 baseline | the clean tree at 690d606; Class 1 and landed-engine hashes re-verified byte-identical after implementation |
| 1 fixtures first | 15 fixtures written before src/gate.rs existed; 13 failed for their declared reasons (the missing subcommand, the absent module, the absent pointer) and 2 invariance controls passed (G-M1 trivially, via clap's deterministic error; L-F3, init determinism) |
| 2 implementation | src/gate.rs (new, self-contained), main.rs (dispatch, GateArgs, the check --proof pointer), init.rs (the penultimate pointer), lib.rs, the three completion scripts. STOPPED at Gate 2 on the R-L-2 conflict with the frozen slice 1 suite; resumed under the Option B ruling (section K) |
| 3 mutations | 13 of 13 CAUGHT by named assertion, control green, none SURVIVED, BUILD-FAILED or APPLY-FAILED (scripts/gate/mutate.sh; the declared 12 plus 09b, splitting the pointer-removal mutant so the init surface and the check --proof surface each die independently) |
| 4 regression | 244 of 244 tests green (the 229 landed plus 15 gate fixtures), zero warnings, ZERO frozen-suite edits; every slice, proof and explain suite byte-green |
| 5 documentation | sections K and M, STATUS.md entry |

### M2. The conflict event, recorded plainly

The ratified R-L-2 placed the pointer as the final line of every output. At
Gate 4's first run, 17 of 40 frozen slice 1 fixtures failed, every one at the
single assert that machine-reads init's LAST line as the SUMMARY. No fixture
failed for any other reason. The arc stopped at Gate 2 per stop condition J;
the founder rejected amending the frozen suite and ruled Option B: the
SUMMARY line is an immutable machine-readable terminator of init output, and
the pointer is penultimate there, final in check --proof. The frozen suite
did exactly what frozen suites are for: it surfaced an interface contract
nobody had written down, and now it is written down.

### M3. Fixture corrections during the arc (stated, not hidden)

- L-F1's two-abstention repository first used an invalid-UTF-8 file; a
  packageless walk skips such a file silently, so only H2 fired. Replaced
  with the deterministic H2 plus K3 AMBIG-RECEIVER construction.
- L-F1's clean branch first tested `contains("ABSTAIN")`, which matches the
  word inside init's standing header text; tightened to the emission prefix
  `# ABSTAIN `. Both were fixture errors; the engine was right both times.

### M4. Files changed

src/gate.rs (new), src/main.rs, src/init.rs, src/lib.rs, completions/invar.{bash,zsh,fish},
tests/gate_test.rs, scripts/gate/ (harness and 13 patches). Everything else
byte-identical to 690d606, including every file the proof and explain arcs froze.
