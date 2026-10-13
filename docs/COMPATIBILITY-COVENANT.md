# The INVAR Compatibility Covenant

Status: RATIFIED 2026-10-12 (founder dispatch, accepted with the
corrections recorded in docs/architecture/strategy/RUNWAY.md).
Every later change appends a dated entry to the change log below and
never edits an existing promise.

This document exists so that anyone who commits an INVAR lock to their
repository knows exactly what we will never break. The promise is the
product. It only compounds if it is public, dated, and kept.

## The promises

1. Lock formats are readable forever. Every lock format ever shipped
   (formats 1 through 4 as of ratification) will be parsed by every future
   engine version. We write the lowest format that expresses a lock's
   content, so older engines keep working wherever possible, and newer
   engines never orphan an old lock.
2. Exit codes are frozen. 0 holds, 1 violated only, 2 ledger break,
   3 error, 4 skipped or not evaluated. Scripts and CI gates written
   against these codes today will mean the same thing in every future
   version.
3. Reason codes are append-only. A reason code, once shipped, is never
   renamed, removed, or given a severity. New codes may be added.
4. The record is never rewritten. The corpus, the measured results, the
   published miss rate, and the ledger chain are append-only. Errors are
   corrected by dated supersession, never by deletion.
5. Unknown means fail closed. An engine that encounters a lock format
   newer than it understands refuses to evaluate rather than guessing.
   Silence will never be manufactured from ignorance.
6. Evidence language is bound. We do not say proven for synthetic
   evidence, we do not say zero-hallucination, we do not say proves
   safety. A violation of this promise in any INVAR publication may be
   reported as a bug.

## What this covenant does not promise

- It does not promise that INVAR catches everything. Our measured miss
  rate is published and we expect it to stay nonzero.
- It does not promise API stability of the Rust crate internals, only of
  the formats, exit codes, and records named above.
- It does not promise that a published lock is true. A lock is a claim
  by its publisher, for coordination, never security.

## Verifying our history independently

The age and integrity of the INVAR record do not depend on trusting us:
the ledger re-verifies by BLAKE3 recomputation, releases are archived at
Software Heritage under durable SWHID anchors, and tagged releases carry
DOIs. A fork of this repository inherits the code but cannot extend this
record as its own; the chain and the external anchors date every entry.

## What this covenant deliberately omits

Latency. Performance is a budget we publish and measure per release,
never a promise in this covenant, because a promise here must be
keepable on every machine forever. Measured numbers live with the
measured record.

## Change log

- 2026-10-12: initial six promises drafted and ratified the same day
  (founder dispatch; engineer corrections in RUNWAY.md).
