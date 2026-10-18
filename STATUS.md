
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
