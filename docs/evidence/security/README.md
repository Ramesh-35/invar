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

## S2 (SEC-D4 terminal neutralization, SEC-D5 walk and path), SECURITY-SPEC r2

S2 binary: d2c449b14aa1. Fixtures: tests/security_s2_test.rs (14), frozen
before any S2 code (r1, then r2 still before code: one killer corrected).

| File | sha256 (16) | What it is |
|---|---|---|
| probes-s2.txt | b85625cf10bf37a4 | terminal injection: no raw control byte; link loop: returns at once; forgery: all exit 1 |
| mutate-s2.log | fd2b26dcf1234d60 | 9 S2 operators, all killed by their declared killer |
| corpus-s2.tsv | f75fa99a85dad2f1 | all 122 corpus cases, S1 vs S2: 122 identical |
| roundtrip-s2.tsv | e7f954b215b4892e | round trips S1 vs S2: every check output identical; 5 locks differ in time-derived fields only |
| roundtrip-s2-locks-verified.tsv | 91862f14d6cebd26 | those 5 locks, identical after recorded_at substitution |

## S3 (SEC-D6 programs INVAR runs, SEC-D7 supply chain), SECURITY-SPEC r3

S3 binary: 60866e1afd13 (built clean twice by repro-check.sh from two
paths: identical). Fixtures: tests/security_s3_test.rs (11), frozen before
any S3 code. Ruling of 2026-10-26: S3-D1 to S3-D4 as recommended.

| File | sha256 (16) | What it is |
|---|---|---|
| git-hostile-s3.tsv | dbde8e7bba43febd | before: INVAR's 7 git invocations against 7 hostile .git/config vectors, each with a firing positive control; only `hash-object` runs a program (filter clean, filter process) |
| git-hostile-s3-after.tsv | 0b6b60777be8e000 | after: `hash-object --no-filters` runs neither filter |
| repro-check-s3-macos.txt | 16e0ba4a9fd3dae8 | native run on the founder's Mac, commit 5f7619e: binary 13be8c9a2f63 and archive 2249ceda31ce identical from two paths (macOS differs from Linux, as a different target must) |
| git-hostile-s3-macos.tsv | 0539d2f2542c4997 | native run on the founder's Mac (git 2.54.0): every row identical to the sandbox (git 2.43.0); only `hash-object` runs the filters, all 7 controls fire |
| run-binpath-s3.tsv | e558fa8a1340651b | before: `run` and `report` execute composite.toml's bin_path |
| run-binpath-s3-after.tsv | 1d536514bfe07eab | after: neither executes it |
| audit-s3.txt | 2a0e6032d6c2aa97 | cargo audit 0.22.2: 57 crates, 1,352 advisories, 0 vulnerabilities, 0 warnings |
| repro-s3.txt | faee9bcd6c4b3280 | before: the S2 binary built from two paths is identical; the release tar.gz of one binary differs between two runs |
| repro-check-s3.txt | edc1778bd02d70f7 | after: scripts/release/repro-check.sh, binary and packed archive identical from two paths |
| s3-fixtures-baseline.log | f8e54fc4010f8457 | the 11 fixtures on the S2 engine: 9 FAIL at their own assertion, 2 guards pass |
| mutate-s3.log | f411ed8ccce2d902 | 10 S3 operators, all killed by their declared killer |
| corpus-s3.tsv | d774514800b32353 | all 122 corpus cases, S2 vs S3: 122 identical |
| roundtrip-s3.tsv | 02c5b8b8055e8856 | round trips S2 vs S3: every check and backtest output identical; 6 locks differ in time-derived fields only |
| roundtrip-s3-locks-verified.tsv | 47a975b6ab045887 | those 6 locks, identical after recorded_at substitution (scripts/rs/locks-verify.py) |

Method note, stated: the sandbox's network now refuses raw GitHub fetches
(403), so the corpus was captured with the 395 pinned texts measured on
2026-10-25 served locally in place of `curl`. Validation before use: the S2
binary run that way reproduced the recorded S2 capture byte for byte on all
122 cases, fetched-text BLAKE3 lines included.

Native (S3-D4): 548 of 548 tests on macOS; the git probe on macOS git
2.54.0 matches the sandbox row for row; repro-check.sh on macOS is
reproducible (binary and archive identical from two paths). Not yet
measured: GitHub running the two new CI jobs (the first push).
