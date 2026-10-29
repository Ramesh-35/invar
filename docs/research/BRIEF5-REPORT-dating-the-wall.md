# Dating the Wall: What Breaks First When Agents Write the Code (INVAR Foresight Brief 5)

Received 2026-10-29. Verdict in one line: a qualified H5. The human
review gate is failing NOW, but the world is dissolving it (LLM review,
unreviewed merges) rather than replacing it with a deterministic
judge-and-receipt substrate. H1's diagnosis survives; its prescription
is disfavored at 20%. The one dated, decidable collision INVAR can
partly capture is H4: the MCP deprecation registry, removals eligible
from 2027-07-28 (55%). Prescription: no new substrate program; keep the
current program; 2 to 3 weeks of warm-keeping only. The prediction set
is logged in docs/research/PREDICTIONS.md and governs reassessment.

## TL;DR

- The review wall is real, and it is being dissolved rather than hit.
  Agent-started PRs went from 17% (March 2026) to more than 40%
  (September 2026) of GitHub PRs (FACT, GitHub Octoverse, 28 Oct 2026).
  Agent PRs now merge within two weeks more often than other PRs (90%
  vs about 86% by September). Faros telemetry shows PRs merged with no
  review up 76.3% while incidents per PR are up only 14.5% (FACT,
  Faros, 18 Sep 2026; vendor data). The gate is being removed or
  automated, not enforced by a new substrate.
- H1's diagnosis survives, but its prescription loses to economics and
  platforms. Price for a given level of LLM performance falls about 47%
  per quarter (FACT, Epoch AI, 22 Sep 2026). On 28 Oct 2026 GitHub
  folded Copilot code review into Enterprise with no separate
  subscription. Rice's theorem limits deterministic judgment to
  declared, decidable properties, so INVAR's addressable surface is the
  declared-interface niche (H4), not "every boundary."
- What to do: no new product. Keep a warm position on the one dated
  collision INVAR can judge: MCP's 2026-07-28 deprecation registry
  (Roots, Sampling, Logging, Dynamic Client Registration removable in
  the first revision on or after 2027-07-28; official registry 43,298
  servers on 14 Oct 2026; about 60 to 62% of servers have only one
  published version). Treat liability receipts (H2), agent identity
  (H3) and runtime authority (H6) as watch items owned by others.

## Hypothesis verdicts (confidence that each is the operative wall by end of 2029)

| Hypothesis | Verdict |
|---|---|
| H1 verification-bandwidth ceiling | Diagnosis right, prescription disfavored. 20% that a deterministic machine-speed trust substrate becomes unavoidable. |
| H2 liability wall | 15% that receipts are a forced purchase. PLD compensates natural persons only; insurers exclude rather than demand evidence. |
| H3 identity wall | 35% real; under 10% that INVAR captures it (OAuth/OIDC/SPIFFE reuse; platforms own the layer). |
| H4 interface wall | 45% in narrow form; 55% that an MCP removal ships by end of 2027. The only hypothesis INVAR can partly capture. |
| H5 null (qualified) | 55% for PR-time trust. The surviving hypothesis. |
| H6 runtime authority wall (new) | 60% real; captured by cloud and identity platforms, not INVAR. |
| H7 remediation wall (new) | 50%; 2027 to 2028; captured by observability and SRE vendors. |

## Key evidence (dated)

- Octoverse 2026 (28 Oct 2026): agent PR share 17% (Mar) to >40% (Sep);
  two-week merge rate 90% agent vs ~86% non-agent; ~110M agent PRs in
  September; counts only detectable agents.
- Chen and Stratton (working paper, 4 Aug 2026, Jellyfish data, ~700
  firms): PR-to-merge time +49% after agent adoption; review comments
  +35%; completed tasks no significant rise. Unrefereed, opted-in
  sample.
