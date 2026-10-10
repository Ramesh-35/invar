# INVAR moat: what we defend, and how we know (2026-10-09)

Founder direction, ratified 2026-10-09: the algorithm is copyable in a week,
so INVAR does not defend code. It defends four assets that take longer than a
week to copy. This file records each asset, its evidence rung today, the gap
between the claim and the evidence, and the next falsifiable step. A moat
here is a hypothesis until its gate is met.

## The four assets

| Asset | Claim | Evidence today | Gap stated plainly |
|---|---|---|---|
| 1. Incident corpus | A replayable corpus of real cross-repository breaks, each with provenance and an honest score, is ground truth no clone has. | 40 scored real breaks (30 in R1, 10 in-scope batch 4) plus controls, pinned commits, a runner, held-out protocol. | INVAR itself still misses 40% in scope. The corpus gives us the map of what we miss, not better detection yet. Once public (MIT/Apache), the files are copyable; what is not copyable is the process that keeps adding scored cases. |
| 2. invar.lock habit | A committed lock that CI and AI agents rely on becomes a standard that is costly to leave. | Format 1 built (alpha.5): `Type.field` slice, BLAKE3, t_r/t_v, waivers, MCP. | Zero external users. A format is a moat only after adoption. Not yet frozen or versioned for compatibility. |
| 3. Cross-org trust | A neutral local-first binary lets two organizations exchange content-addressed assumptions neither has to host. | Content addressing exists (BLAKE3 of every fetched artifact). | Not built: no signatures, no signed assumption exchange. No two-organization pair has used INVAR. The "do not build" list rules out a custom PKI or public log, so signing must reuse existing tools. |
| 4. Speed and simplicity | A small deterministic single binary beats tools that grow dashboards and servers. | 4.7 MB binary; exit codes 0/1/2/3/4 with 1 = VIOLATED only; no server, no dashboard; no new crate since alpha.1. | Not "zero-dependency": 7 crates (bundled SQLite for the ledger) and a curl subprocess. Speed is not measured yet (R6 TTFV benchmark). |

## Plan (each step has a gate; a missed gate is recorded, not reworded)

**Asset 1, corpus: grow it and score against it, every release.**
- Add one held-out in-scope batch of at least 10 real breaks per release,
  scored on the frozen engine before any tuning, same rules as batch 4.
- Attack the four batch-4 miss mechanisms one release at a time: request-side
  writes, destructuring reads, nullability, moved-but-same-name fields. Tune
  on old cases, score only on the next held-out batch.
- Gate: an in-scope held-out miss rate at or below 20% on at least 30 cases
  before any public claim of reliable detection. Until then R1 stays FALSIFIED.
- Publish the corpus format and runner with the repo, so others can add
  cases; the moat is the rate of honest additions, not secrecy.

**Asset 2, lock: freeze it, then earn adoption.**
- Freeze `invar.lock` format 1 with a written spec and a compatibility
  promise (roadmap R7): later INVAR versions read format 1 forever.
- Make the lock the default path: `invar init` proposes contracts, `invar
  lock` writes the slice, the Action checks against it.
- Gate: 3 independent external repositories commit an `invar.lock` and keep
  it for 60 days. Count is reported as is, including zero.

**Asset 3, cross-org trust: build the smallest real version.**
- R4 supply-chain hardening first: signed release binaries and checksums
  using an existing signing tool, verified by the Action before it runs.
- Then signed producer snapshots: a producer publishes its field surface with
  a detached signature from an existing tool; `invar lock` records the
  signer; `invar check` verifies it. No new PKI, no public log, no server.
- Gate: one real pair of independent organizations exchanges a signed
  snapshot and catches or clears a change with it. Until then this asset is a
  hypothesis, and docs say so.

**Asset 4, simplicity: budgets, not intentions.**
- Budgets enforced in CI: binary at most 15 MB; no new crate without founder
  approval; exit codes frozen; no network except fetching declared artifacts.
- Measure time to first value (R6): from install to first real finding on a
  public repo pair, target under 5 minutes, measured and published.
- Simplify further where it costs nothing: consider making the SQLite ledger
  an optional feature so the default binary is smaller.

## Additions (ratified 2026-10-09)

The hard-to-copy part is not secrecy. Any large company can copy what we
ship. What it cannot easily do is act against its own incentives: publish
its blind spots, promise identical answers twice while moving to AI review,
run a neutral benchmark that grades itself, or stay neutral across forges
and agents it competes with. Each addition below leans on one of those.

