# STAGE3-EVAL-002: the pinned manifest, measured with the unlanded SLICE3 engine

Engine: SLICE3 implementation (docs/evidence/slice3/slice3-implementation.patch,
binary c409ff539db6), NOT landed. Same pinned manifest and instrument r2 as
STAGE3-EVAL-001, which is never overwritten.

| Metric | Verdict |
|---|---|
| M-S-1 ISA conformance | PASS, 10 of 10 |
| M-S-2 determinism | PASS, 10 of 10, raw bytes |
| exports-related UNRESOLVED | 6,394 to 423, the pre-registered ceiling exactly |
| proposals | 9 to 186 (pi 113, excalidraw 46, Understand-Anything 13, puppeteer 5) |
| M-S-3 precision | PENDING: 58 proposals (first 20 per repo) in adjudication-packet.md |
| M-S-4 abstention honesty | FAIL: 4 genuine silent drops among 26 sampled constructs (m-s-4-triage.md) |
| M-S-5, M-S-6 | not run: they follow adjudication, and the arc stopped at M-S-4 |
| M-S-7 performance | init 0 to 7 seconds per repository |

Consequence: SLICE3 stays unlanded. INIT-DISCOVERY-SLICE3B (earned silence)
is the drafted remedy; after it, STAGE3-EVAL-003 re-measures every metric.
