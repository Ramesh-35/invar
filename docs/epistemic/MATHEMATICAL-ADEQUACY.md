# INVAR Mathematical Adequacy Audit

This is an AUDIT, not an implementation. No crate, evaluator, ledger, or test
was changed to produce it. It measures what the repository can and cannot establish for each of the six
adequacy primitives. First issued at commit `af1b1b6` (code at Gate 0
`2164ac4`); rows C and F updated at commit `6c68af1`, row B at commit
`0fc1d9a`, row E at commit `71290d6` (source fingerprint `379598f`); GAP-B,
GAP-C, GAP-E, and GAP-F closed on synthetic data, 14/14 acceptance tests. It imports no NEXTLOGIC or VEM vocabulary: every
primitive is stated generically, over any system that presents an invariant.

Method: for each primitive the constitution requirement is matched against the
actual code symbol that would implement it, the test that would exercise it, and
the smallest attack that would expose its failure. A mechanism that is merely
"related to X" is not counted as establishing X.

Overall adequacy is itself scored by weakest-link composition (the rule INVAR
enforces on its own case verdicts in `evaluator::classify`): the composite is
capped by the weakest primitive. One primitive that is ABSENT as an enforced
property caps the whole, exactly as one missing required leg caps a case at
UNKNOWN.

## The six primitives (as commissioned)

| Key | Primitive | Question |
|---|---|---|
| A | Epistemic Standing | Can INVAR establish what epistemic status a result actually has, rather than merely emit a verdict? |
| B | Provenance Independence | Can the evidence establish that the result is independent of the system under examination and not merely self-attested? |
| C | Temporal Admissibility | Can INVAR prevent later information from contaminating an earlier admissibility judgment? |
| D | Reality Boundary | Can INVAR distinguish synthetic/fixture evidence from evidence grounded in an actual external system or event? |
| E | Weakest-link Composition | Does the composite verdict degrade correctly when a required component is UNKNOWN, invalid, missing, or otherwise inadmissible? |
| F | Conservation-with-residual | Does INVAR account for what remains unexplained/uncovered rather than allowing a successful finding to imply total coverage? |

## Adequacy matrix (compact)

| Primitive | Status | One-line basis |
|---|---|---|
| A Epistemic Standing | PARTIAL | Case verdict carries a three-state standing (`EpistemicStatus`) plus rationale and refuses over-claim; no per-evidence standing, no standing algebra. |
| B Provenance Independence | SATISFIED (synthetic) | `provenance::ProvenanceGraph` decides independence from recorded ancestry (`independence` -> Independent/Correlated/Unknown; unknown is never independent) and de-duplicates shared-ancestry carriers so they cannot corroborate as independent (`corroboration` -> Banded/Confident), tests 11-12. GAP-B closed on synthetic sources, commit `0fc1d9a`. Standalone: not yet wired into `classify` (no correlated evidence in the Phase-1 composite). |
| C Temporal Admissibility | SATISFIED (synthetic) | `replay_as_of(as_of_t_r, as_of_t_v)` verifies the full chain then admits only rows with `t_r <= as_of_t_r` and `t_v <= as_of_t_v`; a later-recorded fact is invisible to an earlier reconstruction (tests 7-8). GAP-C closed on synthetic rows, commit `6c68af1`. |
| D Reality Boundary | PARTIAL | `EvidenceTier::FIXTURE`/`REAL` is load-bearing in the verdict language and refuses to promote FIXTURE, but the tier is a self-declared manifest field with no verification, so promotion is on the honor system. |
| E Weakest-link Composition | SATISFIED (synthetic) | Non-execution degradation is proven (test 3); GAP-E adds executed-but-inadmissible degradation on two axes: temporal via `admit_temporal` (a leg beyond the as-of bound becomes INADMISSIBLE, so a required one forces UNKNOWN, test 13) and provenance via `leg_corroboration` (correlated carriers capped at Banded, test 14), commit `71290d6`. Carve-out: the reality-boundary inadmissibility axis is REAL-dependent and stays with D. |
| F Conservation-with-residual | SATISFIED (synthetic) | `composite_residual` records uncovered legs on every case and preserves them even on a catch; `CaseConservation::holds` and `RunReport::conservation_holds` assert the balance (tests 9-10). GAP-F closed on synthetic rows, commit `6c68af1`. Residual is composite-coverage breadth; per-finding residual is future work. |

