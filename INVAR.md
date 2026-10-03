# INVAR Platform — Root Execution Blueprint

> **Authoritative document.** Every forward-looking claim below carries an
> explicit epistemic label. Absence of a label on a capability statement is a
> defect; report it.
>
> Labels: `[PROVEN]` (demonstrated in this repository, reproducibly) ·
> `[HYPOTHESIS]` (testable, not yet tested) · `[TARGET]` (desired, no evidence) ·
> `[UNVERIFIED]` (asserted elsewhere, not established here) ·
> `[STRATEGIC HYPOTHESIS]` (business/positioning bet, not a technical fact).

---

## A. Product thesis

INVAR is the **Executable Invariant Control Substrate (EICS)**: a proposed
substrate that records and checks *invariants* about a software system across
repository, time, and runtime boundaries, where those invariants are exactly the
things that fall between the cracks of existing single-scope tools.

- `[HYPOTHESIS]` There exists an **irreducible category residue** — a class of
  real failures that a realistic composite of Semgrep + OPA + OpenTelemetry +
  Property Testing does not catch, but which an invariant substrate can.
- `[HYPOTHESIS]` The most valuable residue is **cross-repository, bi-temporal,
  and relational**: defects that are locally valid in every file yet violate a
  system-level invariant (e.g. TC-001's silent under-billing).
- **The thesis is unproven.** The entire purpose of Phase 1 is to try to
  *falsify* it. If a fully-exercised composite catches everything, INVAR has no
  category to stand on, and this repository will say so.

## B. Architecture (authorized vs deferred)

| Component | Path | Status |
|---|---|---|
| Falsification instrument | `crates/invar-falsify` | **AUTHORIZED** (Phase 1). |
| Core substrate | `crates/invar-core` | `[DEFERRED]` — no build system. |
| Language server | `crates/invar-lsp` | `[DEFERRED]` — no build system. |
| Structural-break detector | `crates/invar-cusum` | `[DEFERRED]` — no build system. |
| ISV integration package | `packages/invar-isv` | `[DEFERRED]` — no build system. |

The deferred components carry **no `Cargo.toml`/manifest**, are **not workspace
members**, and **cannot build**. They are promoted only when a specific
falsifiable requirement authorizes them (DR-003).

The falsification instrument's internal architecture and the bi-temporal
semantic invariants it depends on are described in
[`docs/architecture/OVERVIEW.md`](docs/architecture/OVERVIEW.md).

## C. Repository boundaries

- INVAR lives **only** at `/Users/nikhilramesh/Documents/Projects/invar-platform`.
- **INVAR is not NEXTLOGIC.** NEXTLOGIC
  (`/Users/nikhilramesh/Documents/Projects/nextlogic`) is a separate enterprise
  decision-intelligence platform with its own source, history, and lifecycle.
- INVAR does not import NEXTLOGIC source, history, or documents. See DR-001,
  DR-002.
- `[TARGET]` NEXTLOGIC may one day *consume* INVAR through an explicit,
  versioned interface. There is **no runtime coupling today** (DR-001).

## D. Falsification-first engineering strategy

Phase 1 ships an instrument, not a product. The instrument (`invar-falsify`)
runs the composite baseline against a corpus and classifies each case into
exactly one of three states. It is designed to be a *skeptic*: it will only
report `PROVEN_RESIDUE` when the composite demonstrably ran and failed. The gate
and required-legs table are in
[`docs/falsification/METHODOLOGY.md`](docs/falsification/METHODOLOGY.md); the
epistemic vocabularies are in
[`docs/epistemic/STATUS.md`](docs/epistemic/STATUS.md).

## E. Current status

- `[PROVEN]` The instrument compiles, is tested (six acceptance tests), and
  classifies the corpus deterministically *(pending the toolchain gate; see the
  run report).*
- `[PROVEN]` A leg that does not execute is never counted as PASS or as "caught
  nothing" — enforced structurally in `evaluator.rs`.
- `[HYPOTHESIS]` TC-001 represents a genuine category residue. On FIXTURE
  evidence it can be no more than a **candidate**.
- Nothing here claims performance, security, correctness, or category
  superiority as fact.

## F. Verified facts (TEST-PROVEN on synthetic fixtures)

1. `[TEST-PROVEN]` The classifier never reads `failure_type`/`expected_status` to
   decide a verdict's polarity (it reads `failure_type` only for leg-coverage).
2. `[TEST-PROVEN]` The ledger keeps valid-time and transaction-time in distinct
   columns and re-verifies its BLAKE3 chain by recomputation, not a stored flag.
   The chain authenticates the transaction-time append log only, not a poset edge-set.
3. `[ADVERSARIALLY-VALIDATED-ON-SYNTHETIC]` A silent row edit in the ledger is detected as a `StateMismatch`.

(Rung: TEST-PROVEN. The six tests are authored in-repo and were last reported
green by Claude Code on this Mac in an earlier session. They are NOT re-executed
or adjudicated this session (the device shell has no `cargo`), and the repo has
no remote, so HAS-IMPL stays UNCONFIRMED under the build gate until a fresh green
`cargo test` is run and adjudicated. Bare `PROVEN` is reserved for REAL,
non-synthetic evidence and is never a synthetic status.)

## G. Hypotheses (testable, untested)

- `[HYPOTHESIS]` Cross-repository schema desync is under-served by single-repo
  static analysis.
- `[HYPOTHESIS]` A bi-temporal, content-addressed lineage substrate can express
  and check the residue invariants economically.
- `[HYPOTHESIS]` The runtime-shaped failures (TC-002/003/004) are residue too —
  currently UNTESTED because their required OpenTelemetry leg cannot run offline.

## H. Unverified targets

- `[TARGET]` Sub-millisecond-to-low-millisecond AST processing. **No benchmark
  exists.** Any specific latency figure is `[UNVERIFIED]`.
- `[TARGET]` Merge-gate enforcement in CI.
- `[TARGET]` Zero-false-positive residue classification on REAL incidents.
- `[UNVERIFIED]` Every prior external claim of category superiority.

## I. Deferred components

`invar-core`, `invar-lsp`, `invar-cusum`, `invar-isv` are placeholders. Notably:

- `[UNVERIFIED]` "Robust CUSUM detects source-code structural breaks." CUSUM is
  a change-point method; its applicability to ASTs is a hypothesis, not a fact
  (DR-006).
- `[UNVERIFIED]` "The CBR-001 market-regime detector transfers to AST analysis."
  It does not transfer for free; no such transfer is asserted here (DR-006).
- `[UNVERIFIED]` "HKDF is a zero-knowledge proof system." HKDF is a **key
  derivation function**. It is not a zero-knowledge proof of anything (DR-006).

## J. Integration boundaries

- **Runtime:** none between INVAR and NEXTLOGIC today (DR-001).
- **Data:** INVAR reads only its own corpus and configuration.
- `[TARGET]` A future, explicit INVAR→consumer interface will be versioned and
  documented before any coupling is introduced.

---

### Monetization architecture

`[STRATEGIC HYPOTHESIS]` An open-core split: the falsification instrument and
corpus schema are community; enterprise value (managed lineage, org-wide merge
gates, cross-repo fleets) is commercial. This is a business bet, not a technical
claim, and the concrete license boundary is an **unresolved decision** (see
[`LICENSE`](LICENSE)).

### Phased execution roadmap

1. **Phase 1 — Falsification (this phase).** Instrument + corpus + honest gate.
2. **Phase 2 — Differential probe.** A minimal cross-repository probe that tries
   to *promote* a FIXTURE residue toward evidence, plus one REAL incident.
3. `[TARGET]` **Phase 3+** — only if a REAL-tier residue survives, author the
   core substrate. Not before (DR-003).

### Simplicity Law

INVAR is bound by the Simplicity Law
([`docs/architecture/SIMPLICITY.md`](docs/architecture/SIMPLICITY.md)): the
smallest correct instrument that can falsify the thesis beats a large one that
merely looks complete.
