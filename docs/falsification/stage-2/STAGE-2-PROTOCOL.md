# STAGE-2-PROTOCOL: the detection-advantage benchmark (FROZEN)

Status: FROZEN at commit time. Ratified by the founder's Stage 2 ruling
of 2026-10-14 (denominator, Category C structural scope, Category D vs
C-benign split all ruled explicitly). From this commit on, this file is
not edited after any engine result is seen; errors become recorded
protocol deviations. Binary under test: the slice 1 engine at 4a7bab6,
or a later commit a ruling names before run 1.

## 1. Hypothesis under test

H-S2: on real historical regressions in TypeScript monorepos, an
invar.lock taken before the break turns at least some real,
runtime-consequential field drifts into VIOLATED verdicts that the
repository's own TypeScript check at the breaking commit does not flag.

## 2. Structural honesty: where Category C can exist at all

This section is part of the protocol so the corpus is mined where the
hypothesis is falsifiable, not where it is impossible.

Slice 1 locks the producer TYPE surface against consumer reads. tsc
sees the same type. So when a consumer reads declared fields through
typed values and the producer type changes, BOTH catch it (Category A);
a corpus of those cannot confirm H-S2 and would not refute it honestly
either. Category C is structurally possible only where the type system's
view and the lock's view diverge:

- C-i. Type-blind consumers: reads through `any`, `as T` casts on
  `JSON.parse`/`fetch` wrappers, untyped destructuring, plain-JS
  consumer files in a TS workspace. tsc checks nothing there; the lock
  still binds the read field names.
- C-ii. Baseline absent or partial: the consumer package is excluded
  from the repo's tsc project, CI runs no typecheck, or `skipLibCheck`/
  config holes hide the producer. Recorded as its own column, never
  silently pooled with "tsc green".
- C-iii. Skew: consumer checked against stale local types while the
  producer moved (pre-build type drift inside the workspace).

Out of scope and stated: cross-repository drift (slice 1 is
intra-workspace), runtime payload drift with an unchanged type
declaration (invisible to BOTH tools; such a case is excluded, not
counted against either side).

## 3. Case definition

A case is (repository, C_prior, C_break, D, R):
- C_prior: a commit where the producer-consumer pair exists and slice 1
  scope holds (TS, 2+ qualifying packages, named import, 2+ non-stoplist
  direct fields).
- C_break: a later commit where an exported shape's direct member that
  the consumer read was removed, renamed, or re-typed incompatibly.
- D: the drifted field name(s), from the diff.
- R: documented runtime consequence: a linked issue, fix commit, test
  fix, or changelog entry stating a runtime failure or wrong behavior.
  "Looks dangerous" does not qualify.

## 4. Corpus mining, blind to the engine

Mining happens BEFORE invar runs on any candidate, by these criteria
only: public TS monorepos; fix commits or issues whose text documents a
runtime failure tied to a shared-type field change (undefined field,
renamed API field, missing property); the C-i/C-ii/C-iii structures
above. Mining tools: web search, GitHub web, clones. The miner may read
source to verify case structure; every mined repository is appended to
an exposure log for this stage before any engine run.

Target: at least 12 accepted cases, at least 6 of them C-i. Fewer after
a bounded search (recorded hours) = BLOCKED-INSUFFICIENT-CORPUS,
reported, no threshold evaluated.

Exclusions, frozen: biosshot/easyeda-copilot and every repository named
as a development case in the Experiment C exposure log; all synthetic
fixtures; the Experiment A/backtest batch repositories (their 38/70
measure belongs to the payload-read mechanism, not this one, and is
never pooled with Stage 2 results).

## 5. Procedure per case, mechanized

1. Checkout C_prior. `invar init` unaided; no hand-editing of
   invar.toml. If init does not propose the pair, record
   NOT-PROPOSED (counts as INVAR miss for coverage; categories below
   apply only to proposed pairs). `invar lock`.
2. Advance the worktree to C_break, keeping invar.toml and invar.lock
   from C_prior. `invar check`.
3. INVAR-CAUGHT iff exit 1 with VIOLATED on that pair naming a field in
   D. A VIOLATED naming only other fields is not a catch; it is scored
   as its own alarm under step 4.
4. Baseline at C_break: the repository's own typecheck as configured
   (tsc version from its lockfile, its tsconfig, its script). Record
   exit code and whether any emitted diagnostic points at the drifted
   usage. No configured typecheck = column "no baseline" (C-ii).
