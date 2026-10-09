# R1: Real-Tier Falsification Corpus

Critical path of the locked roadmap (docs/architecture/ROADMAP.md). Its job
is to find out, on real breaks, whether INVAR catches anything the existing
free tools miss. A result where INVAR adds nothing is a valid, publishable
outcome.

## Status

| Item | State |
|---|---|
| Case format and runner (`corpus-real/`, `scripts/r1-run.sh`) | built; all six outcomes exercised on test cases |
| Real cases admitted | **30 breaks + 6 controls, of 30** (batch 1: 5 + 1; batch 2: 10 + 2; batch 3: 15 + 3) |
| Baselines run | openapi-diff 0.24.1 (npm). oasdiff not run: its release download is blocked here. buf, schema registry, Semgrep, AI reviewer: not run |

`r1-000` is REAL-INCIDENT-001, a constructed demo kept as a harness
self-test. It is marked `real = "false"` and never counts.

## Admission rules (a case enters only if all hold)

1. **Provenance first.** A public source (issue, PR, postmortem or commit)
   shows that a consumer actually broke because a producer changed. Link it in
   `source`; quote the decisive line in `expected.evidence`.
2. **Pinned.** Producer and consumer are pinned to full commit SHAs in public
   repositories, so anyone can replay the case.
3. **Expected residue comes from the source, never from INVAR.** Fill
   `expected.residue` before running INVAR. Do not edit it afterwards to match.
4. **Cases INVAR will likely miss are admitted on equal terms.** Do not select
   cases because they suit INVAR's patterns.
5. **Controls.** Include real producer changes that did *not* break the
   consumer (`residue = "none"`), at least one control per five breaks, to
   measure false positives.
6. **No manufacturing.** No edits to either side. If a minimal reproduction is
   needed, it lives in a separate case marked `real = "false"`.

## Outcomes (scripts/r1-run.sh)

| Outcome | Meaning |
|---|---|
| CAUGHT | every expected field is in INVAR's residue |
| PARTIAL | some found, some missed |
| MISSED | none found, or INVAR said the contract holds on a real break |
| CLEAN | control case, INVAR reported nothing |
| FALSE_POSITIVE | control case, INVAR reported residue |
| ERROR | INVAR could not evaluate (exit 3); never counted as a verdict |

Fields INVAR reports beyond the expected set are listed in `extra`, not
hidden.

## Baselines

Each case records, per tool, `caught`, `missed`, `not_applicable` or
`not_run`, filled by running the tool on the same pinned commits: oasdiff or
openapi-diff (OpenAPI), `buf breaking` at the matching category (Protobuf),
schema-registry compatibility checks (Avro, JSON Schema), Semgrep, and one AI
reviewer as a non-deterministic reference. The runner copies these values; it
never infers them.

## Falsification thresholds (locked)

With 30 or more real cases: the category claim fails if the baselines together
catch 90% or more of what INVAR catches, or if INVAR misses more than 20% of
true breaks.

## Known limit, stated before any case runs

Schema and event contracts find consumer dependencies only as
`payload.<field>` reads. Real consumers name things differently
(`order.amount_cents`, `data["amount_cents"]`, typed models), so INVAR is
expected to MISS many real cases as it stands. That is what R1 is for; misses
will be recorded and will decide whether the consumer side needs a better
extractor before anything else is built.

## Batch 1 results (2026-10-08)

Cases sourced by two research agents from public git history and issue
pages, then re-checked here: every producer and consumer file fetched at its
pinned commit, every claimed field change confirmed by parsing the specs, and
every consumer read confirmed by grep. INVAR binary: `0.1.0-alpha.2` code.

| Case | Break (from source) | INVAR | openapi-diff |
|---|---|---|---|
| r1-001 Immich to immich-go | `duration` string to integer | MISSED | error (spec failed its validator) |
| r1-002 Firefly III to Waterfly III | `account_id`, `account_name` removed | ERROR (YAML spec) | partial (flagged request bodies, not the response fields) |
| r1-003 Docker to Checkmk | `VirtualSize` omitted | ERROR (YAML spec) | missed |
| r1-004 Docker to Portainer | `ContainerConfig`, `Container` removed | ERROR (YAML spec) | missed |
| r1-005 CometBFT to cosmjs | `begin/end_block_events` replaced | ERROR (YAML spec) | missed |
| r1-006 control: Immich adds `createdAt` | none | CLEAN (vacuous) | error |

