//! INVAR falsification instrument — library crate.
//!
//! This crate is a *skeptic*. Its job is not to prove that INVAR constitutes a
//! new category of tool; its job is to try to falsify that claim by checking
//! whether a realistic composite baseline (Semgrep + OPA + OpenTelemetry +
//! Property Testing) already catches every failure in the corpus. A case is
//! only ever classified as `PROVEN_RESIDUE` when the composite demonstrably
//! failed to catch it *and* every leg the case depends on actually executed —
//! never because its `failure_type` sounds like something INVAR should own.
//!
//! Epistemic guarantees enforced structurally:
//!   * A tool that did not run (`SKIPPED`) or errored (`ERROR`) is never PASS
//!     and never "caught nothing".
//!   * The classifier never reads `failure_type` or `expected_status`.
//!   * The bi-temporal ledger keeps valid-time and transaction-time distinct
//!     and re-verifies its BLAKE3 chain by recomputation, not by a stored flag.

pub mod check;
pub mod config;
pub mod evaluator;
pub mod init;
pub mod invariant;
pub mod ledger;
pub mod manifest;
pub mod provenance;
pub mod reality;
pub mod yaml;
pub mod report;
pub mod resolver;
pub mod surface;
