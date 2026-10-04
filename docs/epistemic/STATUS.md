# Epistemic Status — Controlled Vocabularies

This document defines the three vocabularies used throughout the repository.
Mixing them is an error.

## 1. Claim status (used in prose/docs)

| Label | Meaning |
|---|---|
| `[PROVEN]` | Demonstrated in THIS repository, reproducibly. |
| `[HYPOTHESIS]` | A testable proposition, not yet tested. |
| `[TARGET]` | A desired outcome with no supporting evidence yet. |
| `[UNVERIFIED]` | Asserted elsewhere; not established here. |
| `[STRATEGIC HYPOTHESIS]` | A business/positioning bet, not a technical fact. |
| `[TEST-PROVEN]` | Demonstrated by an automated test on SYNTHETIC fixtures; last green run reported, not re-adjudicated. Never equal to `[PROVEN]`. |
| `[ADVERSARIALLY-VALIDATED-ON-SYNTHETIC]` | A test that actively tries to break the property (e.g. ledger tamper detection) passed, on synthetic data. Still synthetic. |


> **Reservation (math-thread ruling; DR-003/DR-006).** Bare `[PROVEN]` is
> reserved for REAL, non-synthetic evidence: a differential residue isolated
> on a real cross-repository incident. Synthetic harness results are at most
> `[TEST-PROVEN]`. A BLAKE3 hash chain authenticates the transaction-time
> append log only, not a poset edge-set.

## 2. Case verdict (the evaluator's output — exactly one per case)

| Verdict | Meaning |
|---|---|
| `CAUGHT_BY_COMPOSITE` | An **executed** composite leg isolated the designated failure (a finding matched the case's `catch_signature`). |
| `PROVEN_RESIDUE` | Every **required** leg executed and none isolated the failure. On FIXTURE evidence this is a *candidate* residue only. |
| `UNKNOWN` | We cannot honestly make either claim — typically because a required leg did not execute. |

The evaluator decides a verdict **only** from executed-leg evidence. It never
reads a case's `failure_type` or `expected_status` to choose the verdict's
polarity. `failure_type` is consulted solely to know which legs are *required*
to have executed (coverage).

## 3. Leg state (per composite tool, per case)

| State | Executed? | Meaning |
|---|---|---|
| `PASS` | **yes** | Ran, found nothing. |
| `FAIL` | **yes** | Ran, found something. |
| `SKIPPED` | no | Did not run (tool absent, disabled, or nothing to observe offline). |
| `ERROR` | no | Tried to run but the result is uninterpretable. |
| `UNKNOWN` | no | Executed but the result cannot be reduced to PASS/FAIL. |
| `INADMISSIBLE` | no | Executed and interpretable, but not allowed to count: temporally outside the as-of bound (GAP-C) or a provenance-correlated duplicate carrier (GAP-B/GAP-E). Never a catch. |

**Load-bearing rule:** `SKIPPED`, `ERROR`, and `INADMISSIBLE` are **never**
`PASS` and **never** "caught nothing". Only `PASS`/`FAIL` count as executed. A
case whose required leg is `SKIPPED`/`ERROR`/`INADMISSIBLE` is `UNKNOWN`, never
`PROVEN_RESIDUE`. This keeps a missing tool, or an executed-but-inadmissible one
(GAP-E), from masquerading as evidence that the composite failed.

## FIXTURE vs REAL evidence

`FIXTURE` cases are hand-built to probe the thesis. They can produce a residue
**candidate** but can never, by themselves, prove a category claim. Only `REAL`
cross-repository incidents can promote a candidate toward evidence, and only
through a confirming differential probe (DR-003). A `REAL` claim is additionally
gated by the reality-admission check (`reality::verify_real_tier`, GAP-D): an
unbacked `REAL` tier is refused and degraded to `UNKNOWN`. That gate is
necessary, not sufficient: it binds a proof to the recorded residue but does not
verify the referenced commits are real, so it does not by itself earn `[PROVEN]`.

## Gate 0 adjudication (2026-10-03)

The committed source was built and its acceptance suite run and adjudicated.
This is a has-implementation confirmation only. It is not REAL-tier evidence.

- Commit tested: `2164ac4`
- Source fingerprint (sha256 over all `.rs` + `Cargo.toml` + `Cargo.lock`): `821cb199c128fcfcbf4962622d7d03dbc59b30950c5914744e5edc73165cbe3d`
- `cargo build --workspace --locked`: GREEN
- `cargo test --workspace --locked -- --nocapture --test-threads=1`: 6/6 PASS (0 failed, 0 ignored, 0 skipped)
- has-implementation: CONFIRMED (was UNCONFIRMED)
- Rung: `TESTED-SYNTHETIC` (ADJUDICATED). Never bare `[PROVEN]`, never REAL.
- Environment: Linux cloud container, cargo/rustc 1.95.0.
- Caveat: NOT Mac-native execution. A founder-side `cargo test` on the authoritative machine remains a separate confirmation.
- Boundary: no REAL-tier evidence exists yet. All six tests are synthetic / FIXTURE evidence.

## Contract kinds (general invariant engine, 2026-10-04)

The engine dispatches evidence to a deterministic checker by declared kind
(`invariant::check_contract`). Residue empty = contract holds; non-empty = the
named violation. These expand the PRODUCT surface, not epistemic adequacy: the
A-F adequacy matrix is unchanged by their addition.

| Kind | Checks | Evidence tier so far |
|---|---|---|
| `SchemaContract` | consumer-read payload fields the producer schema does not DEFINE | REAL (REAL-INCIDENT-001) + synthetic tests 17-18 |
| `EventContract` | consumer-handler fields the producer payload does not EMIT | SYNTHETIC only (test 19); no real incident yet |
| `LockOrderContract` | sequence inversion / missing precondition in a trace | SYNTHETIC only (test 20); no real incident yet |

A kind reaches REAL standing only when run on a real incident of that kind,
through the same reality-admission gate (GAP-D). Until then it is a tested but
synthetic checker. Do not report EventContract or LockOrderContract as
REAL-proven.