**Score on real breaks: INVAR caught 0 of 5. openapi-diff caught 0 of 5
outright (1 partial).** Five cases are far too few to generalize, and the
cases come from the repositories the agents chose to mine, so selection bias
is likely. They are still the first real-tier evidence, and they point the
same way.

### Findings

1. **Format.** 4 of 5 producers publish YAML; INVAR reads only JSON, so it
   could not evaluate them (ERROR, never a verdict).
2. **Consumer extraction.** None of the five consumers reads fields as
   `payload.<field>`: a Go struct tag (`json:"duration"`), Dart
   `json['account_id']`, Python `image["VirtualSize"]`, and JS/TS
   `data.ContainerConfig`, `data.begin_block_events`. Even with YAML support,
   INVAR as built would find none of them.
3. **Type changes are not modelled.** r1-001 is a type change, which a
   presence-only check cannot see.
4. **The published spec lagged the break in 3 of 5 cases** (r1-003, r1-004,
   r1-005): at the breaking commit the producer's spec still declared the
   removed fields; in two cases the spec file was byte-identical before and
   after. No tool that diffs or reads the published spec (INVAR, openapi-diff,
   oasdiff) can catch these from the spec. The truth was in the producer's
   implementation (Go struct tags, route code). This bears directly on R2's
   falsification test, which assumes a usable producer surface.
5. **Defect found and fixed.** A non-JSON producer without a `pointer` was read
   as "defines nothing": HOLDS when no reads were recognised, invented residue
   otherwise. It is now an ERROR (test 25).
6. **The control is vacuous.** INVAR reported nothing because it recognised no
   reads in the consumer, not because it reasoned about compatibility.

### What this means (for decision, not decided here)

On this batch INVAR misses every real break, above the locked 20% threshold,
but the threshold applies at 30 cases. The evidence says the next work is not
R2: it is (a) reading YAML producers, (b) recognising real consumer read
styles (`x.field`, `x["field"]`, struct field tags), (c) modelling type
changes, and (d) deciding whether INVAR should read the producer's
implementation, not only its published spec. Each needs founder direction.

## Engine overhaul (tuned on batch 1 only)

Founder directive 2026-10-08: YAML specs, broad consumer reads, type
changes, dual-source producers (published spec plus producer code). Built
and tuned against batch 1, then frozen at commit `0804555` (engine
fingerprint `6e973b22dfaa18b1`) before batch 2 was run.

Disclosure: the batch 2 case descriptions were read before the engine was
written (the sourcing agents returned early). The engine design was fixed
from batch 1 and the directive's list before any code, and no rule was
added for a batch-2-only pattern. Examples of patterns deliberately NOT
added: case-insensitive or camelCase/snake_case name matching, Go struct
fields without tags on the consumer side, JS producers, structural (root
type) changes. A reader can check this against the commit.

Batch 1 after tuning (not a score, the engine was fitted to it): 3 of 5
caught (r1-001 as a type change, r1-002, r1-005 via producer code), 2 missed
(r1-003, r1-004: the fields stayed in both spec and code and were dropped
only at runtime), control clean. r1-005 also flags `tx_result` and
`txs_results`, whose Go type names changed; a Go type-name change is not
necessarily a JSON change, so these are possible false alarms.

## Batch 2 results (held out, frozen engine)

