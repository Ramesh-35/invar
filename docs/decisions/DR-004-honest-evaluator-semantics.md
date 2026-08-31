# DR-004 — Honest evaluator semantics

- **Status:** Accepted
- **Date:** 2026-08-31

## Context

An earlier, speculative evaluator design risked classifying a case as residue
because its `failure_type` was *associated* with a capability INVAR wants to own
(e.g. "bi-temporal lineage"). That converts a hypothesis into a fabricated
result.

## Decision

The evaluator is a skeptic with structural guarantees:

1. Leg states `PASS`/`FAIL` are the **only** executed states. `SKIPPED`/`ERROR`
   are never `PASS` and never "caught nothing".
2. A missing or errored tool is never silently a pass.
3. `classify` never reads `failure_type` or `expected_status` to choose a
   verdict's polarity. `failure_type` is used only to compute which legs are
   *required* to have executed (coverage).
4. `CAUGHT_BY_COMPOSITE` requires an **executed** leg whose finding matched the
   case's `catch_signature`. A stray advisory finding (no signature match) is
   **not** a catch.
5. `PROVEN_RESIDUE` requires every required leg to have executed and none to have
   caught. Otherwise the verdict is `UNKNOWN`.

## Consequences

- The classifier's behaviour is pinned by acceptance tests: signature-gated
  catch, advisory-is-residue, missing-leg-is-unknown.
- No verdict can be manufactured from a case's declared type or hoped-for
  outcome.
