> **HISTORICAL ARCHIVE:** This document records early Phase 0/1 architectural research and is superseded by `README.md`, `docs/reference/INVAR-LOCK.md`, and `docs/epistemic/STATUS.md`.

# INVAR Sovereignty Report

Phase 0 of the INVAR commission. Read-only audit of the authoritative repository,
reconciled against the staging folder and the math-thread ruling. Verified on
device `mac-lan` on 2026-10-03. No architecture or feature was changed during the
audit; the only writes this turn are the four commission documents and a rung
correction to `INVAR.md`/`docs/epistemic/STATUS.md` (Section H).

Rungs used (lowest to highest): FORMALIZED, IMPLEMENTED, TESTED (synthetic),
ADVERSARIALLY-TESTED (synthetic), REACHABLE, PRODUCTION-CONFORMANT,
REAL-DATA-VALIDATED, INDEPENDENTLY-VALIDATED, REALITY-PROVEN. Bare "PROVEN" is
reserved for REAL evidence and is never a synthetic status.

## A. Repository identity
- Path: this repository (authoritative).
- Branch: `master`. HEAD: `fdf6551adc63adf4e8619eaca11bce2d53ca7e89`.
- Commits: 1 (`chore: establish independent INVAR platform repository`).
- Remotes: none. Never pushed. Working tree: clean.
- Distinct from the founder's separate enterprise platform and from the staging
  folder (`~/Documents/PROJECT INVAR`). No nesting, no shared path.

## B. Existing implementation
- One workspace member: `crates/invar-falsify` (the only `Cargo.toml` besides the
  workspace root). Modules: `main, lib, manifest, config, evaluator, ledger,
  report`, plus `tests/harness_test.rs`.
- Six acceptance tests authored: `catch_requires_signature_match`,
  `advisory_finding_is_residue_not_catch`, `missing_required_leg_is_unknown`,
  `ledger_chain_verifies_with_distinct_times`, `ledger_detects_silent_row_edit`,
  `ledger_uses_wal_on_disk`.
- Deferred placeholders carry NO `Cargo.toml` and are not workspace members:
  `crates/invar-core`, `crates/invar-cusum`, `crates/invar-lsp`,
  `packages/invar-isv`. Correct per DR-003.
- Corpus: 5 cases (`tc001`..`tc005`). Rules: `rules/semgrep/falsification_rules.yml`,
  `rules/opa/falsification_policy.rego`.
- Docs: `OVERVIEW, SIMPLICITY, laws/M1, laws/M2, strategy/CUDA-THESIS`,
  `decisions/DR-001..006`, `epistemic/STATUS`, `falsification/METHODOLOGY`.
- CI: PRESENT at `.github/workflows/ci.yml` (59 lines): on push to master/main
  and on pull request it installs the Rust stable toolchain, runs `cargo build`
  and `cargo test --workspace --locked`, installs Semgrep, validates the
  manifest, runs the harness to `runs/ci`, and verifies the ledger. DEFINED,
  NEVER EXECUTED: the repo has no remote, so this workflow has not run once.
  (`action/` holds only `.gitkeep`; the composite GitHub Action is deferred.)

## C. Mathematical / constitutional inventory
Implemented in code (crate):
- Evaluator `classify`: three-state verdict (CAUGHT_BY_COMPOSITE / PROVEN_RESIDUE
  / UNKNOWN), coverage-gated, polarity-blind (never reads `failure_type` /
  `expected_status` to choose polarity). Five leg states PASS/FAIL/SKIPPED/ERROR/
  UNKNOWN; only PASS/FAIL count as executed. Rung: TESTED (synthetic), see H.
- Ledger: two distinct clocks (t_v, t_r), `state_hash = BLAKE3(content)`,
  `lineage_hash = BLAKE3(prev ++ state_hash ++ context)`, `verify()` recomputes
  both hashes while re-walking rows (no stored-flag trust), SQLite WAL. Rung:
  TESTED + ADVERSARIALLY-TESTED (tamper) (synthetic), see H.

Formalized only, NOT present in the repo (they live in the math-thread
adjudication artifact, not on disk here):
- Cross-repo AST schema morphism `f` (reconstructed correspondence with standing;
  Desync a partial typed predicate). FORMALIZED.
- 3-axis verdict product (Standing x Relation x Comparability). FORMALIZED.
- Poset lower-set view `down(s)`. DESIGN.
- Set-valued recovery `C_min`. DESIGN.
- HKDF key isolation. SPECIFIED (no formal definition; not ZK).
- Statistical code-metric change-point. UNVERIFIED / BLOCKED (no CBR-001 transfer).

