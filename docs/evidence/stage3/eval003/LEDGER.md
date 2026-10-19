# STAGE3-EVAL-003: SLICE3 + SLICE3B on the pinned manifest

Engine: INIT-DISCOVERY-SLICE3 and SLICE3B, release binary 3e0bb89880be.
Manifest hash 2f5b6b0f798b, instrument r2, the same ten pinned
repositories as STAGE3-EVAL-001 and -002, both of which stand unmodified.

| Metric | Verdict |
|---|---|
| M-S-1 ISA conformance | PASS, 10 of 10 (every init exit 0) |
| M-S-2 determinism | PASS, 10 of 10, raw bytes |
| proposals | 186, byte-identical to STAGE3-EVAL-002 (9 in EVAL-001) |
| new abstentions vs EVAL-002 | 194 added, 0 removed; every other output line byte-identical |
| pre-registered prediction (B4) | 205; measured 194; all 17 differences attributed, 0 unexplained (below) |
| M-S-4 abstention honesty | PASS: 0 genuine silent drops (detector as registered: 1 stoplist-aligned flag, the S3-F-14 spread false positive; spread-corrected detector: 0) |
| the 4 EVAL-002 drops | all now abstain and name their declaration sites (listMarker.ts:193; Configuration.ts:87, :127, :167) |
| M-S-3 precision | PASS: 41 of 58 new code-pair proposals TRUE (71 percent) under the conservative merge of two independent blind adjudications; pooled with the founder's 8 of 9 STAGE3-EVAL-001 strings verdicts, 49 of 67 (73 percent) against the 60 percent floor; TRUE proposals in 4 repositories (adjudication/) |
| M-S-5 round trip | PASS: all 41 TRUE contracts lock, check exits 0 with the durability line, second check byte-identical (Understand-Anything 13, excalidraw 17, pi 9, puppeteer 2); the 8 TRUE strings contracts passed in STAGE3-EVAL-001 with check and lock byte-identical since |
| M-S-6 backtest | PASS: 4 of 4 complete, exit 0, deterministic; 0 absence windows, 0 never-present bindings, 0 blind spots over up to 55 replayed producer commits; 8 absent events are field introductions, 6 of 6 spot-checked as truly absent at that commit |
| M-S-7 performance | init 0 to 7 seconds per repository |

## Reconciliation against the pre-registered 205 (reconcile/, reconcile.py)

| Repository | Census | Engine | Differences and cause |
|---|---|---|---|
| excalidraw | 72 | 59 | 13: `UI` named as a type only inside comments or strings; the engine masks them and sees the value (engine right) |
| pi | 77 | 78 | 1: `InteractiveMode` whose value declaration the census read inside a string (engine right) |
| puppeteer | 49 | 48 | 1: `export const enum PageEvent`, which the census regex misread (engine right) |
| playwright | 6 | 8 | 2: `Playwright`, `ChannelOwner`, loud under the evidence-completeness rule added at the rehearsal |
| freeCodeCamp | 1 | 1 | none |
| Understand-Anything | 0 | 0 | none |
| total | 205 | 194 | 17 differences, 0 unexplained |

## New findings

S3-F-14 (detector bound) The M-S-4 detector matched the spread
`...options` as a property read `.options` (pi harness-host-inputs.test.ts,
`Models`), the one stoplist-aligned flag of this run. The engine found the
declaration and correctly counted one distinctive read. Recorded as
S3-INT-10: the detector no longer counts spreads; both counts are kept.

## Adjudication protocol deviation (stated, not hidden)

S-D3 named the founder as adjudicator. On 2026-10-19 (15:53) the founder
delegated the 58 judgments to Claude. Because Claude built the engine, the
judgments were made by two fresh, independent agents with no knowledge of
the engine or of this evaluation, each given only rubric D, the packet and
the pinned source, working blind to each other. A proposal is TRUE only when
both said TRUE; a disagreement is UNSURE and scores NOT TRUE (rubric D).
Agreement 49 of 58 (84 percent), Cohen's kappa 0.55. Raw verdicts, the
merge and the scoring script are in adjudication/. This is AI adjudication,
not the human adjudication S-D3 registered; precision is reported under
that label.

## Exploratory probes (not pre-registered)

Renaming EditorInterface.canFitSidebar (a name that occurs once in the
producer) made check exit 1, naming all three stale reads in App.tsx (473,
3730, 4665). Renaming formFactor, whose name also survives elsewhere in the
producer file (`EditorInterface["formFactor"]`), made check exit 4 NOT
EVALUATED, naming the reads: it cannot tell a rename from a move, and
declines to pass rather than guess. Neither is ever a false HOLD.

S3-F-15 (expected behavior confirmed; the precision limiter) Every NOT TRUE
verdict traces to init's declared bound, "receiver of field reads not
verified": the consumer reads same-named fields off a different object.
It concentrates in pi (9 of 20 TRUE), whose shapes use common field names
(content, isError, auth, details, source) shared by many lookalike types.
