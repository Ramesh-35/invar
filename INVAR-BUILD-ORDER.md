# INVAR Build Order

Vertical, gated build order for INVAR. Grounded in the audited repository at
`fdf6551`, not in aspiration. Each phase runs the working discipline
(INSPECT -> STATE RUNG -> CONTRACT GAP -> STOP/BUILD -> RED -> GREEN -> TYPECHECK
-> REAL RUNTIME -> ADVERSARIAL -> FULL SUITE -> DOCUMENT EVIDENCE -> COMMIT ->
VERIFY -> ADJUDICATE RUNG). No phase claims a rung the evidence does not support.

Two standing gates sit above every phase:
- FALSIFICATION-FIRST GATE (DR-003): no production substrate
  (`invar-core/lsp/cusum`, `invar-isv`) until a REAL-tier, probe-confirmed
  PROVEN_RESIDUE exists.
- MATH-ADEQUACY GATE: no new primitive without REALITY -> phenomenon ->
  representation -> six-primitive composition test -> expressiveness gap ->
  red-team A-H -> smallest extension. A-D fail => reject.

## Current position
Phase 0 COMPLETE this turn (sovereignty audit + rung correction + these four
documents). Nothing downstream is started.

## Phase 0 - Sovereignty (DONE)
Repository verified, reconciled, corrected. Highest honest rung today: TESTED
(synthetic) for the classifier/ledger; category at HYPOTHESIS (FIXTURE candidate).

## Phase 1a - Adjudicate the existing rung (cheap, next)
Run `cargo test`, `run`, `verify`, and the tamper check on the Mac (which has the
toolchain), capture the output, and record it as evidence in the repo. This moves
the classifier/ledger from TESTED-pending to TESTED-adjudicated. It does NOT raise
the category rung. No code change expected; if a test fails, that is the finding.

## Phase 1b - Core engine integrity (only if 1a surfaces a gap)
Keep the smallest trustworthy engine: input -> parse -> represent -> establish
invariant -> evaluate -> detect -> preserve evidence -> deterministic result.
No hidden LLM authority, no fabricated evidence, no silent fallback. Apply the
math ruling's dispatch corrections if any are not yet in the code (hand-written
`partial_cmp` returning None on incomparable points; no `Ord`; chain documented as
append-log-only).

## Phase 2 - REAL-world falsification (THE gate; founder-dispatched)
Source exactly ONE genuine cross-repository incident. Provenance first; observed
vs inferred separated; no manufacturing to resemble a fixture. Build the minimal
reproduction preserving the real cross-repo boundary. Run the composite baseline;
run the INVAR differential probe. Classify honestly. Preserve negative results.
Do not generalize from one incident. A CAUGHT or UNKNOWN verdict is a successful,
honest experiment and may falsify the category. Outcome recorded with full
traceability: incident -> reconstructed failure -> baseline observation -> INVAR
observation -> verdict.

## Phase 3 - Developer model (only if Phase 2 yields a real residue)
Smallest mental model a competent developer needs: invariant, assertion, evidence,
violation, scope, temporal validity, standing, remediation, verification, replay.
Then remove every concept a developer does not strictly need.

## Phase 4 - Developer interface
install -> init -> define -> run -> violation -> inspect -> fix -> verify. The
interface is simple even where the internals are not.

## Phase 5 - Repository / CI integration
Local CLI, CI gate, Git hooks where justified, machine- and human-readable output,
exit codes, baseline/migration semantics. Only integrations that remove real
friction.

## Phase 6 - Evidence and explanation
Every violation answers what changed, which invariant, which evidence, when valid,
which dependency, what is known, what is uncertain, what is reproducible. No
invented explanations; every explanation grounded in machine-available evidence.

## Phase 7 - Falsification hardening
For every core law: positive + negative + minimal mutant + mutant caught. A law is
not HELD until the mutant proves the test can detect its breakage. Extend the
existing conformance harness; do not fork a parallel test system.

## Phase 8 - Real project validation
Move from fixtures to several materially different real repositories. Measure
install success, config effort, false positives/negatives, runtime, comprehension,
remediation time, CI friction, reproducibility, trust. Do not call it validated
until the evidence supports the rung.

## Phase 9 - Productization
Build only the product surfaces the developer workflow proves necessary.

## Phase 10 - Operationalization
Versioning, release, signing where justified, reproducible builds, CI,
compatibility/migration policy, telemetry/privacy policy, security model,
vulnerability handling, rollback, observability.

## Phase 11 - Real developer onboarding
A developer who never spoke to the team completes the full journey unaided
(INVAR-DEVELOPER-JOURNEY.md). Observe, do not guide. Record every failure. Fix.
Repeat.

## Phase 12 - INVAR 1.0
Declared only when a real external developer independently adopts INVAR,
integrates it into a real repository, receives trustworthy invariant enforcement,
diagnoses violations from evidence, and operates it continuously
(INVAR-OPERATIONAL-DEFINITION.md).

## Anti-duplication rule
Before introducing any engine / truth store / verification / temporal / evidence
model: find the existing owner, reuse if semantically sufficient, and if not,
prove the expressiveness gap first.