Gap: the founder's six-primitive Mathematical-Adequacy framework (Epistemic
Standing, Provenance Independence, Temporal Admissibility, Reality Boundary,
Weakest-link Composition, Conservation-with-residual) is named by the commission
as a permanent gate but is NOT instantiated as INVAR's own documented constitution
in this repo. This is a Constitutional Research Gap, not an implemented layer.

## D. Product surface
- CLI `invar-falsify` exists as SOURCE (subcommands run/validate/report/verify).
- No built/released binary, no published package (crates.io/npm), no installer, no
  SDK, no editor/LSP integration, no GitHub Action (placeholder only).
- A human-readable `EPISTEMIC_REPORT.md` and machine-readable `run_results.json`
  are produced per run (seen under `runs/latest/`, correctly gitignored).

## E. Operational state
- Build from a clean machine: NOT possible in this device shell (Linux VM has
  `git` but no `cargo`/`rustc`/`semgrep`). The Mac itself has rust 1.98 and a
  semgrep venv, so it builds THERE, not here.
- New-developer install: NOT possible (nothing published, no install path).
- Run / understand result / CI integration / safe failure / upgrade / reproducible
  release: NONE established. Operational rung: pre-REACHABLE.

## F. Claims vs evidence (truthful rungs)
| Claim | Highest truthful rung |
|---|---|
| Classifier is polarity-blind, coverage-gated | TESTED (synthetic); not re-adjudicated this session |
| Ledger keeps t_v/t_r distinct; verify recomputes the chain | TESTED (synthetic) |
| Ledger detects a silent row edit | ADVERSARIALLY-TESTED (synthetic) |
| BLAKE3 chain integrity | authenticates the transaction-time APPEND LOG only, not a poset edge-set |
| Category residue exists | HYPOTHESIS; FIXTURE-tier CANDIDATE only (HE=0) |
| CUDA five-flaw thesis | STRATEGIC HYPOTHESIS / UNVERIFIED |
| Performance (sub-15ms, 100k LOC, zero races, 100% gate) | UNVERIFIED / TARGET |
| HKDF multi-tenant isolation | SPECIFIED (not ZK) |
| CUSUM for AST detection | UNVERIFIED / BLOCKED (no CBR-001 transfer) |

No claim stands at REAL-DATA-VALIDATED or above. HE=0 everywhere: there is no
real cross-repository defect corpus, no exercised composite baseline against one,
and no reproduced differential isolation.

## G. Reconciliation (repo vs staging master vs math ruling)
- Repository `fdf6551` is AUTHORITATIVE. `MASTER-INVAR-PROJECT.md` does not exist
  in the repo; it lives only in `~/Documents/PROJECT INVAR`, is now SUPERSEDED,
  and still carries `[PROVEN in-repo]` / `[PROVEN present]` labels that the
  math-thread ruling downgrades. Per instruction it is LEFT UNEDITED to prevent
  divergence; the repo is the single source of truth.
- Staging folder also holds an `INVAR PLATFORM.zip` (Aug 31) and scaffold
  `Cargo.toml`/`composite.toml`: all stale pre-build artifacts, not authoritative.
- Math-thread ruling applied to the repo this turn (Section H).

## H. Corrections applied this turn (authoritative location only)
- `docs/epistemic/STATUS.md`: added `[TEST-PROVEN]` and
  `[ADVERSARIALLY-VALIDATED-ON-SYNTHETIC]` to the claim-status vocabulary, with a
  note that bare `[PROVEN]` is reserved for REAL evidence.
- `INVAR.md` Section F: the three facts relabelled from `[PROVEN]` to
  `[TEST-PROVEN]`; fact 2 scoped to the append log; the "become fully PROVEN once
  cargo test runs" parenthetical replaced with the honest HAS-IMPL-unconfirmed
  statement (tests authored; last green run reported on the Mac in an earlier
  session; not re-executed or adjudicated this session; no remote, so the build
  gate keeps HAS-IMPL UNCONFIRMED until a fresh adjudicated green run).

## I. Top contradictions / gaps
1. Staging master overstates rungs vs the math ruling. Resolved: repo corrected, master superseded.
2. CI is defined (`.github/workflows/ci.yml`) but has never executed: the repo
   has no remote. The residual gap is execution and adjudication, not absence.
3. Math-registry primitives are formalized off-repo, absent from disk. Gap.
4. Tests cannot be re-run in the device VM (no cargo). The cheap unblock is a
   remote-adjudicated `cargo test` on the Mac to move the rung from
   TESTED-pending to TESTED-adjudicated.

## J. Shortest rigorous path to a real developer using INVAR
Re-run the suite on the Mac (adjudicate the rung) -> source ONE real
cross-repository incident -> build the minimal differential probe -> run the
composite baseline and the probe -> honest CAUGHT/RESIDUE/UNKNOWN verdict. Only a
REAL-tier probe-confirmed residue opens production (DR-003). Developer-facing
install/onboarding is built only after the category survives that test.
