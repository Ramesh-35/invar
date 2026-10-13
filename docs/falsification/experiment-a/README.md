# Experiment A: MCP output-contract breaks across releases

Pre-registered decision experiment from vacancy pass G
(../vacancy-gpass-2026-10-13.md). Status: CLOSED, REFUTED on condition 2 (zero public
consumers reading a broken field); condition 3 not run, reason recorded.

- PREDICTIONS.md: method, thresholds and predictions, frozen before any
  data, with the addendum, smoke-test corrections and the aborted run 1.
- RESULT-MINING.md: verified result. 24 breaking output-change events in
  a seeded 30-server sample, every flag verified by hand; concentrated in
  3 servers and 5 of 229 release transitions.
- stage3.jsonl / stage3-aborted-run1.jsonl: per-server records.
- frame.tsv: the registry frame (repo, tag count, outputSchema files).
- Scripts: ../../../scripts/experiment-a/ (static only; no third-party
  code is ever executed). extract.py is a research extractor, NOT an
  INVAR feature: INVAR does not parse MCP output schemas today.
