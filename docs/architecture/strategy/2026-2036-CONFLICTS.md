# 2026-2036 spec: conflicts and rulings (founder adjudication 2026-10-10)

The 2026-2036 report (2026-2036-SPEC.md, STATUS: RECEIVED) was first issued
with a ratification wrapper ordering Phase 0 execution; that wrapper was
withdrawn by the founder's correction-of-record dispatch the same evening
after the conflicts below were surfaced. This file records each conflict
and the standing ruling. Where this file and the report disagree, this
file governs.

## 1. Wrong repository audited

The report's "actual implementation baseline" audits `anomly-labs/invar`,
an LLM inference provenance server, and concludes the contract verifier,
invar.lock, five-state exit codes and SSH receipt pipeline are unbuilt.
The canonical codebase is `Ramesh-35/invar-platform`, where all of those
are shipped and measured: lock format 1 frozen since alpha.5, the exit map
live since the early alphas, SSH-signed releases since alpha.7, receipts
shipped in alpha.12 with the cross-platform gate met (run 38047172189).
RULING: Phase 0 ("strip contract claims from the legacy repository") is
void; Ramesh-35/invar-platform is the sole canonical codebase; no
clean-room re-implementation (crates/invar-core split DECLINED).

## 2. ADR-001 exit-code remap

Report map: ERROR=2, LOCK_DRIFT=3. Shipped frozen map: 0 HOLDS, 1 VIOLATED
only, 2 ledger break, 3 error (and LOCK_DRIFT for `invar lock`),
4 SKIPPED / NOT EVALUATED. The report does not know the ledger exists.
RULING: DECLINED. The shipped map stays frozen; the five-state algebra the
report argues for already exists under it.

## 3. ADR-002 strict in-toto conformance

Receipt format 1 shipped 2026-10-10 with a compatibility promise and a
green cross-platform gate. Making in-toto Statement v1.0 the native format
would break that promise the day it was made.
RULING: format 1 stays canonical. in-toto v1.0 is ACCEPTED as an additive
export only (`invar receipt export --in-toto`), A2.1.

## 4. Tree-sitter AST engine and synthetic gates

The report's Phase 1 mandates tree-sitter parser frontends (a new
dependency family, against the no-new-dependencies rule) gated on 5,000
synthetic AST mutations (synthetic evidence, which three rules in a row
proved predicts nothing held out). It reopens the rule-expansion race
frozen by founder adjudication after batch 8.
RULING: DECLINED. The freeze holds; the substrate-asset pivot stands.

## 5. Internal contradictions in the report

Waiver cap stated as 90 days (ADR-003) and 30 days (Scenario B); the
shipped, ratified cap is 90 days. "Zero dynamic runtime dependencies"
alongside mandatory tree-sitter. Citation markers with no attached source
list. RULING: where the report contradicts itself, shipped reality wins.

## Adopted from the report

- The banned-patterns list and the kill criteria, enshrined in MOAT.md.
- The two-phase baseline ceremony, adapted to our architecture as an
  additive detached SSH signature over invar.lock (namespace invar-lock),
  verified fail-closed when a signers file is supplied. No pending-file
  mechanics: `invar lock` already writes the manifest; the ceremony adds
  provenance, not workflow.
- The untrusted-local-hooks / authoritative-CI threat framing (already our
  Action hardening posture, now stated explicitly).
- in-toto as an export (point 3).
