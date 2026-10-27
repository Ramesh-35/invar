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

Promises people already declare (ODCS data contracts, dbt contracts),
enforced at the moment of change by this engine: deterministic because the
contract is declared, not inferred; single-player on day one (a team
checks its own change against its own contract, no consumer adoption
needed); same hold / approve / log spine as A2.

| Track | What | Metric | Window | Kill rule |
|---|---|---|---|---|
| B1 kill gate | real public ODCS and dbt projects: lock declared contracts, replay real historical breaking changes | actionable catches; false alarms on controls | 7 days | below the pre-registered threshold (set in the B1 spec before measurement) kills Program B before any product code |
| B2 slice | the contract reader plus hold / approve / log, frozen fixtures first | ships, gated as always | 14 days after B1 passes | B1 failing |
| B3 money | the same 40-team outreach instrument pointed at data teams | written paid-pilot intents | 14 days from first contact | fewer than 2 kills the data tier |

## Standing rules

- Every claim in public material is backed by the measured record; the
  banned-phrases list applies to marketing exactly as to docs.
- A dead track is recorded dead here with its final number.
- The engine stays one binary, zero required runtime services, no LLM in
  any verdict, open core.