| # | Addition | Why it is painful to copy | Gate (proves it or not) |
|---|---|---|---|
| A1 | **Stated blind spots.** Where INVAR sees a read it does not evaluate (destructuring, payloads the consumer writes, nullability changes it does not compare), the contract reports "not evaluated here, and why" instead of HOLDS. Extends SKIPPED inside supported files. | Large vendors do not publish their tools' blind spots. | Silent miss rate (a real break reported as HOLDS) falls on the next held-out batch versus batch 4's 40%. Batch 4 is tuning data from now on. |
| A2 | **Replayable receipts.** Every check emits a content-addressed receipt (input hashes, engine version, verdict); `invar verify-receipt` re-runs it and must reproduce the same bytes. | Review is moving to AI, which cannot promise the same answer twice. Determinism is our counterweight. | A receipt from one machine verifies byte-identical on another (Linux and macOS) for every corpus case. |
| A3 | **Open benchmark.** The corpus published as the scoreboard for cross-repository breaks: every tool scored on it, INVAR included, its own miss rate first. Also usable to evaluate AI coding agents ("does your agent break downstream consumers?"). | A vendor cannot credibly run the neutral benchmark that grades its own product. | At least one tool or agent we do not build is scored on it by someone other than us. Requires the repo to be public. |
| A4 | **Field-level dependents graph.** Committed locks state who reads which field; a producer asks "who breaks if I remove this?" before removing it (R3 blast radius, fed by real declarations). | Owning the format and being first. Public locks can be crawled by anyone, so this is not secrecy. | A producer uses it to change or delay a removal in a real repository. |
| A5 | **`invar bisect`.** Given a lock and a range of producer refs, find the first producer commit that violates it. | Turns a postmortem from hours into one command; small because the lock already says what to check. | Finds the breaking commit for every corpus case that has a pinned producer history. |
| A6 | **Neutral across forges and agents.** Same behaviour against GitHub, GitLab, Gitea, local files, and any agent through MCP. | Platform owners are disincentivised from neutrality toward rivals. | Verified on at least two forges besides GitHub. |

## Bottlenecks, ranked

1. **Detection.** No moat compensates for missing 40%. A1 and A5 attack it from
   the honesty side; targeted fixes attack it directly.
2. **Private, zero users.** A2 to A4 only work once the repo is public and
   used outside. Going public after doc review is the gating decision
   (founder's).
3. **No design partner.** One team with real cross-repository pain tests every
   claim here faster than any benchmark.
4. **Releases depend on the founder's Mac being online.** alpha.5 waited a day.
   Fix: CI cuts the release from a tag the founder approves.

## Sequence

1. Land alpha.5 (batch 4 record, invar.lock). Waiting on the Mac.
2. alpha.6: A1 stated blind spots; lock format 1 frozen and specified;
   batch 5 held-out scored on the frozen engine (A1 tuned on batches 1 to 4
   only).
3. alpha.7: A2 receipts; R4 signed releases verified by the Action; TTFV
   measured; A5 bisect.
4. alpha.8: A4 dependents graph (R3); signed producer snapshots; repo public
   with A3 benchmark, if the founder approves; first external adopters and a
   design partner sought.

## What would prove this direction wrong

- Held-out in-scope miss rate does not fall below 40% after two releases of
  targeted work: detection is the bottleneck, and no moat compensates.
- No external repository keeps an `invar.lock` after honest outreach:
  the format moat does not exist.
- No two organizations want signed assumption exchange: the boundary moat is
  a story, not a need.

## Adjudication 2026-10-10: detection rule expansion frozen

Founder decision after batch 8. D1, E1 and F1 each scored perfectly on
tuning data and changed zero verdicts on their held-out batch; the held-out
catch rate has sat near 54% for five batches; the misses keep arriving one
syntactic notch outside each frozen rule's grammar. The incremental
textual-rule approach to the 20% gate is falsified and STOPPED. F2 (widened
selector grammar for r1-113's enum-variant case) is declined.

What this does NOT change: the corpus keeps growing one held-out batch per
release (Asset 1), the honest scoring stands, R1 stays FALSIFIED, and every
doc keeps saying so. A1-style stated non-evaluation stays, because it is
honesty, not detection.

New sequence: A2 evidence receipts (spec drafted, docs/architecture/
A2-RECEIPTS.md, awaiting ratification), then A4 field-level dependents
graph. Both deliver value that does not depend on winning the detection
race; the detection bottleneck remains recorded as bottleneck #1, unsolved.
