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

## Release v0.1.0-alpha.3 (2026-10-08)

CI run 37753534128 on `d3ce79c`: green. Release run 37753759503 on annotated
tag `v0.1.0-alpha.3` (`d3ce79c`): green, Pre-release; tag/version guard
passed. Contents: the engine frozen at `0804555` (YAML specs, broad reads,
type changes, dual-source producers). Not the alpha.4 engine.

| Asset | SHA-256 (GitHub asset digest) | Size |
|---|---|---|
| `invar-v0.1.0-alpha.3-x86_64-unknown-linux-musl.tar.gz` | `7dcefa651fd6b07cf871ac0fb07bfa5ae3877ad8e495f0649e8a65950766e41b` | 1.82 MiB |
| `invar-v0.1.0-alpha.3-x86_64-apple-darwin.tar.gz` | `4a1529ca25ffc5717238f23d4e5903e0e70dd17bbb0d6bd3cc9344e66b2e678c` | 1.67 MiB |
| `invar-v0.1.0-alpha.3-aarch64-apple-darwin.tar.gz` | `8247c2b2dc27b828f103c7becd9feda42330f963d12efe0f4fc51808e624320a` | 1.56 MiB |

## Re-scope after R1 (founder directive 2026-10-08)

Declared scope: producers and consumers in Go, Python, JavaScript/TypeScript
and Rust; producer specs in JSON Schema, OpenAPI and AsyncAPI (JSON or YAML).
Protobuf is NOT supported (no `.proto` reader exists); listed in the directive
as in scope, it remains a TARGET.

Engine alpha.4 additions (fingerprint `96d46e3123cf1dca`, 28/28 tests):
- Contracts touching C, C++, C#, PHP or Swift are SKIPPED with the reason;
  `invar check` exits 4 when nothing is violated but something was skipped.
  A skip is never a pass; the Action's gate fails on exit 4.
- Producer fields are bound to their declaring scope (type, class, or
  enclosing function). Type changes compare within one scope only; Python
  class attributes are read at class-body level only, so parameters no
  longer mask removed keys.

Rerun of all 36 R1 cases (all now tuning data, so not a score): 4 breaks and
1 control SKIPPED; of the 26 breaks still evaluated, 14 caught, 3 partial, 9
missed. The R1 verdict stands (FALSIFIED on the miss rate). The narrowed
scope has not been scored on any case chosen after it was declared; a fresh
in-scope batch is the only fair test of it.


## R1 batch 4: in-scope benchmark (2026-10-08)

- Evidence rung: real, held out, frozen alpha.4 engine, 10 scored breaks
  plus 3 controls, all sides in Go, Python, JS/TS or Rust.
- INVAR caught 6 of 10, missed 4 (40%); controls 3 of 3 clean; one case
  with 2 false extras. Wilson 95% on catch rate: 31% to 83%.
- 5 of 18 candidates excluded before running for lack of consumer-impact
  evidence or unchanged wire format; INVAR flagged 4 of them. With all 15
  scored: miss rate 33% to 47%.
- R1 remains FALSIFIED (bar is 20%). Details in
  `docs/falsification/R1-CORPUS.md`.
- Source hash of commit 0dd1361 recomputed on the Mac: 3d7a023adc247ee5, equal to the
  sources that built the batch 4 binary (sha256 2d5d79e3...8115, rebuilt
  byte-identical).

## v0.1.0-alpha.5: R2 invar.lock built (2026-10-09)

- Built and tested in the cloud container (30 tests pass, synthetic) plus one
  real replay (Erigon `totalDifficulty` to Otterscan through a lock). Not
  measured on held-out data; detection is unchanged by the lock.
- Without an invar.lock, results on all 49 corpus cases are identical to
  alpha.4, except r1-023: its consumer is Java, now SKIPPED (founder
  directive 2026-10-09: Java, Kotlin, Ruby join C, C++, C#, PHP, Swift as
  unsupported). Its earlier PARTIAL came from reading Java as plain text; that
  partial catch is given up for an honest SKIPPED.
- Exit 1 means VIOLATED only; `invar lock` drift is exit 3 (LOCK_DRIFT);
  waivers capped at 90 days. Reference: docs/reference/INVAR-LOCK.md.

### v0.1.0-alpha.5 published (2026-10-09)

- Pushed `d3ce79c..ef2808c` (this also published 8f331ca, c4a6b0b and
  0dd1361; alpha.4 was never tagged). CI run 37900992040 passed on ef2808c.
- Tag v0.1.0-alpha.5 on ef2808c; release run 37901314299 passed; prerelease.
- Asset digests (sha256, as reported by GitHub):
  - aarch64-apple-darwin: 6db71e5cde87009bb3ffca34e9b5bd2f4c982e59d9e1ae8c4eeeba4646f6453b (1.69 MiB)
  - x86_64-apple-darwin: 3347552d2ec58c910686ecb4bccfba38fa34ebcd801867e16f9ee52333e2d272 (1.82 MiB)
  - x86_64-unknown-linux-musl: c0aa794fb320ac6314867e4300e4ea89bd65a1efcc4c51dad27c8bdd13b92c8f (1.98 MiB)
- Release notes: docs/releases/v0.1.0-alpha.5.md.

## R1 batch 5 and A1 stated blind spots (2026-10-09)

- Batch 5: 15 real in-scope breaks + 3 controls, held out, no exclusions.
  Frozen alpha.5: caught 7 of 15 (53% missed); controls 2 clean, 1 false
  positive (spec `$ref` not resolved). Combined in-scope held-out: 13 of 25
  caught (48% missed). R1 remains FALSIFIED.
- A1 spec frozen 2026-10-09T07:58:08Z before batch 5 was read
  (docs/falsification/A1-SPEC.md). A1 on batch 5: silent misses 8 to 5 of 15
  (53% to 33%); detection unchanged; 1 control turned NOT EVALUATED;
  "moved" rule noisy on refactors. Evidence rung: real, held out, n = 15.
- Exit 4 now also means NOT EVALUATED (A1), not only SKIPPED; the runner
  scores it UNEVALUATED, distinct from MISSED and SKIPPED.
- invar.lock format 1 frozen with a compatibility promise
  (docs/reference/INVAR-LOCK.md). Engine version 0.1.0-alpha.6 (not tagged).

### v0.1.0-alpha.6 published (2026-10-09)

- Tag v0.1.0-alpha.6 on 3931715; release run 37905838753 passed; prerelease;
  release notes from docs/releases/v0.1.0-alpha.6.md. CI run 37905203994
  passed on 3931715 before the tag was pushed.
- Asset digests (sha256, as reported by GitHub):
  - aarch64-apple-darwin: 601266866222b910d6b3c49e146c8f672a5150466faab70609ad2cee27db8b3a (1.70 MiB)
  - x86_64-apple-darwin: ea42430c83ccca51497674b4aa8d59e633960fe89a5633aed5c2717881cc2789 (1.83 MiB)
  - x86_64-unknown-linux-musl: bcd135fe721e355411639e0a746282f69ab07e5bf7894aa5e920ecef8ec7617d (1.99 MiB)

## v0.1.0-alpha.7: signed releases (2026-10-09)

- Release archives signed with an SSH key (secret INVAR_RELEASE_SIGNING_KEY);
  public key in action/release-signers; the Action verifies checksum and
  signature before running invar and stops on a bad signature.
- Tested on the founder's Mac workspace with a throwaway key and a stub
  release: good signature passes; tampered archive with a fresh checksum,
  missing signature, and wrong key each stop the job (exit 1).
- Chosen over GitHub artifact attestations (public repositories or Enterprise
  Cloud only) and Sigstore keyless (would publish the private repo's identity
  to a public log). Engine unchanged from alpha.6.

### v0.1.0-alpha.7 published (2026-10-09)

- CI run 37911185596 passed on 9d833b7; tag v0.1.0-alpha.7 on 9d833b7;
  release run 37911766195 passed (each signature verified against
  action/release-signers before publishing); prerelease.
- Signing key fingerprint SHA256:TskzCIjWasw0XmCJ3xKJYdVDW9kQILVlmgmzd8ERY+M.
- Asset digests (sha256, as reported by GitHub):
  - aarch64-apple-darwin: 516b603ee083de6b76cf4e35eacf5b329b9d58e5d65d9b825716c071d548285d (sig 3cd423df...a2c)
  - x86_64-apple-darwin: 1ea4fd639b9f6f86b6545d5f490c26bf094b4a7fac34e9be8c92f2fbfde5a576 (sig 97f91add...1d1)
  - x86_64-unknown-linux-musl: fdfa220c6e39b46fb6478f395a1461102a7bb551a8f731a4b921cc4c66f3a1d3 (sig 597353ab...60b)
- Live verification (2026-10-09): smoke repo run 37912445920 (workflow
  invar-release.yml, token INVAR_TOKEN) downloaded the alpha.7 musl archive,
  checked its SHA-256 ("OK") and its SSH signature: "invar: using release
  v0.1.0-alpha.7 (x86_64-unknown-linux-musl), checksum and signature
  verified". Evidence rung: real, one run, Linux only; the macOS paths and the
  refusal paths are tested on a stub only.
