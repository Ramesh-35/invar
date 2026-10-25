# G3 evidence (RECALL-ATTACK-1 phase 1)

Spec: docs/falsification/G3-SPEC.md (r2, frozen) and G3-SPEC-r3.md (r3,
the bounded repair; no rule change from r2). Freeze records:
G3-SPEC.frozen and G3-SPEC-r3.frozen. Fixtures: tests/g3_test.rs (r2, 17)
and tests/g3r3_test.rs (r3, 7). Baseline: aed8988 (release binary
bca55be8506e). G3 binary: b62c0dbffa53. Sandbox: Linux x86_64. Native:
macOS, founder's machine, 2026-10-25.

| File | sha256 (16) | What it is |
|---|---|---|
| mutate-r2.log | 7a11e7c396f62ee2 | r2 harness, 14 operators: 12 behavior-killed, M4 survived (non-equivalent), M8 equivalent |
| mutate-r3-aborted.log | 261e0ccd13d21759 | first r3 run, aborted after M1: the harness lacked --no-fail-fast, so the r3 fixture file did not run whenever the r2 file failed |
| mutate-r3-aborted-harness.sh | c801aba3ce7b8e5d | that defective harness, kept as run |
| mutate-r3.log | 74427333c445b33b | corrected r3 harness, 18 operators: 17 killed by their declared killer, M8 equivalent (argued in G3-SPEC-r3.md section 13) |
| corpus-before-after.tsv | d1e52bc68b048d80 | all 122 corpus cases, full `invar check --ci` output byte-compared, baseline vs G3 |
| r1-113.diff | 4a8737d8fbb7b6b6 | the one changed case: silent HOLDS (exit 0) to stated NOT EVALUATED (exit 4) |
| roundtrip-original.tsv | 2e325fd48a4b205c | EVAL-007/008 round trips, baseline vs G3, declared normalization (times, engine ids): 18 raw-identical, 12 identical after normalization, 6 locks DIFFERENT |
| roundtrip-locks-corrected.tsv | 0f827a457ce9eab4 | separate corrected comparison of those 6 locks: every entry hash first verified to recompute from its own entry, then normalized: all 6 identical |
| probe-ab-variants.tsv | bdeff652c569506f | every selector-typed numeric variant in all 273 corpus producer texts: 8 literals, all in r1-113, all locally declared |

Limitation of the original round-trip comparison, kept explicit: it did
not normalize the entry hash, which is a function of `recorded_at`. The
mechanism was then reproduced for all 64 hold records (substituting the
baseline `recorded_at` makes candidate entries byte-identical with
identical hashes), and the corrected comparison above is a separate
artifact; the original result stands as recorded.

Native verification: aed8988 425 of 425 (2026-10-25), then with G3 landed
449 of 449 (2026-10-25), every pre-existing target unchanged, g3_test 17,
g3r3_test 7.

Not claimed: held-out recall (the sealed record stays 38 of 70; only a
frozen-engine batch 9 can move it); mechanical certification; any
semantic ownership of a value inside an open P6 window beyond the stated
grammar.

Harnesses: scripts/g3/ (r2: mkmutants.py, mutate.sh, corpus-capture.sh,
roundtrip-compare.py) and scripts/g3r3/ (r3: mkmutants.py, mutate.sh).
