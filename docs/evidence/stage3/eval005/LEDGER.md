# STAGE3-EVAL-005: INIT-DISCOVERY-SLICE4 plus SLICE5, measured together

Engine: SLICE4 (bounded barrel chains) and SLICE5 (receiver verification)
together (ruling S5-D2), release binary 237fbfef5b0b, built from c406834
plus docs/evidence/slice4/slice4-implementation.patch plus
docs/evidence/slice5/slice5-implementation.patch. Manifest hash
2f5b6b0f798b, instrument r2, the same ten pinned repositories as
STAGE3-EVAL-001 to -004, all of which stand unmodified.

Verdict: FAIL on M-S-5 (87 of 88), every other metric PASS. The failing
contract exposes a defect in lock and check, not in init: a producer type
written on one line locks zero bindings, and check then reports a vacuous
HOLDS. By ruling of 2026-10-20 nothing lands; LOCK-SLICE1 fixes lock and
check, and the three slices land together once all seven metrics pass.

| Metric | Verdict |
|---|---|
| M-S-1 ISA conformance | PASS, 10 of 10 (every init exit 0) |
| M-S-2 determinism | PASS, 10 of 10, raw bytes |
| proposals | 113: code pairs 206 to 104 (Understand-Anything 13 to 10, pi 139 to 63, excalidraw 46 to 26, puppeteer 8 to 5), strings 9 unchanged |
| M-S-3 precision, out of sample (registered, S5-D5) | **PASS: 43 of 50 TRUE (86.0 percent)** against the 60 percent floor (pi 32 of 38, excalidraw 11 of 12); two blind agents, conservative merge, agreement 50 of 50, kappa 1.00 |
| M-S-3 in sample (reported, not tested) | 45 of 54 TRUE (83.3 percent) re-judged on the new evidence (Understand-Anything 10 of 10, pi 18 of 25, excalidraw 14 of 14, puppeteer 3 of 5); agreement 48 of 54, kappa 0.45 |
| retention (reported) | 41 of the 57 pairs adjudicated TRUE in EVAL-003 and -004 still propose (S5-D6) |
| M-S-4 abstention honesty | PASS: 0 genuine silent drops. Every dropped shape is exactly one RECV line (168); no other abstention line removed or added. Detector as registered: 3 sampled flags, all in Understand-Anything files init never proposed in any evaluation, each stoplist-aligned (S3-INT-9: 1 distinctive read); exploratory flags identical to EVAL-003 and -004 |
| M-S-5 round trip | **FAIL: 87 of 88 TRUE contracts** lock, check exits 0 with the durability line, second check byte-identical. pi `code:output.ts->tool.ts` locks 0 bindings and checks as a HOLDS with no durability line (finding S5-F-01) |
| M-S-6 backtest | PASS: 4 of 4 complete, exit 0, byte-identical reruns, clones at the instrument's depth; 367 bindings, all present at the pinned commit; 18 absent events, every one before the field's introduction ("never absent after introduction"); 0 never-present, 0 blind spots |
| M-S-7 performance | init 0 to 8 seconds per repository |

## The pre-registration (S5-D5, founder 2026-10-20), against measurement

| Pre-registered | Measured |
|---|---|
| code-pair proposals about 104 (from 206) | 104 |
| RECV abstentions +168 | +168 |
| M-S-3 at least 60 percent on the 50 out-of-sample survivors | 86.0 percent |
| M-S-4 silent drops 0 (hard gate) | 0 |
| M-S-5 100 percent of TRUE contracts round-trip | **87 of 88: FAIL** |
| 10 of 10 repositories exit 0, deterministic | 10 of 10 |

The engine's output equals the throwaway prototype's line for line on the
five cloned repositories, apart from the ratified wording (S5-D4 status
and header, the RECV reason): the draft's prediction was not adjusted.

## New findings

S5-F-01 (the failure; lock and check, pre-existing) pi
packages/chord/src/output.ts:7 declares
`export type OutputLimits = { readonly maxBytes: number; readonly maxLines: number; readonly retain: "head" | "tail" };`
on one line. lock's TypeScript field reader (surface.rs `ts_fields`) reads
one member per line and finds none, so `invar lock` writes the contract
with 0 bindings, and `invar check` reports HOLDS with nothing compared.
The same pair is proposed on main today (EVAL-003 and -004, through other
read lines), so the defect is reachable without SLICE4 or SLICE5; this
evaluation exposed it because it is the first to round-trip that pair. It
is the only zero-binding contract among the 88 (every other contract binds
every one of its evidence fields).

S5-F-02 (what the remaining NOT TRUE verdicts are) Every out-of-sample
NOT TRUE verdict (7) and every in-sample disagreement (6) is a cited read
the adjudicators placed on a value of another type: the same receiver
name declared differently elsewhere in the file (`appState` is
InteractiveCanvasAppState in groups.ts; `frame` is a local WatchFrame in
observation.ts), a generic parameter, or a union. That is bound B4 (names,
not scopes), declared at ratification; a parser would remove it. No
NOT TRUE verdict cites a write, and none cites a receiver whose only
declaration in the file is untyped: where an untyped value is involved
(pi 28-reviewer.ts, `message = entry.model?.[0]`), the same name is also
declared as the shape elsewhere in the file, which is B4.

## Adjudication

Two packets (build_packet.py): out-of-sample, every surviving proposal
whose (producer, consumer) pair was never adjudicated (50); in-sample, the
54 that were (labeled-pairs.tsv). Four fresh agents, two per packet, with
no knowledge of the engine or of earlier verdicts, rubric D, the packet and
the pinned source only, blind to each other; TRUE only when both say TRUE.
This is AI adjudication, reported under that label.

## Closing re-measurement after LOCK-SLICE1 (2026-10-20)

The record above stands as written (M-S-5 FAIL on 237fbfef5b0b). By the
ruling that followed it, LOCK-SLICE1 fixed lock and check
(docs/architecture/LOCK-SLICE1.md), and the evaluation was re-measured on
release binary 26824a08805e (SLICE4 plus SLICE5 plus LOCK-SLICE1), same
manifest, same pinned clones (after-lock-slice1/).

| Metric | Verdict on 26824a08805e |
|---|---|
| M-S-1, M-S-2 | PASS, 10 of 10, exit 0, deterministic |
| init output | byte-identical to 237fbfef5b0b on all 10 repositories, so M-S-3 (43 of 50, 86.0 percent) and M-S-4 (0 silent) stand unchanged |
| M-S-5 round trip | **PASS: 88 of 88** TRUE contracts lock with bindings (0 zero-binding contracts; every contract binds every one of its evidence fields), check exits 0 with the durability line on all 88, second check byte-identical. pi `code:output.ts->tool.ts` now binds OutputLimits maxBytes, maxLines, retain |
| M-S-6 backtest | PASS: 4 of 4, exit 0, byte-identical reruns; 370 bindings, all present, every absent event before the field's introduction; 0 never-present, 0 blind spots. The only change from 237fbfef5b0b is the 3 new OutputLimits bindings, each present since its introduction |
| M-S-7 | init 0 to 8 seconds per repository |

STAGE3-EVAL-005: CLOSED, all seven metrics PASS. The first OmniRoute
fetch of the re-run failed on disk space (PIN-FAILED, as S3-F-02); space
was freed and the repository re-run alone; its output is byte-identical.