5. Every command, exit code, and output hash goes in the case record.

## 6. Categories (per proposed pair evaluated at C_break)

- A BOTH_CAUGHT: baseline flags the drifted usage AND INVAR-CAUGHT.
- B BASELINE_ONLY: baseline flags it; INVAR proposed but did not catch.
- C INVAR_ONLY_REAL: baseline exit 0 (or no diagnostic at the drifted
  usage) AND INVAR-CAUGHT AND R documents the runtime consequence.
- D INVAR_ONLY_SPURIOUS: VIOLATED whose named field did not in fact
  change presence, name, or compatible type between C_prior and C_break
  (false mechanics). A true field drift with benign outcome is recorded
  separately as C-benign and is neither C nor D.
- E INVAR_ABSTAINED: init abstained on the pair with one of its stated
  codes (H2, H2-ENTRY, H2-MEMBER, SLICE-SCOPE, R5, R6, UNSUPPORTED-SHAPE,
  BARREL2, AMBIG-SHAPE, AMBIG-PACKAGE, UNRESOLVED, UNREADABLE,
  UNREPRESENTABLE). A safe, explicit boundary, never pooled with B.
Plus NOT-PROPOSED from step 5.1, reported alongside.
C-benign (ruled): a VIOLATED whose named field genuinely drifted but
whose runtime path did not fail is true structural drift, recorded as
C-benign; it is neither C (no documented runtime consequence) nor D
(the mechanics were correct).

## 7. Pre-registered thresholds (RULED, frozen before any run)

Noise rate = Category D / (all VIOLATED verdicts emitted in the corpus
runs), i.e. D / (A + C + C-benign + D). Ruled by the founder: the
denominator is triggered alarms, because alarm fatigue is felt only when
INVAR actively alarms, and silent passing proposals must not dilute it.
- CONFIRM H-S2: at least 3 Category C cases AND noise rate below 15%.
- REFUTE H-S2: 0 Category C cases after a corpus meeting section 4's
  minimum, OR noise rate at or above 30%.
- Between: NOT ESTABLISHED, reported as such with the full table.
Secondary diagnostic, recorded but never governing: D / (all proposals
evaluated at C_break).

## 8. Discipline

No protocol edit after the first engine run on any mined case; corpus
list frozen (named commits) before run 1; every deviation recorded in
the results document; all case records, logs and hashes committed under
docs/falsification/stage-2/. Classifications available to results:
MECHANIZED for the runs, CONFIRMED-C only when baseline-green, INVAR
VIOLATED, and R are all evidenced. No general-capability claim follows
from any outcome; the result is a statement about this corpus.

## 9. Stop conditions

BLOCKED if: mining cannot reach the section 4 minimum; a case needs
engine changes to run (engine changes are a separate directive, never
mid-evaluation); baseline tooling for a case cannot be reproduced
(record and drop the case before its engine run, never after).

## Amendments (founder ruling 2026-10-15, before any engine run)

A1. Wave 2 cap (Option 1): one further mining wave of about 16
repositories selected for tsc blind spots (JS or JSX consumer packages,
absent or partial typecheck in CI, distinctive shape members). If the
corpus then meets section 4's minimum, run 1 proceeds; if not,
BLOCKED-INSUFFICIENT-CORPUS is recorded with the logged mining effort.
No further waves without a new ruling.

A2. External-boundary column (Option 2): the corpus table gains an
EXTERNAL BOUNDARY column for cases where the drift broke consumers of
the repository's published contract (API clients, SDK users) while the
intra-workspace code was updated atomically (the formbricks class).
Scientific fence, ruled explicitly: external-boundary cases NEVER count
toward Category C, are expected to score E or NOT-PROPOSED for slice 1,
and leave every H-S2 threshold and the noise-rate denominator untouched.
They document where the break mass sits, nothing more.

A3. Record correction: the INVAR exit-code ISA has five codes, in the
binary's own words (main.rs): 0 holds, 1 violated, 2 ledger break,
3 error or lock drift, 4 skipped or not evaluated.

A4. Miner method note: member-name matching is receiver-blind (the D1
limitation applies to the miner too), so wave 2 adds a distinctiveness
filter (compound or repo-rare member names) before surviving-read
analysis.

