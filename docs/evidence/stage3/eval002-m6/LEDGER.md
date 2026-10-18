# STAGE3-EVAL-002-M6: backtest re-measurement after INVAR-BACKTEST-002

A new identifier per STAGE3-EVAL-001 B-S-4; the EVAL-001 record is
unchanged. Same pinned SHAs, same accepted manifests and locks as
EVAL-001 phase 2, depth-200 pinned fetches, `invar backtest --depth 100`.
Engine: INVAR-BACKTEST-002 (backtest.rs only changed).

| repository | EVAL-001 | EVAL-002-M6 |
|---|---|---|
| thedotmack/claude-mem @ b85cd123daf8 | 17 absent, 0 present (all false, S3-F-05) | 17 present, 0 absent, shallow history stated, exit 0 |
| diegosouzapw/OmniRoute @ 3b52f62d9cc3 | 44 absent, 0 present (all false, S3-F-05) | 44 present, 0 absent, shallow history stated, exit 0 |

Both: two runs byte-identical; JSON key set identical to EVAL-001's, only
event states changed. M-S-6 adjudication: every absence event is now
absent (none exist), and the one present event per binding matches what
check reads on the same bytes.

Bound: the clones are shallow (S3-F-07), so 1 to 3 producer commits were
replayed; this re-measures the reader, not deep history.
