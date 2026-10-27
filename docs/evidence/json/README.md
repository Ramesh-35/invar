# JSON-1 evidence (AGENT-1 P1, structured verdicts for agents)

Spec: docs/falsification/JSON-SPEC.md (r0), freeze record JSON-SPEC.frozen.
Fixtures: tests/json_test.rs (14), frozen before any code. Ruling of
2026-10-26: JSON-D1 to JSON-D3 as recommended. Baseline: the S2 engine
(d2c449b14aa1). P1 binary: 967f9f8ac5c4.

| File | sha256 (16) | What it is |
|---|---|---|
| measure-baseline.txt | 5a97c716fd26af94 | section C: scripts/json/measure.py over the S2 corpus capture |
| premises-baseline.log | c5fb780362e9e2ef | the fixtures with every JSON-1 assertion removed (json_premise_test.rs), on the S2 engine: 14 of 14 pass, so every premise is real behavior |
| json_premise_test.rs | a11af4e0cdd19d07 | that premise-only variant |
| fixtures-baseline.log | 59ec10e42e60cb0a | the frozen fixtures on the S2 engine: 13 FAIL at a JSON-1 assertion, the text guard passes |
| mutate.log | f362305ea4e2de0f | scripts/json/mutate.py: 16 operators, all killed by their declared killer |
| corpus-text.tsv | d774514800b32353 | all 122 corpus cases, CLI text S2 vs P1: 122 identical |
| corpus-json.txt | 74fd54d3515a5b7e | scripts/json/corpus_json.py: MCP check on all 122 cases; existing keys identical to S2 on 122 of 122; rules 108 absent, 18 retyped, 1 request-side; producers named 127 of 127; 3 candidates; 96 statements all classified, none `unclassified`; 6 skips `unsupported-language` |
| roundtrip.tsv | 02c5b8b8055e8856 | round trips S2 vs P1: every check and backtest output identical; 6 locks differ in time-derived fields only |
| roundtrip-locks-verified.tsv | 47a975b6ab045887 | those 6 locks, identical after recorded_at substitution |

Method note, stated: the corpus was captured with the pinned texts served
locally in place of `curl` (the sandbox network refuses raw GitHub
fetches); the S2 binary run that way reproduced the recorded S2 capture
byte for byte on all 122 cases first. See docs/evidence/security/README.md
(S3).
