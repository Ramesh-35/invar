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

**Load-bearing rule:** `SKIPPED` and `ERROR` are **never** `PASS` and **never**
"caught nothing". Only `PASS`/`FAIL` count as executed. A case whose required
leg is `SKIPPED`/`ERROR` is `UNKNOWN`, never `PROVEN_RESIDUE`. This is what keeps
a missing tool from masquerading as evidence that the composite failed.

## FIXTURE vs REAL evidence

`FIXTURE` cases are hand-built to probe the thesis. They can produce a residue
**candidate** but can never, by themselves, prove a category claim. Only `REAL`
cross-repository incidents can promote a candidate toward evidence, and only
through a confirming differential probe (DR-003).