| Case | Break | INVAR | Note |
|---|---|---|---|
| r1-007 Mealie to aiomealie | `isFood`, `disableAmount` removed | MISSED | flagged `is_food` at the right three lines (the producer's Python name; the consumer's alias is `isFood`); `disableAmount` lives in a different producer file. Scored MISSED by the exact-name rule |
| r1-008 Frigate to HA integration | `temperatures` moved | MISSED | producer builds the dict in code, no declared fields |
| r1-009 authentik to terraform provider | `auth`, `auth_password` write-only | ERROR | the 1.8 MB spec exceeds INVAR's 1 MB fetch limit |
| r1-010 RomM to homepage | `FILESIZE` renamed | **CAUGHT** | producer TypedDict |
| r1-011 evcc to homepage | `gridPower` nested | MISSED | key names are Go constants, not struct fields |
| r1-012 cargo-audit to audit-check | `advisory` restructured | **CAUGHT** | producer Rust struct |
| r1-013 podman to toolbox | `Created` string to int64 | **CAUGHT** | type change from producer Go struct |
| r1-014 Trivy to DefectDojo | root array to object | MISSED | structural change, no field changed |
| r1-015 gitleaks v8 to DefectDojo | 12 fields renamed | **CAUGHT** | all 12, no extras |
| r1-016 npm 7 to audit-ci | `advisories` replaced | ERROR | producer is JavaScript, not readable |
| r1-017 control, RomM adds fields | none | CLEAN | |
| r1-018 control, gitleaks adds `Link` | none | CLEAN | |

**Held-out score: INVAR caught 4 of 10 real breaks (2 errors, 4 misses);
both controls clean, no extra residue anywhere in batch 2.** openapi-diff
was applicable to one case and was killed on the 1.8 MB spec; 8 of the 10
producers publish no usable JSON or YAML spec for the payload, so no
spec-diff tool could have run on them.

Still 15 real cases of 30, so the locked thresholds do not apply yet; on
this batch INVAR misses 60% (including errors), well above the 20% bar.

### What batch 2 says about the next engine version (to be scored on batch 3)
- Name forms: producer `is_food` vs consumer alias `isFood` (r1-007).
- Size: specs over 1 MB (r1-009) need a higher or configurable limit.
- Producers that build payloads dynamically (dicts, Go constants, JS) are
  invisible to declared-field extraction (r1-008, r1-011, r1-016).
- Structural changes (root type) are outside a field-set model (r1-014).

## Engine alpha.4 (tuned on batches 1 and 2)

Founder directive 2026-10-08: naming-form matching (case and `_`/`-`
ignored), payloads assembled at runtime in producer code (dict/map/object
literal keys, keyed assignments; JS/TS added as a producer language), and a
configurable remote size limit (default 5 MB, `[check] max_remote_mb`, 1 to
50). Frozen on branch `engine-alpha4`, commit `8f331ca`, fingerprint
`88844f1512e83e06`, before batch 3 was sourced. The runner now compares names
by the same naming form, for every batch.

Rule fixed before batch 3 was run: a producer code file is used only if its
language is supported (Go, Python, JS/TS, Rust); otherwise the case uses the
published spec alone, or is an ERROR if there is none. Producers in PHP, C,
C++ and C# therefore count against INVAR.

## Batch 3 results (held out, frozen alpha.4 engine)

