# DR-003 — Falsification-first gate before any production runtime

- **Status:** Accepted
- **Date:** 2026-08-31

## Context

The strong temptation is to start building the INVAR core substrate (lineage
engine, LSP, detectors) immediately. Doing so would encode the category claim as
an assumption rather than a finding.

## Decision

**No production runtime is authorized until a probe-confirmed, `REAL`-tier
residue survives a fully-exercised composite baseline.** Concretely, Phase 1
ships only `crates/invar-falsify`. The deferred crates (`invar-core`,
`invar-lsp`, `invar-cusum`) and `packages/invar-isv` carry **no build system**,
are **not workspace members**, and cannot build.

Promotion past this gate requires all of:

1. a minimal cross-repository differential probe demonstrating an INVAR primitive
   isolating a failure the composite missed, and
2. at least one `REAL` cross-repository incident of the same shape, and
3. reproducible attribution of the detection to that primitive.

## Consequences

- FIXTURE evidence can produce a residue **candidate**, never authorization to
  build the substrate.
- The workspace `Cargo.toml` lists exactly one member. Adding a deferred crate a
  `Cargo.toml` before this gate is met is a process violation.
