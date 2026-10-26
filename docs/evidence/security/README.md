# SECURITY-1 evidence (S0 threat model, S1 reuse authenticity)

Spec: docs/falsification/SECURITY-SPEC.md (r0). Freeze record:
SECURITY-SPEC.frozen. Fixtures: tests/security_test.rs (9, new); revised
under SEC-D2: tests/check_incr1_test.rs (helper only) and
tests/gate_test.rs (g_m1), prior versions kept here. Baseline: the
self-audit tree, release binary 14e0dce4e181. S1 binary: 01f9ae96b7d2.

| File | sha256 (16) | What it is |
|---|---|---|
| probes-s0.txt | 92bdb32545c7b79d | every S0 probe run: forgery on the pre-S1 and S1 engines, robustness, token redirect, terminal injection, walk loop |
| mutate-s1.log | 2dd3a360fb90f004 | 6 S1 operators, all killed by their declared killer |
| corpus-before-after.tsv | b85feb887a069a86 | all 122 corpus cases (`--ci`), self-audit vs S1: 122 identical |
| roundtrip-original.tsv | 2f1984529a41996d | EVAL-007/008 round trips: check outputs differ in reuse wording only; every contract's verdict identical; the first local check equals the pre-S1 full evaluation byte for byte on all three repositories |
| check_incr1_test.pre-s1.rs | 1cc886b8c9973e9d | CHECK-INCR-1 fixtures as frozen before SEC-D2 |
| gate_test.pre-s1.rs | d2be9e2a16de9e08 | gate fixtures as before SEC-D2 |

Confirmed and NOT yet fixed (S0, for ruling): terminal control-byte
injection through the witness line (path 4); exponential directory walk
over symbolic-link loops, reachable from a repository's own invar.toml
(path 5). Probes: scripts/security/.
