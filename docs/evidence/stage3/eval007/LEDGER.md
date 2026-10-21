# STAGE3-EVAL-007: INIT-DISCOVERY-SLICE7 on the pinned manifest

Engine: INIT-DISCOVERY-SLICE7 (receivers assigned from an imported
function's declared return type), uniform scope (S7-D1), on b0a8c79,
release binary 029c0c286ecf. Manifest hash 2f5b6b0f798b, instrument r2,
the same ten pinned repositories as STAGE3-EVAL-001 to -006, all of which
stand unmodified.

Verdict: PASS on every gate of the ruling. The counts land one proposal
and two shapes under the uniform forecast; the whole difference is the
two prototype conversions the ruled return forms exclude (below).

| Gate or target | Measured |
|---|---|
| M-S-1 ISA conformance | 10 of 10, every init exit 0 |
| M-S-2 determinism | 10 of 10, raw bytes |
| byte preservation (S7-D2) | 7 repositories byte-identical to EVAL-006; in the other 3, 0 proposal blocks removed, 39 added, 1 changed: pi `code:output.ts->tool.ts`, which gains the combined status line and OutputSlice's E2, E4, E5 and evidence lines, every other line of it unchanged (m-s-4-compare.txt) |
| M-S-4 silent drops (hard gate) | 0: 40 abstention lines removed (39 R6, 1 RECV), 0 added; each removed line is exactly one of the 40 new shapes, in its consumer file |
| M-S-3 precision, every new shape (S7-D5, floor 60 percent, hard gate) | **39 of 40 TRUE (97.5 percent)**; two blind agents, conservative merge, agreement 40 of 40 |
| the faux pattern, both functions (fauxProvider, registerFauxProvider) | 32 of 32 TRUE (100 percent) |
| registerFauxProvider only | 5 of 5 TRUE (100 percent) |
| excluding the faux pattern | 7 of 8 TRUE (87.5 percent) |
| excluding registerFauxProvider only | 34 of 35 TRUE (97.1 percent) |
| M-S-5 round trip, the new TRUE contracts | 39 of 39 (38 new pairs and the mixed pair): lock exits 0, every evidence field bound (85 + 3 + 4 bindings), check HOLDS with the durability line, second check byte-identical |
| M-S-6 backtest, the same contracts | 3 of 3 repositories complete, exit 0, byte-identical reruns, clones at depth 200; 2 absent events, both before the field's introduction; 0 never-present, 0 blind spots |
| M-S-7 performance | init 0 to 9 seconds per repository |

## The pre-registration (draft section F, uniform), against measurement

| Pre-registered | Measured |
|---|---|
| proposals 113 to 153 | 113 to 152 |
| new shapes 42 | 40 |
| R6 -40 | -39 |
| RECV -2 | -1 |
| every other repository and existing proposal byte-identical except S7-D2 | yes |
| M-S-3 at least 60 percent over every new shape | 97.5 percent |
| M-S-4 0 | 0 |

Every deviation is one cause, explained, not tuned: the prototype accepted
a return type written `S | undefined`; the ruled forms (S, Readonly<S>,
Promise<S>, S[], Array<S>) do not, and fixture S7-F10 pins that. Two
prototype conversions use it: `getOpenAICodexWebSocketDebugStats` (pi, an
R6 shape on a new pair: the -1 proposal, the -1 R6 and one shape) and
`getCurrentSystemMessage` (pi, SystemMessage on an existing pair: the -1
RECV and the other shape). Every other prototype conversion is in the
engine's output, and the engine adds none the prototype did not have.

## The one NOT TRUE verdict

pi `code:colors.ts->system-theme.ts`, RgbColorValue: the reads `color.r`,
`color.g`, `color.b` (system-theme.ts:392) are on a parameter `color` of
another function, typed RgbColor (a different interface with the same
field names); the file's `color = okhslColor(...)` is a different binding
of the same name. Bound B3 (names, not scopes, as SLICE5 B4), declared
before any code; the R6 probe found the same item NOT TRUE.

## Adjudication

One packet (build_packet.py): one item per shape whose E4 line says it was
resolved via an imported function's return type. The script first checks
that this set equals the set of shapes added since EVAL-006, (repository,
proposal, shape) by (repository, proposal, shape), and stops otherwise; it
was equal (40). Two fresh agents with no knowledge of the engine or of
earlier verdicts, the packet, the rubric and the pinned source only, blind
to each other; TRUE only when both say TRUE (score.py). This is AI
adjudication, reported under that label. In-sample caveat, stated in the
draft: the R6 probe judged 39 of these 40 shapes before the rule existed
(38 TRUE); these verdicts are on the engine's own evidence lines.

The faux pattern, stated before scoring: pi's test files obtain a test
provider either from `fauxProvider()` (FauxProviderHandle, 27 shapes) or
from `registerFauxProvider()` (FauxProviderRegistration, 5). The R6 probe's
README called it one function and one shape; its own TSV shows both. Both
readings are reported above.

## Closing re-measurement after S7-D6 (2026-10-21)

Ruling S7-D6 corrected the header sentence init writes at the top of
every invar.toml (it said every shape was imported by name). Release
binary 681fc1463275, the same manifest and runner: 10 of 10 exit 0,
deterministic; every output differs from the evaluation above in that
sentence only, 3 lines replaced by 4, and in nothing else
(closing-s7d6/header-diff.txt). The M-S-5 and M-S-6 inputs carry no
header, so they stand as measured.

## Files

phase1/ the init outputs and ledger of both runs (run.sh, unchanged);
m-s-4-compare.txt (compare.py, EVAL-006 against EVAL-007); adjudication/
(packet, items, both verdict files, merged verdicts, the prompts,
score.py, accepted.tsv); m-s-5/ and m-s-6/ (roundtrip.sh over the
accepted contracts: the accepted invar.toml, lock, both checks and both
backtests per repository); closing-s7d6/ (the re-measurement's outputs,
ledger and diff).
