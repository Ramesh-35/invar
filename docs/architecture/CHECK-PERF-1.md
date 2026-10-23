# CHECK-PERF-1: memoized check analyses (latency of `invar check` and `invar mcp`)

Status: RATIFIED (P-D1 to P-D4, founder, 2026-10-22), implemented through
Gates 0 to 5 and LANDED; sections J and K. Sections 0 to I are the draft r1
text, unchanged. Parent: the
founder's direction of 2026-10-21 (profile `check`, measure the MCP round
trip, evaluate memory-mapping the lock). Baseline: 93e93ce, release binary
502689914b48. Measured on the sandbox (Linux x86_64); the Mac will differ,
so the same scripts are to be re-run there.

## 0. Evidence, measured before this draft (throwaway prototype)

Workload: the 18 EVAL-008 contracts of pi (4 distinct producers, 17
consumers, 516 KB of consumer text), plus Understand-Anything (6) and
excalidraw (1). Wall time is the median of 20 runs; instructions are
callgrind counts.

| Path | Baseline | Prototype |
|---|---|---|
| process start (`invar --help`) | 2.0 ms | 2.0 ms |
| `invar check`, pi, cold process | 33.2 ms, 160 M instructions | 19.8 ms, 96 M |
| `invar check`, Understand-Anything | 8.6 ms | 6.4 ms |
| `invar check`, excalidraw | 4.5 ms | 4.5 ms |
| `invar mcp` tool `check`, warm server, pi | 31.5 ms | **2.9 ms** |
| `invar mcp` tools `contract`, `coverage` | 0.9, 0.7 ms | unchanged |

Where the baseline time goes (callgrind, pi): producer surfaces 53 percent
(the same 4 producer files re-read and re-parsed for each of 18
contracts; a zod scan of every TypeScript producer although no TypeScript
contract names a zod shape), consumer reads 36 percent (per-line substring
searches for four keys build a searcher on every line), the rest is
building and comparing the maps. File I/O is negligible: 43 opens and 83
reads, under 1 ms in total, so memory-mapping `invar.lock` (a few KB) has
nothing to remove.

The prototype (docs/evidence/perf/memo-prototype.patch), three changes, no new
dependency:

M1 Per-process memo of the pure analyses, keyed by the text's BLAKE3 (and
the artifact name where the analysis depends on it): a producer's live
surface (with its pointer, code companion and named zod shapes in the
key), a consumer's reads. A file whose bytes change gets a new key, so a
warm `invar mcp` sees edits at once.
M2 `broad_reads` searches each of its four keys once over the whole text
and replays the matches per line in the original order.
M3 Shared, not cloned, cached maps.

Equivalence, measured:
- `check` output byte-identical to the baseline on all three repositories;
- 397 of 397 tests pass;
- `broad_reads` output identical over 18,206 corpus files (2,037,643
  reads, every `.ts`, `.tsx`, `.js`, `.py`, `.rs` and `.go` file of the six
  cloned repositories);
- warm-server change detection (scripts/perf/mcpchange.py): the same
  server answers before a producer edit (18 hold), after it (13 hold, 5
  violated) and after the revert (18 hold), each answer equal to a fresh
  process's.

## Bounds

B1 No verdict, exit code, output byte or file format changes.
B2 Memo keys are content hashes; nothing is cached across processes or on
disk, so nothing can be stale.
B3 Zero new dependencies.
B4 The stated latency is the measured one, on the stated machine and
workload, never a general SLA.

## Decision points for ruling

