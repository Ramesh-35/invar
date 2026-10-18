# STAGE3-EVAL-001: pre-registered evaluation on unseen repositories

Status: DRAFT r1 for ruling. Execution NOT authorized. Engine frozen at
f9c4b29 for the whole evaluation. Prepared to the house standard.

## 0. Purpose and honest bounds

Stages 1 and 2 proved the engine against itself: declared fixtures and 97
mutants across seven harnesses, all caught. Stage 3 asks the only question
left: is the tool useful and honest on code it has never seen? The protocol
is pre-registered: every rule, metric and threshold below is ratified and
committed BEFORE any candidate repository is opened. The repositories are
the mutants now, and this protocol is the fixture.

Bounds, stated before any repo:

B-S-1 Stage 3 measures discovery precision and refusal honesty on real
code. It cannot prove the absence of false HOLDS; that burden stays with
the mutation regime, which carries it already.
B-S-2 N repositories is a smoke population, not statistics. The report
states counts, never percentages implying generality beyond the sample.
B-S-3 Adjudication is one human against a written rubric; inter-rater
reliability is out of scope in v1 and said so in the report.
B-S-4 The engine is frozen at f9c4b29 for the entire run. Findings are
recorded, never fixed mid-run. A re-run against a fixed engine is a NEW
evaluation with a new identifier; the first result is never overwritten.

## A. Frozen scope

A-S-1 Engine: the f9c4b29 release build, built once; the runner records
the binary hash into every log so a swapped binary is self-evident.
A-S-2 Ecosystem: TypeScript/JavaScript repositories only, init's v1 target.
A-S-3 Phase 1 is automated (clone pinned, init twice, determinism, exit
codes, timing, ledger row). Phase 2 is manual per the rubric (adjudicate,
lock, check, backtest), recorded verbatim.

## B. Repository selection (rule first, repos never)

R-S-1 Population: GitHub repositories with primary language TypeScript,
ordered by stars at ratification time, from one recorded search.
R-S-2 Filters, applied in rank order: at least 100 .ts or .tsx files; not
a type-definition or documentation corpus (the DefinitelyTyped class is
excluded by this rule, not by name after peeking); OSI-approved license.
R-S-3 Sampling: every 2nd qualifying repository by rank until N = 10.
R-S-4 Pinning: each selected repository's default-branch HEAD SHA at
selection time goes into scripts/stage3/repos.txt, committed BEFORE any
clone happens.
R-S-5 No repository is replaced after the manifest commit. One that fails
to clone or run stays in the ledger as a finding, never swapped out.

## C. Procedure

Phase 1 (scripts/stage3/run.sh): for each manifest entry, two fresh clones
at the pinned SHA; `invar init` in each; raw stdout, stderr, exit code and
wall seconds recorded; byte comparison of the two runs' stdout and
invar.toml; one ledger row per repository. No normalization anywhere.
Phase 2 (manual, per repository): adjudicate every proposal against the
rubric in D; lock the accepted set; `invar check`; `invar backtest` over
the last 100 commits; every outcome recorded verbatim in the ledger.

## D. Adjudication rubric

A proposal is TRUE iff reading the producer and consumer sites shows a
cross-file agreement where renaming or removing one side without the other
is a bug a reviewer would flag. UNSURE scores as NOT TRUE; the conservative
error is counted against the tool, never for it. Every judgment is one
ledger line: repository, contract id, TRUE or NOT TRUE, one sentence.

## E. Metrics and thresholds (declared now, before any repo exists)

M-S-1 ISA conformance: init exits 0 or 3 on every repository, no panic,
nothing outside the frozen exit codes. Threshold: 10 of 10, hard.
M-S-2 Determinism: the two fresh runs byte-identical in stdout and
invar.toml. Threshold: 10 of 10, hard.
M-S-3 Proposal precision: TRUE divided by all adjudicated proposals,
pooled across repositories, capped at the first 20 proposals per
repository in file order. Threshold: at least 60 percent, and at least one
repository yields at least one TRUE proposal.
M-S-4 Abstention honesty: in 10 randomly sampled files per repository that
produced no proposal, every candidate-shaped construct either appears as a
proposal elsewhere or carries an ABSTAIN line. Threshold: zero silent
drops, hard.
M-S-5 Round trip: on each repository with at least one TRUE proposal,
lock then check exits 0 with the durability line present, and a second
check is byte-identical. Hard where applicable.
M-S-6 Backtest: completes within the ISA on every repository attempted;
every VIOLATED is adjudicated as a real historical break (a success story)
or an engine error (a finding).
M-S-7 Performance: wall seconds per repository, recorded, no threshold.
This measurement is what gates the refused optimization backlog: any
future performance charter must cite M-S-7 evidence, never intuition.

