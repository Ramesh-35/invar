# STAGE3-EVAL-004: INIT-DISCOVERY-SLICE4 on the pinned manifest

Engine: INIT-DISCOVERY-SLICE4 (bounded barrel chains), release binary
2823f90b0161, built from aa03983 plus docs/evidence/slice4/
slice4-implementation.patch. Manifest hash 2f5b6b0f798b, instrument r2,
the same ten pinned repositories as STAGE3-EVAL-001 to -003, all of which
stand unmodified.

Verdict: FAIL on M-S-3 (precision on the delta below the founder's
pre-registered floor). The engine is NOT landed (ruling S4-D8,
2026-10-20); it stays an evidence patch until a receiver-verification
slice lands with it and the pair is re-measured.

| Metric | Verdict |
|---|---|
| M-S-1 ISA conformance | PASS, 10 of 10 (every init exit 0) |
| M-S-2 determinism | PASS, 10 of 10, raw bytes |
| proposals | 215: +29 (pi 113 to 139, puppeteer 5 to 8); the other 8 repositories byte-identical to EVAL-003 |
| M-S-3 precision on the delta | **FAIL: 16 of 29 TRUE (55.2 percent) against the pre-registered 60 percent floor** (pi 13 of 26, puppeteer 3 of 3); conservative merge of two blind adjudications, agreement 29 of 29, kappa 1.00 |
| M-S-3 cumulative (reported, not the registered test) | code pairs EVAL-003 plus EVAL-004: 57 of 87 (65.5 percent); pooled with the EVAL-001 strings verdicts: 65 of 96 (67.7 percent) |
| M-S-4 abstention honesty | PASS: 0 genuine silent drops. Detector as registered: 1 sampled flag (puppeteer browsercontext-cookies.test.ts:7, DeleteCookiesRequest), stoplist-aligned (S3-INT-9: 1 distinctive read); exploratory flags identical to EVAL-003. Run on the four repositories whose output changed or whose clone is held here; Understand-Anything and freeCodeCamp produce byte-identical init output to EVAL-003 on the same pinned source, so their EVAL-003 M-S-4 results (0 silent) stand |
| M-S-5 round trip | PASS: 16 of 16 TRUE contracts lock, check exits 0 with the durability line, second check byte-identical (pi 13, puppeteer 3) |
| M-S-6 backtest | PASS: 2 of 2 complete, exit 0, byte-identical reruns; 100 bindings, all present, 0 absence windows, 0 never-present, 0 blind spots |
| M-S-7 performance | init 0 to 7 seconds per repository (unchanged) |

## The pre-registration (J of INIT-DISCOVERY-SLICE4), against measurement

| Pre-registered | Measured |
|---|---|
| +29 proposals (pi 113 to 139, puppeteer 5 to 8) | +29 exactly |
| BARREL2 abstentions -118 | -118 (pi 118 to 0) |
| unreached-type abstentions -13 | -13 (pi -4, puppeteer -9) |
| 0 new AMBIG-SHAPE | 0 (0 before, 0 after) |
| every cycle guarded, no loop or timeout | yes: S4-F4 under a wall-clock limit; mutant 11 caught; no run over 7 seconds |
| M-S-3 at least 60 percent on the delta | **55.2 percent: FAIL** |
| M-S-4 silent drops 0 (hard gate) | 0 |

## Every abstention change against EVAL-003 (transitions/)

| Repository | Removed | Added | Net |
|---|---|---|---|
| pi | 122: BARREL2 118, unreached-type 4 | 44: UNSUPPORTED-SHAPE 31, R5 13 | -78 (2933 to 2855) |
| puppeteer | 13: unreached-type 9, not-declared 3, R6 1 | 1: UNSUPPORTED-SHAPE | -12 (179 to 167) |

Each removed line is now a proposal, a more specific abstention, or a
silence. The 9 that became silence (7 pi, 2 puppeteer) were each checked
against the source: every one reaches its declaration and the consumer
reads 0 or 1 distinctive fields of it, so the unchanged two-field rule
keeps it silent, exactly as it would one level down (earned-silence.txt).

Observed and stated: a chain follows a re-export that names another
workspace package (puppeteer.ts:7 `export * from 'puppeteer-core'`). The 3
not-declared removals resolved that way; the proposals they produced
(Issue, Page) were adjudicated TRUE.

## New findings

S4-F-01 (existing id scheme) Three pi proposal ids shift their `#N`
suffix because new pairs with the same file basenames sort ahead of them.
No proposal's content changes.

S4-F-02 One existing pi pair gains a shape (`Usage`, through a two-level
chain). Adjudicated separately as G1: TRUE.

S4-F-03 (the failure, by cause) All 13 NOT TRUE verdicts are in pi.
12 are S3-F-15: the consumer reads same-named fields off an unrelated
receiver (`vm.fs.readFile` counted for WriteOperations; `usageTotals`
counted for Usage; `agent.state` counted for AssistantMessage). 1 is new:
a field WRITE counted as a read (`details.matchLimitReached = ...` on a
local typed GrepToolDetails, gondolin/index.ts:296 to 304). SLICE4 did not
create either error; it reached more shapes in the repository where the
receiver bound already cost the most (pi, 9 of 20 in EVAL-003).

S4-F-04 (instrument) The prototype clones were depth 1, which gives a
one-commit backtest. M-S-6 was run after fetching both clones to the
instrument's depth 200 at the pinned commits; the depth-1 outputs were
discarded, not reported.

## Adjudication

Same protocol as EVAL-003 (S4-D6): two fresh agents with no knowledge of
the engine, rubric D, the packet and the pinned source only, blind to each
other; TRUE only when both say TRUE. A first pair launched on 2026-10-19
was lost to a session limit before reporting; a second pair with the
identical prompt produced the verdicts here. No two of the 60 evidence
sentences are identical. Raw verdicts, the merge and score.py are in
adjudication/. This is AI adjudication, reported under that label.
