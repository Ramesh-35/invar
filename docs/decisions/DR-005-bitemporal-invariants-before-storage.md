# DR-005 — Bi-temporal invariants defined before storage

- **Status:** Accepted
- **Date:** 2026-08-31

## Context

It is easy to pick a database first and let its schema dictate meaning. For a
lineage substrate whose entire value is temporal honesty, that inverts the
dependency.

## Decision

The ledger's **semantics are fixed before its storage**:

- Two clocks, always distinct: **valid time `t_v`** (when a fact was true) and
  **transaction time `t_r`** (when it was recorded). They are separate columns
  and are never collapsed.
- `state_hash = BLAKE3(content)`.
- `lineage_hash = BLAKE3(prev_lineage ++ state_hash ++ context)`,
  `context = "{kind}|{subject}|{t_v}|{t_r}"`.
- `append_now` sets `t_v == t_r` (honest when the event *is* the run);
  `append(t_v, …)` carries an independent valid time. Both route through one
  private `commit`.
- **Verification is recomputation.** `verify()` re-walks rows in id order and
  recomputes both hashes; it never reads a stored "valid" flag.

Storage (SQLite with WAL, `synchronous=NORMAL`, `foreign_keys=ON`) is chosen to
*serve* these invariants, not define them.

## Consequences

- A silent edit to any committed row is detected as a `StateMismatch`/
  `LineageBroken` at the offending id.
- Swapping the storage engine later must preserve these invariants unchanged.
