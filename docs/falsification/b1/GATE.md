# B1 kill gate: MET, 2026-10-28 (cap was 2026-11-01)

Rule (DATA-KILLGATE-SPEC r0): find 10 or more pinnable real breaking
changes to DECLARED, cross-boundary data contracts, at least 3 with
documented downstream impact, or kill Program B.

Result: 10 admitted, 6 rejected. Four ecosystems: Snowplow (2),
OpenTelemetry (3), Kubernetes (3), Terraform (2). Mean restatement
count across admitted cases: 3.2 coping notations per broken promise.
PR-time enforcement observed in any ecosystem: none.

The ledger is WORKSHEET.tsv. Each cases/b1-NNN/ holds the case record
(case.tsv) and the SHA-256 manifest of the pinned bytes; the bytes
themselves are reproducible from the upstream repositories at the
commits and tags the case records name, via harvesters/.
