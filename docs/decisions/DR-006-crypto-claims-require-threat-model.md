# DR-006 — Cryptographic claims require a threat model; no CBR-001 transfer

- **Status:** Accepted
- **Date:** 2026-08-31

## Context

Prior narratives attached strong, unearned claims to cryptographic and
statistical primitives: that HKDF provides zero-knowledge proofs, and that the
CBR-001 market-regime detector transfers to AST/source analysis. Both are
category errors that would poison the evidence base if encoded as fact.

## Decision

1. **No cryptographic capability is claimed without a written threat model.**
   In particular: **HKDF is a key derivation function, not a zero-knowledge
   proof system.** It derives keys from input keying material; it proves nothing
   in zero knowledge. Any ZK claim requires its own construction and threat
   model.
2. **No transfer of CBR-001 (market-regime detection) onto AST analysis is
   asserted.** A detector validated on market data is not, by that fact, valid on
   source-code structure. Such a transfer is a `[HYPOTHESIS]` requiring its own
   labelled evaluation (see M2).
3. `invar-cusum` stays `[DEFERRED]` until a source-code change-point signal is
   defined and its false-positive behaviour is measured, not assumed.

## Consequences

- BLAKE3 is used for **content and chain identity** only — an integrity/hashing
  role — and is described as such, never as a proof system.
- Any figure like "<15 ms AST processing", "100% merge-gate enforcement", or
  "zero false positives" is `[UNVERIFIED]` until a benchmark exists.
