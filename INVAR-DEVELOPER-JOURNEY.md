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

## Current reality (verified at `fdf6551`, 2026-10-03)
A real developer today reaches STEP 0. Honest per-step status:
1. Discover: NO. Nothing published, no public page, no package.
2. Install: NO. No release artifact, no installer, no crates.io/npm presence.
3. Init: NO. `invar-falsify` is source-only; no `init` scaffold for a user project.
4. Define: PARTIAL as internals. Invariants today are hand-built FIXTURE corpus
   cases plus Semgrep/OPA rule files, not a developer-authored invariant format.
5. Run: ONLY with the full repo + Rust toolchain + a semgrep venv present. Not a
   developer path.
6-8. Violate / useful failure / evidence: the engine produces a verdict,
   `EPISTEMIC_REPORT.md`, and a BLAKE3-chained `run_results.json` for the FIXTURE
   corpus. It is honest, but it is not yet a developer-facing failure on the
   developer's own code.
9-12. Fix / rerun / CI / operate: NOT established.

## The gap, stated plainly
INVAR is, today, an internally-run falsification instrument, not a developer
product. The journey above is the backlog, gated behind Phase 2: the developer
surface is built only after the category survives a REAL incident (DR-003).
Building install/init/CI before that would be polishing a product whose reason to
exist is unproven.

## Guardrails for when this journey is built
- Simplicity is the standard: a developer must not need posets, epistemic
  lattices, provenance algebra, or internal ledgers to use INVAR. Those make it
  trustworthy; they must not become cognitive load.
- No hidden AI authority in the trust path: evidence -> deterministic/
  constitutional computation -> standing -> result. Never prompt -> model opinion
  -> truth. AI may assist discovery/analysis/explanation, never manufacture
  evidence or upgrade standing.
