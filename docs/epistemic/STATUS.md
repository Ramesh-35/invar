# Epistemic Status — Controlled Vocabularies

This document defines the three vocabularies used throughout the repository.
Mixing them is an error.

## 1. Claim status (used in prose/docs)

| Label | Meaning |
|---|---|
| `[PROVEN]` | Demonstrated in THIS repository, reproducibly. |
| `[HYPOTHESIS]` | A testable proposition, not yet tested. |
| `[TARGET]` | A desired outcome with no supporting evidence yet. |
| `[UNVERIFIED]` | Asserted elsewhere; not established here. |
| `[STRATEGIC HYPOTHESIS]` | A business/positioning bet, not a technical fact. |
| `[TEST-PROVEN]` | Demonstrated by an automated test on SYNTHETIC fixtures; last green run reported, not re-adjudicated. Never equal to `[PROVEN]`. |
| `[ADVERSARIALLY-VALIDATED-ON-SYNTHETIC]` | A test that actively tries to break the property (e.g. ledger tamper detection) passed, on synthetic data. Still synthetic. |


> **Reservation (math-thread ruling; DR-003/DR-006).** Bare `[PROVEN]` is
> reserved for REAL, non-synthetic evidence: a differential residue isolated
> on a real cross-repository incident. Synthetic harness results are at most
> `[TEST-PROVEN]`. A BLAKE3 hash chain authenticates the transaction-time
> append log only, not a poset edge-set.

## 2. Case verdict (the evaluator's output — exactly one per case)