## F. Findings discipline

Every anomaly gets an identifier S3-F-nn, classified as engine defect,
protocol defect, or expected behavior confirmed. The numbering is
monotone, so a trimmed ledger is visible by its gaps. The report publishes
every finding and the raw logs. Engine defects spawn their own chartered
arcs AFTER the evaluation closes, per B-S-4.

## G. Countermodels (ways this evaluation could lie, declared)

CM-S-1 cherry-picked repositories: killed by R-S-1 to R-S-5, rule first
and manifest committed before any clone.
CM-S-2 engine quietly fixed mid-run: killed by the binary hash in every
log and B-S-4.
CM-S-3 trimmed findings: killed by monotone numbering and committed logs.
CM-S-4 post-hoc thresholds: killed by this section being ratified and
committed while the manifest does not yet exist.
CM-S-5 a crash scored as an abstention: killed by verbatim exit codes in
the ledger; a panic is an M-S-1 failure, full stop.
CM-S-6 determinism checked after normalization: killed by raw byte
comparison; nothing is normalized before cmp.

## H. Decision points for ruling

S-D1 Population and filters (R-S-1, R-S-2).
S-D2 N = 10 and the every-2nd sampling rule (R-S-3).
S-D3 Adjudicator: you, under the D rubric, one line per judgment.
S-D4 The thresholds in E, especially the 60 percent precision bar.
S-D5 Placement: scripts/stage3/ at the repository root, since this
evaluates the platform, not one crate's internals.
S-D6 Publication: the ledger and raw logs commit into
docs/evidence/stage3/.
S-D7 Backtest depth: the last 100 commits per repository.

## I. Stop conditions

Stop and report on: any urge to amend selection, thresholds or rubric
after a repository is opened; any engine or fixture edit during the run;
any finding left out of the ledger; any repository swapped after the
manifest commit; any metric computed over normalized rather than raw bytes.

## J. Rulings of 2026-10-18 (recorded)

S-D1 to S-D7 RATIFIED as drafted (23:11). Execution GO: selection search,
manifest commit, phase 1. Phase 2 GO after adjudication (23:31).

## K. Execution ledger (appended at close; sections 0 to J are unchanged)

Status: EXECUTED. Engine frozen at f9c4b29 (binary eb93c9b16869) for every
run. Evidence: docs/evidence/stage3/ (selection, phase1, phase2, findings).

### K1. Verdicts against the pre-registered thresholds

| Metric | Verdict |
|---|---|
| M-S-1 ISA conformance | PASS, 10 of 10: every init exited 0, no panic |
| M-S-2 determinism | PASS, 10 of 10: raw byte-identical stdout and invar.toml across two fresh pinned clones |
| M-S-3 precision | PASS: 8 TRUE of 9 adjudicated (89 percent against a 60 percent floor), TRUE proposals in 2 repositories; S3-F-04 requests a re-ruling of the one NOT TRUE, which cannot change the verdict |
| M-S-4 abstention honesty | PASS: 55 candidate constructs in 60 sampled files, 0 silent, each matched to an ABSTAIN line naming its exact file:line and imported name (stricter than registered); exploratory over all files: 1,767 constructs, 0 unaccounted |
| M-S-5 round trip | PASS, 2 of 2 applicable: 8 of 8 TRUE contracts lock, check exit 0 with the durability line, second check byte-identical |
| M-S-6 backtest | completed 2 of 2 within the ISA (exit 0); adjudication: 61 of 61 absence events are engine errors (S3-F-05, S3-F-06). Backtest is NOT validated for strings contracts |
| M-S-7 performance | init 0 to 7 seconds, check and backtest under 1 second per repository; recorded, no threshold |

### K2. What the evaluation taught (stated, not hidden)

- The core held on unseen code: deterministic, inside the ISA, and every
  declined candidate coded, never guessed. An exploratory probe (not
  pre-registered) renamed one OmniRoute schema field: check exited 1 and
  named both stale consumers by file and line, untouched contracts HOLD.
- One engine defect class, in backtest: strings contracts are replayed
  with the wrong extractor (S3-F-05) and a never-present binding renders
  as reassurance (S3-F-06). Found because M-S-6 required every absence
  event to be adjudicated.
- One yield lever: package.json "exports" maps account for 83 percent of
  UNRESOLVED and blocked code-pair discovery in 6 of 8 multi-package
  repositories (S3-F-08).
- Per B-S-4, nothing was fixed mid-run. Each defect spawns its own
  chartered arc; a re-measurement after any fix is STAGE3-EVAL-002 on the
  same pinned manifest, and this result is never overwritten.