| Case | Break | INVAR | openapi-diff |
|---|---|---|---|
| r1-019 Grocy to pygrocy | `qu_factor_purchase_to_stock` removed | **CAUGHT** (spec) | error |
| r1-020 Syncthing to Syncthing Tray | browse `type` string to int | MISSED (marshaller change; C++ consumer) | not applicable |
| r1-021 Memos to MoeMemos | `openId` removed | MISSED (Swift `let` field not recognised) | **caught** |
| r1-022 Uptime Kuma to uptime-kuma-api | `version` dropped from event | **CAUGHT** (JS producer) | not applicable |
| r1-023 Syncthing to syncthing-android | `pendingDevices`, `pendingFolders` | PARTIAL (`pendingFolders` is in another producer file) | not applicable |
| r1-024 fortitude to nvim-lint | `row` to `line` | MISSED (Rust payload built without declared fields) | not applicable |
| r1-025 sqlfluff 3 to ALE | `line_no`, `line_pos` renamed | MISSED (a same-named function parameter masked the removed dict key) | not applicable |
| r1-026 ansible-lint to ALE | `lines` to `positions` | MISSED (key still emitted in some cases) | not applicable |
| r1-027 Gitea go-sdk to Drone | `base_branch`, `head_branch` removed | **CAUGHT**, plus 3 extras (`labels`, `mergeable`, `state`: type collisions between structs in one file, likely false alarms) | not applicable |
| r1-028 proselint 0.16 to flycheck | envelope and keys reshaped | PARTIAL (found `errors`, `start`, `end`, `severity`; missed `data`, `check`; extras `line`, `column`) | not applicable |
| r1-029 LND to RTL | `chains` strings to objects | **CAUGHT** (type change in spec) | **caught** |
| r1-030 Core Lightning to lnbits | msat fields removed | MISSED (spec never defined them; producer is C) | missed |
| r1-031 Bitcoin Core to btc-rpc-explorer | `warnings` string to array | ERROR (producer is C++) | not applicable |
| r1-032 Sonarr v4 to Bazarr | `languageProfileId` removed | MISSED (spec not regenerated; producer is C#) | missed |
| r1-033 Bitcoin Core to btc-rpc-explorer | `fee` moved under `fees` | ERROR (producer is C++) | not applicable |
| r1-034 control, Grocy adds `battery` | none | CLEAN | error |
| r1-035 control, sqlfluff adds `name` | none | CLEAN | not applicable |
| r1-036 control, Bitcoin Core adds `fees` | none | ERROR (C++) | not applicable |

**Held-out score: INVAR caught 4 of 15 real breaks outright, 2 partially;
7 missed, 2 errors.** Controls: 2 clean, 1 error. openapi-diff, where it
could run: caught 2 (one of them, r1-021, a case INVAR missed), missed 2,
errored on 2.

## R1 verdict at 30 real cases (2026-10-08)

The locked falsification test (ROADMAP R1): with 30 or more real cases, the
category claim fails if the baselines catch 90% or more of what INVAR
catches, **or if INVAR misses more than 20% of true breaks**.

| Measure | Caught | Not caught |
|---|---|---|
| Held-out only (each batch scored by the engine frozen before it) | 8 of 30 (+2 partial) | 20 to 22 of 30 (67% to 73%) |
| Current alpha.4 engine on all 30 (batches 1 and 2 were tuning data) | 13 of 30 (+3 partial) | 14 to 17 of 30 (47% to 57%) |

**Verdict: FALSIFIED on the miss-rate criterion.** On every reading, INVAR
misses far more than 20% of real cross-repository breaks. The claim that
INVAR, as built, catches real cross-repository contract breaks reliably does
not hold.

The baseline criterion does **not** trigger: of INVAR's 4 held-out catches in
batch 3, openapi-diff caught 1; across batches 2 and 3, most INVAR catches
came from producer code where no spec-diff tool can run. That is evidence of
a real, distinct residue, just not a reliable one.

Why it misses, across all 30 (each is a separate fact, not a ranking):
- Producers in languages INVAR cannot read (PHP, C, C++, C#).
- Breaks that change behaviour, not declared shape (fields still declared,
  omitted at runtime or by a custom marshaller).
- Consumer languages and idioms outside the patterns (Swift, C++, Elisp
  `let-alist`, Lua via undeclared fields).
- File-wide field sets: names collide across structs (false extras) or a
  same-named parameter masks a removed key.
- Structural changes (root type, envelopes) outside a field-set model.

Adequacy is unchanged by this verdict (D PARTIAL, overall NOT ESTABLISHED);
the decision on what INVAR claims and builds next belongs to the founder.


## Batch 4: fresh in-scope benchmark (held out, frozen alpha.4 engine)

Founder directive 2026-10-08: before R2, score the frozen alpha.4 engine on
10 to 15 unseen real breaks with both sides in Go, Python, JS/TS or Rust.

Engine: binary `invar 0.1.0-alpha.4` built from the alpha.4 sources (commit
`0dd1361`), unchanged for this batch. Container binary sha256
`2d5d79e35246b28c...8115`, concatenated `src/*.rs` sha256 prefix
`3d7a023adc247ee5`, stamped 2026-10-08T12:46:57Z before the first run.
Run with `GITHUB_TOKEN` unset so fetches use the public raw endpoint (the
session token is not valid for these repositories); this changes transport
only.

Rules fixed before running, applied to all 18 candidates the sourcing agents
returned (15 breaks, 3 controls):
1. The producer change must be visible in the pinned producer files, and the
   consumer file at its pin must read the field.
2. There must be public evidence the consumer was affected: a fix commit, or
   the consumer's later code reading the new shape. This is the template's
   existing `source` requirement.
3. Use the published spec when the producer has one at both pins, plus one
   supported code file. When the code moved between pins, each pin uses its
   own file.
4. Request-side and behavioural breaks are admitted and labelled.

Five candidates failed rule 1 or 2 and are **not scored**:

| Candidate | Reason excluded | INVAR (unscored) |
|---|---|---|
| Ruff `fix` fields to ruff-lsp | break plausible, no evidence the consumer was affected | flagged `content`, `fix` |
| Rspack stats defaults to webpack-bundle-analyzer | behavioural; no evidence found | flagged `assets`, `chunks`, `entrypoints`, `modules` |
| Lighthouse SSE `msg` to Siren | Siren's later code still reads top-level `msg`; no evidence | HOLDS (would be MISSED) |
| Tekton `resourcetemplates` case rename to tektoncd/cli | no evidence; cli reads the Go field, not the JSON key | flagged `resourcetemplates` |
| Argo `deleteDelayDuration` to Hera | wire format unchanged (`metav1.Duration` marshals to a string) | flagged `deleteDelayDuration` (likely a false alarm on the wire) |

The exclusions went against INVAR: it flagged 4 of the 5. Sensitivity
below shows the verdict does not depend on them.

| Case | Break (from source) | INVAR | openapi-diff |
|---|---|---|---|
| r1-037 Lemmy to pythorhead | `post_id` dropped from MarkPostAsRead (request side) | MISSED (consumer writes the key; INVAR reads only reads) | not applicable |
| r1-038 headscale to Headplane | `forcedTags`, `validTags` replaced by `tags` | MISSED (consumer reads by destructuring `({ validTags, forcedTags })`) | partial |
| r1-039 Zulip to zulip-terminal | `email_address` removed from subscriptions | **CAUGHT** | missed (listed as non-breaking) |
| r1-040 Zulip to zulip-terminal | edit limit integer to nullable | MISSED (nullable not treated as a type change) | missed |
| r1-041 Zulip to zulip-terminal | peer events `stream_id`, `user_id` pluralised | **CAUGHT** | error |
| r1-042 Cargo to cargo_metadata | `debuginfo` u32 to enum | **CAUGHT** | not applicable |
| r1-043 rustdoc JSON to cargo-public-api | `attrs` strings to structured | **CAUGHT** | not applicable |
| r1-044 bundle-tracker to django-webpack-loader | `name`, `path`, `publicPath` moved from chunks to assets | MISSED (names still emitted elsewhere in the file) | not applicable |
| r1-045 Erigon to Otterscan | `totalDifficulty` removed | **CAUGHT** | not applicable |
| r1-046 Harbor to harbor_exporter | `storage` object to array | **CAUGHT** as a removal, plus extras `free`, `total` (false: they moved into generated models, still on the wire) | not applicable |
| r1-047 control, headscale adds user fields | none | CLEAN | clean |
| r1-048 control, cargo adds `workspace_default_members` | none | CLEAN | not applicable |
| r1-049 control, Harbor adds `current_time` | none | CLEAN | error |

**In-scope held-out score: INVAR caught 6 of 10 real breaks, 0 partial,
4 missed, 0 errors, 0 skipped. Miss rate 40%. Controls: 3 of 3 clean.**
95% Wilson interval on the catch rate: 31% to 83% (n = 10, wide).
openapi-diff, where a spec existed (4 breaks): 0 caught, 1 partial, 2
missed, 1 error.

Sensitivity, all 15 candidates admitted and scored from their sources: 8
caught, 2 partial, 5 missed: miss rate 33% to 47%. Either way above the 20%
bar.

Reading: inside the supported languages INVAR does better than across the
full corpus (40% missed here against 67% to 73% held-out before), but it
still misses more than twice the locked bar. The four misses are each a
separate mechanism: request-side writes, destructuring reads, nullability,
and fields that move but keep their names. R1 stays FALSIFIED on the
miss-rate criterion; this batch is evidence about the in-scope rate, not a
reversal. Results: `r1-batch4-results.tsv`.

## Batch 5: held out, in scope (2026-10-09)

Sourced after v0.1.0-alpha.5 was published and after the A1 spec was frozen
(A1-SPEC.md, sha256 88133c41...0ebe5, 2026-10-09T07:58:08Z); batch 5 cases
were first read after that time. Same admission rules as batch 4. All 18
candidates returned by three sourcing agents passed rules 1 and 2 (pinned
files fetched, change and consumer read confirmed); none was excluded.
Concentration: AdGuard Home appears in 3 cases, zwave-js-server in 2.

Scored first on the frozen alpha.5 engine (the released code), then on the
A1 engine built only from the frozen spec.

| Case | Break | alpha.5 | A1 | openapi-diff |
|---|---|---|---|---|
| r1-050 zigbee2mqtt to frontend | `permit_join_timeout` removed | MISSED | NOT EVALUATED (destructuring read stated) | n/a |
| r1-051 AdGuard to adguardhome-sync | set_rules body text to JSON (request side) | MISSED | MISSED | missed |
| r1-052 zwave-js-server to python | device class CCs only below schema 36 | MISSED | NOT EVALUATED (via "moved", noisy: 7 sites) | n/a |
| r1-053 paperless-ngx to paperless-gpt | `status` moved into pagination envelope | MISSED | MISSED | n/a |
| r1-054 zwave-js-server to python | `isHealNetworkActive` renamed for schema 32 | MISSED | MISSED | n/a |
| r1-055 gopls to vscode-go | `Vulns` removed | CAUGHT | CAUGHT | n/a |
| r1-056 Poetry to micropipenv | `files` moved to packages | MISSED | NOT EVALUATED (moved, stated) | n/a |
| r1-057 Turborepo to vercel | `pipeline` renamed `tasks` | CAUGHT | CAUGHT | n/a |
| r1-058 rust-analyzer to coc-rust-analyzer | `cargoExtraArgs` removed | CAUGHT | CAUGHT | n/a |
| r1-059 deno_lockfile to esbuild_deno_loader | `specifiers` moved (v4 default) | MISSED | MISSED | n/a |
| r1-060 NetBox to ansible_modules | `device_role` renamed (request side) | MISSED | MISSED | n/a |
| r1-061 nuclei to faraday_plugins | `matched`, `templateID` renamed | CAUGHT | CAUGHT | n/a |
| r1-062 Glances 4 to python-glances-api | `rx`, `tx` removed | CAUGHT, extra `time_since_update` | same | n/a |
| r1-063 AdGuard to python-adguardhome | `rulesCount` to `rules_count` | CAUGHT | CAUGHT | error |
| r1-064 grype to scan-action | `fixedInVersion`, `links` replaced | CAUGHT, extras `description`, `severity`, `version` | same | n/a |
| r1-065 control, AdGuard safesearch additive | none | FALSE_POSITIVE (`$ref` not resolved) | same | false positive |
| r1-066 control, coverage.py adds regions | none | CLEAN | NOT EVALUATED (moved noise) | n/a |
| r1-067 control, grype adds ignoredMatches | none | CLEAN | CLEAN | n/a |

**Detection (frozen alpha.5): caught 7 of 15, missed 8 (53%).** Combined
in-scope held-out (batch 4 on alpha.4, batch 5 on alpha.5): caught 13 of 25,
48% missed. R1 remains FALSIFIED; batch 5 was harder than batch 4.

**A1 (stated blind spots), measured on batch 5:** silent misses (a real break
reported HOLDS) fell from 8 of 15 (53%) to 5 of 15 (33%). Detection is
unchanged (7 of 15): A1 states, it does not catch. Cost: 1 of 3 controls
became NOT EVALUATED, and the "moved" rule is noisy where producers refactor
types or functions (r1-052 lists 7 sites, r1-066 is a control). Contracts
reported NOT EVALUATED: 4 of 18 in batch 5; 7 of 68 across all batches (including the constructed demo).

On tuning data (batches 1 to 4, read before A1): r1-040 now CAUGHT
(nullability), r1-037, r1-038 and r1-014 NOT EVALUATED; no control changed.

Still silent after A1: request-side changes with no field mention
(r1-051, r1-060), an envelope added around a field (r1-053), a key kept only
for older schema versions (r1-054), and a default format switch (r1-059).
Results: r1-batch5-alpha5.tsv, r1-batch5-a1.tsv.
