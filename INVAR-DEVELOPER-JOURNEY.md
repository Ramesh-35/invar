# INVAR Developer Journey

The commission's definition of success: a developer who has never spoken to the
team can discover INVAR, understand it, install it, integrate it into a real
project, get a trustworthy result, understand why INVAR reached it, and keep using
it in production without founder or internal help. This document is the operational
target and the yardstick every later phase is measured against.

## The twelve steps
1. Discover INVAR and grasp the value proposition.
2. Install it.
3. Initialize a project.
4. Create a first invariant.
5. Run it.
6. Intentionally violate the invariant.
7. Receive a useful, specific failure.
8. Understand the evidence behind the failure.
9. Fix the violation.
10. Re-run successfully.
11. Integrate it into CI.
12. Operate it continuously and upgrade it safely.

## The five questions a developer must answer immediately (quality test)
1. What does INVAR protect?
2. How do I tell it what matters?
3. What happens when something violates it?
4. Why should I trust the result?
5. How do I put it into my existing workflow?
If any answer is unclear, the product is not ready.

## Current reality (verified at `0197eca`, 2026-10-04)
A developer who builds the binary from source can now run INVAR against two real
checkouts and get an honest, specific verdict. Honest per-step status:
1. Discover: NO. Nothing published, no public page, no package.
2. Install: NO. No release artifact or crates.io/npm presence; must `cargo build`
   from source.
3. Init: PARTIAL. `invar.toml` is the config format ([producer]/[consumer] paths
   plus repo/commit provenance); there is no `invar init` scaffold yet.
4. Define: PARTIAL. For the AST_SCHEMA_DESYNC contract the invariant is implicit
   (every payload field the consumer reads must be a producer-DEFINED property);
   there is no general developer-authored invariant language yet.
5. Run: YES (from source). `invar check --config invar.toml` runs the cross-repo
   differential probe over local checkouts, the mode CI uses.
6. Violate: YES. REAL-INCIDENT-001 is a real violation; `invar check` exits 1.
7. Useful failure: YES. It names the exact residue (`["amount_cents"]`), the
   derived BLAKE3 probe hash, and the reality-gate result.
8. Evidence: YES, honest. `invar audit` verifies the bi-temporal ledger and lists
   its nodes; the composite-baseline comparison is explicit, never fabricated.
9. Fix: YES. Restore the field (or update the consumer) and re-run.
10. Re-run: YES. A holding contract reports HOLDS and exits 0.
11. CI: PARTIAL. `check` has CI-grade exit codes; there is no packaged, reusable
    GitHub Action for users yet (the repo's own ci.yml is INVAR's build).
12. Operate / upgrade: NO. No user-facing release, versioning, or upgrade path.

## The gap, stated plainly
INVAR has crossed from an internally-run instrument to a runnable developer tool
for one contract type (cross-repository schema desync), and its reason to exist
survived a REAL incident (REAL-INCIDENT-001, DR-003 cleared for that one case).
What remains before the journey is complete: discoverability and a real install
path (steps 1-2), an `invar init` scaffold and a general invariant language
(steps 3-4), a packaged CI action (step 11), and a user release and upgrade path
(step 12). The engine is honest and runnable; it is not yet packaged or general.

## Guardrails for when this journey is built
- Simplicity is the standard: a developer must not need posets, epistemic
  lattices, provenance algebra, or internal ledgers to use INVAR. Those make it
  trustworthy; they must not become cognitive load.
- No hidden AI authority in the trust path: evidence -> deterministic/
  constitutional computation -> standing -> result. Never prompt -> model opinion
  -> truth. AI may assist discovery/analysis/explanation, never manufacture
  evidence or upgrade standing.
