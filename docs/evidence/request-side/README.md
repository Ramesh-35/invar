# Request-side evidence (RECALL-ATTACK-1 phase 2)

Spec: docs/falsification/REQUEST-SIDE-SPEC.md (r2, frozen; r1 and r0
preserved). Freeze records: REQUEST-SIDE-SPEC.frozen (r2) and
REQUEST-SIDE-SPEC-r1.frozen. Fixtures: tests/rs_test.rs (r1, 22) and
tests/rs_r2_test.rs (r2, 1). Baseline: 201d193, G3 release binary
b62c0dbffa53. Request-side binary: 4852d4438957 (identical for r1 and r2:
r2 changed no code). Ids are sha256 prefixes. Sandbox: Linux x86_64.

| File | sha256 (16) | What it is |
|---|---|---|
| r1-118-pdfformat-inventory.tsv | 1c98bd3bd1999ed0 | every case-insensitive `pdfformat` in the r1-118 head producer: 21 on 15 lines, no string literal, no B2 read |
| mutate-r1.log | d975d1846ac81764 | r1 harness, 16 operators: 15 killed by their declared killer, RS-M2 SURVIVED (non-equivalent) |
| rs-m2-survivor-probe.txt | a018fb4088011c04 | throwaway workspace showing RS-M2 is a false conviction: head still reads `req.body.pdfFormat`; engine HOLDS, mutant VIOLATED |
| mutate-r2.log | e83cfe6844e10252 | r2 harness, same 16 patch bodies, both fixture files: 16 of 16 killed by their declared killer |
| corpus-before-after.tsv | 479adc1b53b9e12e | all 122 corpus cases, full `invar check --ci` output byte-compared, G3 vs request side |
| r1-118-gotenberg-pdfformat-paperless.diff | 55e4286f8560667d | silent HOLDS (exit 0) to VIOLATED (exit 1) on `pdfFormat` |
| r1-060-netbox-device-role-ansible.diff | 59f11fd24efe596b | silent HOLDS (exit 0) to stated NOT EVALUATED (exit 4), read-only-field |
| roundtrip-original.tsv | ea6cfb317bfe7c07 | EVAL-007/008 round trips, G3 vs request side: 32 of 36 identical (raw or after the declared time and engine-id normalization), 4 locks DIFFERENT in time-derived fields only |

The 4 locks were then checked by scripts/rs/locks-verify.py: every entry
hash recomputes from its own rendered entry, and substituting the baseline
`recorded_at` makes all 61 candidate entries byte-identical with identical
hashes; block hashes and held_input lines are identical.

Corpus movement, as pre-registered: r1-118 and r1-060 only; 120 cases
byte-identical, including all 15 held-out controls. Both moved cases are
held-out breaks examined during specification, so the movement is tuning
rung: the sealed record stays 38 of 70.

RS-0 has no corpus evidence: it fires on zero cases, and its fixtures
(rs_f06, f07, f08, f08b, f08c) are synthetic.

Not claimed: held-out recall; route association (r1-051 stays
unsupported); keys not written as literals; trailing-comment masking;
a separate SARIF location for RS-1; RS-1's no-duplicate rule (code reading
only, no fixture).

Harnesses: scripts/rs/ (r1: mkmutants.py, mutate.sh; r2: mkmutants-r2.py,
mutate-r2.sh; locks-verify.py). Corpus capture and round trips reuse
scripts/g3/corpus-capture.sh and scripts/g3/roundtrip-compare.py.