## Final adjudication

```
MATHEMATICAL ADEQUACY
A Epistemic Standing:        PARTIAL
B Provenance Independence:   SATISFIED (synthetic)
C Temporal Admissibility:    SATISFIED (synthetic)
D Reality Boundary:          PARTIAL
E Weakest-link Composition:  SATISFIED (synthetic)
F Conservation-with-residual:SATISFIED (synthetic)
Overall adequacy: NOT ESTABLISHED
```

Overall is NOT ESTABLISHED by INVAR's own weakest-link rule. As of commit
`71290d6`, B, C, E, and F are SATISFIED on synthetic data (GAP-B, GAP-C, GAP-E,
GAP-F closed, 14/14 acceptance tests). "(synthetic)" means TESTED-SYNTHETIC:
proven on synthetic data, never REAL. E is satisfied on its temporal and
provenance inadmissibility axes; its third axis, reality-boundary
inadmissibility, is REAL-dependent and carried by D. The composite is now capped
by the two remaining PARTIALs, A and D, and both are REAL-dependent: A needs a
real case that the trichotomy demonstrably mislabels, and D needs a genuine
external artifact. Synthetic adequacy is at its limit; overall stays NOT
ESTABLISHED until REAL evidence arrives.

## Per-primitive evidence

### A. Epistemic Standing: PARTIAL

1. Constitution requires: a result must carry its actual epistemic standing, not
   a bare pass/fail.
2. Currently exists: `EpistemicStatus { CAUGHT_BY_COMPOSITE, PROVEN_RESIDUE,
   UNKNOWN }`; `classify` returns the status plus a rationale string; FIXTURE
   residue is annotated "candidate only"; `report::category_status` cannot emit
   any VALIDATED/PROVEN-category wording on fixture evidence (caps at
   `CATEGORY_RESIDUE_CANDIDATE`).
3. Implemented by: `evaluator::classify`, `manifest::EpistemicStatus`,
   `report::RunReport::category_status`.
4. Tested by: tests 1-3 (`catch_requires_signature_match`,
   `advisory_finding_is_residue_not_catch`, `missing_required_leg_is_unknown`).
5. Can establish: the standing of a single case verdict among three states, and
   that the system will not label fixture evidence as proven.
6. Cannot establish: standing of an individual piece of evidence; a graded
   standing lattice; any standing attached to ledger records. Standing exists
   only at the granularity of a whole-case verdict.
7. Smallest falsification attack: feed a case whose evidence warrants "weak
   support" rather than one of the three states; the system has no way to
   express it and must flatten it into the trichotomy.
8. Status: PARTIAL.
9. Gap requires: new representation (a standing type richer than the trichotomy),
   not merely new evidence.

### B. Provenance Independence: SATISFIED (synthetic)

1. Constitution requires: the result must be shown independent of the system
   under examination, not self-attested, and multiple carriers descending from
   one observation must not count as independent corroboration.
2. Currently exists: nothing that models evidence provenance or ancestry. The
   adjacent and real property that does exist is on a different axis:
   `classify` never reads `failure_type`/`expected_status` for verdict polarity,
   so the verdict is independent of the human's a-priori claim.
3. Implemented by: (the adjacent property) the structure of `classify` and the
   `expected_status` field being carried but unused for polarity; (this
   primitive) no symbol.
4. Tested by: the adjacent property is implied by tests 1-3 passing with
   `expected_status: None`; this primitive has no test.
5. Can establish: that a verdict is not copied from the declared expectation.
6. Cannot establish: that evidence is independent of the subject; that two legs
   are not correlated carriers of one underlying signal; that a finding is not
   self-attested by the system under test.
7. Smallest falsification attack: supply two legs that are the same observation
   relabeled; the composite treats them as independent coverage.
8. Status: SATISFIED on synthetic sources (TESTED-SYNTHETIC). UPDATE commit
   `0fc1d9a`: `provenance::ProvenanceGraph` models ancestry and decides
   independence (Independent/Correlated/Unknown; unknown never independent) and
   de-duplicates shared-ancestry carriers (Banded vs Confident), tests 11-12.
