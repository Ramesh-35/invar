# INVAR Operational Definition

What "operational" and "INVAR 1.0" mean, so the words cannot inflate.

## The rung ladder (every claim carries its highest truthful rung)
FORMALIZED -> IMPLEMENTED -> TESTED (synthetic) -> ADVERSARIALLY-TESTED
(synthetic) -> REACHABLE -> PRODUCTION-CONFORMANT -> REAL-DATA-VALIDATED ->
INDEPENDENTLY-VALIDATED -> REALITY-PROVEN.

- Synthetic evidence never crosses into REAL. A passing test is TESTED, not
  PRODUCTION-CONFORMANT. A production build is not developer adoption.
- Bare "PROVEN" is reserved for REAL, non-synthetic evidence and is never a
  synthetic status (math-thread ruling; DR-003/DR-006).
- Today INVAR's highest honest rung is TESTED/ADVERSARIALLY-TESTED (synthetic) for
  the instrument, HYPOTHESIS for the category. HE=0.

## INVAR is NOT operational when
- the code compiles,
- the tests pass,
- the CLI runs,
- the repository looks complete,
- a production build exists.
None of these is adoption, and none raises the category rung.

## INVAR 1.0 (the only definition of done)
A real external developer, unaided, can:
1. discover INVAR and understand what it protects,
2. install and initialize it on a real repository,
3. define an invariant that matters to them,
4. run it and intentionally violate it,
5. receive a useful, specific failure grounded in machine-available evidence,
6. understand why it violated, which invariant, which evidence, and when valid,
7. fix it, re-run green, and wire it into CI,
8. operate it continuously and upgrade it safely.
And the enforcement they receive is trustworthy: the category residue INVAR claims
to catch has been shown, on at least one REAL cross-repository incident, to be a
defect a fully-exercised composite baseline misses and an INVAR primitive catches
(REAL-DATA-VALIDATED), reproducibly.

## What must be true before 1.0 can even be attempted
- A REAL-tier, probe-confirmed PROVEN_RESIDUE exists (Phase 2; DR-003). Without
  it, there is no category to productize and 1.0 is undefined.
- The security model is designed in, not appended: trust boundaries, malicious
  repositories and invariants, parser and config-injection surfaces, supply-chain
  and CI-privilege risks, path traversal, secret exposure, cache/artifact
  substitution, denial-of-service. INVAR must not become the new attack surface it
  exists to reduce.

## Standing failure definition
If a fully-exercised composite baseline catches every real incident INVAR is
tested against, the category is FALSIFIED and the repository must say so. That is a
legitimate, successful outcome of the program, not a defeat to engineer around.

## Owner responsibilities (from the commission)
Mathematical adequacy, faithful implementation of the constitution, one coherent
authority (no duplication), a real developer problem solved, usable without
internal guidance, no new attack surface, buildable/shippable/observable/
recoverable, external onboarding success, and every claim carrying its actual rung.
