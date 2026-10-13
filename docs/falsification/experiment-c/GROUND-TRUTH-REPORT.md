# INIT-DISCOVERY ground-truth audit: report (2026-10-13)

Research only. Nothing in the repository was changed, committed or
pushed. Definition frozen before any repository was opened
(DEFINITION.md, unchanged during the audit).

## A. Frozen corpus and seed

The 30 monorepos of Experiment B (seed 20261014), re-cloned at depth 1;
HEAD commits recorded in corpus.tsv. Corpus unchanged.

## B. 30-row audit matrix

| # | repo | label | basis |
|---|------|-------|-------|
| 1 | matthewwilliams949-ops/nakodo | NEGATIVE | 2 packages, 0 cross-package imports (two methods) |
| 2 | iceywu/icloud-calendar-mcp | NEGATIVE (not a monorepo) | pnpm-workspace.yaml holds build settings only, no packages |
| 3 | nikicat/mcp-wallet-signer | NEGATIVE | 3 packages, 0 cross-package imports |
| 4 | biosshot/easyeda-copilot | POSITIVE | see C1 |
| 5 | krovacloud/krova-node | AMBIGUOUS | see D1 |
| 6 | apegurus/web3agent | NEGATIVE | only imported shape has 1 distinctive field (fails 4) |
| 7 | mihai-dinculescu/tapo | POSITIVE | see C2 |
| 8 | lemmaoracle/lemma | POSITIVE | see C3 |
| 9 | piyushpathakqa/vigilis | POSITIVE | see C4 |
| 10 | lusrodri/css-sota-mcp | NEGATIVE | 2 packages, 0 cross-package imports |
| 11 | tokencanopy/e2a | POSITIVE | see C5 |
| 12 | mrrefactoring/atlassian-dc-mcp | POSITIVE (marginal) | see C6 |
| 13 | beatapi/beatdesign | NEGATIVE (not a monorepo) | pnpm-workspace.yaml settings only |
| 14 | apollographql/apollo-mcp-server | NEGATIVE | 8 candidates: Options::default(), method-only use, 1-field shapes (fail 4) |
| 15 | csacanam/voulti | NEGATIVE | @voulti/shared declared, imported by no sibling (fails 3) |
| 16 | ouranos-labs/pseolint | POSITIVE | see C7 |
| 17 | jordanburke/ssh-client-mcp-server | NEGATIVE | see D2 |
| 18 | ruvnet/claude-flow | NEGATIVE | 1 candidate, 1 distinctive field read (fails 4) |
| 19 | ftwagency/wpconvert-devtools | NEGATIVE | cross-package imports carry no named shapes |
| 20 | clira-dev/ezstat-mcp-server | NEGATIVE (not a monorepo) | pnpm-workspace.yaml settings only |
| 21 | datahogo/datahogo | POSITIVE | see C8 |
| 22 | sou0327/knowmint | NEGATIVE | 5 candidates, none read 2 distinctive fields (fails 4) |
| 23 | dinglebear-ai/rgotify | NEGATIVE (not a monorepo) | Cargo [workspace] with no members (single crate) |
| 24 | pspdfkit/nutrient-document-engine-mcp-server | NEGATIVE (not a monorepo) | pnpm-workspace.yaml settings only |
| 25 | otakit/otakit | POSITIVE (example consumer) | see C9 |
| 26 | stahldavid/sensegrep | NEGATIVE | only cross-package type is a string union |
| 27 | theluckystrike/mcp-servers | POSITIVE | see C10 |
| 28 | minipuft/claude-prompts-mcp | NEGATIVE | 1 declared workspace package (cli); see D3 |
| 29 | dinglebear-ai/yarr | POSITIVE (tooling consumer) | see C11 |
| 30 | digitalbrewllc/playbykey-dev-tools | NEGATIVE | cross-package imports are const objects and unions |

Totals: POSITIVE 11, AMBIGUOUS 1, NEGATIVE 18 (of which 5 are not
monorepos at all; see H).

## C. Evidence for every positive (one verified TRUE relationship each)

C1 easyeda-copilot: PreviewPcbInput (@copilot/shared types/pcb/preview.ts)
-> extension/src/eda/pcb-preview.ts; `previewPcb(input: PreviewPcbInput)`
reads input.zoom, layers, highlight_net, highlight_net_colors,
highlight_component, highlight_component_colors, padding_mm. Production.
C2 tapo: RecordingHubResult (tapo crate) -> tapo-py
responses/smart_cam/recordings_hub_result.rs; reads self.0.start_time,
end_time, video_type. Python-binding package.
C3 lemma: CircuitMeta (@lemmaoracle/spec, declared `Readonly<{...}>`) ->
packages/sdk/src/prover.ts; reads meta.verifiers, circuitMeta.artifact.
Production.
C4 vigilis: AttestationBundle (@argus/core) -> packages/cli/src/index.ts;
parsed `as AttestationBundle`, reads records, count, headHash, actor.
C5 e2a: EmailReceivedData (@e2a/sdk/v1) -> cli/src/commands/listen.ts;
`notification: EmailReceivedData` reads message_id, received_at,
header_from.
C6 atlassian-dc-mcp (marginal): HttpClientConfig (datacenter-mcp-core) ->
packages/jira/src/jiraClient/core/client.ts; reads config.softValidation,
config.headers. Exactly the 2-field threshold.
C7 pseolint: Template (@pseolint/core) ->
apps/web/src/components/audit/template-card.tsx; reads template.variance,
auditedUrls, totalDiscoveredUrls.
C8 datahogo: FindingData (@datahogo/core) -> packages/cli/src/sarif.ts;
`toSarif(findings: FindingData[])` reads vulnerability_id,
description_technical, description_simple, fix_description.
C9 otakit: OtaKitState (@otakit/capacitor-updater) ->
examples/demo-app/app/page.tsx (declared workspace package); reads
staged, current, fallback, builtinVersion. Example-app consumer.
C10 mcp-servers: ComputedLine (@theluckystrike/mcp-invoice) ->
servers/billing-docs/src/index.ts; `negateLine(l: ComputedLine)` reads 6
*_minor fields.
C11 yarr: OperationSpec (yarr crate) -> xtask/src/live/contract.rs; reads
op.method, op.path_params (op.path stoplisted). Build-tooling consumer.

