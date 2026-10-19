
## 2026-10-18: INVAR-HOLDS-001 implemented (the durability line)

Every HOLDS outcome of `invar check` now carries one indented sub-line,
`durability: held since <date> (<N> days)`, computed from the lock's own
since dates (max, the youngest acceptance), with zero git, zero new
dependencies, and check.rs untouched: the whole change is fourteen guarded
lines in main.rs's outcome loop. Violated and skipped outcomes carry no age,
proof imports keep their epoch axis, and receipts, SARIF, report-md and MCP
stay byte-identical. 11 fixtures written first, 10 of 10 mutations caught on a clean re-run, 255 of 255
tests, zero warnings. Gate 1 and Gate 3 each falsified fixtures while the
engine was right every time; the corrections are recorded in the spec's
section K2, including the evidence-bounded re-acceptance loop that is the
only honest way durability resets. Record: docs/architecture/INVAR-HOLDS-001.md.

## 2026-10-18: STAGE3-EVAL-001 executed (pre-registered evaluation on unseen repositories)

Ten repositories selected by a committed rule before any clone, engine
frozen at f9c4b29. Hard thresholds met: ISA 10 of 10, determinism 10 of
10, abstention honesty 0 silent drops, round trip 8 of 8 HOLDS with the
durability line; precision 8 of 9. One engine defect class found and not
fixed mid-run: backtest replays strings contracts with the wrong extractor
(61 of 61 absence events false). One yield lever: "exports" maps cause 83
percent of UNRESOLVED. Record: docs/architecture/STAGE3-EVAL-001.md section
K and docs/evidence/stage3/.

## 2026-10-19: INVAR-BACKTEST-002 strings replay (from STAGE3-EVAL-001 S3-F-05 to S3-F-07)

Backtest now reads a strings contract's producer with read_any_shape, the
same Zod key-set reader check uses, so the two cannot disagree on the same
bytes. A binding never present in the window says so instead of being
reassured; a shallow history is stated. Text only: JSON surface and
egress/schema replay byte-identical. backtest.rs only; 10 fixtures first,
11 of 11 mutations caught on a clean re-run, 265 of 265 tests, zero warnings. Re-measured on unseen code as
STAGE3-EVAL-002-M6: the 61 false absences now read present. Record:
docs/architecture/INVAR-BACKTEST-002.md.

## 2026-10-19: INIT-DISCOVERY-SLICE3 and SLICE3B land together (STAGE3-EVAL-003 PASS)

SLICE3 (package "exports" maps resolved to declared sources) passed its
own gates but STAGE3-EVAL-002 found 4 silent drops it made reachable, so
it was held. SLICE3B makes silence earned: an import that resolves but
finds no shape is silent only when the target package's own sources
prove the name is a value, class or enum and nothing is unread; otherwise
init abstains and names where the type is declared or which file it could
not read. STAGE3-EVAL-003 on the same pinned manifest: 186 proposals
(from 9 in EVAL-001), 194 new abstentions against a pre-registered 205
with every difference explained, 0 genuine silent drops, determinism and
ISA 10 of 10. Records: docs/architecture/INIT-DISCOVERY-SLICE3.md,
INIT-DISCOVERY-SLICE3B.md, docs/evidence/stage3/eval003/.
