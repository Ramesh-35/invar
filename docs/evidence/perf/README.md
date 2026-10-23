# CHECK-PERF-1 evidence

memo-prototype.patch: the throwaway prototype measured before the draft
(on 93e93ce), kept for the record; the landed code is lock.rs and
surface.rs as committed.

Method: callgrind instruction counts (stable) and medians of wall time
(the sandbox is a shared machine; wall time there varies by up to half).
The scripts that produced the numbers are in scripts/perf/: bench.py
(self-contained synthetic workspace, or `--dir` for a real one) and
mcpchange.py (a running server against a fresh one across edits).