Rejected candidates inside positive repos are recorded in the working
notes (e.g. vigilis CloudReceipt is CONSTRUCTED, not read; atlassian
BinaryAsset is passed through only).

## D. Negative and ambiguous cases, explicit

D1 krova-node, AMBIGUOUS. packages/cli reads share.counterpartySpaceName
and share.expiresAt (2 distinctive fields of BackupShare), so condition 4
holds. But BackupShare is declared `components["schemas"]["BackupShare"]`,
an indexed alias into generated OpenAPI types; the frozen "object type
alias" was written with `type X = {...}` in mind. Resolved as AMBIGUOUS
rather than in the direction that helps the score; reported both ways.
D2 ssh-client, NEGATIVE as written, recorded as a definition-excluded
coupling: orchestrator/src/index.ts reads r.stdout, r.stderr, r.code
(CommandResult's fields), but r's type is inferred from runner.exec and
CommandResult is never imported by name, so condition 3 fails.
D3 claude-prompts-mcp: server/ has a package.json but is not a declared
workspace package (workspaces = ["cli"]). Consequence: Experiment B's only
TRUE proposal (server/resources -> server/tests) was INTRA-package and is
not cross-package ground truth.
Construction is not reading: candidates whose consumer only builds a
value of S (vigilis CloudReceipt) fail condition 4.

## E. Frozen qualifying-contract definition

As in DEFINITION.md, unchanged: (1) distinct declared workspace packages;
(2) S a named data shape exported by P (TS interface or object type alias,
zod object, Go exported struct, Rust pub struct, Python typed class); (3)
F in C imports S by name (package import or relative path, one barrel
level); (4) F reads at least 2 non-stoplisted fields of S by property
access, verified by hand.

## F. Stoplist and threshold (frozen)

id, name, type, description, data, value, payload, key, label, title,
status, error, message, result, items, url, path, version, created_at,
updated_at; naming-form matching; at least 2 surviving fields.

## G. Pre-registered metric definitions

- Ground-truth coverage = (POSITIVE repos where INIT-DISCOVERY emits at
  least 1 TRUE proposal) / (POSITIVE repos). Primary denominator 11.
  Sensitivity: krova counted POSITIVE, denominator 12.
- Precision = (TRUE proposals) / (ALL proposals emitted across all 30
  repos, including NEGATIVE ones). Counted per proposal, not per repo.
- A proposal is TRUE when its (producer file, consumer file) pair
  instantiates a relationship meeting E, judged by hand at evaluation
  time with the same definition; producer file = the file declaring S
  (or the barrel it is imported through).
- Per-repository results are reported alongside aggregates; no
  repository is dropped.

## H. Acceptance targets and limits on interpretation

Targets (as ruled): coverage at least 60% (at least 7 of 11; at least 8
of 12 in the sensitivity) AND precision at least 70%.
Limits:
1. n = 11 is small. 7 of 11 (64%) has a 95% Wilson interval of roughly
   35% to 85%. The benchmark can distinguish "mostly works" from "mostly
   fails", not fine gradations.
2. The frame is MCP-adjacent and TypeScript-heavy; results do not
   generalize beyond such repositories.
3. Experiment B's frozen monorepo detector over-counted: 5 of its 30
   (iceywu, beatdesign, ezstat, pspdfkit: pnpm-workspace.yaml holding
   settings only; rgotify: Cargo workspace without members) are not
   multi-package workspaces. B's verdict (FAILS) is unchanged; its
   honest denominator is 25 monorepos. Recorded against my own method.
4. Definition-excluded couplings exist (D2 inferred types; D1 indexed
   aliases). The benchmark measures named-import, field-reading
   contracts only.

## I. Recommendation: PROCEED

Eleven repositories with verified cross-package ground truth (plus one
ambiguous) is enough to measure the capability at the resolution limit
1 allows. Three findings for the spec, which need ratification before
the build (no spec change made in this increment):
- R4: producer extraction must include `export type X = Readonly<{...}>`.
  The audit's own locator missed it first; lemma went from 0 to 63 shape
  imports once it was recognized. The drafted spec names only
  `export type X = {`.
- R5: rule on indexed-access aliases of generated types (krova): in or
  out of v1.
- R6: rule on inferred-type consumers (ssh-client): stated out of v1
  scope (recommended), since resolving inferred types is type checking.
Precision note, as ruled: proposals emitted in the 18 NEGATIVE
repositories count against precision, so the target penalizes noise on
repositories with nothing to find, which is the stranger's most common
case in this frame.