| Verdict | Meaning |
|---|---|
| `CAUGHT_BY_COMPOSITE` | An **executed** composite leg isolated the designated failure (a finding matched the case's `catch_signature`). |
| `PROVEN_RESIDUE` | Every **required** leg executed and none isolated the failure. On FIXTURE evidence this is a *candidate* residue only. |
| `UNKNOWN` | We cannot honestly make either claim — typically because a required leg did not execute. |

The evaluator decides a verdict **only** from executed-leg evidence. It never
reads a case's `failure_type` or `expected_status` to choose the verdict's
polarity. `failure_type` is consulted solely to know which legs are *required*
to have executed (coverage).

## 3. Leg state (per composite tool, per case)

| State | Executed? | Meaning |
|---|---|---|
| `PASS` | **yes** | Ran, found nothing. |
| `FAIL` | **yes** | Ran, found something. |
| `SKIPPED` | no | Did not run (tool absent, disabled, or nothing to observe offline). |
| `ERROR` | no | Tried to run but the result is uninterpretable. |
| `UNKNOWN` | no | Executed but the result cannot be reduced to PASS/FAIL. |
| `INADMISSIBLE` | no | Executed and interpretable, but not allowed to count: temporally outside the as-of bound (GAP-C) or a provenance-correlated duplicate carrier (GAP-B/GAP-E). Never a catch. |

**Load-bearing rule:** `SKIPPED`, `ERROR`, and `INADMISSIBLE` are **never**
`PASS` and **never** "caught nothing". Only `PASS`/`FAIL` count as executed. A
case whose required leg is `SKIPPED`/`ERROR`/`INADMISSIBLE` is `UNKNOWN`, never
`PROVEN_RESIDUE`. This keeps a missing tool, or an executed-but-inadmissible one
(GAP-E), from masquerading as evidence that the composite failed.

## FIXTURE vs REAL evidence

`FIXTURE` cases are hand-built to probe the thesis. They can produce a residue
**candidate** but can never, by themselves, prove a category claim. Only `REAL`
cross-repository incidents can promote a candidate toward evidence, and only
through a confirming differential probe (DR-003). A `REAL` claim is additionally
gated by the reality-admission check (`reality::verify_real_tier`, GAP-D): an
unbacked `REAL` tier is refused and degraded to `UNKNOWN`. That gate is
necessary, not sufficient: it binds a proof to the recorded residue but does not
verify the referenced commits are real, so it does not by itself earn `[PROVEN]`.

## Gate 0 adjudication (2026-10-03)

The committed source was built and its acceptance suite run and adjudicated.
This is a has-implementation confirmation only. It is not REAL-tier evidence.

- Commit tested: `2164ac4`
- Source fingerprint (sha256 over all `.rs` + `Cargo.toml` + `Cargo.lock`): `821cb199c128fcfcbf4962622d7d03dbc59b30950c5914744e5edc73165cbe3d`
- `cargo build --workspace --locked`: GREEN
- `cargo test --workspace --locked -- --nocapture --test-threads=1`: 6/6 PASS (0 failed, 0 ignored, 0 skipped)
- has-implementation: CONFIRMED (was UNCONFIRMED)
- Rung: `TESTED-SYNTHETIC` (ADJUDICATED). Never bare `[PROVEN]`, never REAL.
- Environment: Linux cloud container, cargo/rustc 1.95.0.
- Caveat: NOT Mac-native execution. A founder-side `cargo test` on the authoritative machine remains a separate confirmation.
- Boundary: no REAL-tier evidence exists yet. All six tests are synthetic / FIXTURE evidence.

## Contract kinds (general invariant engine, 2026-10-04)

The engine dispatches evidence to a deterministic checker by declared kind
(`invariant::check_contract`). Residue empty = contract holds; non-empty = the
named violation. These expand the PRODUCT surface, not epistemic adequacy: the
A-F adequacy matrix is unchanged by their addition.

| Kind | Checks | Evidence tier so far |
|---|---|---|
| `SchemaContract` | consumer-read payload fields the producer schema does not DEFINE | REAL (REAL-INCIDENT-001) + synthetic tests 17-18 |
| `EventContract` | consumer-handler fields the producer payload does not EMIT | SYNTHETIC only (test 19); no real incident yet |
| `LockOrderContract` | sequence inversion / missing precondition in a trace | SYNTHETIC only (test 20); no real incident yet |

A kind reaches REAL standing only when run on a real incident of that kind,
through the same reality-admission gate (GAP-D). Until then it is a tested but
synthetic checker. Do not report EventContract or LockOrderContract as
REAL-proven.

`invar check` dispatches all three kinds from `[[invariant]]` blocks in
`invar.toml` and exits non-zero if any contract is violated. The reality gate
and the PROVEN_RESIDUE epistemic verdict stay in the resolving path, not in
`check`.

## Phase I: `invar init` and `invar check --ci` (2026-10-06)

`invar init` scans a workspace and proposes an `invar.toml`. Discovery is
HEURISTIC (file names and text patterns; only JSON is actually parsed). It
proposes runnable contracts only for JSON Schema and OpenAPI-JSON component
schemas paired with a consumer by field-name overlap. YAML specs, AsyncAPI,
protobuf, Axum/Actix/Express/FastAPI routes and Pydantic models are listed as
found-but-not-checkable. A proposal is not a verified contract.

`invar check --ci` resolves `github:OWNER/REPO@REF:PATH` references (REF may be
`{base}`, from GITHUB_BASE_REF) into memory through a `Transport` seam, hashes
the bytes with BLAKE3, records them as provenance, and never writes them to
disk. Tested with a mock transport and demonstrated live (below).

Live transport: `CurlTransport` (founder decision 2026-10-06). The
zero-dependency build has no HTTPS client, so the constraint relaxed was "no
shell-out", not "zero dependencies". curl runs as an argv subprocess (never a
shell), HTTPS-only including redirects, size- and time-bounded, with the token
passed on stdin, never on the command line. curl is inside the trust boundary,
as a TLS crate would have been. Live run (commit `8830086`): `invar check --ci`
fetched REAL-INCIDENT-001's consumer at C1 in-process, 229 bytes, BLAKE3
`ecd0a60aa8fd380e574fae70e75810da57f3b43c744ec03d492bc3b21c576f69`, identical to
an independent BLAKE3 of the out-of-band copy; VIOLATED `amount_cents`, exit 1.
A bogus-SHA control returned HTTP 404 and exited 3 (ERROR, not a verdict).

`invar check` exit codes: 0 all contracts hold; 1 at least one VIOLATED; 2
ledger integrity break; 3 the manifest could not be evaluated (an ERROR, never
a verdict).

Primitive D stays PARTIAL. In-process resolution has now run against a live
remote artifact, which removes the "externality only out-of-band" limit for the
`--ci` path. Still open: a single constructed demo incident, no run on an
actual CI runner yet, and `check` does not run the reality gate or the
composite baseline. The adequacy matrix headline is unchanged.

## Phase 3: GitHub Action and release pipeline (2026-10-06)

Built and committed (`d4a3c93`): `action.yml` (composite) with inputs `config`,
`base_ref`, `github_token`, `post_comment`, `fail_on_violation` and `version`,
and outputs `exit_code`, `verdict` and `report`. The PR comment is rendered by
tested Rust (`check --report-md`): ids and residue taken from scanned repos are
always escaped inline code, and the report states that a violation is not a
PROVEN_RESIDUE verdict. An ERROR (exit 3) fails the job even in report-only
mode and is never presented as a verdict.

