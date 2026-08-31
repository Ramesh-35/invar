# invar-cusum — DEFERRED

**Status: `[DEFERRED]`.** No `Cargo.toml`. Not a workspace member. Cannot build.

Intended home for a structural-break / change-point detector over the lineage
ledger (see [M2](../../docs/architecture/laws/M2.md)).

`[UNVERIFIED]` That a robust CUSUM — or the CBR-001 market-regime detector — is a
proven method for source-code structural breaks. Per
[DR-006](../../docs/decisions/DR-006-crypto-claims-require-threat-model.md), this
crate stays deferred until a source-code change-point *signal* is defined and its
false-positive behaviour is **measured, not assumed**.
