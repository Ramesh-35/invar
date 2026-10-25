# G2 evidence (RECALL-ATTACK-1 phase 4)

Spec: docs/falsification/G2-SPEC.md (r1, a measurement erratum; r0 frozen
with the fixtures before code and ruled 2026-10-25: G2-D1, G2-D2, G2-D3 as
recommended; preserved as G2-SPEC-r0.md with G2-SPEC-r0.frozen). Freeze
record: G2-SPEC.frozen. Fixtures: tests/g2_test.rs (14, unchanged from r0).
Baseline: 9e89f33, G1-E release binary 806d759a5cf1. G2 binary:
d596dac5d3c5. Ids are sha256 prefixes. Sandbox: Linux x86_64.

| File | sha256 (16) | What it is |
|---|---|---|
| candidates.tsv | d27f156c7bc4e293 | every one-way rename candidate in the corpus, measured before code with the engine's own field extraction |
| mutate-run1.log | eb67d57f39685ab3 | first run, 11 operators: 10 killed by their declared killer; G2-M10 INVALID (the operator's text did not compile: a reference comparison in the mutant) |
| mutate-run2.log | 62e3f0f60a314ee4 | after correcting that operator's text only (rule and fixtures unchanged): 11 of 11 killed by their declared killer |
| corpus-before-after.tsv | 77e2d86d7278d753 | all 122 corpus cases, full `invar check --ci` output byte-compared, G1-E vs G2 |
| r1-076-woodpecker-link-url-terraform.diff | ea6fb02eeec84dbb | silent HOLDS (exit 0) to VIOLATED (exit 1) on `link_url`, read as `Link`, candidate `forge_url` |
| r1-010-romm-filesize-homepage.diff | 7a5c1bb49802c7d8 | verdict unchanged; witness gains candidate `TOTAL_FILESIZE_BYTES` |
| r1-070-vikunja-list-id-project-id-frontend.diff | 94638171a586bd52 | verdict unchanged; witness gains candidate `project_id` |
| roundtrip-original.tsv | 2f28374efe391805 | EVAL-007/008 round trips: 31 of 36 identical (raw or after the declared normalization), 5 locks DIFFERENT in time-derived fields only |
| roundtrip-locks-verified.tsv | 06a845608c1f1057 | scripts/rs/locks-verify.py on those 5 locks: every entry hash recomputes, all 63 entries byte-identical after substituting the baseline recorded_at |

Corpus movement: r1-076, r1-010, r1-070; 119 cases byte-identical,
including all 15 held-out controls. r0 pre-registered r1-057 as a fourth
witness change; that was a measurement error (its convicted field is a
retype, which the frozen rule never annotates), corrected in G2-SPEC r1
section J. An existing fixture (harness_test, change-scoped markdown)
exposed r0's stated rendering limitation; the implementation renders the
candidate's values in their own code spans, plain text unchanged, the
fixture unmodified. Tuning rung: the sealed record stays 38 of 70.

Not claimed: held-out recall; that the consumer should read the
candidate; Go renames that keep the JSON tag; r1-106 (conditional
emission of a surviving key).

Harness: scripts/g2/ (measure.py and candidate-probe.patch, the r0
measurements; mkmutants.py; mutate.sh).
