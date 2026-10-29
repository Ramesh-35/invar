# Prediction ledger (opened 2026-10-29)

Foresight under the same discipline as the engine: dated forecasts with
pre-stated falsification, scored on their review dates, wrong calls kept
in this file. A prediction is settled only by its stated condition,
never by vibes. Source of the first set: Brief 5
(docs/research/BRIEF5-REPORT-dating-the-wall.md). Predictions 1 to 3 are
H1 tripwires: if they fail, the verification-bandwidth thesis is
materially damaged and this file says so.

| # | Statement | Review date | Falsified if | Confidence | Status |
|---|---|---|---|---|---|
| 1 | A refereed version of Chen and Stratton, or an independent replication on post-March-2026 firm data, reports a PR-to-merge time increase of at least 25% after agent adoption | 2027-10-31 | No significant increase, or below 25% | 55% | OPEN (H1 tripwire) |
| 2 | Faros's H1 2027 report shows incidents per PR rising at least 25% again as unreviewed merges keep growing | 2027-06-30 | Incidents per PR stay at or below +15% while unreviewed merges rise | 35% | OPEN (H1 tripwire) |
| 3 | GitHub, GitLab, Cursor or Anthropic ships a default-on, deterministic (non-LLM) contract or compatibility gate for agent-authored PRs | 2027-12-31 | No such default-on deterministic gate ships | 25% | OPEN (H1 tripwire) |
| 4 | Octoverse 2027 reports agent-started PRs above 50%, with the agent two-week merge rate at or above the non-agent rate | Octoverse 2027 publication | Share below 50%, or agent merge rate 3+ points below non-agent | 60% | OPEN |
| 5 | A published firm-level dataset shows AI generating more than 50% of review comments (baseline 23.3%, March 2026) | 2027-12-31 | No published dataset above 40% | 55% | OPEN |
| 6 | MCP publishes a specification revision removing at least one of Roots, Sampling, Logging or Dynamic Client Registration | 2027-12-31 | No removal on or after 2027-07-28, or the date formally extended | 55% | OPEN |
| 7 | No EU court judgment applying the new PLD to software turns on the presence or absence of change records | 2028-12-31 | Such a judgment is published | 85% | OPEN |
| 8 | No top-20 P&C insurer publishes an underwriting requirement for machine-checkable change evidence for AI coding use | 2027-12-31 | One does, in a public filing or product document | 80% | OPEN |
| 9 | At least two more publicly reported incidents of an agent destroying production data through runtime credentials rather than merged code | 2027-06-30 | Fewer than two reported | 80% | OPEN |
| 10 | NIST NCCoE publishes a draft project description for software and AI agent identity and authorization | 2027-06-30 | Not published by that date | 50% | OPEN |

Review protocol: each quarter, check every row whose review date has
passed or whose falsification condition may have fired; mark SETTLED
TRUE, SETTLED FALSE, or AMBIGUOUS with one line of evidence and a date.
A SETTLED FALSE on rows 1 to 3 triggers a written reassessment of the
H1 line in docs/research/BRIEF5-REPORT-dating-the-wall.md.