- Faros (Apr 2026 vs 18 Sep 2026, methods differ, not a time series):
  Q3 shows incidents per PR +14.5%, unreviewed merges +76.3%, monthly
  incidents +125.4%, mean time to remediate +25.8%, QA time +300.6%;
  agentic review associated with 24.4% lower change failure rate.
- Epoch AI (22 Sep 2026): price of fixed capability falling ~13x/year.
- METR (10 Mar 2026): about half of test-passing SWE-bench Verified PRs
  would not be merged by maintainers (296 PRs, 4 maintainers, 3 repos).
- 2026 agent incidents were runtime-credential failures, not merged
  code: DataTalks.Club terraform destroy (26 Feb), PocketOS volume and
  backups deleted (~9 seconds, 24 Apr), two OpenAI training pauses (Jul,
  Sep). No PR-time judge would have caught them.
- EU PLD 2024/2853 applies to products placed after 9 Dec 2026; natural-
  person damage only; Germany adopted its act 8 Oct 2026. ISO published
  generative-AI exclusion endorsements with a Jan 2026 edition; carriers
  exclude rather than require change evidence.
- MCP 2026-07-28 spec: formal deprecation policy (SEP-2596), 12-month
  minimum window, public deprecated-features registry; registry server
  count 3,012 (Mar 2026) to 43,298 (14 Oct 2026); ~60.7 to 62% of
  servers have one published version.

## Invariants assessed

Human reading hours bind only where review is mandatory, and review is
becoming optional, so it behaves as policy, not physics. Semantic review
is undecidable (Rice), so deterministic judgment covers declared,
decidable properties only: the market is the size of the declared
surface. Liability attaches to a legal person and requires disclosure
capability, not per-change human names. Incident remediation and runtime
authority cannot be skipped and are the strongest coming walls, owned by
other categories of vendor.

## Capture map (the licensed work)

| Coming moment | INVAR component | Warm-keeping preparation | Cost |
|---|---|---|---|
| MCP removal revision, on or after 2027-07-28 | judge + MCP surface | A reader for the MCP deprecated-features registry so judge can flag servers and clients using Roots, Sampling, Logging or DCR. Publish nothing until a removal revision is announced. | 1 to 2 weeks |
| Platform LLM review becomes default reviewer | judge GitHub Action | Emit verdicts in formats platform review agents already read (check-run annotations, SARIF) so the LLM reviewer cites INVAR's decidable verdict instead of guessing. | ~1 week |
| PLD disclosure orders | locks and receipts | None; recheck Predictions 7 and 8 quarterly. | 0.5 day/quarter |
| Agent identity convergence | receipts' signer field | None; watch Prediction 10. | 0 |
| Runtime authority, remediation | agent hook | None; outside decidable scope. | 0 |

Premature capability (do not build): agent identity or attestation
services; insurer-facing or PLD evidence packs; a runtime
destructive-action policy engine behind the hook; blocking MCP gates;
an LLM-review competitor.

## Steelman of the null, answered

Mostly conceded for PR-time trust. Not conceded: the cost has moved
rather than vanished (monthly incidents +125.4%, remediation +25.8%),
and METR's maintainer gap shows LLM-judged "passing" is not
"mergeable". Both point at runtime and remediation walls that INVAR
does not own. Under H5 the operator builds nothing beyond the current
program plus the two warm-keeping items; if B2 fails its kill rule,
stop, and do not repurpose the primitives toward H2 or H3.

## Caveats and gaps (abridged)

Load-bearing numbers come from interested vendors (GitHub, Faros,
LinearB); Faros's two reports use different methods; Chen and Stratton
is unrefereed; DORA is survey-based and correlational. No public series
exists for reviewer headcount, review latency over time, price per
merged agent PR, non-vendor incident rates per agent PR, or
declared-contract adoption. Unsourced claims excluded from findings:
any company pausing agent merges over review overload; any insurer
pricing on change evidence; GitHub requiring human approval on Copilot
cloud-agent merges (secondary only); LLM-review reliability convergence
(vendor-run benchmarks conflict, 36% to 82% for one tool).