9. Remaining scope: not yet wired into `classify`, because the Phase-1 composite
   has no correlated multi-carrier evidence to de-duplicate in the live path;
   wiring is deferred until such evidence exists. The model itself is complete.

### C. Temporal Admissibility: SATISFIED (synthetic)

1. Constitution requires: later-known facts must not participate in a
   reconstruction whose knowledge cut precedes them; replay at T must match what
   was knowable at T.
2. Currently exists: a bitemporal ledger that keeps `valid_time_us` (t_v) and
   `transaction_time_us` (t_r) distinct and never collapses them, with a
   BLAKE3 chain re-walked by `verify()` that recomputes both hashes.
3. Implemented by: `ledger::Ledger::append` / `append_now` / `commit` /
   `verify`, context `"{kind}|{subject}|{t_v}|{t_r}"`.
4. Tested by: test 4 (`ledger_chain_verifies_with_distinct_times`) asserts t_v
   is preserved and distinct from t_r; test 5 detects out-of-band edits.
5. Can establish: that the two clocks are recorded distinctly and that the log
   is tamper-evident.
6. Cannot establish: that an earlier reconstruction excludes later-known rows.
   There is no as-of query, no `replay(T)` filtering by `t_r <= T`, and no
   admissibility gate. The commissioned replay test cannot be run today.
7. Smallest falsification attack: append a fact at t_r = T2 with t_v = T1, then
   ask for the state as known at T1; nothing restricts the answer to rows with
   t_r <= T1.
8. Status: SATISFIED on synthetic rows (TESTED-SYNTHETIC). UPDATE commit
   `6c68af1`: `replay_as_of` implements the admissibility gate; tests 7-8 prove a
   later-recorded fact is invisible to an earlier reconstruction and that an
   out-of-band tampered row is still caught during reconstruction. Not REAL.
9. Gap requires: new code (an as-of reconstruction query plus an admissibility
   gate) over the EXISTING ledger columns. No schema change needed.

### D. Reality Boundary: PARTIAL

1. Constitution requires: synthetic/model-generated/self-attested evidence must
   not cross into verified real-world standing merely by being persisted.
2. Currently exists: `EvidenceTier::FIXTURE|REAL`; `classify` tags FIXTURE
   residue "(FIXTURE tier - residue CANDIDATE only, not proven)" and never
   promotes it; `report::category_status` / `has_real_residue` treat even a REAL
   residue as a candidate "requires probe confirmation"; STATUS.md reserves bare
   `[PROVEN]` for REAL evidence.
3. Implemented by: `manifest::EvidenceTier`, `evaluator::classify` tier note,
   `report::RunReport::{category_status, has_real_residue}`.
4. Tested by: tests 1-3 operate at FIXTURE tier and confirm no promotion; there
   is no test that attempts an illegitimate FIXTURE-to-REAL promotion.
5. Can establish: that fixture evidence is never spoken of as proven, and that
   the boundary word exists and gates the output language.
6. Cannot establish: that a case claiming `evidence_tier = REAL` is actually
   grounded in an external system. The tier is a self-declared field with no
   verification, so the boundary is honored, not enforced.
7. Smallest falsification attack: author a synthetic case with
   `evidence_tier: REAL`; the system accepts the REAL standing with no external
   grounding. The commissioned M4 attack passes today.
8. Status: PARTIAL (distinction present, load-bearing, tested on the FIXTURE
   side; forgery-resistance absent).
9. Gap requires: new architecture (an admission check that REAL tier must be
   backed by an external, independently attestable artifact), plus a reality
   dependency (a real artifact to attest).

### E. Weakest-link Composition: SATISFIED (synthetic)

1. Constitution requires: the composite may not be stronger than its weakest
   admissible input, and must degrade when a required component is UNKNOWN,
   invalid, missing, or otherwise inadmissible.
2. Currently exists: `LegState::executed()` admits only PASS/FAIL; a catch needs
   an executed leg with a `catch_signature` match; `classify` forces UNKNOWN
   when any `required_legs(failure_type)` did not execute; SKIPPED/ERROR never
   count.
