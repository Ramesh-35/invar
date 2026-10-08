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

## Release v0.1.0-alpha (2026-10-07)

Release workflow run 37596752643 on tag `v0.1.0-alpha` (commit `b4be483`),
green in 3m50s: `cargo test --workspace --locked`, then all three builds, then
publish. Published by github-actions[bot] as a Pre-release.

| Asset | SHA-256 (GitHub asset digest) | Size |
|---|---|---|
| `invar-v0.1.0-alpha-x86_64-unknown-linux-musl.tar.gz` | `28953639e3c6c414ed7521d298124c9a67c3871638259cbc8c719ed5597b4b41` | 1.75 MiB |
| `invar-v0.1.0-alpha-x86_64-apple-darwin.tar.gz` | `84985680ccc49c899497abd84c2ea471133ffed2bf1a9729a77c5103ee880bd2` | 1.60 MiB |
| `invar-v0.1.0-alpha-aarch64-apple-darwin.tar.gz` | `c44b3b862d8fff83e62bbfdf67403a2d8b21e3a41d8d166ef3d38e2f13a07c41` | 1.49 MiB |

Each tarball ships with a `.sha256` sidecar (113, 112 and 118 bytes).

Established by the green run (each is a step in `release.yml`):
- All three targets built with `--locked` after the test job passed.
- Linux musl: `ldd` found no dynamic dependencies (static), and
  `invar --version` ran on the runner.
- aarch64-apple-darwin: `invar --version` ran on the macos-14 runner.
- x86_64-apple-darwin: built only. The arm64 runner does not execute it, so
  this binary has not been run anywhere.

NOT yet verified, stated plainly:
- The sidecar contents were not compared to the digests above in this
  session. The digests are GitHub's own; the founder can confirm with
  `gh release download v0.1.0-alpha --pattern '*.sha256'`.
- The Action's binary download path (`gh release download` plus checksum check
  in `action/install.sh`) has never run. The self-test uses `uses: ./`, which
  has no version ref and therefore builds from source.
- Evidence source: `gh run list` and `gh release view` output pasted by the
  founder; this session has no read access to the repository.

Adequacy is unchanged: D PARTIAL, overall NOT ESTABLISHED. A shipped binary is
a packaging fact, not evidence about reality.

## Release binary used from outside the repository (2026-10-07)

Private test repository `Ramesh-35/invar-action-smoke` (owned by the founder)
runs `uses: Ramesh-35/invar-platform@v0.1.0-alpha` on `ubuntu-latest`. The
manifest pins both sides of REAL-INCIDENT-001 as remote references
(producer `7c08557...:schema.json`, consumer `fe73889...:client.js`).

Run 37609090956 (commit `9c1d39f`, attempt 2), 2026-10-07T10:49Z:
- `invar-v0.1.0-alpha-x86_64-unknown-linux-musl.tar.gz: OK` (sidecar
  checksum verified on the runner; 1,836,046 bytes, matching the published
  1.75 MiB asset).
- `invar: using release v0.1.0-alpha (x86_64-unknown-linux-musl), checksum
  verified`.
- `[VIOLATED] real-incident-001 (schema): ["amount_cents"]`.
- The workflow's assertion step (binary installed by the release path,
  `--version` reports 0.1.0-alpha, verdict `violated`, `amount_cents` and
  consumer BLAKE3 `ecd0a60a...` in the report) logged no error. The run's
  final status line was not in the pasted excerpt.

This is the first execution of the Action's binary-download path, and the
first run of the shipped musl binary on a runner outside this repository.

How it got there (each failure was real and is kept):
- Attempt 1 failed before INVAR ran: "Unable to resolve action". The private
  repository's Actions access level was `none`; set to `user`.
- Attempt 2 (run 37605355380, rerun) fell back to a source build: the
  fine-grained token lacked read access to `invar-platform` releases (HTTP
  403, found by a temporary diagnostic step). The assertion step caught the
  fallback and failed the job, as designed. The token was corrected.
- Defect found and fixed: `action/install.sh` discarded the download error,
  so a 403 read as "no release asset". It now prints the reason before
  falling back. Not yet in a release: `v0.1.0-alpha` still has the old
  message.

Constraints recorded: GitHub lets only private repositories owned by the same
user use an Action from a private repository, and a caller's `GITHUB_TOKEN`
cannot read another private repository's releases, so private consumers need
a read-only token. Both go away when `invar-platform` is public.

Adequacy unchanged: D PARTIAL, overall NOT ESTABLISHED. This is distribution
evidence, not evidence about reality.

## Release v0.1.0-alpha.1 (2026-10-07)

Confirmed green before tagging:
- `invar-action-smoke` run 37609090956 (attempt 2, the external run recorded
  above): green.
- `invar-action-smoke` run 37610639232 on `87fa285` (diagnostic step
  reverted, workflow back to its original form): green.
- `invar-platform` CI run 37610640967 on `1b29058` (install-script fix):
  green, including `action-selftest`.

Release workflow run 37610852839 on annotated tag `v0.1.0-alpha.1`
(commit `1b29058`), green in 3m49s; published by github-actions[bot] as a
Pre-release.

