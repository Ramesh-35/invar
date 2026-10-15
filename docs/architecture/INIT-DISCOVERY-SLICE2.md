# INIT-DISCOVERY-SLICE2: egress locks and zod-value producers

Status: FROZEN as revision 2, ratified without further edits by the
founder's ruling of 2026-10-15 17:10, which also granted the lock.rs
and check.rs expansions (Class 3) and authorized gates 0 and 1. The
only edit at landing is this status line, recorded in the commit.
Changes after this freeze require an explicit revision and renewed
review. Prepared to the 86427 standard: numbered requirements with
falsifiable acceptance, abstention contract, fixture families with
declared expected outcomes, mutation set, gates 0 to 5 frozen before
any Rust is written. Parent: docs/architecture/EMITTED-SHAPE-DISCOVERY.md
(4393a89). Changes after freeze require an explicit revision and
renewed review.

## 0. Decision-record mapping (requested by ruling 13003)

The committed feasibility doc numbered S2-D1..D6 one way; the 13003
ruling table ruled on substantive proposals. Mapping, so nothing is
silently assigned:

| Committed record | 13003 disposition |
|---|---|
| S2-D1 slice boundary (K1+K2, K3 to 2b) | APPROVED IN PRINCIPLE |
| S2-D2 boundary identification for K1 | NOT YET RULED; this spec proposes R-K1-1/R-K1-2 below for that ruling |
| S2-D3 K1 additive-tolerant semantics | APPROVED FOR SPECIFICATION (also ruled earlier as "additive tolerance") |
| S2-D4 K2 type-blind pairing | APPROVED (ruled as "type-blind scope rule") |
| S2-D5 development-case quarantine (formbricks, strapi) | APPROVED (ruled as "quarantine") |
| S2-D6 Stage 3 evaluation protocol | Covered by 13003's boundary: no official evaluation or general-detection claim authorized by this spec |
| (13003's own "abstention rules for dynamic expressions") | Specified in section C |
| (13003's own "ISA invariance") | Specified in R-ISA-1..3 |

## A. Frozen scope and architecture

In scope:
- K1: egress-lock contracts (`kind = "egress"`): a self-contract on a
  schema VALUE surface that the repository emits at a declared
  boundary. Producer = a zod object-literal schema (K2 extraction);
  boundary evidence = the route or export site referencing it.
- K2: zod-value producer extraction and (a) feeding K1, (b) pair
  proposals (`kind = "schema"`, existing semantics) ONLY for type-blind
  consumers.

Two contract classes, kept distinct in every output: an EMITTED-SHAPE
contract (K1: a promise about what the repo emits; no consumer
evidence) versus a PRODUCER/CONSUMER contract (K2 pairs: evidence of
reads, correctness of the consumer never claimed).

Architecture:
- NEW module `valuepairs.rs` (name final at implementation): zod
  object-literal extraction with SHAPE ATTRIBUTION (keys belong to the
  exported const that owns the literal, never pooled across schemas in
  a file), boundary-evidence scan, K2 type-blind consumer scan. Reuses
  codepairs.rs masking, membership, resolution, stoplist as library
  functions; does NOT modify surface.rs.
- init.rs: render egress proposals and K2 pairs (evidence keys as in
  86428 pass 2).
- Path accounting, four distinct classes (revision-2 reconciliation):
  1. BASELINE-PROTECTED paths, byte-identical at every gate; their
     hashes are recorded mechanically at gate 0 from the tree of the
     freeze commit and re-verified at gate 4:
     crates/invar-falsify/src/surface.rs, backtest.rs, invariant.rs,
     config.rs, yaml.rs; crates/invar-falsify/Cargo.toml; root
     Cargo.toml; Cargo.lock.
  2. VERIFICATION ARTIFACTS, byte-identical always:
     docs/falsification/experiment-c/DEFINITION.md, corpus.tsv,
     labels.tsv, GROUND-TRUTH-REPORT.md, EVALUATION-PROTOCOL.md; plus
     the slice 1 fixture outputs, which must remain byte-identical as
     behavior evidence (R-ISA-3).
  3. REQUESTED EXPANSION paths, modifiable ONLY if the implementing
     directive explicitly grants them; nothing in this specification or
     its acceptance grants them: crates/invar-falsify/src/lock.rs (an
     egress branch building the locked surface from the valuepairs
     extraction, shape-attributed; slice 1's schema path untouched) and
     crates/invar-falsify/src/check.rs (evaluate `kind = "egress"`
     additive-tolerantly; extend the 86428 pass 3 no-lock rule to
     egress). Status reconciliation against the record: both files were
     protected under 86427 (EVIDENCE.md section 8); each received one
     bounded, ruled amendment under the 86428 scope expansion (passes 2
     and 3, landed in 4a7bab6); they returned to protected status with
     that directive's closure. The prior amendment confers no standing
     authorization; a new grant must be explicit.
  4. PROPOSED IMPLEMENTATION paths, to be confirmed as the authorized
     list by the implementing directive:
     crates/invar-falsify/src/valuepairs.rs (new), init.rs, main.rs
     (init output only), lib.rs (module line),
     crates/invar-falsify/tests/init_slice2_test.rs (new),
     scripts/slice2/ (mutation harness), and the documentation paths
     the implementing directive names.
  Authoritative baseline identity: the 86427 evidence package
  (EVIDENCE.md sections 2 and 8) and the published commits 4a7bab6 and
  4393a89; gate 0 records the concrete hashes from origin/main at
  freeze time rather than this document restating them.

K3 (string couplings) is excluded: same-package scope and
receiver-collision noise need their own feasibility and countermodel
review (slice 2b); fixture CM-6 below guards that slice 2 does not
absorb K3 behavior accidentally.

## B. Numbered requirements (each with its counterexample)

Extraction (K2):
- R-X-1: `export const <Name> = z.object({ <members> })` yields shape
  <Name> with its depth-1 member keys. Accept: fixture K2-F1 proposes
  with exactly those fields. Counterexample: K2-N1 (nested z.object
  members must not count; the analog of slice 1's D2).
- R-X-2: `.extend`, `.merge`, `.pick`, `.omit`, `.passthrough`,
  computed keys, and `...spread` inside the literal make the shape
  unrepresentable: ABSTAIN UNSUPPORTED-DYNAMIC, never a guess. Accept:
  K2-U1/U2 show the abstention line. Counterexample: any proposal from
  such a shape fails the fixture.
- R-X-3: stoplist and the 2-field minimum apply unchanged. Accept:
  K2-N2. Counterexample: a 1-field or all-stoplist overlap proposing.
- R-X-4: shape attribution is per-const: two schemas in one file never
  pool fields. Accept: K2-N3 (the N11 analog).

K2 pairing:
- R-P-1: a pair proposal requires a type-blind consumer: a .js/.jsx
  file importing the schema module, or a .ts consumer reading ONLY via
  bracket literals. Accept: K2-F2 (js consumer) proposes. 
- R-P-2: a typed consumer (z.infer or typed property reads) yields
  ABSTAIN TSC-GOVERNED, stated, never a proposal and never silence.
  Accept: K2-N4. Counterexample: a proposal on a typed consumer.
- R-P-3: proposals carry slice 1's evidence discipline: E-lines,
  machine-readable evidence keys, lock restricted to evidence (86428
  pass 2 semantics via the egress-aware path). Accept: the K2 lock on
  K2-F2 binds exactly the evidence fields, scoped.
- R-P-4: a proposal is never presented as proof of consumer
  correctness; every status line carries the heuristic disclaimer.
  Accept: string scan of every proposal in every fixture.

K1 boundary and binding:
- R-K1-1 (ruled 2026-10-15, with the eligibility clarification): the
  two evidence forms establish DISCOVERY ELIGIBILITY of a file only,
  never an emitted contract: (a) the file lies under a versioned API
  path segment (api/v<N>/ or v<N>/), or (b) the file or directory is
  declared under [egress] in invar.toml. Every proposal prints which
  eligibility evidence applied. A file with neither is not eligible;
  an eligible file with no establishable schema association yields
  ABSTAIN AMBIG-EGRESS, and boundaries are never inferred from
  proximity or naming guesses. Counterexample: K1-N1 (unversioned,
  unmarked route) must not propose. The four-layer semantics are part
  of this contract: eligibility (path or marking), association (named
  import plus reference), response-side association (R-K1-2), and
  runtime serialization governance, which NO static evidence in this
  slice establishes and which every proposal disclaims.
- R-K1-2 (revised per the return ruling): association alone is not
  enough; the reference must be in a RESPONSE-PRODUCING POSITION to
  the extent the static scanner can establish it. Concretely: the
  schema identifier must occur in a masked statement that also contains
  a member of the frozen response-marker set {`return`, `.json(`,
  `.send(`, `new Response`, `successResponse(`}, and a reference whose
  statement instead matches the frozen request-marker set {`req.`,
  `request.`, `body`, `params`, `searchParams`, `safeParse(`} with no
  response marker is request-side. Both marker sets are frozen contract
  vocabulary, like the stoplist; extending either requires a ruling.
  Rules: (1) at least one response-position reference is required for a
  proposal; (2) references that are request-side only, or that cannot
  be classified by the marker sets, yield ABSTAIN AMBIG-EGRESS, never a
  proposal; (3) the proposal's evidence names the boundary file, the
  schema, and the file:line of each response-position reference; (4)
  the mandatory disclaimer stays on every proposal, "PROPOSED
  (heuristic; response-position reference found; runtime serialization
  not verified)", because even a response-position reference does not
  prove runtime serialization; (5) if the relationship cannot be
  established within this pattern scope, the scanner abstains; no AST,
  type-system, or runtime analysis may be introduced to resolve it.
  Accept: K1-F1 proposes citing a response-position reference line;
  K1-U1 abstains (no reference); K1-U4 abstains (request-side only).
- R-K1-3: additive tolerance at check: a locked emitted field removed
  or renamed in the schema's current extraction is VIOLATED (exit 1)
  naming the field; an added field HOLDS (exit 0). Accept: K1-M-series
  fixtures flip exactly these. Counterexample: an addition alarming, or
  a removal holding.
- R-K1-4: a locked schema that becomes unextractable (turned dynamic)
  is NOT EVALUATED (exit 4) with the reason, never HOLDS. Accept:
  K1-U2.
- R-K1-5: no lock means no verdict: an egress contract without a lock
  entry is SKIPPED (exit 4), extending the 86428 pass 3 rule. Accept:
  K1-F1 checked before `invar lock` exits 4.
- R-K1-6: version families: schemas living in a version-family layout
  (v/<N> path segments, or sibling consts <Name>V<N>/V<N>_SCHEMA
  naming) are never paired across versions, and an egress lock binds
  one version only: ABSTAIN VERSION-FAMILY on cross-version candidates.
  Accept: K1-N2, the hoppscotch negative control, reproducing the
  verzod layout: 0 cross-version proposals, 0 alarms from
  reorganization. This fixture is a MANDATORY gate condition.

ISA and regression:
- R-ISA-1: manifest changes are additive only: new kind value(s), new
  evidence/boundary keys; the flat parser grammar is unchanged; slice 1
  manifests parse identically.
- R-ISA-2: the lock file grammar is unchanged; egress contracts use the
  existing line forms. The five exit codes (0 holds, 1 violated, 2
  ledger break, 3 error or lock drift, 4 skipped or not evaluated)
  are untouched in meaning and use.
- R-ISA-3: all slice 1 tests (40) and the harness suite pass unchanged;
  slice 1 fixture outputs stay byte-identical; F01 init output
  unchanged except any NEW egress/zod sections, with its 6 schema
  proposals byte-identical. Accept: gate 4.

## C. Abstention contract (exact codes, defined before code)

Reused as-is: UNSUPPORTED-SHAPE (type aliases; unchanged meaning), H2
family, R5, R6, BARREL2, AMBIG-SHAPE, AMBIG-PACKAGE, UNRESOLVED,
UNREADABLE, UNREPRESENTABLE, and the category labeling for test and
fixture files (never presented as production).
New:
- UNSUPPORTED-DYNAMIC: spread, computed keys, builder chains; the shape
  cannot be stated statically.
- AMBIG-EGRESS: boundary or serialization binding cannot be
  established; includes insufficient evidence for an emitted-shape
  contract.
- VERSION-FAMILY: candidate crosses a schema version family;
  compatibility machinery presumed, pairing declined.
- TSC-GOVERNED: producer and consumer are bound by the type system;
  INVAR adds no claim there.
No duplicate codes for equivalent conditions; an abstention is never
converted into a HOLDS or a VIOLATED.

## D. Fixture families (declared expected outcome, before execution)

K1 positives: K1-F1 (versioned route imports and references one
z.object schema: one egress proposal; lock; removal mutation of a field
violates; addition holds), K1-F2 (explicit invar.toml marking as the
boundary evidence), K1-F3 (two routes, two schemas, two contracts,
alpha.29 id uniqueness).
K1 negatives/ambiguous: K1-N1 (unversioned route: no proposal), K1-N2
(hoppscotch verzod negative control: versioned family reorganized, 0
proposals across versions, 0 alarms), K1-N3 (schema imported but never
referenced: AMBIG-EGRESS), K1-U1 (two schemas imported, none
referenced: AMBIG-EGRESS), K1-U2 (locked schema turned dynamic: exit
4), K1-U3 (no lock: exit 4, never HOLDS), K1-U4 (schema imported and
referenced ONLY in request-validation position, e.g.
`Schema.safeParse(await request.json())`: ABSTAIN AMBIG-EGRESS, no
proposal).
K2 positives: K2-F1 (extraction exactness), K2-F2 (js consumer pair
with evidence-bounded lock), K2-F3 (bracket-literal .ts consumer).
K2 negatives/ambiguous: K2-N1 (nested members), K2-N2 (stoplist and
minimum), K2-N3 (two schemas, one file, never pooled), K2-N4 (typed
consumer: TSC-GOVERNED), K2-U1 (.extend chain: UNSUPPORTED-DYNAMIC),
K2-U2 (spread member), CM-6 (a string field list naming schema fields:
K3 territory, must produce NO proposal under slice 2), CM-7 (test
fixture lists: category-labeled), K2-F4 (zod producer with BOTH a
typed consumer and a js consumer: exactly one pair, for the js file).
Shared: determinism (every fixture runs init twice, byte-identical),
and every proposal scanned for evidence lines and disclaimers.
All declared outcomes above are frozen with this document.

## E. Mutation set (harness reports CAUGHT / SURVIVED / BUILD-FAILED;
a BUILD-FAILED mutant is NOT a kill, and the gate requires every
mutation to BUILD and be CAUGHT by its named test)

1. nested zod members counted -> K2-N1
2. dynamic chain extracted anyway -> K2-U1
3. fields pooled across schemas in a file -> K2-N3
4. typed consumers proposed -> K2-N4
5. unversioned route proposed -> K1-N1
6. cross-version pairing allowed -> K1-N2
7. schema-without-reference proposed -> K1-N3 / K1-U1
8. addition alarms (additive tolerance broken) -> K1-F1's hold branch
9. removal holds (detection broken) -> K1-F1's violation branch
10. unextractable schema returns HOLDS -> K1-U2
11. unlocked egress returns HOLDS -> K1-U3
12. egress lock ignores evidence keys -> K2-F2's lock assertion
13. string-list coupling absorbed into slice 2 -> CM-6
14. request-side reference treated as response-side -> K1-U4

## F. Gates 0 to 5

| Gate | Entry | Operation | Evidence | Acceptance | Stop | Exit | Authority |
|---|---|---|---|---|---|---|---|
| 0 Baseline | spec FROZEN by ruling | hash HEAD, protected files, clean tree, on the Mac; container tree hash-matched | hash manifest | all match | any mismatch | PASS/FAIL | permits gate 1 only |
| 1 Implementation | gate 0 PASS; scope expansion for lock.rs/check.rs granted or refused in the implementing directive | write valuepairs.rs, init/main wiring, granted expansions; nothing else | changed-path list vs authorization | paths exactly as authorized | any unauthorized path; a protected-file need | PASS/FAIL/BLOCKED | permits gate 2 only |
| 2 Fixtures | gate 1 | run the full fixture set | all K1/K2/CM fixtures pass; determinism; disclaimer scans | 100% pass | K1-N2 alarming is an immediate BLOCKER (13003 stop condition 5) | PASS/FAIL | permits gate 3 only |
| 3 Mutations | gate 2 | control run, then every mutation | per-mutant BUILD + CAUGHT status | all build, all caught by the named test | a SURVIVED or BUILD-FAILED mutant | PASS/FAIL | permits gate 4 only |
| 4 Compatibility | gate 3 | full suite; slice 1 byte-identity; F01 re-run; protected-file hashes | 93+ tests; slice 1 outputs byte-identical; untouched-list hashes equal | all green, zero warnings | any slice 1 behavioral change; any untouched-file drift | PASS/FAIL | permits gate 5 only |
| 5 Documentation | gate 4 | slice doc evidence tables, STATUS append, exposure entries | docs landed, hash-verified | founder review | record conflicts | PASS/FAIL | permits the commit RULING only; never evaluation |

No gate authorizes broader work implicitly; gate 5 does not authorize
Stage 3, corpus work, or any general-detection claim.

## G. Corpus and evaluation boundaries

formbricks/formbricks and strapi/strapi are development cases,
quarantined from any Stage 3 corpus; hoppscotch's verzod family is the
mandatory negative control and also exposure-logged; K3 is outside
slice 2; Stage 2's terminal record and acceptance criteria are
unchanged; no official evaluation and no general-detection claim arise
from this specification or its gates.

## H. Regression and compatibility plan

Evidence required at gate 4: lock format unchanged (byte-compare slice
1 locks regenerated); ISA behavior unchanged except additive kind
values; all five exit codes preserved in meaning; slice 1 fixture
outputs byte-identical; frozen Experiment C artifacts and the untouched
list of section A hash-equal to baseline. The protected-path list in
section A is copied from the 86427 evidence package, with lock.rs and
check.rs carried as named expansion REQUESTS only.

## I. Evidence ledger (classifications as of this draft)

| Claim | Classification |
|---|---|
| formbricks route and schema counts (114/53/3/84; 315 z.object) | PROBE-REPORTED |
| assembled_keys parses plain zod member lines | PROBE-REPORTED |
| K2 reduces F01's z.infer abstentions | HYPOTHESIS |
| K1 detects breaking emitted-shape changes | NOT YET ESTABLISHED |
| K1 tolerates additive changes without false alarms | NOT YET ESTABLISHED |
| K2 pair proposals correct for type-blind consumers | NOT YET ESTABLISHED |
| hoppscotch negative control non-alarming | REQUIRED ACCEPTANCE CONDITION, NOT YET ESTABLISHED |
| general detection capability | NOT ESTABLISHED |

## I2. Change summary, revision 2 (every amendment to revision 1)

1. Status line: revision 2, not frozen, submitted for freeze review.
2. Section A path accounting rewritten into the four ruled classes
   (baseline-protected, verification artifacts, requested expansions,
   proposed implementation paths), with the lock.rs/check.rs status
   reconciled against the 86427 and 86428 records and the explicit
   statement that the prior amendment confers no standing
   authorization; concrete hashes are recorded at gate 0 from
   origin/main rather than restated here.
3. R-K1-1 restated as a discovery-ELIGIBILITY rule with the four-layer
   semantics (eligibility, association, response-side association,
   runtime governance) and the no-proximity-inference clause.
4. R-K1-2 revised: frozen response-marker and request-marker sets;
   response-position reference required; request-side or unclassifiable
   references abstain (AMBIG-EGRESS); evidence must cite file, schema,
   and reference file:line; disclaimer wording updated; explicit bar on
   resolving ambiguity via AST, type-system, or runtime analysis.
5. New fixture K1-U4 (request-side-only reference abstains) and new
   mutation 14 targeting it.
6. No other requirement, fixture identity, abstention code, mutation,
   or gate changed.

## J. Stop conditions (verbatim adoption of 13003 section 6)

Blockers, each stopping the affected slice and never widening scope:
K1 unable to bind schema to emitted response without guessing; K2
needing AST or type-system capability; any exit-code or lock-format
semantic change; an unrepresentable countermodel; the hoppscotch
control alarming; a mutation kill that is actually a load failure; any
need to expand the corpus or alter Stage 2 criteria; a path-list
conflict with 86427; any frozen Experiment C artifact or
section-A-untouched file needing change.