3. Implemented by: `evaluator::{classify, required_legs, LegState::executed}`.
4. Tested by: test 3 (missing required leg -> UNKNOWN), reinforced by tests 1-2.
5. Can establish: that missing, skipped, errored, or non-executed required legs
   degrade the composite to UNKNOWN rather than to a false residue. This is the
   one adequacy behavior the repository genuinely proves.
6. Cannot establish: degradation for a leg that executed but is inadmissible on
   other grounds (compromised provenance, stale, out-of-boundary). "Otherwise
   inadmissible" collapses to "not executed" today; executed-but-inadmissible is
   unmodeled. Also `required_legs` covers only two failure-type groups.
7. Smallest falsification attack: supply a required leg that executed (PASS/FAIL)
   but whose evidence is inadmissible; the composite still counts it as coverage.
8. Status: SATISFIED on synthetic data (TESTED-SYNTHETIC). UPDATE commit
   `71290d6` (GAP-E): `admit_temporal` demotes a leg beyond the as-of bound to
   INADMISSIBLE so a required one degrades the case to UNKNOWN (test 13), and
   `leg_corroboration` caps provenance-correlated carriers at Banded (test 14).
   classify() is untouched; degradation flows through the existing weakest link.
9. Carve-out: the third inadmissibility axis, an executed leg whose REAL tier is
   forged or unverified, is REAL-dependent and stays with GAP-D. E is therefore
   SATISFIED (synthetic), not unconditionally.

### F. Conservation-with-residual: SATISFIED (synthetic)

1. Constitution requires: a successful finding must not imply total coverage;
   the uncovered/unexplained remainder must be accounted for and preserved.
2. Currently exists: `PROVEN_RESIDUE` names the uncaught remainder per case;
   UNKNOWN preserves ignorance instead of defaulting to pass; the report prints
   a partition (CAUGHT / RESIDUE / UNKNOWN counts) and an integrity ledger of
   what the run does NOT assert.
3. Implemented by: `manifest::EpistemicStatus::{PROVEN_RESIDUE, UNKNOWN}`,
   `report::RunReport::{count, to_markdown}` case partition and integrity ledger.
4. Tested by: test 3 preserves UNKNOWN as a first-class outcome; the partition
   and a post-catch residual are not asserted as an invariant or tested.
5. Can establish: that ignorance is preserved as UNKNOWN and that a run states
   its own non-assertions.
6. Cannot establish: that a CAUGHT verdict still carries residual uncertainty (a
   catch is treated as fully explained), and there is no asserted conservation
   invariant (caught + residue + unknown == total with nothing silently
   dropped).
7. Smallest falsification attack: a case that is partly caught and partly
   uncaught; the trichotomy forces one label and the uncaught part of a caught
   case is lost.
8. Status: SATISFIED on synthetic rows (TESTED-SYNTHETIC). UPDATE commit
   `6c68af1`: `composite_residual`, `CaseConservation::holds`, and
   `RunReport::conservation_holds` record and assert the residual, preserved
   even on a catch (tests 9-10). Residual is composite-coverage breadth.
9. Remaining (not blocking closure): a richer per-finding residual within a
   partially-explained divergence. Future work.

## Gap ledger

### GAP-B (provenance independence) - CLOSED on synthetic sources (commit `0fc1d9a`)
- WHY IT MATTERS: without it, correlated or self-attested evidence can masquerade
  as independent corroboration, which is the most dangerous way a skeptic engine
  can lie. It caps overall adequacy today.
- CURRENT EVIDENCE: none for evidence independence; only verdict-vs-expectation
  independence (a different axis) exists.
- MINIMAL FALSIFICATION TEST: two legs that are one observation relabeled must
  not jointly satisfy coverage; the composite must detect shared ancestry or
  return UNKNOWN.
- MINIMAL IMPLEMENTATION REQUIRED: a provenance tag on each leg/finding and an
  independence check in `classify` that refuses to count carriers sharing an
  origin. No new crate.
- REALITY DEPENDENCY: none to define the check; a real multi-source case to
  prove it on.

