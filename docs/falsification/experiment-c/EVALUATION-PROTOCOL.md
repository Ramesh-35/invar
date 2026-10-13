# INIT-DISCOVERY evaluation protocol (proposed for freezing)

Accepted as the working baseline by directive 58317 (2026-10-13). Its
contents are research evidence prepared before any INIT-DISCOVERY code
exists; it becomes frozen in the repository only when landed under
separate authorization, and is never edited after engine results are
seen. Errors found later are recorded as protocol deviations, appended
below, never silently corrected.

## Frozen inputs

- Corpus: the 30 repositories in corpus.tsv, at the recorded HEADs
  (seed 20261014, Experiment B's sample). Unchanged.
- Labels: labels.tsv (11 POSITIVE, 1 AMBIGUOUS, 18 NEGATIVE), from the
  hand audit in GROUND-TRUTH-REPORT.md.
- Definition: DEFINITION.md (conditions 1 to 4), unchanged since it was
  written before the audit.
- Stoplist and threshold: DEFINITION.md; at least 2 surviving fields.
- Test and other consumers: included and labeled (R2).
- Scope: R4 in; R5 and R6 out of v1.

## Metrics (exact)

- PRIMARY COVERAGE = |POSITIVE repos with >= 1 TRUE proposal| / 11.
- SENSITIVITY COVERAGE = |POSITIVE or krova-node repos with >= 1 TRUE
  proposal| / 12.
- PRECISION = |TRUE proposals| / |ALL proposals emitted in all 30
  repositories|, counted per proposal, including proposals emitted in
  NEGATIVE repositories.
- A proposal is TRUE when its (producer file, consumer file) pair
  instantiates a relationship meeting DEFINITION.md conditions 1 to 4,
  judged by hand at evaluation time. The producer file is the file that
  declares S, or the barrel through which the consumer imports it.
- A NEGATIVE repository can never be an engine false negative.

## Thresholds

- PRIMARY COVERAGE >= 7 of 11, and
- PRECISION >= 70%.
- Reported alongside: SENSITIVITY COVERAGE against >= 8 of 12.

## Reporting (all required)

Raw counts; a 30-row per-repository table (proposals emitted, TRUE,
false, labels); primary and sensitivity coverage; aggregate precision;
the small-sample statement below; every protocol deviation.

## Interpretation limit

n = 11. A primary coverage of 7 of 11 (63.6%) has a 95% Wilson interval
of 35.4% to 84.8%. The benchmark separates broad success from broad
failure only; it supports no fine-grained performance claim. The frame
is MCP-adjacent and TypeScript-heavy; no generalization beyond it.

## Scope rulings affecting evaluation (directive 64182)

- H1: schema files are inventory-only in v1. Schema inventory entries
  are not proposals: they never count as TRUE and never enter the
  precision numerator or denominator. Only actionable contract
  proposals are evaluated.
- H2: init treats a repository as a multi-package workspace only when
  its markers establish at least 2 qualifying packages (evidence table:
  INIT-DISCOVERY.md "Workspace evidence"; clarified by directive 73814).
  This does NOT change the evaluation frame: all 30 repositories stay in
  the per-repository table with their recorded labels, including every
  repository that is not a multi-package workspace. Experiment B's
  corrected denominator of 25 is historical: it was counted on the
  audit's basis (a marker listing no members: no `packages:` list, or a
  Cargo [workspace] without `members`), a different predicate from the
  clarified H2 (at least 2 qualifying packages), so the two counts are
  not interchangeable. It describes B's monorepo population on that
  basis only and does not replace this frame. A preliminary
  approximation under the clarified H2 (a rough script, not the engine)
  puts 6 repositories outside the multi-package population, adding
  minipuft/claude-prompts-mcp; it is an impact analysis, not a
  relabeling. Any count under H2 belongs to a new, separately identified
  evaluation and overwrites neither Experiment B nor Experiment C.

## Outcome accounting (directive 73814)

Added before any INIT-DISCOVERY code exists. It clarifies what the
Metrics section counts; the metric and threshold lines are unchanged.

| Outcome | Meaning | Counts as a proposal? |
|---|---|---|
| Actionable proposal | a contract init writes to invar.toml, asserting a candidate relationship | Yes |
| Explicit abstention | init declines to propose: evidence or supported scope is insufficient | No |
| Inventory-only observation | a file or structure recorded with no asserted relationship (H1) | No |

- Proposals are counted from the contracts in the invar.toml init
  writes, never from its printed output. Abstentions and inventory
  entries are never written there, so they enter neither the numerator
  nor the denominator of PRECISION.
- An abstention is never TRUE. A POSITIVE repository whose outcomes are
  only abstentions or inventory counts as NOT covered; the coverage
  denominators stay 11 and 12.
- Abstentions are reported separately, each with: repository and its
  frozen label; reason code (H2, R5, R6, unsupported syntax); the
  pattern responsible and its location; coverage impact (whether a
  POSITIVE or krova-node repository is left uncovered); and the
  evaluator's classification: SCOPE (the reason is correct and the case
  is outside v1 as ruled) or FAILURE (the case is inside v1 scope, or
  the stated reason is wrong). The engine's own reason code is not taken
  as the classification.
- Every uncovered POSITIVE repository is reported as either abstained
  (with reason) or silent (no proposal and no abstention). Inventory
  entries are reported as counts per repository and never judged TRUE
  or false.

Required cases. These are requirements, not results: none passes any
test until implemented and tested.
1. Abstention outside the denominator (evaluation rule): with one
   abstention and zero contracts in invar.toml, the proposal count is
   0, not 1.
2. Inventory cannot become a proposal (engine requirement; fixtures:
   the 3 H1 builder-schema cases): each yields one inventory entry and
   zero contracts, although the field rule passes them.
3. Field match alone is not a proposal (engine requirement; fixture: a
   synthetic consumer reading 2 or more distinctive fields of a shape it
   does not import by name, the R6 pattern): zero contracts, one R6
   abstention.
4. Abstention still costs coverage (evaluation rule): a POSITIVE
   repository with an R5 abstention and no TRUE proposal counts 0 in
   the coverage numerator, with the denominator unchanged.

## Measurement limitations (directive 73814)

- Recall within positive repositories is not measured. The audit
  verified one TRUE relationship per POSITIVE repository, not an
  exhaustive set. Coverage is repository-level and is never to be
  described as contract-level recall; a covered repository may hold
  further relationships init missed.
- One human rater. Labels and the TRUE or false judgment of each
  proposal are made by a single rater, the engineer who wrote the
  definition and will build the engine. The labels are not
  independently adjudicated, which limits the evaluation's independence
  and reproducibility.
- Reproduction is partly manual. Checking out each recorded HEAD is a
  manual step; the scripts do not read corpus.tsv's head column. No
  end-to-end automated reproduction is claimed.

## Protocol deviations

(none recorded)