P-D1 Land M1 to M3 (recommended: warm MCP check 31.5 to 2.9 ms on pi,
cold CLI 33 to 20 ms, byte-identical).
P-D2 Memory-mapping the lock: REJECT (recommended: I/O is under 1 ms of
33 ms; mmap adds unsafe code for nothing measurable).
P-D3 Latency claim: state only measured figures, per path ("warm MCP
check: 2.9 ms on pi's 18 contracts, Linux x86_64"); no blanket sub-5 ms
SLA (recommended: cold CLI check is 4 to 20 ms here and scales with
consumer text).
P-D4 A later, separate slice for cold CLI: an allocation-free reads
scanner (most of the remaining 96 M instructions are per-read String
allocations and map building), measure first.

## Gates if ruled

Gate 1 fixtures before code: warm-server change detection (edit, revert),
key separation (same text under two artifact names; same producer, two
zod-shape sets), a golden check output, and the broad_reads corpus
equivalence as a recorded measurement. Gate 3 mutants: content hash
dropped from a key, zod shapes dropped from a key, the artifact name
dropped, a stale entry served after an edit, the key replay out of order.
Gate 5: `check` byte-identical on every EVAL-008 round-trip contract, and
the latency table re-measured with the release binary.

## J. Rulings of 2026-10-22 (recorded)

P-D1 RATIFIED: land the memo, the single-pass key scan and shared maps.
P-D2 RATIFIED: memory-mapping the lock rejected.
P-D3 RATIFIED: latency is published per path, as measured, never as a
blanket SLA. The founder's sentence ("Sub-3ms verification for running
`invar mcp` agent sessions; 4 to 20 ms for cold CLI process executions
depending on codebase scale") is recorded here as the intent; the figures
published are the gate's own (K), because the gate measured 2.9 to 4.4 ms
for a running session on this machine, not a steady sub-3 ms.
P-D4 RATIFIED: a fresh-process speed-up is a later slice, measured first.
Also directed: the bench scripts in scripts/perf/.

## K. Results

Gate 0 PASS: 284 crate and script files identical at 93e93ce (Mac and
sandbox, 9fc09fda44ed).
Gate 1 PASS: tests/check_perf1_test.rs frozen before code (cfca63f8ce9c):
P-F1 a running server sees every producer and consumer edit and answers
as a fresh process does; P-F2 the same bytes under two artifact names are
read twice; P-F3 key reads across lines and CRLF; P-N1 check output
golden. Invariance fixtures: all pass on the baseline, as they must.
Gate 2 PASS: lock.rs and surface.rs only. Two levels of memo, every key
(artifact name, BLAKE3 of the bytes): a code file's fields and its zod
objects (shared by every contract on that file), a contract's whole live
surface (with its pointer, zod shapes and companion in the key), a
consumer's reads. `broad_reads` searches each key once per text.
Gate 3 PASS: 9 of 9 mutants caught from scratch (scripts/check-perf-1).
Gate 4 PASS: 404 of 404 tests, 0 compiler warnings, release binary
f48021c24137.
Gate 5 PASS: `invar lock` output and lock file, `invar check` output and
exit, and `invar backtest` output byte-identical to 93e93ce on all six
EVAL-007 and EVAL-008 round-trip sets; `broad_reads` identical over 18,206
corpus files (2,037,643 reads).

| Measured (sandbox, Linux x86_64) | 93e93ce | CHECK-PERF-1 |
|---|---|---|
| `invar check`, pi 18 contracts, fresh process | 160 M instructions, about 33 ms | 85 M, about 20 to 26 ms |
| `invar mcp` check, each call on a running server, pi | 161 M instructions, about 31 ms | 18 M, about 3 to 4.4 ms |
| `invar mcp` check, first call | about 31 ms | about 22 to 26 ms |
| scripts/perf/bench.py synthetic workspace, running server | about 54 ms | about 5 ms |

Wall times on the shared sandbox vary by up to half between runs;
instruction counts (callgrind) do not, and are the stated basis. Re-run
`python3 scripts/perf/bench.py target/release/invar` on the target machine
for its own figures.

Corrections to this draft, stated:

P-F-01 Gate 2 first memoized the analyses only (fields, zod, reads); a
running server then spent its time rebuilding each contract's surface
(5.9 ms). The whole surface is memoized too, which needs the contract's
pointer, zod shapes and companion in its key; P-F4 (zod shapes) was added
during Gate 2 before that code, P-F5 (companion) and P-F6 (pointer) at
Gate 3, each below a marker.
P-F-02 Mutant 05 (pointer dropped from the key) survived P-F6 as first
written: its two pointers had different last segments, so the binding
scopes, also in the key, told them apart. P-F6 now uses two pointers with
the same last segment.