| Asset | SHA-256 (GitHub asset digest) | Size |
|---|---|---|
| `invar-v0.1.0-alpha.1-x86_64-unknown-linux-musl.tar.gz` | `257d6754d82ac353620d135a0b911a315a9b3a58d6d39641264583469c611b94` | 1.75 MiB |
| `invar-v0.1.0-alpha.1-x86_64-apple-darwin.tar.gz` | `3915d5d71a8433f0220edd5a3aaa4213f8fe7f2dfe259afc451e048377a579cf` | 1.60 MiB |
| `invar-v0.1.0-alpha.1-aarch64-apple-darwin.tar.gz` | `6b81ffc9446815211e516c0c1f6b5cdac0b573e3f69338287ea5d8860368aa6b` | 1.49 MiB |

Each tarball ships with a `.sha256` sidecar (120, 114 and 115 bytes). The
same release.yml steps as v0.1.0-alpha ran: tests, `--locked` builds, musl
static-linkage check, `--version` smoke test on Linux and Apple Silicon
(Intel macOS built only, never executed).

What this release changes: only `action/install.sh` (it now reports why a
release download failed). The `invar` binary's code is unchanged from
v0.1.0-alpha.

Known inconsistency, stated plainly: `Cargo.toml` still says
`0.1.0-alpha`, so these binaries report `invar 0.1.0-alpha` while the
assets are named `v0.1.0-alpha.1`. `release.yml` does not check that the
tag matches the crate version. To fix in the next release: bump the version
and add a release step that fails on a mismatch.

Not yet verified: no run has consumed `@v0.1.0-alpha.1` (the smoke repo
pins `@v0.1.0-alpha`), so the new failure message has run only against a
stub `gh`, not on a runner.

Evidence source: `gh run list` and `gh release view` output pasted by the
founder. Adequacy unchanged: D PARTIAL, overall NOT ESTABLISHED.

## v0.1.0-alpha.1 consumed from outside the repository (2026-10-07)

`invar-action-smoke` run 37613226396 on commit `114e7a8` (workflow pinned to
`uses: Ramesh-35/invar-platform@v0.1.0-alpha.1`), exit 0, 2026-10-07T11:18Z:
- `invar-v0.1.0-alpha.1-x86_64-unknown-linux-musl.tar.gz: OK` (checksum).
- `invar: using release v0.1.0-alpha.1 (x86_64-unknown-linux-musl), checksum
  verified`.
- `[VIOLATED] real-incident-001 (schema): ["amount_cents"]`; producer BLAKE3
  `639cf21a...`, consumer BLAKE3 `ecd0a60a...`, both unchanged from every
  earlier run.

This closes the "no run has consumed `@v0.1.0-alpha.1`" gap above. The
install log names the tag; the assertion's `--version` check cannot tell
the two releases apart (both binaries report `0.1.0-alpha`). The new failure
message itself was not exercised (the download succeeded).

## PR comment path, live (2026-10-07/08)

`invar-action-smoke` PR #1 (branch `invar-pr-test`), workflow
`invar (pull request)` using `Ramesh-35/invar-platform@v0.1.0-alpha.1` with
`post_comment: 'true'`, `fail_on_violation: 'true'` and the workflow's own
token (`pull-requests: write`). Contract: producer schema pinned to
REAL-INCIDENT-001's P1, consumer `pr/client.js` on the PR branch.

| Push | Commit | Run | Check | Comment 6036978332 |
|---|---|---|---|---|
| 1, reads `amount_cents` | `9dfe6ab` | 37614304630 | red | posted, `VIOLATED (1 of 1)` (2026-10-07T11:29:49Z) |
| 2, reads `amount_micro_units` | `29d4c19` | 37740968482 | green | same comment updated, `ALL HOLD (1 contract(s))` (2026-10-08T07:03:18Z) |
| 3, reads `amount_cents` again | `fa5d0bd` | 37741819357 | red | same comment updated, `VIOLATED (1 of 1)` (2026-10-08T07:12:17Z) |

One comment throughout: created once, refreshed in place, never duplicated,
and a stale VIOLATED comment did not outlive the fix. The red checks are the
gate failing on a violation, by design (`At least one cross-repository
contract is VIOLATED`).

Also observed on a runner for the first time: the install script's new
failure message, `not downloaded: release not found`, then a source build.
The workflow token cannot read `invar-platform`'s private releases; GitHub
reports that as "release not found", so the message is accurate but does not
name the access cause. Finding: the Action uses one token both to download
the binary and to comment, which forces either a broadened token or a source
build in private consumers. A separate download-token input is the fix.

Not verified: rendering was checked by heading text; the table's appearance
on the PR page was not inspected in this session. This run used the
`alpha.1` binary, so its comments carry no line locations.



## Roadmap re-grounded and locked (2026-10-08)

The founder adopted the external research report "INVAR: What We Are
Missing" (October 2026) as binding. Order and "Do not build" list:
docs/architecture/ROADMAP.md. CUDA thesis retired. New labels:
`SOURCED (vendor study, not about contracts)` for AI-defect figures;
`UNVERIFIED` for incident-share figures not backed by Azure HotOS '19 or
SOSP '21. Report citations were not re-verified in this repository and are
labelled SOURCED (external report) until checked.

