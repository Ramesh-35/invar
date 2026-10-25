# G1-E evidence (RECALL-ATTACK-1 phase 3)

Spec: docs/falsification/G1-E-SPEC.md (r0, frozen; ruled 2026-10-25:
G1E-D1 rule approved, G1E-D2 r1-068 a binding limitation, G1E-D3
`send_result(` payload is the second argument, all as recommended).
Freeze record: G1-E-SPEC.frozen. Fixtures: tests/g1e_test.rs (15).
Baseline: fb85ae9, request-side release binary 4852d4438957. G1-E binary:
806d759a5cf1. Ids are sha256 prefixes. Sandbox: Linux x86_64.

| File | sha256 (16) | What it is |
|---|---|---|
| r1-068-url-isolation.txt | a331fbe13b2c3588 | the bound consumer never reads `url`; it is read in dialog-thread-dataset.ts:52 through an object passed at thread-config-panel.ts:340 |
| mutate-run1.log | ed9153a084545846 | first run, 12 operators: 11 killed by their declared killer; G1E-M2 INVALID (the mutant made an unguarded `opq[0]` index reachable and the engine panicked) |
| mutate-run2.log | 71d0ce2e977e3139 | after the index was made total (`opq.first()`; unreachable in production, where condition 2 guarantees a site): 12 of 12 killed by their declared killer |
| corpus-before-after.tsv | 8d8fb41a5947ae32 | all 122 corpus cases, full `invar check --ci` output byte-compared, request side vs G1-E |
| r1-107-linkwarden-tags-envelope-homepage.diff | 45e15c09acbd4711 | the one changed case: silent HOLDS (exit 0) to stated NOT EVALUATED (exit 4) |
| roundtrip-original.tsv | c39a2f09a567ae05 | EVAL-007/008 round trips: 33 of 36 identical (raw or after the declared normalization), 3 locks DIFFERENT in time-derived fields only |
| roundtrip-locks-verified.tsv | 22dc4c92fb3d29cc | scripts/rs/locks-verify.py on those 3 locks: every entry hash recomputes, and all 60 entries are byte-identical after substituting the baseline recorded_at |

Corpus movement, as pre-registered: r1-107 only; 121 cases byte-identical,
including all 15 held-out controls. r1-107 is a held-out break examined
during specification, so the movement is tuning rung: the sealed record
stays 38 of 70.

Not claimed: held-out recall; which head site replaced which base site;
member-access or call payloads; emission outside the seven frozen forms;
masking of comments inside multi-line arguments; cross-file object flow
(r1-068, r1-044).

Harness: scripts/g1e/ (measure.py, the r0 measurement counter;
mkmutants.py; mutate.sh). Corpus capture and round trips reuse
scripts/g3/corpus-capture.sh, scripts/g3/roundtrip-compare.py and
scripts/rs/locks-verify.py.
