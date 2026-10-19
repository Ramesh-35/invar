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
| M-S-3, M-S-5, M-S-6 | NOT RUN: they require the founder's rubric D adjudication of the 58-proposal packet; on 2026-10-19 the founder classified all 58 as unverified hypotheses rather than adjudicating them, so precision is not scored |
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
