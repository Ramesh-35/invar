# AUDIT evidence (AGENT-1 P2: self-audit)

Spec: docs/falsification/AUDIT-SPEC.md (r1, bounded repair; r0 frozen
with its fixtures before code and ruled 2026-10-25: AU-D1 schema scope,
AU-D2 whole tokens; preserved as AUDIT-SPEC-r0.md with
AUDIT-SPEC-r0.frozen). Freeze record: AUDIT-SPEC.frozen. Fixtures:
tests/audit_test.rs (r0, 12) and tests/audit_r1_test.rs (r1, 1). Baseline:
a4aa356, G2 release binary d596dac5d3c5. Audit binary: 14e0dce4e181
(identical for r0 and r1 gate runs: r1 changed no code). Ids are sha256
prefixes. Sandbox: Linux x86_64.

| File | sha256 (16) | What it is |
|---|---|---|
| mutate-r0.log | 234cdd135fdb8261 | r0 harness, 9 operators: 8 killed by their declared killer; AU-M5 SURVIVED (non-equivalent) |
| au-m5-survivor-probe.txt | 39d59eab24522594 | throwaway test: for `row["template-id"]` the engine accepts and the AU-M5 mutant wrongly rejects |
| defect-injection-r0.log | 3e055343e55ba670 | DI-1 (site lines off by one) and DI-2 (wrong file name), r0 harness: 65 of 65 corpus convictions become exit-3 self-audit failures, none prints [VIOLATED] |
| mutate-r1.log | e4ba0f531bfcdaa4 | r1 harness, same 9 patch bodies, both fixture files: 9 of 9 killed by their declared killer |
| defect-injection-r1.log | 3e055343e55ba670 | DI-1 and DI-2 rerun: 65 of 65 trapped, 0 printed, 40 fixture tests fail under each defect |
| corpus-before-after.tsv | f292a0a8d1121fed | all 122 corpus cases, full `invar check --ci` output byte-compared, G2 vs audit: 122 identical |
| roundtrip-original.tsv | ea6cfb317bfe7c07 | EVAL-007/008 round trips (lock mode, where the audit also runs): 32 of 36 identical, 4 locks DIFFERENT in time-derived fields only, no audit text anywhere |
| roundtrip-locks-verified.tsv | 63f8f2b780d1fbfc | scripts/rs/locks-verify.py on those 4 locks: all 61 entries byte-identical after substituting the baseline recorded_at |

The existing fixture suite found r0's lock-mode gap on the first full run
(13 tests): lock mode shows `Type.field` while the consumer reads
`field`; the lock path now audits the bound field it matched, the rule
not loosened. Recorded in AUDIT-SPEC r1 section I.

Not claimed: that a cited read is semantically a read of that field
(r1-116 passes the audit); audit of non-schema contract kinds.

Harness: scripts/audit/ (r0: mkmutants.py, mutate.sh, inject.sh; r1:
mkmutants-r1.py, mutate-r1.sh, inject-r1.sh; measure.py, the r0
measurement).
