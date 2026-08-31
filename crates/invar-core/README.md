# invar-core — DEFERRED

**Status: `[DEFERRED]`.** No `Cargo.toml`. Not a workspace member. Cannot build.

This crate will hold the INVAR invariant substrate (lineage engine, invariant
model, checkers). It is intentionally empty of a build system: per
[DR-003](../../docs/decisions/DR-003-falsification-first-gate.md), no production
runtime is authorized until a probe-confirmed, `REAL`-tier category residue
survives a fully-exercised composite baseline. Until then, this file is the only
content here.
