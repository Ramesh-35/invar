# STAGE3-EVAL-008: INIT-DISCOVERY-SLICE8 and LOCK-SLICE2 on the pinned manifest

Engine: INIT-DISCOVERY-SLICE8 (receivers reached through a property
declared as the shape, at most two hops), uniform scope, with LOCK-SLICE2
(one-line `Readonly<{ ... }>` bodies in lock), on db401f4, release binary
502689914b48. Manifest hash 2f5b6b0f798b, instrument r2, the same ten
pinned repositories as STAGE3-EVAL-001 to -007, all of which stand
unmodified.

Verdict: PASS on every gate. The yield is larger than the draft's
forecast; every deviation is explained below, nothing was tuned.

| Gate or target | Measured |
|---|---|
| M-S-1 ISA conformance | 10 of 10, every init exit 0 |
| M-S-2 determinism | 10 of 10, raw bytes (puppeteer re-fetched once after a transient TLS failure in its second clone; both of its runs then completed) |
| byte preservation | 7 repositories differ from EVAL-007 in the header sentence only (S8-D3); in the other 3, 0 blocks removed, 0 ids renamed, 10 added, 15 changed, each only by its status line and the added shape's E2, E4, E5 and evidence lines (m-s-4-compare.txt) |
| M-S-4 silent drops (hard gate) | 0: 34 abstention lines removed (27 R6, 7 RECV), 0 added; each is exactly one of the 34 new shapes, in its consumer file |
| M-S-3 precision, every new shape (floor 60 percent) | **34 of 34 TRUE**; two blind agents, conservative merge, agreement 34 of 34 |
| the probe's 20 (in sample) | 20 of 20 TRUE |
| the 14 the probe did not find (out of sample) | 14 of 14 TRUE |
| R6 conversions / RECV rescues | 27 of 27 / 7 of 7 TRUE |
| M-S-5 round trip, the 25 contracts touched (10 new, 15 changed) | 25 of 25: lock exits 0, every evidence field bound (85 + 56 + 4 bindings), check HOLDS with the durability line, second check byte-identical. Before LOCK-SLICE2: 24 of 25 (finding S8-F-05) |
| M-S-6 backtest, the same contracts | 3 of 3 repositories complete, exit 0, byte-identical reruns, depth 200; 14 absent events, every one before the field's introduction |
| M-S-7 performance | init 0 to 9 seconds per repository |

## Against the draft (section F)

| Expected | Measured |
|---|---|
| conversions near +17 (one hop) or +20 (two hops) | 27 R6 conversions and 7 RECV rescues (34 shapes) |
| concentrated in pi and Understand-Anything | pi 21 shapes, Understand-Anything 12, excalidraw 1 |
| proposals | 152 to 162 (10 new pairs, 13 shapes; 21 shapes added to 15 existing pairs) |
| every other repository and proposal byte-identical | yes, apart from the ruled header and status lines |

Why more: the engine ties receivers in every SLICE5 form (casts, declared
returns of local functions, parameters), where the probe matched
annotations only, and it opens `type X = Readonly<{ ... }>` bodies the
probe's declaration reader skipped (7 more R6 lines); and the uniform
rule rescues 7 RECV lines the probe never looked at. All 14 extra shapes
were judged out of sample and all are TRUE.

## Adjudication

One packet (build_packet.py): one item per shape whose E4 says it was
resolved via a property; the script first checks that this set equals the
shapes added since EVAL-007 and stops otherwise (equal, 34). Two fresh
agents, the packet, the rubric and the pinned source only, blind to each
other; TRUE only when both say TRUE (score.py). AI adjudication, reported
under that label.

## Files

phase1/ (run.sh outputs and ledger, puppeteer's from its re-run),
m-s-4-compare.txt (compare.py of EVAL-007), compare_pairs.py (pairs by
their E6 line, so a renamed id shows), adjudication/, m-s-5/ and m-s-6/
(roundtrip.sh over accepted.tsv).