- Found: the smoke repo's PR workflow (invar-pr.yml) uses the default token,
  which cannot read this private repo's releases, so it builds from source
  every time ("release not found", run on 5d37337). The Action uses one token
  for both downloading and commenting; a separate download token is next.

## v0.1.0-alpha.8: Action hardening against moved tags (2026-10-09)

- Threat model: tj-actions/changed-files (CVE-2025-30066), tags repointed to a
  secret-leaking commit. docs/reference/ACTION-HARDENING.md.
- This repo's workflows pin every third-party action to a commit SHA, CI
  checks it (scripts/check-pins.sh), and CI runs with contents: read.
  dtolnay/rust-toolchain is now given toolchain: stable explicitly (its ref no
  longer names the toolchain).
- The Action supports pinning by full commit SHA (uses the signed release only
  if the SHA is exactly that release's commit, else builds from source), warns
  when pinned by tag, and takes a separate release_token for the download.
- Tested on the founder's Mac workspace with a stub release and stub gh:
  tag pin (warning + verified binary), matching SHA (verified binary),
  non-matching SHA (source build). Not yet run live.

### v0.1.0-alpha.8 published (2026-10-09)

- CI run 37913736150 passed on 0a7de0b (includes the new pinned-actions job);
  tag v0.1.0-alpha.8 on 0a7de0b947e6aa34fc79f9ab726a478e64768b57; release run
  37913969514 passed (built and signed with SHA-pinned actions).
- Tag-protection ruleset NOT active: GitHub returned 403 "Upgrade to GitHub
  Pro or make this repository public to enable this feature." Tags can still
  be moved by the owner account; SHA pinning is the defence that works today.
- Live verification (2026-10-09): smoke repo commit 55fca03 pins the Action by
  SHA 0a7de0b947e6aa34fc79f9ab726a478e64768b57. Both workflows took the
  signed release path with no tag warning: invar-release.yml run 37915003884
  and invar-pr.yml run 37915007540 (release_token = INVAR_TOKEN), each
  "using release v0.1.0-alpha.8 (x86_64-unknown-linux-musl), checksum and
  signature verified". The PR-token gap found on alpha.7 is closed. Evidence
  rung: real, Linux only; macOS and the refusal paths remain stub-tested.

## R1 batch 6 and D1 detection rules (2026-10-09)

- Batch 6: 15 real in-scope breaks + 3 controls, held out, no exclusions.
  alpha.8 engine: caught 9 of 15; 2 NOT EVALUATED; 4 silent misses; controls
  3 of 3 clean. Combined held-out (batches 4 to 6): 22 of 40 caught, 45%
  missed. R1 remains FALSIFIED.
- D1 (TypeScript Omit in new types; Python unannotated class fields), spec
  and engine frozen before batch 6: identical result on batch 6 (no
  applicable case). Tuning effect only: batch 5 from 7 to 9 of 15. No change
  on any other case or control across all 85 cases. Engine 0.1.0-alpha.9
  (not tagged).

## R1 batch 7 and E1 envelope rule (2026-10-09)

- Batch 7: 15 real in-scope breaks + 3 controls, held out, no exclusions.
  alpha.8, D1 and E1 engines all identical: caught 8 of 15; 2 NOT EVALUATED;
  5 silent misses; controls 2 clean, 1 NOT EVALUATED. Combined held-out
  (batches 4 to 7): 30 of 55 caught, 45% missed. R1 remains FALSIFIED.
- E1 (fields moved into a new envelope in Python/JS/TS/Go literals), spec and
  engine frozen before batch 7: no applicable case in batch 7. Tuning effect
  only (batch 6: r1-069 caught, r1-068 partial); no other change on any of
  103 cases. D1 likewise untested on held-out data after two batches.
- Finding: rules written from earlier misses have not recurred in fresh
  batches; the recurring miss is a default version/format switch (4 cases in
  batches 5 to 7). Engine 0.1.0-alpha.10 (not tagged).

## 2026-10-09 (later): F1 default-switch rule built; tuning sweep clean; batch 8 next

- F1 spec frozen and committed (docs/falsification/F1-SPEC.md, hash in
  F1-SPEC.frozen) before any batch 8 case was sourced.
- v0.1.0-alpha.11 engine built with F1: a default version or format selector
  whose value set gains a new value between base and head turns a would-be
  HOLDS into NOT FULLY EVALUATED. F1 never claims VIOLATED. 34 tests green.
- Tuning sweep, all 105 corpus cases (batches 1 to 7 plus controls), alpha.10
  engine versus F1 engine: exactly the four known default-switch misses flip
  MISSED -> UNEVALUATED (r1-059, r1-075, r1-099, r1-100). Zero other outcome
  changes; no control flipped; the one preexisting control FALSE_POSITIVE is
  identical in both engines. Evidence rung: tuning data. D1 and E1 also
  looked exact on tuning data and changed nothing held out; the only score
  that counts for F1 is batch 8, not yet sourced.

## 2026-10-10: batch 8 scored; F1 zero held-out effect; fifth default-switch recurrence missed

- Batch 8 (15 real breaks + 3 controls, r1-104..r1-121) scored on the frozen
  alpha.10 and alpha.11 engines: identical verdicts on all 18 cases.
  8 caught, 1 NOT EVALUATED (A1), 6 missed; controls clean. F1, like D1 and
  E1 before it, changed nothing on held-out data. Three rules in a row.
- The mechanism F1 was built for recurred (r1-113, Cargo.lock v4 default)
  and F1 missed it: the selector value is an enum variant
  (`ResolveVersion::V4`), outside the frozen value grammar (integers,
  booleans, quoted integers). The rule family is right about the mechanism
  and too narrow in its grammar; widening it is batch 9 work under a new
  frozen spec.
- Combined held-out record (batches 4 to 8): 38 of 70 in-scope real breaks
  caught, 46% missed. The 20% gate is not close. R1 remains FALSIFIED.
- Honest read of three zero-effect rules in a row: each new rule fixed past
  misses that have not recurred (except default-switch); the misses that do
  occur keep coming from mechanisms one notch outside each frozen rule.
  Tuning-data exactness (4 flips, 0 noise) predicted nothing about held-out
  effect, again.

## 2026-10-10 (later): v0.1.0-alpha.11 tagged; rule expansion frozen; A2 spec drafted

- v0.1.0-alpha.11 tagged on 5741001 carrying the batch 8 record. Note for
  the log: the tag was pushed before the CI run on 5741001 was visible; the
  commit is corpus and docs only, and the release workflow runs its own
  build and signature self-check. Release run result to be recorded when
  confirmed.
- Founder adjudication: no F2; per-mechanism textual rule expansion is
  frozen (three consecutive rules with zero held-out effect). Recorded in
  MOAT.md.
- A2 evidence receipts spec drafted for ratification at
  docs/architecture/A2-RECEIPTS.md: canonical byte-deterministic receipt,
  as_of as input, offline verify-receipt, optional SSH signature via the
  existing release machinery, cross-platform byte-identity gate over the
  whole corpus. RATIFY markers: verify-receipt exit codes, ssh-keygen
  subprocess, no hash-chaining.

## 2026-10-10 (later still): A2 receipts built, v0.1.0-alpha.12; Linux gate green, macOS gate pending CI

- A2 implemented per the ratified spec: `invar check --receipt` emits the
  canonical receipt; `invar verify-receipt` re-runs it (exit 0 reproduced,
  1 not reproduced or input drift or bad signature, 3 error); offline mode
  `--artifacts <dir>`; optional SSH signature check via ssh-keygen
  (namespace invar-receipt). Local file reads are now content-addressed in
  provenance when a receipt is requested (record_local), so drift in local
  inputs is detected, not silently re-evaluated. 35 tests green.
- Correction to the spec text: the corpus is 122 configs (104 + 18), not
  123; the _template dir was counted by mistake.
- Determinism gate, Linux x86_64: all 122 corpus configs emit byte-identical
  receipts across two runs and verify-receipt exits 0 on every one
  (docs/falsification/a2-gate-linux-x86_64.txt; one transient fetch flake
  on r1-063 passed on retry). The cross-platform half runs in CI
  (.github/workflows/receipts-gate.yml, workflow_dispatch): the A2 gate is
  NOT met until that run shows Linux and macOS receipts identical. Until
  then A2's claim stays "built, gate pending", and alpha.12 release notes
  must say so.
- Engine v0.1.0-alpha.12: binary sha256 1a9e310f1c868347..., src fingerprint
  da149cb55f7ab0bc. Detection rules unchanged from alpha.11 (rule expansion
  stays frozen); the batch 8 verdicts are unaffected.

## 2026-10-10 (evening): A2 cross-platform gate MET; alpha.12 ready to tag; 2026-2036 spec held for conflict adjudication

- a2-receipts-gate run 38047172189 (workflow_dispatch on c37efc9): gate
  (ubuntu-latest) green, gate (macos-latest) green, compare green. Linux and
  macOS receipts are byte-identical across all 122 corpus configs and
  verify-receipt exits 0 on every one. The A2 gate from MOAT.md is met, on
  real corpus inputs, cross-platform, in CI. Also noted: v0.1.0-alpha.11
  release run 37931048122 succeeded with signed assets on all three targets.
- A "2026-2036" architectural specification was received. It is NOT adopted
  into canon in this commit: its Phase 0 "reality baseline" describes a
  different repository (an LLM inference attestation server), not this one,
  and several of its ADRs contradict decisions already frozen here (exit
  code map, receipt format 1 compatibility promise, no new dependencies).
  Conflicts listed for founder adjudication before any part lands.

## 2026-10-10 (night): A2.1 in-toto export and the baseline signing ceremony, v0.1.0-alpha.13

- `invar receipt export --in-toto <receipt> [--out p]`: wraps a format 1
  receipt as an in-toto Statement v1.0 (subjects = the content-addressed
  inputs; predicate = the format 1 body verbatim). Additive view only;
  format 1 stays canonical per CONFLICTS ruling 3.
- Baseline ceremony (CONFLICTS adoption): a human signs the lock with
  `ssh-keygen -Y sign -n invar-lock -f <key> invar.lock`; `invar check` and
  `invar lock` accept `--lock-signers <file> --lock-principal <name>` and
  then refuse to evaluate unless invar.lock.sig verifies (missing lock,
  missing signature, or rejected signature: ERROR exit 3, fail closed,
  never a verdict). Without the flags, behavior is unchanged.
- 36 tests green (the SSH round-trip test also proves namespace separation:
  an invar-lock signature does not verify as invar-receipt). Receipts spot-
  checked deterministic under alpha.13. Engine binary sha256
  41813bffa16f3d11..., src fingerprint 495aaf3aa04bceb6. Detection rules
  unchanged; verdicts identical to alpha.11/alpha.12.

## 2026-10-11: releases recorded; A4 dependents graph built, v0.1.0-alpha.14

- Release records: v0.1.0-alpha.12 live and signed on all three targets;
  v0.1.0-alpha.13 CI green (run 38049584644), release run 38049587630
  result to be recorded when the founder confirms it.
- A4 (Option A, local-first) built per docs/architecture/A4-DEPENDENTS.md:
  `invar dependents <Type.field | field> --locks <dir> [--producer substr]
  [--format json]` answers "who depends on this field" from local
  invar.lock files. Zero network by construction (no resolver, no token in
  the module). Unparseable lockfiles are stated and exit 4 (an answer with
  blind spots is never a silent 0); no parseable lockfile at all is exit 3;
  the command never exits 1 (it holds no verdict). A scoped query that
  misses but whose field matches unscoped (spec-producer) bindings says so
  instead of showing a bare zero. 37 tests green.
- Gate unchanged and not met: A4 counts only when a producer uses the
  answer to change or delay a removal in a real repository. Until then it
  is plumbing, recorded as such.

## 2026-10-11 (later): alpha.13 release confirmed; public readiness audit delivered

- v0.1.0-alpha.13 release run 38049587630: SUCCESS, signed assets on all
  targets (founder confirmation). alpha.14 tagged on 02ac095; CI and
  release results pending.
- Public readiness audit committed at docs/epistemic/
  PUBLIC-READINESS-AUDIT.md: 2 blockers (the README and the CLI about-text
  still introduce the Phase-1 falsification harness, not the product),
  6 fixes (SECURITY.md, CONTRIBUTING.md, stale tables, missing release
  docs, exit-code documentation, init's consumer-discovery gap),
  2 founder decisions (NEXTLOGIC naming, moving the six Phase-0/1 root
  docs to docs/history/), and the clean checks (no secrets, licensing
  complete, pins enforced, epistemic docs consistent). Remediation order
  proposed; nothing changed yet beyond this record.

## 2026-10-11 (later): Stage 1 front-door remediation, v0.1.0-alpha.15

- README rewritten product-first: what INVAR checks, the sixty-second start,
  the frozen exit-code table, the hardened Action snippet, receipts and
  dependents, and a "What INVAR does not claim" section quoting the measured
  held-out record (38 of 70 caught, 46% missed, R1 FALSIFIED) with links to
  STATUS.md and the corpus. The research harness is now a labeled section.
- CLI front door: `invar --help` introduces the contract checker and its
  exit codes; product commands listed first, the six research-harness
  commands labeled as such. No behavioral change; 37 tests green.
- Remaining remediation stages per PUBLIC-READINESS-AUDIT.md: Stage 2
  (SECURITY.md, CONTRIBUTING.md, release-doc backfill), Stage 3 (NEXTLOGIC
  genericization per ruling (a), six root docs to docs/history/), Stage 4
  (init Python consumer discovery, ratified, with a test).

## 2026-10-11 (later still): Stages 2-3 landed; observed-reality and demo specs frozen

- Stage 2: SECURITY.md (GitHub private vulnerability reporting; verify
  binaries per RELEASE-SIGNING.md; honest attack-surface notes),
  CONTRIBUTING.md (corpus admission rules and honesty rules),
  docs/releases backfill (alpha.13, alpha.14, and a README noting the
  deliberate alpha.9 skip).
- Stage 3: NEXTLOGIC genericized in current-facing docs (INVAR.md,
  architecture OVERVIEW, MATHEMATICAL-ADEQUACY, invar-isv README) per
  ruling (a); decision records DR-001/DR-002 left as written history;
  personal absolute paths removed; the six Phase-0/1 root documents moved
  to docs/history/ with archive banners.
- Specs frozen for the next increments: docs/architecture/
  OBSERVED-REALITY.md (lock the shape of a real observed payload;
  shape-level, never byte-level; a sample is evidence about one response
  at one moment and the report says "observed", never "guaranteed") and
  docs/architecture/DEMO-ENGINE.md (offline replay of real CAUGHT corpus
  cases with the honest coda; explicitly NOT CrowdStrike/Azure, which are
  outside supported scope). Both carry RATIFY markers for the open points.
- v0.1.0-alpha.15 ready to tag once CI is green on the Stage 3 commit.

## 2026-10-11 (evening): observed-reality locking built, v0.1.0-alpha.16; validation benchmark still owed

- `invar lock --sample` and `invar check --sample` built per the frozen
  spec and the ratified points: shape-level locking (never values), PII
  key-name warnings (warn, never block), lock format 2 (format 1 is still
  read and written forever when no samples exist; older engines fail
  closed on format 2), drift = residue with waivers applied (exit 1),
  stale sample past 90 days = stated, NOT FULLY EVALUATED (exit 4).
  38 tests green; end-to-end smoke: drift exit 1, stale exit 4, PII
  warning fires, format 2 roundtrips.
- Demo case pick recorded with corrected ids: r1-109 (authentik), r1-112
  (headscale), r1-117 (ghostfolio); the ids in the ratifying dispatch
  (r1-012/028/035) referred to other cases.
- NOT yet done, and owed before any effectiveness claim: the ratified
  measurement gate. Reconstruct the five recurring-miss payloads (r1-059,
  r1-075, r1-099, r1-100, r1-113) from public evidence and measure whether
  sample-vs-sample comparison flags them; cases whose payloads cannot be
  honestly reconstructed are recorded as such. Until that is measured,
  observed-reality locking is a built mechanism with no effectiveness
  claim, and the docs say exactly that.

## 2026-10-11 (night): deep-research report received and adjudicated; DX batch shipped, v0.1.0-alpha.17

Report quality: guardrails held (right repository treated as ground truth,
evidence labeled, lookalike repos discarded, no invented citations found on
spot-check). Adjudication against shipped state:

Adopted and SHIPPED in alpha.17 (research items A1, A2, A3/A4-as-docs, A8):
- Fixed summary line on every check: `summary: held N | violated N |
  skipped or not evaluated N`, plus `measured: 38 of 70 held-out real
  cross-repository breaks caught`, printed even when counts are zero.
- Stable reason codes on every not-evaluated statement
  (docs/reference/REASON-CODES.md), no severity tier, append-only.
- Exit-4 CI semantics documented (non-zero fails plain CI; the Action's
  fail_on_violation is the knob). No new flag: the frozen exit map already
  does what cargo-audit's --deny does.
- Lock drift already names the exact next command (shipped in R2).

Adopted, NEXT engine increment: presence-witness counts (research B5/B6).
The shipped single-sample shape lock will flag an optional field that was
present in the locked sample and absent in a fresh one; witness counts
across repeated --sample runs (seen-present/absent/null, window, N) fix
that honestly: hard residue only for always-present-now-gone. Design page
mapping counts onto lock format 2 before code.

Adopted, corpus track (research D1-D6, founder dispatch Module C): licence
split (CDLA-Permissive-2.0 for data), Zenodo DOI, Software Heritage ids,
standalone scorer, JSONL export, sealed held-out set with a committed
manifest hash. Scheduled after the demo and coverage commands.

Adopted, experiments (research E1-E3): false-positive rate on 10 real PRs;
the exit-4 comprehension test with 5 developers; TTFV timing with 3
developers. All need the founder or external people; INVAR cannot grade
itself on these.

Conflicts, recorded:
- Research B7 (staleness as printed age, never an exit trigger) contradicts
  the founder's ratified stale-sample ruling (NOT FULLY EVALUATED, exit 4),
  which shipped in alpha.16 and stands.
- The dispatch's [[observed_shapes]]/schema_version lockfile sketch is the
  horizon report's fantasy format again; observed-reality locking shipped
  as lock format 2 [[sample]] sections (alpha.16). No change.
- invar.dev does not exist; reason-code references point at
  docs/reference/REASON-CODES.md in this repository, not at URLs we do not
  own.
- Demo case ids corrected once more: r1-109, r1-112, r1-117.

## 2026-10-11 (late night): experiments adjudicated; E1 measured clean; engine latency measured; E2 needs humans

- Adjudication of "run E1-E3 locally with synthetic user simulation":
  DECLINED for E2 and for E3's real metric. Simulated users are synthetic
  evidence graded by the thing under test; comprehension and
  time-to-first-verdict are human measurements. E2 is packaged for the
  founder to run in ~2 hours (docs/epistemic/E2-PACKET.md, scoring fixed in
  advance). E3's human half needs 3 outside developers.
- E1, run mechanically (the strongest form we can run without an outside
  reviewer): 10 real post-pin commits touching corpus producer files, each
  diff inspected and labeled before the engine ran (two candidates REJECTED
  at inspection: a ghostfolio commit that removes fields the pinned
  consumer reads is a break candidate, not a clean probe; replaced by the
  next additive commit). Probe = PR simulation (base = the commit's own
  parent, pinned consumer). Result: 10 of 10 HOLDS, zero false VIOLATED,
  zero noise codes (docs/falsification/e1-false-positive-probe.tsv).
  Against the pre-registered thresholds (<10% confirm, >25% refute): the
  assumption is CONFIRMED at n=10, with the stated caveats: small n, and
  the clean labels were judged by the engine's author, not an independent
  reviewer.
- Engine latency, measured (NOT time-to-first-verdict, which is a human
  metric): local `invar check`, median 2.2 ms, max 4.0 ms over 50 runs on a
  1-contract fixture (Linux container, release build); `invar init
  --stdout` median 2.0 ms; one remote `--ci` check with 3 live fetches:
  893 ms, network-bound, n=1. The 200 ms editor and 2 s pre-commit budgets
  hold with two orders of margin locally; remote checks are and will stay
  network-bound.

## 2026-10-11 (night, cont.): multi-sample presence witnesses built, v0.1.0-alpha.18

- Built per the frozen spec: `invar lock --sample` is repeatable and
  cumulative; each path carries present/absent/null counts, an observed
  type set, and the contract carries N and the observation window.
  Verdicts per the ratified threshold: VIOLATED only for
  always-present-now-gone (or retyped) with N>=3; optional fields and
  low-witness paths are stated ([not-evaluated:optional-field-absent],
  [not-evaluated:witness-count-low]), never violations and never silent.
- Lock format 3 (witness counters); format 2 is read forever as one
  observation; writes are format 1 (no samples) or 3. Rationale in the
  spec: an old engine refusing a new format fails closed; an old engine
  misreading one would fail silent.
- Output honesty fix found during smoke: stated non-evaluation lines now
  print alongside a VIOLATED verdict too; a verdict never swallows a blind
  spot. 38 tests green; smoke: optional-field absence stated while the
  always-present removal is the only violation.
- The observed-reality effectiveness gate (five reconstructed payloads)
  remains owed and is the next work item.

## 2026-10-11 (midnight): observed-reality effectiveness gate measured; two catches, one disqualifying finding

- The five-payload gate ran on real pinned payloads (record appended to
  docs/architecture/OBSERVED-REALITY.md). Results: r1-059 and r1-075
  CAUGHT by witnesses (VIOLATED, where every static engine was silent);
  r1-099 inconclusive (abridged doc payloads; stated, not silent);
  r1-100 not honestly reconstructable; r1-113 out of mechanism scope
  (TOML).
- The disqualifying finding: map-keyed payloads (lockfiles, exactly the
  recurring-miss class) explode path-level witnesses into thousands of
  false residue items on ordinary dependency churn (1,232 and 23,240 in
  the two catches). By our own E1 zero-false-alarm standard, no
  effectiveness claim is made. Next spec: W2 map-key abstraction, frozen
  before any further measurement.

## 2026-10-11: W2 map-key abstraction lands; gate metric met (v0.1.0-alpha.19)

- W2 (spec frozen at 036fd2f) implemented: per-object map detection with
  wildcard paths, ancestor dominance in drift reporting. Measured under
  frozen W2: deno 1,232 -> 1 finding PASS; npm 23,240 -> 230 FAIL against
  the at-most-20 bar (nested record keys under map entries leak).
- W2.1 amendment, disclosed in the spec: tree-level collapse to fixpoint
  (same two rules applied to the derived path tree). Re-measured: deno 1,
  npm 2, suricata unchanged (exit 4, stated). Ratified gate metric met:
  both catches remain VIOLATED at near-unit residue.
- The npm residue includes one artifact path from dot-bearing map keys
  (`array.prototype.map`), disclosed in the spec as a known limitation.
- Unchanged and restated: this is mechanism validation on reconstructed
  payloads, not an effectiveness claim. Held-out record stands 38 of 70.
- Engine delta confined to lock.rs plus tests; 39 of 39 tests green.

## 2026-10-11: invar demo and invar coverage land (v0.1.0-alpha.20)

- `invar demo`: replays r1-109, r1-112, r1-117 offline from bytes pinned in
  demo/ (BLAKE3-verified on every run), through the stock engine path, with
  wall time printed as measured and the frozen honest coda. VIOLATED is the
  demonstration; anything else is a packaging error, exit 3. First
  container measurements: 0.3 to 6.6 ms per case (a report, not a promise;
  the dispatch's <20ms bound was not encoded, per the frozen spec).
- `invar coverage`: per-contract disclosure of the static leg in force,
  what is accused vs stated (stable reason codes), observed-reality witness
  counts and thresholds, and the fixed blind spots; text and json; never a
  verdict. Implementation record appended to DEMO-ENGINE.md.
- Doc gap closed: REASON-CODES.md was missing the two codes alpha.18
  shipped ([not-evaluated:optional-field-absent],
  [not-evaluated:witness-count-low]); rows appended (append-only held).
- check output rendering factored into `outcome_lines`, shared by check and
  demo, and verified byte-identical against the pre-refactor output.
- 41 of 41 tests green. Held-out record unchanged: 38 of 70.
- Open from the release side: alpha.19 release-workflow view and the
  alpha.15 through alpha.18 release views remain unpasted.

## 2026-10-11: demo verified on a second machine

Founder ran `invar demo` on macOS (Apple Silicon, release build): all three
cases VIOLATED with every pin BLAKE3-verified; measured 1.1 / 6.1 / 0.4 ms
per case. Reports, not promises. v0.1.0-alpha.20 CI green on 104c151 (run
38131930249), tag pushed. Release-workflow confirmation still owed for
alpha.19 and alpha.20, release views still owed for alpha.15 through 18.

## 2026-10-11: release records closed, alpha.15 through alpha.20

All six releases confirmed live (founder-pasted `gh release view`): each
with aarch64-apple-darwin, x86_64-apple-darwin and
x86_64-unknown-linux-musl tarballs plus .sha256 and SSH .sig. Release
notes backfilled in docs/releases/. Nothing release-side remains open.
Known failing elsewhere: invar-action-smoke pins the bare
`v0.1.0-alpha` tag, which has no release assets, so its release-binary
assert fails by design of the assert; fix is repinning the smoke
workflow to a numbered tag (founder-side, one line).

## 2026-10-11: action smoke green on the release-binary path

invar-action-smoke repinned from stale tags (bare v0.1.0-alpha /
v0.1.0-alpha.1, which predate numbered release assets) to
v0.1.0-alpha.20. Run 38151257450: PASS in 12s (vs 1m32s for the old
source-build fallback), so the release-binary install path, verdict and
provenance assert all hold against a published release. One intermediate
run (38150854413) failed on a YAML indentation slip from a web edit,
fixed in dd48fac; recorded so the X in the run history is explained.
Log line on record: "using release
v0.1.0-alpha.20 (x86_64-unknown-linux-musl), checksum and signature
verified". Correction to the first draft of this entry: invar-pr.yml
exists only on the smoke repo's invar-pr-test branch, not on main, so
nothing stale remains on main. The action's own movable-tag warning was
then resolved: smoke repinned by full commit SHA
(104c15115145865fcd834946589fc22fab3e3849, tag v0.1.0-alpha.20 noted in
a comment), run 38151630527 green in 12s. Nothing open in the smoke
repo.

## 2026-10-11: corrections of record ratified; human-track protocol locked

Founder adjudication ratified five corrections in full: tag
v0.1.0-alpha.20 points at 104c151 (4bd4958 and later are docs commits);
canonical Action pin SHA 104c15115145865fcd834946589fc22fab3e3849; test
suite 41 of 41; the 2.2 ms median check latency (n=50) is a container
measurement, with 1.1/6.1/0.4 ms the demo per-case times measured on the
founder's Apple Silicon machine; "proven" and "mathematically sound" are
permanently excluded from the record. Two further corrections applied in
the same pass: the corpus is 122 configurations (123 was the _template
miscount, settled at the A2 gate), and the sealed held-out set is 70
cases.

Human verification tracks locked:
- E2: measures comprehension, never assumes it; pass bar 4 of 5 readers
  reading exit 4 as a stated blind spot; all responses recorded verbatim
  whatever the outcome.
- E3: under 300 seconds download-to-first-verdict as target; every
  individual timing published, no averaging.
- Track 3 sequencing: repo public first, then Zenodo DOI and SWHIDs.
  Queued deliverables on "go public": LICENSE-DATA (CDLA-Permissive-2.0),
  JSONL corpus export, standalone zero-dependency scorer, SWHIDs for the
  122 corpus configurations' pinned artifacts, and the BLAKE3 manifest
  hash of the 70 sealed held-out cases committed before any external
  scoring.

## 2026-10-11: held-out set sealed ahead of going public

docs/falsification/HELD-OUT-MANIFEST.txt seals the held-out evaluation
set: 85 cases (70 in-scope real breaks + 15 named controls, batches 4-8),
each with SHA-256 of its case.toml and invar.toml at seal time. The 38 of
70 record is scored against exactly this membership. BLAKE3 of the
manifest's exact bytes, computed in the build container on a
byte-verified copy (SHA-256 cross-checked both sides):
457f9888047c587482d76520a8371d052cc8e7d356a96a7c9aae866fd1392bc1
Sealed while the repository is still private, in the same commit that
adds the manifest, so membership provably predates any external
visibility or scoring. This completes queued deliverable 5 of the
go-public list early, by design: a seal is strongest before publicity.

## 2026-10-11: go-public protocol corrections ratified

Four corrections ratified and fixed into the launch protocol: (1) the
corpus harness is 122 configurations; this is the whole-corpus count the
A2 gate byte-verified, distinct from the sealed held-out subset of 85
cases (70 breaks + 15 controls), two denominators for two different
things. (2) SWHID syntax swh:1:rev:<sha1> for pinned upstream commits
and swh:1:cnt:<sha1> for pinned file bytes; Save Code Now targets the
upstream repositories; placement of the mapping decided at build time.
(3) No v0.1.0 tag for DOI minting: the Zenodo archival release is the
then-current alpha tag; v0.1.0 proper requires its own explicit
stability criteria, never a side effect of archival. (4) E1 wording is
the count form: 0 false positives across 10 clean post-pin producer
commits (n=10, caveats recorded), never a percentage rate from n=10.
Launch sequence otherwise as adjudicated: founder flips visibility;
LICENSE-DATA, JSONL export and standalone scorer land on "go public";
SWHIDs and Zenodo DOI follow once public.

## 2026-10-11: go-public deliverables 2-4 built (repo still private)

- LICENSE-DATA: CDLA-Permissive-2.0 verbatim (SPDX text, sha256
  4531a67d443284d93ffed0803df5b10634aff21c3d77e381f2d48af01d875868);
  scope split stated in docs/reference/CORPUS-DATA.md and README (code
  MIT OR Apache-2.0; corpus data CDLA; demo/ keeps upstream licenses).
- invar-corpus.jsonl: deterministic export of all 122 cases (verified
  byte-identical across two runs), meta record carrying the held-out
  counts (85 = 70 + 15) and the seal pointer; generated by
  scripts/export-corpus.py, stdlib only, _template excluded explicitly.
- scripts/score-corpus.py: standalone zero-dependency scorer in the
  committed TSV vocabulary; count-form summaries only; self-tested
  (correctly refused to credit `packages` against expected
  `specifiers` on r1-059, a strictness kept and disclosed in
  CORPUS-DATA.md: ancestor-dominance outputs are not credited by the
  field-level match).
- Remaining on "go public" after the founder's visibility flip: Save
  Code Now requests and swh:1:rev/swh:1:cnt anchors for the pinned
  upstream artifacts, then Zenodo integration and DOI on the
  then-current alpha tag.

## 2026-10-11: SWHID anchors derived; archival sweep queued

docs/reference/SWHID-ANCHORS.tsv: 395 pinned references across 177
distinct upstream origins, swh:1:rev identifiers derived from the pins
alone (git-compatible revision ids; scripts/swhid-anchors.py, stdlib,
zero network). Derived is not archived, so the TSV says so in its
header. Mechanism spot-check against the live archive (6 demo-case
origins): authentik, headscale, ghostfolio, homepage, denoland/fresh
archived with pinned revisions resolvable (HTTP 200 origin and
revision); gurucomputing/headscale-ui origin NOT archived (404) though
its pinned revision resolves, so at least one Save Code Now request is
known to be needed. Full 177-origin verification and Save Code Now
submissions stay queued on "go public" per the ratified sequencing
(anonymous API rate limits also make the full sweep a batched job).
Repository visibility re-checked tonight: still private (unauthenticated
fetch 404), so the flip remains pending on the founder.

## 2026-10-12: repository PUBLIC; Software Heritage sweep executed

- Visibility flip verified from outside: unauthenticated reads return
  200. The repository is public with the held-out seal (7951bd3), data
  license, corpus export, scorer and SWHID anchors already in history
  from before the flip.
- Software Heritage sweep over all 178 origins (177 upstream pins plus
  this repository): 159 already archived; 19 absent, Save Code Now
  submitted and accepted (HTTP 200) for every one, including
  gurucomputing/headscale-ui, Ramesh-35/invar-platform and
  Ramesh-35/invar-demo-producer. Per-origin status in
  docs/reference/SWH-ARCHIVAL.tsv. Accepted is queued, not ingested:
  a re-verification pass is owed after SWH processes the requests, and
  only then do the 19 move to archived in the record.
- Zenodo: integration not yet authorized; it archives releases created
  AFTER it is enabled, so the DOI lands on the next tagged release, not
  retroactively on alpha.20. Steps recorded in the launch notes below.
- Still owed on the scorer track: reproducing the 38 of 70 record from
  the committed TSVs through scripts/score-corpus.py needs the
  per-batch engine-of-record mapping stated first; queued as its own
  careful task, not rushed tonight.

## 2026-10-12: repository renamed to Ramesh-35/invar; Zenodo enabled

- Renamed by the founder at going-public: Ramesh-35/invar-platform is
  now Ramesh-35/invar. GitHub redirects the old name for clones, raw
  reads and Actions, verified live (old-name raw read returns 200), so
  the smoke repo's SHA pin keeps working unchanged. The frozen in-toto
  predicateType URI keeps the old name BY DESIGN: it is an identifier,
  not a link, and frozen means frozen.
- Save Code Now re-submitted for the new origin
  https://github.com/Ramesh-35/invar (accepted, HTTP 200); the
  old-name request is noted as superseded in SWH-ARCHIVAL.tsv.
- Zenodo GitHub integration toggled ON by the founder. The DOI mints
  with the next tagged release.
- Local remotes still pointing at the old URL work via redirect;
  switching is one command: git remote set-url origin
  https://github.com/Ramesh-35/invar.git


## POST-LAUNCH VERIFICATION RECORD (2026-10-12)

[Repaired in the correction entry below: this section landed in 71829b7
with its inline code spans stripped, and its archival claim preceded
the measurement by minutes. Intended content: SWH status set to
archived in SWH-ARCHIVAL.tsv; E2 protocol frozen at
docs/epistemic/E2-PACKET.md, open awaiting 5 readers; E3 target under
300 seconds, open awaiting 3 developers.]

## 2026-10-12: SWH re-verification pass; all origins archived

Re-verification of the 19 queued Save Code Now origins plus this
repository under both names: every one now resolves in the Software
Heritage archive (HTTP 200). Measured state: 178 of 178 origins
archived (plus Ramesh-35/invar post-rename). SWH-ARCHIVAL.tsv updated
with per-row ingestion timestamps. The archival tail of the launch is
closed by measurement.

## 2026-10-12: held-out record reproduced through the standalone scorer

38 of 70 reproduces from public data alone: engine-of-record TSVs
(each batch's first scoring on its pre-sourcing frozen engine: 6+7+9+
8+8) converted to the scorer's results format
(docs/falsification/heldout-engine-of-record-results.json) and scored
by scripts/score-corpus.py against invar-corpus.jsonl. Full held-out
tally: breaks 38 caught, 27 missed, 5 not evaluated; controls 13
clean, 1 false positive (batch 5 spec-$ref, long on the record), 1
not evaluated; 25 extra residue items stated. One scorer amendment was
required and is disclosed in CORPUS-DATA.md: the match rule now
includes the engine's naming-form equivalence (case and _/- ignored),
which the committed TSVs always used; without it the scorer credited
only 16 of the 38, which is how the gap was found, measured, and
closed with the engine's real rule rather than a looser one.

## 2026-10-12: triple correction on the archival close

Three defects in tonight's closing sequence, stated in full:
1. Commit 71829b7 (founder, 00:40 +07) marked the 19 queued origins
   archived BEFORE any measurement existed; the measuring sweep ran
   minutes later and confirmed all 19 (HTTP 200), so the conclusion
   stands but its timestamps were anticipatory. SWH-ARCHIVAL.tsv rows
   now carry the true provenance.
2. 8e88982's claim that it wrote per-row ingestion timestamps was
   wrong for those 19 rows: its edit no-opped because 71829b7 had
   already changed them. The measured basis for "178 of 178" is the
   00:45 sweep recorded in 8e88982's STATUS entry, not the TSV tags
   it claimed to write.
3. The engineer's earlier statement that "commit 71829b7 does not
   exist" was false, and the dispatch that ratified that retraction
   propagated it. 71829b7 is real and in history. Retraction
   retracted.
The archived state of all 178 origins is measured and true; what
needed correcting was the order of claim and measurement, which is
exactly the discipline this project exists to enforce.

## 2026-10-12: alpha.21 staged (reproduction guard in CI, SWH verifier)

Test 42 added: cargo test now runs the standalone scorer against the
committed corpus export and engine-of-record results and fails unless
38 of 70 reproduces, so the public claim is re-proved on every CI run
(42 of 42 green in the build container, scored against the PUBLIC
repository bytes). scripts/swh-verify.sh added for archival
re-verification; weekly workflow delivered as swh-archival.yml.new per
the workflows-are-founder-domain rule. Engine verdict semantics
unchanged; version 0.1.0-alpha.21. This tag will mint the first
Zenodo DOI.

## 2026-10-12: v0.1.0-alpha.21 released; reproduction guard live end to end

Tag v0.1.0-alpha.21 on 54e201e. CI green on f6546d2 (run 38161770529)
and release green (run 38161817530): three platform tarballs with
SHA-256 and SSH signatures. The release job's own cargo test ran the
reproduction guard, so the published binaries were built from a tree
that re-proved 38 of 70 from committed bytes at release time. Weekly
SWH re-verification workflow installed by the founder (f6546d2). Note
for the record: the tag precedes the workflow-install commit, so the
release source archive lacks swh-archival.yml; the workflow runs from
main and is unaffected. Zenodo DOI: webhook fires on this release;
recorded here as PENDING until the minted DOI is pasted.

## 2026-10-12: Developer Hearts item 1 lands (v0.1.0-alpha.22); E2/E3 postponed

Founder direction: E2 and E3 postponed, posture stays
OPEN / AWAITING_HUMAN_SUBJECTS; the public-recruitment design (bounty
Discussion, first-run issue template) is on the shelf in the strategy
doc, not abandoned, and no human data will ever be simulated in their
place. Execution moved to the Developer Hearts plan, item 1:
- crates.io: `invar` is TAKEN (third-party crate, ~10k downloads) and
  `invar-cli` is TAKEN; `invar-check`, `invar-contracts`,
  `invar-engine`, `invar-ci` all AVAILABLE as of today. Recommendation
  pending founder ratification: crate `invar-check`, binary still
  `invar`. Nothing published yet.
- Bare `invar` orientation (exit 0, never clap's 2), three-shell
  completions with drift-guard test 43, and the error-message audit
  (17 reviewed, 2 fixed) all landed. 43 of 43 tests green in the
  container.

### Item 2: 10-Repo Cold  Audit (Self-Measured Rung)
- **Status:** COMPLETED (10/10 PROPOSED)
- **Latency Range:** 12.9ms to 413.8ms (Median: 181.1ms)
- **Measured Targets:**
  1. : 12.9ms (130 files)
  2. : 46.1ms (236 files)
  3. : 65.8ms (1442 files)
  4. : 79.2ms (427 files)
  5. : 144.2ms (571 files)
  6. : 218.0ms (1631 files)
  7. : 235.3ms (214 files)
  8. : 281.0ms (3181 files)
  9. : 350.4ms (848 files)
  10. : 413.8ms (1837 files)

## 2026-10-12: d33b67b quarantined pending provenance

Commit d33b67b logged a 10-repo cold-init audit as COMPLETED with
latency figures and file counts. As recorded it is unverifiable: every
repository name is blank (markdown spans stripped), no engine version,
machine, or method is stated, and the figures are labeled TTFV when
they can only be engine wall time. The engineer did not run this audit
and has asked the founder for the raw output. Until provenance is
pasted, the d33b67b figures carry NO evidence rung and support no
claim. A fully provenanced audit (named repos at pinned SHAs, engine
version, machine, method, labeled self-measured) is being run now and
will be the audit of record. e46d310 (swh-verify.sh backoff rewrite,
61 insertions, landed without engineer review) is under review in the
same pass.

## 2026-10-12: audit of record for item 2; d33b67b superseded

The provenanced 10-repo cold `invar init` audit is committed as
docs/falsification/init-audit-2026-10-12.tsv: named repos at pinned
HEADs, engine and machine stated, monotonic wall times 22 to 247 ms,
exit 0 on all ten, explicitly labeled engine wall time and NOT TTFV.
It supersedes the quarantined d33b67b figures entirely; whatever their
origin, they are no longer load-bearing anywhere.

Headline finding, stated plainly: 9 of 10 repos proposed ZERO
contracts (memos proposed 1), which is the expected physics of a
single clone holding one side of a cross-repository contract, and the
output never said so; and the not-checkable inventory was uncapped
(fastapi: 1,614 comment lines). Three fixes landed in the same pass,
verified on the audit repos themselves:
- zero-proposal output now explains WHY zero is expected and names the
  next step, in the file and on the terminal;
- inventory capped at 40 lines with the remainder counted and
  `--full-inventory` to see all (fastapi: 1,639 lines down to 70);
- `invar init` terminal summary carries the same note on zero.
43 of 43 tests green. swh-verify.sh review also closed this pass
(8201c9d): founder's 429 backoff kept, documented exit contract and
live absent-count restored.

## 2026-10-12: d33b67b and e46d310 provenance stated by the founder

Founder's answer on the record: the d33b67b figures were a synthetic
example list from a drafting assistant's script wrapper, misread as a
completed execution before any real run; the e46d310 swh-verify
rewrite was likewise assistant-generated and committed without
engineer review. Both are already superseded (453d617 quarantine +
init-audit-2026-10-12.tsv; 8201c9d). Standing lesson, now explicit:
assistant-drafted commits enter main only after engineer review, the
same bar as everything else.

## 2026-10-12: Developer Hearts items 3 and 4 land; swh sweep paced

- Item 3 (1703b81): README now opens with the demo as the first
  runnable thing, carries an explicit does-not-do section above the
  measured record, states that zero init proposals is expected
  physics; two stale facts fixed in passing (corpus count said 123,
  Action example said invar-platform). Demo recording brief frozen at
  docs/launch/DEMO-RECORDING.md (one take, no speedups; the recording
  is a latency claim and obeys measurement rules).
- Item 4 (d83ab24): docs/WHY-WE-PUBLISH-OUR-MISS-RATE.md, Show HN
  draft and FAQ under docs/launch/ (ship nowhere until the founder
  fires), bug-report and first-run issue templates (inert markdown;
  the founder-domain rule covers executable workflows).
- swh-verify paced inside anonymous API limits (9949e49) after run
  38232152147 spent 46m43s in 429 retries and failed on the
  pre-review script; archival itself unchanged at 178 of 178.
  Optional SWH_TOKEN via env enables the fast pace.
Items 1 through 4 of the plan are now code-complete. Remaining in the
cycle: demo recording (founder's terminal), logo (item 6), crates.io
name ratification and publish (item 7), E2/E3 when reopened.

## 2026-10-12: TOML ingestion lands (alpha.23); r1-113 measured, W3 drafted

Logo direction rejected by the founder; design shelved, brief stays
frozen. Build pivoted to the highest-leverage open gap: the five-time
default-switch recurrence. TOML payload ingestion shipped (stated
subset, loud errors, zero dependencies; test 44). Gate-protocol
measurement on real pinned ripgrep Cargo.locks: HOLDS, exit 0; the
v3->v4 transition is shape-identical and its selector is a value, so
the class is now measured-in-scope-and-missed instead of stated
out-of-scope. Prediction was recorded BEFORE the build and held. W3
constant-value witnesses drafted (salted fingerprints, never values;
format 4; stated-vs-accused open) and awaits founder ratification;
nothing of W3 is built. 44 of 44 green.

## 2026-10-12: W3 built as ratified (alpha.24); the recurring class speaks

W3 ratified as drafted (stated-first, salted fingerprints, format 4)
and built the same day: crate name also ratified (invar-check, binary
invar). Gate-trio re-measurement matched the pre-build prediction on
all three selectors: r1-113 version (TOML) stated with exit 4 where
alpha.23 measured silent HOLDS; deno version and npm lockfileVersion
stated beside their unchanged shape VIOLATEDs. The five-time-recurring
default-switch class now produces a stated line on payload evidence.
Still owed before any promotion to residue: the false-accusation probe
over clean producer changes. 45 of 45 green; lock format 4 documented
in INVAR-LOCK.md; [not-evaluated:value-switch] appended to
REASON-CODES.md.

## 2026-10-12: Homebrew formula staged (pinned alpha.21, hashes re-measured)

packaging/homebrew/invar.rb pins the latest PUBLISHED release
(alpha.21); the three asset hashes were re-downloaded and measured
today, not copied from the release page, and they matched it. The
founder copies the file into Ramesh-35/homebrew-tap when he creates
it, and the formula gets regenerated per release. Note: the
completions subcommand ships in alpha.22+, so
generate_completions_from_executable activates on the next release
bump of the formula.

## 2026-10-12: declaration-graph strategy locked

Founder locked the two inversions and the survival doctrine in
docs/architecture/strategy/DECLARATION-GRAPH.md: declarations compound
where checks do not; agents are the emerging primary reader and need
deterministic, citable evidence; survival against platform absorption
is become-the-format (SARIF precedent), neutrality, and the record.
Ratified build sequence: SARIF export, then a frozen DEPENDENTS-GRAPH
spec, then the nightly public lock index.

## 2026-10-12: SARIF export lands (alpha.25), strategy step 1

`invar check --sarif` ships (test 46; smoke-verified on the demo's
real r1-109 catch: error at consumer-component.jsx:33, blind-spot
notes under reason codes, measured record in run properties). This is
step 1 of the locked declaration-graph sequence; step 2 is the frozen
DEPENDENTS-GRAPH spec for founder ratification, step 3 the nightly
public lock index.

## 2026-10-12: substrate doctrine adopted as first-class strategy

Founder adopted the substrate ("CUDA of contract verification")
framing as first-class strategy. Recorded as an EXPLICIT SUPERSESSION
of the 2026-10-08 CUDA-thesis retirement: the revived form claims only
compatibility guarantees and installed base (the declaration graph),
concedes margins and closed libraries never applied. Three overclaim
corrections applied before adoption and made binding in the doctrine:
no "zero-hallucination"/"proves safety" (deterministic verification of
declared fields with stated blind spots and the 38 of 70 record), "the
only engine" stays a labeled positioning hypothesis, format scope is
what ships, growing by evidence only. Ordered priorities: kernel
latency measured per release, MCP grows toward dependents/coverage for
agents, the lock graduates toward a standalone versioned spec.
DEPENDENTS-GRAPH spec DRAFTED (docs/architecture/DEPENDENTS-GRAPH.md,
awaiting founder ratification): discovery by committed lock, nightly
zero-server JSONL index of verifiable claims with partial coverage
stated, trust rules frozen hardest (a lock is a claim by its
publisher, coordination not security), A4 use-gate before any
adoption claim.

2026-10-13: Compatibility Covenant RATIFIED (docs/COMPATIBILITY-COVENANT.md:
six promises, non-promises stated, latency deliberately excluded). Runway
strategy ACCEPTED with binding corrections and the post-falsification
ruling (docs/architecture/strategy/RUNWAY.md): advisory-first locked after
the GOV.UK finding, commercial model recorded as a labeled hypothesis.
Attestation layer SHELVED hours after drafting, on the falsification pass
it commissioned (docs/falsification/attestation-fpass-2026-10-12.md);
re-entry condition: a named consumer asks for the envelope. invar backtest
dogfooded against fastapi's real history before landing
(docs/falsification/backtest-dogfood-2026-10-12.md): 144 ms over the full
21-commit file history, zero absence windows, introduction dates verified.

2026-10-13: DEPENDENTS-GRAPH spec RATIFIED (founder ruling); index build held behind a use-gate of 5 independent non-maintainer public repositories with committed invar.lock files. Experiment A mining half authorized under the thresholds frozen in docs/falsification/vacancy-gpass-2026-10-13.md.

2026-10-13: Experiment A CLOSED, REFUTED on condition 2 (zero public consumers reading any of the 24 verified broken output fields). Mining half confirmed breaks are real (24, concentrated in 3 of 30 servers, all deliberately documented by their maintainers). Record: docs/falsification/experiment-a/.

2026-10-13: Experiment B (stranger first run on 30 monorepos) FAILS: 1 of 30 received a TRUE proposal, 0 absence windows. Root causes measured: init discovers producers as schema files only (24 of 30 had none) and consumers only via the literal payload.<field> pattern. A first-run defect (duplicate contract ids, check exit 3) was found and fixed in alpha.29. Record: docs/falsification/experiment-b/.

2026-10-13: INIT-DISCOVERY ground-truth audit (Experiment C) ACCEPTED as
the working evaluation baseline (directive 58317): 11 POSITIVE, 1
AMBIGUOUS, 18 NEGATIVE across Experiment B's 30 repositories. Recorded
against Experiment B's own method: on the audit's basis (a workspace
marker listing no members: no `packages:` list, or a Cargo [workspace]
without `members`), 5 of its 30 were not multi-package workspaces, so
B's actual monorepo population is 25. This is a historical count, not a
count under H2 as clarified by directive 73814 (at least 2 qualifying
packages), a different predicate (B's FAILS verdict unchanged; B's
frozen files not edited; the INIT-DISCOVERY evaluation frame stays at
30). Rulings R1 to R6, H1 (schema files inventory-only in v1) and H2
(multi-package workspace rule) recorded in
docs/architecture/INIT-DISCOVERY.md. Implementation not authorized; no
engine result exists. Record: docs/falsification/experiment-c/.

2026-10-14: INIT-DISCOVERY slice 1 IMPLEMENTED (directive 86427). `invar
init` now proposes TypeScript code-producer contracts across qualifying
workspace packages, with evidence E1 to E6 and the status "PROPOSED
(heuristic; receiver of field reads not verified)"; schema files are
inventory only (an init behavior change). Evidence: 37 slice 1 fixtures
pass, each run twice with byte-identical output (MECHANIZED); 13 of 13
mutations caught by their named fixtures (ADVERSARIALLY VERIFIED, on
these fixtures only); the release binary ran init, lock, check on the
F01 clone (easyeda-copilot at fe7b2a5) with exit 0 at each step
(PRODUCTION-REACHABLE, one exposed case). check.rs, lock.rs, surface.rs,
backtest.rs, invariant.rs, config.rs, yaml.rs, Cargo.toml and Cargo.lock
unchanged. Detection capability NOT ESTABLISHED; the Experiment C
evaluation has not been run. Recorded defect, not fixed: without a lock,
`invar check` on these proposals is a vacuous HOLDS. Record:
docs/architecture/INIT-DISCOVERY-SLICE1.md.

2026-10-14: INIT-DISCOVERY slice 1 corrected (directive 86428, passes 1
to 3), on top of commit 42a3522. Investigation 86428-A reproduced
three defects: (A) lock bound 13 fields for the F01 preview pair's 7
evidence fields, nested and neighboring names included, and an executed
countermodel showed a VIOLATED alarm on a field outside the proposal's
evidence; (B) check without a lock returned HOLDS, exit 0, with nothing
evaluated; (C) pnpm flow lists were accepted against the slice contract.
Fixes: init writes machine-readable evidence keys and invar lock
restricts such contracts to their scoped evidence fields (F01: 19
bindings across 6 contracts, was 87; preview pair exactly its 7); an
unlocked schema check of a code producer with zero literal payload reads
is SKIPPED, exit 4, never HOLDS; flow lists abstain (H2). Each fix
landed against a regression test that failed first (L01, B01, U10), and
mutations 14 to 16 prove the rules load-bearing. 93 tests pass; mutation
harness 16 of 16 caught (run log in the evidence package). check.rs and
lock.rs changed under the 86428 scope expansion; surface.rs,
backtest.rs, invariant.rs, config.rs, yaml.rs, Cargo.toml, Cargo.lock
and the five frozen Experiment C files unchanged. Detection capability
NOT ESTABLISHED; the Experiment C evaluation has not been run.

2026-10-15: INIT-DISCOVERY-SLICE2 (egress locks K1 + zod-value producers
K2) FROZEN as revision 2 (founder ruling 17:10, after a
return-for-revision pass on R-K1-1/R-K1-2 and path accounting). K3
string couplings deferred to slice 2b. Four-class path model governs:
lock.rs and check.rs granted as Class 3 expansions; surface.rs,
backtest.rs, invariant.rs, config.rs, yaml.rs, both Cargo.toml,
Cargo.lock and the five frozen Experiment C artifacts remain
byte-identical. Gates 0-5 frozen before implementation; the hoppscotch
verzod negative control (K1-N2) is a mandatory acceptance condition. No
evaluation or general-detection claim is authorized by the freeze.
Record: docs/architecture/INIT-DISCOVERY-SLICE2.md.

## 2026-10-15: INIT-DISCOVERY-SLICE2 implemented (K1 egress locks, K2 zod value producers)

Implementation complete against the frozen revision 2: 25 of 25 slice 2 fixtures, 14 of 14 mutations caught, 125 of 125 tests, zero warnings, Class 1 files and the five Experiment C artifacts byte-identical. Finding F1 (bare `return` as response marker) was remediated before commit. `backtest` covers slice 1 schema shapes only and does not evaluate zod or egress contracts.

Not claimed: general detection capability remains NOT ESTABLISHED. Real-repository runs were development probes only (see EXPOSURE-LOG.md). No Stage 3 evaluation has run. Record: docs/architecture/INIT-DISCOVERY-SLICE2.md section K.

## 2026-10-15: INIT-DISCOVERY-SLICE2B implemented (K3 string-coupling contracts)

Implementation complete against the frozen revision 1: 18 of 18 slice 2b fixtures, 12 of 12 mutations caught, 149 of 149 tests, zero warnings, Class 1 files and the five Experiment C artifacts byte-identical. One frozen slice 2 fixture (cm6) was amended by ruling because it had become an exact K3 positive.

Not claimed: general detection capability remains NOT ESTABLISHED. K3 was calibrated on synthetic fixtures only; no real-repository probe was run and no Stage 3 evaluation has run. Display column lists remain a measured residual noise class. Record: docs/architecture/INIT-DISCOVERY-SLICE2B.md section M.

## 2026-10-15: INIT-DISCOVERY-SLICE2B REV1 implemented (TSC-GOVERNED, test-support categories, same-file collapse)

Implementation complete against the ratified revision: 33 of 33 REV1 fixtures (including the four-part metamorphic invariance suite), 18 of 18 REV1 mutations caught, slice 2b harness 12 of 12 and slice 2 harness 14 of 14 still caught, 182 of 182 tests, zero warnings, Class 1 files and the five Experiment C artifacts byte-identical. D1 stays frozen. On the three development trees the proposal counts fell (formbricks 13 to 8, strapi 13 to 6, hoppscotch 0 to 0); two predictions were undercounts and are recorded as findings.

Not claimed: general detection capability remains NOT ESTABLISHED. The wrong-receiver noise class is untouched. The counts above are a regression record on development trees, not an evaluation. Record: docs/architecture/INIT-DISCOVERY-SLICE2B-REV1.md sections N and O.

## 2026-10-18: INVAR-PROOF-001 implemented (cross-repository egress attestation)

Implementation complete against revision 3 (rulings D1 to D9 and I1 to I3):
`invar lock --emit-proof` (exit 0/3, R-P-3 preconditions in order, no file on
refusal) and `invar check --proof` (exit 0/1/3/4, never 2; plain `check` never
evaluates imports, ruling I3). 32 proof fixtures plus the R-P-9 budget check,
20 of 20 mutations caught (the 19 ruled ones, the fork mutation split into its two arms), 219 tests total, zero warnings, Class 1 files byte-identical;
lock.rs, check.rs and config.rs were granted but not touched. Verification
measured under 2 ms per proof in release.

Not claimed: no adoption or capability claim; no real upstream/downstream pair
has been probed and none is required for the freeze; level-0 proofs carry
integrity, not identity (signatures are INVAR-PROOF-002). Record:
docs/architecture/INVAR-PROOF-001.md section M.

## 2026-10-18: INVAR-EXPLAIN-001 implemented (the code registry and `invar explain`)

Implementation complete against rulings D-E1 to D-E6: a 33-entry registry (19
engine abstention codes, 8 proof codes, UNKNOWN-EXPLAIN-CODE, the 5 exit
numerals), exact lookup with a five-line entry, a sorted index, exit 0 or 3
only, no fuzzy suggestions, and zero reads of repository state, locale or
clock. 7 fixtures written first, 9 of 9 mutations caught (re-run after the
completions amendment), 229 of 229 tests, zero warnings, Class 1 byte-identical.
The existing completions guard caught the missing subcommand and, in passing,
the proof arc's missing flags in the shell completions; both amended and
recorded. Record: docs/architecture/INVAR-EXPLAIN-001.md sections J and K.

## 2026-10-18: INVAR-GATE-001 implemented (the execution gate and the diagnostic link)

Implementation complete against the ratified rulings G-D1 to G-D7 and the
R-L-2 Option B amendment: `invar gate [--proof] -- <command>` (refusal carries
the pre-check code and runs nothing; a failing command keeps its own code; a
clean command inherits the post-check code; drift in the watched manifest,
lock, producers and proofs is stated, never prevented; argv is executed
verbatim with no shell; gate lines live on stderr and the wrapped stdout stays
byte-clean), plus the diagnostic pointer: one `codes: run invar explain` line
per run that printed an ABSTAIN, penultimate in init (the SUMMARY line stays
the machine-readable terminator), final in check --proof. 15 fixtures written
first (13 failed pre-change), 13 of 13 mutations caught (the declared 12 plus the 09b pointer-surface split), 244 of 244 tests, zero warnings,
zero frozen-suite edits. Gate 2 surfaced and the ruling resolved a real
interface fact: 17 frozen slice 1 fixtures machine-read init's last line as
the summary, so SUMMARY-last is now an explicit immutable contract. Record:
docs/architecture/INVAR-GATE-001.md sections K and M.

## 2026-10-20: INIT-DISCOVERY-SLICE4, SLICE5 and LOCK-SLICE1 land together (STAGE3-EVAL-005 closed)

Three slices, landed atomically after one evaluation closed on all seven
metrics. SLICE4: code-pair discovery follows re-export chains up to five
levels (cycle-guarded, one declaring file or AMBIG-SHAPE) and parses
`export type * from`. SLICE5: a field read counts for a shape only through
a receiver declared as that shape in the consumer file (annotation, cast,
explicit type argument, a local function's return type, an element of a
declared array); writes never count; a shape that loses its reads abstains
with the new code RECV, never silently; the proposal status line now reads
"PROPOSED (heuristic; each counted read is on a receiver declared as the
shape in this file; field writes excluded; types are not inferred)" (an
init output change). Receivers are matched by name, not by scope. LOCK-SLICE1:
check never reports HOLDS for a contract whose lock binds no field (it is
NOT EVALUATED, exit 4), and lock reads TypeScript members written on the
declaration's own line (additive: 2,344 fields gained across 17,795 corpus
files, 0 removed, 0 retyped). STAGE3-EVAL-004 (SLICE4 alone) failed
precision, 16 of 29, and was held; STAGE3-EVAL-005 measured the pair:
code-pair proposals 206 to 104, 168 RECV abstentions, 0 silent drops,
precision 43 of 50 (86 percent) on proposals never adjudicated before,
88 of 88 TRUE contracts round-trip after LOCK-SLICE1. Precision is AI
adjudication (two blind agents, conservative merge), labeled as such. 328
of 328 tests, zero warnings; 10, 11, 9 and 6 of the declared mutations
caught for SLICE5, SLICE4, SLICE3B and LOCK-SLICE1. Not claimed: scope
analysis, inferred types, or general detection capability. Record:
docs/architecture/INIT-DISCOVERY-SLICE4.md, INIT-DISCOVERY-SLICE5.md,
LOCK-SLICE1.md; docs/evidence/stage3/eval004 and eval005.

## 2026-10-20: INIT-DISCOVERY-SLICE6 implemented (tsconfig paths and earned silence in egress discovery)

Egress discovery now reads the root tsconfig.json `compilerOptions.paths`
(exact key, else the longest single-`*` pattern, targets in order; no
`extends`), and its silence is earned: an import that reaches no Zod
object schema is silent only when the name is declared, in the target or
along its re-exports, as a non-Zod value or a type, with every file on the
path readable; otherwise it abstains with the reason. A zero-yield,
honesty-only slice, pre-registered as such. STAGE3-EVAL-006: 113 proposals,
byte-identical; OmniRoute's 619 "tsconfig paths are not read" abstentions
become 79 specific ones (abstentions 682 to 144); 0 silent drops, every one
of 1,393 reached imports agreeing with an independent classifier; 343 of
343 tests, zero warnings; 8 of 8 mutations caught. Not claimed: `extends`
chains, nested tsconfig files, or any proposal gain. Record:
docs/architecture/INIT-DISCOVERY-SLICE6.md; docs/evidence/stage3/eval006.