### GAP-D (reality-boundary promotion is unenforced)
- WHY IT MATTERS: a synthetic case can claim REAL standing with no external
  grounding, which would let fixture work leak into a REAL claim. Directly
  blocks any future REAL rung.
- CURRENT EVIDENCE: `EvidenceTier` gates language but not admission; no promotion
  test exists.
- MINIMAL FALSIFICATION TEST: a synthetic case with `evidence_tier: REAL` and no
  external artifact must be refused REAL standing.
- MINIMAL IMPLEMENTATION REQUIRED: an admission rule that REAL requires a
  referenced, independently checkable external artifact (for example a real
  repository commit and its probe output), enforced before `classify` grants
  REAL-tier language.
- REALITY DEPENDENCY: a genuine external artifact to attest. This gap cannot be
  fully closed with fixtures.

### GAP-C (as-of temporal reconstruction) - CLOSED on synthetic rows (commit `6c68af1`)
- WHY IT MATTERS: without replay(T), INVAR cannot show that an earlier judgment
  was not contaminated by later knowledge, which is the core temporal guarantee.
- CURRENT EVIDENCE: bitemporal columns and tamper-evident chain exist; no as-of
  query.
- MINIMAL FALSIFICATION TEST: append t_v=T1 at t_r=T2; a reconstruction as-known-
  at-T1 must exclude that row and be byte-identical to the T1 state.
- MINIMAL IMPLEMENTATION REQUIRED: a read-only `reconstruct_as_of(t_r)` over the
  existing `lineage` table plus a test. No schema change.
- REALITY DEPENDENCY: none; provable on synthetic rows.

### GAP-F (conservation after a catch) - CLOSED on synthetic rows (commit `6c68af1`)
- WHY IT MATTERS: treating a catch as total explanation lets a partial success
  hide an uncovered remainder.
- CURRENT EVIDENCE: residue and UNKNOWN exist; no residual after a catch, no
  conservation invariant.
- MINIMAL FALSIFICATION TEST: a mixed case must report both its caught part and
  its residual; the partition must sum to the total with nothing dropped.
- MINIMAL IMPLEMENTATION REQUIRED: a residual field on `CaseResult` and an
  asserted, tested partition invariant in `report`.
- REALITY DEPENDENCY: none.

### GAP-A (standing granularity)
- WHY IT MATTERS: a three-state case verdict cannot express weak or mixed
  support, forcing real nuance into UNKNOWN and losing information.
- CURRENT EVIDENCE: the `EpistemicStatus` trichotomy plus rationale.
- MINIMAL FALSIFICATION TEST: a case warranting "weak support" must not be
  indistinguishable from "cannot say".
- MINIMAL IMPLEMENTATION REQUIRED: a richer standing type, introduced only if a
  real case proves the trichotomy loses needed information.
- REALITY DEPENDENCY: a real case that the trichotomy demonstrably mislabels.

### GAP-E (executed-but-inadmissible degradation) - CLOSED on synthetic axes (commit `71290d6`)
- WHY IT MATTERS: an executed leg that is temporally out of bounds or a
  correlated duplicate carrier must not grant coverage or corroboration it did
  not earn.
- CURRENT EVIDENCE: `admit_temporal` + `LegState::INADMISSIBLE` (temporal axis),
  `leg_corroboration` (provenance axis), tests 13-14.
- REMAINING: the reality-boundary inadmissibility axis (forged or unverified
  REAL tier) is REAL-dependent and tracked under GAP-D.

## Sequencing note

UPDATE (commit `71290d6`): GAP-B, GAP-C, GAP-E, and GAP-F are CLOSED on synthetic
data, 14/14 acceptance tests GREEN in the Linux cloud container (not Mac-native).
Primitives B, C, E, and F are SATISFIED (synthetic); A and D remain PARTIAL, so
overall adequacy is still NOT ESTABLISHED by weakest-link. This is the limit of
what synthetic local harnesses can prove: both remaining gaps are REAL-dependent.
A (a richer case-verdict standing) is held by this audit to a real case that the
trichotomy demonstrably mislabels; D (the reality boundary, and with it E's
third inadmissibility axis) needs a genuine external cross-repository artifact.
The next build is GAP-D with the first REAL evidence. The founder confirms.