Critical path is R1, the real-tier corpus. Known limit stated before it runs:
the consumer side of schema and event contracts recognizes only
`payload.<field>` reads. Real consumers rarely use that exact name, so R1 is
expected to expose misses; they will be recorded, not hidden.

## Release v0.1.0-alpha.2 (2026-10-08)

CI run 37742923843 on `fe0af65`: green. Release run 37743250048 on annotated
tag `v0.1.0-alpha.2` (`fe0af65`): green, published as a Pre-release. The new
"tag must match the crate version" step passed (crate `0.1.0-alpha.2`). The
version is now derived from Cargo.toml in the CLI and run reports.

| Asset | SHA-256 (GitHub asset digest) | Size |
|---|---|---|
| `invar-v0.1.0-alpha.2-x86_64-unknown-linux-musl.tar.gz` | `19d0019ab32c5c43e5783e1322a412f29869548822650b1b06926a49940fa83d` | 1.76 MiB |
| `invar-v0.1.0-alpha.2-x86_64-apple-darwin.tar.gz` | `d0f971752309ec1ddd4ea35255bb75cd3958ed17c0f008685c7082f007e62f59` | 1.61 MiB |
| `invar-v0.1.0-alpha.2-aarch64-apple-darwin.tar.gz` | `01e8a94740b1b94703de16a2822c91b3170cad1810414b35bf035cecdfdc68f6` | 1.50 MiB |

Contents: witness locations (`0bcc0e0`). Not yet consumed by any run outside
this repository. Evidence source: output pasted by the founder.

## R1 batch 1 (2026-10-08)

Five real cross-repository breaks and one control, sourced from public git
history and issue pages and re-verified here (docs/falsification/R1-CORPUS.md,
raw results in docs/falsification/r1-batch1-results.tsv). INVAR caught 0 of
5 (1 MISSED, 4 ERROR because the producers publish YAML); openapi-diff 0.24.1
caught 0 of 5 outright (1 partial, 3 missed, 1 error). In 3 of 5 cases the
producer's published spec still declared the removed fields at the breaking
commit, so no spec-reading tool could catch them. None of the five consumers
reads fields as `payload.<field>`. A fail-open defect (non-JSON producer read
as "defines nothing") was found and fixed. Five cases are not enough to apply
the locked thresholds (they apply at 30), and selection bias is likely; the
direction of the evidence is nonetheless negative for INVAR as built.
Adequacy unchanged: D PARTIAL, overall NOT ESTABLISHED.

## Live witness locations and the engine overhaul (2026-10-08)

`invar-action-smoke` PR #1 on `@v0.1.0-alpha.2` (run 37749259452): the
refreshed comment 6036978332 reads `` `amount_cents` read at
`pr/client.js:3` · not in `schema.json /properties` ``. First line-level
location on a real PR.

Engine overhaul (founder directive 2026-10-08), tuned on R1 batch 1 and
frozen at `0804555` (fingerprint `6e973b22dfaa18b1`) before batch 2: YAML
specs (zero-dependency reader, matches PyYAML on 397 files), broad consumer
reads, type changes, change-scoped checking against `producer_base`, and
dual-source producers (published spec plus producer code). Not released.

R1 batch 2, held out (10 real breaks, 2 controls): INVAR caught 4 of 10
(gitleaks v8, all 12 fields; podman `Created` type change; RomM; cargo-audit),
missed 4, errored on 2 (a 1.8 MB spec over the 1 MB fetch limit; a
JavaScript producer). Both controls clean; no extra residue. 8 of 10
producers publish no usable spec, so the catches came from producer code.
Total real cases: 15 of 30; thresholds not yet applicable. Disclosure: the
batch 2 descriptions were read before the engine was written; see
docs/falsification/R1-CORPUS.md. Adequacy unchanged.

## R1 verdict at 30 real cases (2026-10-08)

Engine alpha.4 (naming forms, runtime-assembled producer payloads, 5 MB
default fetch limit configurable to 50 MB), tuned on batches 1 and 2, frozen
at `8f331ca` (fingerprint `88844f1512e83e06`) before batch 3 was sourced.

Batch 3 (held out, 15 real breaks, 3 controls): INVAR caught 4 outright and 2
partially; 7 missed; 2 errors (C++ producers). Controls: 2 clean, 1 error.
openapi-diff, where it could run: caught 2 (including r1-021, which INVAR
missed), missed 2, errored on 2.

Across all 30 real breaks: held-out scores (each batch run by the engine
frozen before it) catch 8 of 30 (+2 partial); the current engine catches 13
of 30 (+3 partial). **The locked R1 test is FALSIFIED on the miss-rate
criterion**: INVAR misses 47% to 73% of real breaks, against a 20% bar. The
baseline criterion does not trigger: most INVAR catches came from producer
code that no spec-diff tool can read. Details and causes:
docs/falsification/R1-CORPUS.md. The next step is a founder decision.
Adequacy unchanged: D PARTIAL, overall NOT ESTABLISHED.

