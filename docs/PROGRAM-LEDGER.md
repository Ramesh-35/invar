# Program ledger (opened 2026-10-27)

The anti-zombie instrument. Every track has one metric, one date, one kill
rule, pre-registered. A track whose date passes without its number is
reported dead in this file, not quietly extended. Dates are set at each
track's start and recorded here; windows below are from that start.

## Program A: the breaking-change firewall for code (what we launch)

The engine as it is, measured 38 of 70 held-out recall, stated everywhere.
Program A is not sold as detection. Its jobs:

| Track | What | Metric | Window | Kill rule |
|---|---|---|---|---|
| A1 launch | v0.1.0 release, README, launch post with the live demo PRs | reach: stars, hook installs, demo-PR visits, inbound issues | 14 days from post | no track dies here; A1 feeds A2, A3 and B |
| A2 governance | hold / approve / log on VIOLATED (the one sourced payer pattern: Buf's enterprise workflow) | written paid-pilot intents from the 40-team outreach | 14 days from first contact | fewer than 2 intents kills the paid tier; the free Action stays |
| A3 autopsy | fixed-fee break autopsy: a team's own history replayed by backtest, witnesses, delivered in days | paid autopsies sold during the same outreach | same window as A2 | 0 sold kills the service |
| A4 benchmark | the 122-case corpus published as the public benchmark for real cross-repository breaks | external citation or use (a tool, a paper, a vendor) | 60 days from publication | none; the corpus compounds regardless, cost is packaging only |

Non-goal, recorded: recall rungs are last priority. They feed only the
falsified detection thesis.

## Program B: the breaking-change firewall for data (the bet)

Promises people already declare, enforced at the moment of change by this
engine: deterministic because the contract is declared, not inferred;
single-player on day one (a team checks its own change against its own
contract, no consumer adoption needed); same hold / approve / log spine as
A2. The B1 corpus fixed the contract families: Iglu JSON Schema, OTel
semconv ledgers, Kubernetes prerelease-lifecycle declarations, Terraform
provider schemas (ODCS and dbt remain candidates; the Fivetran family is
rejected as declaration without enforcement semantics).

| Track | What | Metric | Window | Kill rule |
|---|---|---|---|---|
| B1 kill gate | real public ODCS and dbt projects: lock declared contracts, replay real historical breaking changes | actionable catches; false alarms on controls | 7 days | below the pre-registered threshold (set in the B1 spec before measurement) kills Program B before any product code |
| B2 slice | `invar judge`: one stateless subcommand, CI against base-branch diffs; declared upstream contract vs downstream consumer reads; exit-coded HOLDS / VIOLATED (key and line witnesses) / NOT EVALUATED (stated, never silent), signed receipt | external independent repositories running `invar judge` in an active PR pipeline | 14 days from B2 release | fewer than 2 external independent repositories kills the standalone data slice; the judge stays in the engine |
| B3 money | the same 40-team outreach instrument pointed at data teams | written paid-pilot intents | 14 days from first contact | fewer than 2 kills the data tier |

B1 result, recorded 2026-10-28: GATE MET, four days before the 2026-11-01
cap. 10 admitted, 6 rejected, four ecosystems (Snowplow 2, OTel 3,
Kubernetes 3, Terraform 2); mean restatement count 3.2 coping notations
per broken promise; PR-time enforcement observed in any ecosystem: none.
Evidence: docs/falsification/b1/ (ledger, manifests, harvesters).

B2 WINDOW OPEN, recorded 2026-10-29: v0.2.0 released and
stranger-verified (checksum, ed25519 signature against the committed
anchor, judge verdict replayed from the released binary). The 14-day
window closes 2026-11-12; the kill rule is measured by
docs/falsification/b2-watch.sh, one row per day. Distribution note,
honest: HN Show HN blocked by a new-account restriction (moderators
emailed), first r/rust post held by moderation (modmail sent);
distribution is accreting via direct channels during the window.

B2 definition, ratified 2026-10-28:
- Surface: a single stateless `invar judge` subcommand, run in CI against
  the base branch exactly as SIB-3 reads trust (never from the PR tree).
  Inputs: the upstream declared contract (Iglu JSON Schema first, because
  the B1 corpus supplies pinned pre/post bytes as frozen fixtures; OTel
  semconv and OpenAPI follow) against downstream consumer declarations or
  queries. Outputs: the fixed exit-code interface, VIOLATED with exact
  key and line witnesses, NOT EVALUATED stated with a reason code, and a
  signed receipt. A skip is never a pass.
- First adopter: platform engineers, data-infrastructure leads, and
  maintainers of shared schema registries or APIs who absorb
  cross-repository consumer breakage today by hand (the b1-003 and
  b1-006 coping patterns).
- Build gate: spec and fixtures frozen before code, as always; the B1
  pinned cases are the first fixture source.
- Kill rule, pre-committed: fewer than 2 external independent
  repositories (not ours, not demos) running `invar judge` in an active
  PR pipeline within 14 days of the B2 release kills the standalone data
  slice without negotiation; the judge remains an engine capability.

The gated backlog behind these tracks, item by item with unlock gates
and kill conditions: [BACKLOG.md](BACKLOG.md).

## Standing rules

- Every claim in public material is backed by the measured record; the
  banned-phrases list applies to marketing exactly as to docs.
- A dead track is recorded dead here with its final number.
- The engine stays one binary, zero required runtime services, no LLM in
  any verdict, open core.