Verified in this environment: 23/23 acceptance tests; all three YAML files
parsed with a real YAML parser; scripts shellcheck-clean and run end to end in
a simulated runner (release-less source build, live in-process fetch of
REAL-INCIDENT-001's consumer with the same BLAKE3, every gate branch, and every
comment branch against a stub `gh`).

NOT yet verified, stated plainly:
- No GitHub Actions runner has executed any of this. The repository has no
  remote, and the standing rule is no push without explicit instruction.
- `.github/workflows/release.yml` and the `action-selftest` job in
  `.github/workflows/ci.yml` are protected from remote writes; they were handed
  to the founder to place. Until placed and pushed they are not in the repo.
- No release binary has been built. The musl target could not be built here
  (toolchain download blocked) and the macOS targets need a macOS runner, so
  the first tagged release run is the first real test of all three targets.
- `cross` cannot build macOS targets: the pipeline uses `cross` for
  `x86_64-unknown-linux-musl` and native `cargo` on `macos-14` for both Apple
  targets.
- The crate license is still `LicenseRef-INVAR-PENDING`; publishing release
  binaries needs a license decision first.

Primitive D is unchanged (PARTIAL) and the adequacy matrix headline is
unchanged.

## Release decisions (2026-10-06)

- License: MIT OR Apache-2.0 (founder decision; supersedes the "still pending"
  note above). LICENSE-MIT and LICENSE-APACHE added; README and INVAR.md
  updated. Commercial components, if any, must live outside this repository.
- CLI binary renamed to `invar`; the internal package stays `invar-falsify`.
- Repository: GitHub, private first (reversible), to be made public after a
  review of the strategy and commission docs. Pushed by the founder from their
  Mac; this session holds no GitHub credentials.
- First tag: `v0.1.0-alpha`, published as a prerelease. It matches the binary's
  `0.1.0-alpha` and the NOT ESTABLISHED adequacy.
- Still unverified until the push: the runner self-test, and the release builds
  for all three targets. Primitive D is unchanged (PARTIAL).

## Runner evidence (2026-10-07)

Pushed by the founder to the private repository `Ramesh-35/invar-platform`
(`main`). This resolves the first item of "NOT yet verified" in Phase 3 for the
self-test only.

CI run 37590676958 on commit `8a6daa5` (workflow `invar-falsify CI`, push to
`main`), both jobs green on `ubuntu-latest`:
- `falsify`: green.
- `action-selftest`: the Action (`uses: ./`) resolved the binary on the runner
  (no release existed yet, so this was the source-build path), fetched the
  consumer in-process and reported, at 2026-10-07T07:59:42Z:
  `[VIOLATED] real-incident-001 (schema): ["amount_cents"]`, fetched
  `github:Ramesh-35/invar-demo-consumer@fe73889bf250275711339d065ecc37f21879fc72:client.js`
  (229 bytes, BLAKE3
  `ecd0a60aa8fd380e574fae70e75810da57f3b43c744ec03d492bc3b21c576f69`),
  byte-identical to the out-of-band copy and the earlier local `--ci` run.
  The assertion step checked verdict `violated` and both strings in the
  report. The bogus-SHA control produced `ERROR, not a verdict (exit 3)`
  and its assertion step checked verdict `error` and exit code 3.

Evidence source: `gh run view` output and log excerpts pasted by the founder;
this session has no read access to the repository and did not observe the run
directly.

What this does and does not change:
- Primitive D: the "not yet run on a CI runner" limit is removed. D stays
  PARTIAL: one constructed demo incident, not a wild defect, and the reality
  gate and PROVEN_RESIDUE verdict are still not wired into `check`. Overall
  adequacy stays NOT ESTABLISHED.
- The runner output is contract residue, not a PROVEN_RESIDUE verdict.
- Release: the first two `v0.1.0-alpha` runs (37590691338, 37592065161)
  failed because a stale `release.yml` packaged `invar-falsify` after the
  binary was renamed `invar`. They could not publish (publish needs all
  builds). Fixed in `b4be483`; the tag was moved to `b4be483`. The release
  builds for all three targets remain unverified until that run is reported.
- Runner notices, not failures: actions/checkout, cache and upload-artifact v4
  target Node.js 20 (forced to Node 24), and `ubuntu-latest` moves to Ubuntu
  26 from 2026-10-19.

