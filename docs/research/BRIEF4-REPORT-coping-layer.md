# INVAR, Brief 4: The Coping Layer Is the Unowned Codebase

One candidate passes all five hard constraints, and only narrowly. It is H2, the compat compiler. Its job: turn a producer's declared change into the exact coping code one specific consumer needs, plus a signed statement of what that code does not cover. The first market test is the OpenTelemetry Java agent 3.0, which removes legacy database and code attribute emission and was targeted for October 2026. Money is the weak point. I found no named source showing anyone pays for generated coping code, so the winner is a bet on labor people already do by hand, not on a stated budget.

## TL;DR

- **The exhaust says:** producers now publish change as data, but consumers still absorb it as hand-written code in several places at once. That coping layer is now partly written by agents, and nothing checks it against the producer's own declared diff. INVAR's value sits there, not in detection.
- **The winner is H2, the compat compiler.** A new output surface reads the declared rename diff, intersects it with what the consumer's committed `invar.lock` actually binds, and emits OTTL, JSON, TypeScript or Go alias tables. It also lists NOT COVERED changes (merges, value changes, unit changes) with reason codes and a signed receipt. The forcing event is the OpenTelemetry Java agent 3.0 cutover, where the project's own preview post hands users an AI prompt to produce the same old-to-new mapping.
- **The 14-day kill rule:** if fewer than 3 externally merged or CI-run INVAR coping artifacts appear, from at least 2 independent organizations, by day 14 after release, H2 is dead. H1, H4 and H5 fail at least one hard constraint. H3 passes them but loses on the base rate that detection feeds go unpaid.

## 1. The signals table

I read this from the exhaust alone, before any external search.

| Item | Measured fact (October 2026) | What this says the world is becoming |
|---|---|---|
| E1 | OpenTelemetry declares 151 renames machine-readably; 12 old names still read in Jaeger and SigNoz production paths years later | Producers already publish change as data, but declaring a change no longer makes consumers move. The gap between declared and absorbed is measured in years. |
| E2 | SigNoz hand-maintains backward_compat_keys.go | Consumers are re-typing the producer's own diff by hand. The coping artifact is derivable but is made manually, once per consumer. |
| E3 | Kubernetes moved the HPA v2beta2 removal from 1.25 to 1.26 | The schedule is itself a moving contract. Any coping code tied to a date or version is tied to a target that moves. |
| E4 | terraform-aws-modules needed two more commits to fix bugs its own absorption code introduced | Coping code is real code with its own defect rate. Each migration creates a second, unchecked codebase. |
| E5 | Fivetran YAML column declarations carry no enforcement semantics | Declaration without enforcement is the default. Most declared contracts are documentation, so the coping layer is where behavior actually gets decided. |
| E6 | One changed promise lives in a mean of 3.2 downstream coping notations; enforcement at the moment of change observed: zero | Each change multiplies into about three hand-made artifacts per consumer, and none of them is verified when the change lands. |
| E7 | Agents write code at both ends; INVAR's hook and MCP server already feed witnesses to agents | The coping layer is moving from human hands to agent hands, so it will be produced faster and in larger volume. |
| E8 | Detection recall 38 of 70, FALSIFIED against its own gate; the durable value is stated non-silence | Users do not need another detector. They need an honest account of what was and was not handled. |

**Synthesis, the single trend line.** Read together, the eight items describe one shift. The contract is now machine-readable at the producer (E1, E3). The work of absorbing a change is still done by hand at each consumer (E2), repeated about three times per change (E6), defect-prone in its own right (E4), unenforced by default (E5), and increasingly written by agents (E7). The coping layer is turning into the largest unowned, unverified body of code at every boundary, and its input, the declared diff, is already data. INVAR's own falsification (E8) says its edge is not finding breaks but stating, reproducibly, what was handled and what was not. The NVIDIA-style reading is that INVAR's capture vehicle is not the gate. It is the engine that already holds both halves of the join: the producer's declared change and the consumer's committed field reads. That join is exactly the input needed to generate the coping layer, and to say honestly where generation stops.

## 2. The winner in five sentences

The job is to turn a producer's declared change into the exact coping code one consumer needs, and to sign a statement of every change it could not translate. Our exhaust predicted it: E2 shows humans hand-compiling what the declared diff already contains, E6 shows that work repeated 3.2 times per change, E4 shows it breeds its own defects, and E8 says the durable value is stated non-silence, which here means every change left untranslated gets a reason code. The dated forcing event is the OpenTelemetry Java agent 3.0. Its release candidate, 2.32.0, was announced on 6 October 2026 with 3.0 "targeted for October 2026". It makes stable database and code conventions the only mode, and the project's preview post offers users an AI prompt to "return a cited old-to-new mapping and draft query changes". The four-week slice is a new `invar compat` output command on the existing engine: it reads the declared rename set, intersects it with the bindings in the consumer's `invar.lock`, emits OTTL transform statements, JSON, TypeScript and Go alias tables, attaches NOT COVERED reason codes, and writes a signed receipt. The 14-day adopters are maintainers of repositories that consume OpenTelemetry attributes (the class that includes Pydantic Logfire, Laminar, Sentry and SigNoz, all of which shipped incomplete hand-written mappings) and platform teams preparing the Java agent 3.0 cutover.

**Which existing component captures it.** The LOCKS supply the consumer side (only fields actually bound, via the same data `invar dependents` uses). The RECEIPTS sign the generated artifact and its NOT COVERED list, and `invar verify-receipt` checks them. The JUDGE then evaluates the generated coping file on the next pull request like any other consumer read, unchanged. `invar backtest` replays the generated map over the producer's own history. No new engine, no runtime service, no LLM in any output, and no new verdict or exit code: the command exits 0 on generation and 3 on error, per the existing covenant.

## 3. The candidate table

| Candidate | Instant compatibility | Imminence (dated, within 12 months) | Primitive | Falsifiable | Novel | Why it won or lost |
|---|---|---|---|---|---|---|
| H1 Machine-speed boundary | PASS | WEAK. The MCP 2026-07-28 spec already shipped; agent PR share is a trend, not a date | PASS | PASS | FAIL. The hook and MCP server already do "consult before acting" | Already built. A rewording of shipped assets. |
| **H2 Compat compiler** | **PASS** (output format over locks, declared diff, receipts) | **PASS.** Java agent 3.0, targeted October 2026; Kubernetes 1.38 planned 16 December 2026 | **PASS** | **PASS** (kill number below) | **PASS.** Generation of coping code is in no listed backlog item | **Wins.** The only candidate whose output is an artifact people already produce by hand. Money unsourced. |
| H3 Promise ledger | PARTIAL. Needs new readers for each lifecycle registry (for example the MCP deprecated-features page) | PASS. MCP deprecated features become removal-eligible on or after 2027-07-28; Kubernetes 1.38 on 16 December 2026 | PASS | PASS | PASS | Loses on base rate. A feed is detection, and three prior briefs found no WTP for detection. Folded into H2 as a reason code. |
| H4 Upgrade intelligence | FAIL. Real "what breaks" answers need live cluster and Helm state; INVAR skips that by policy | PASS. EKS extended support bills $0.60 versus $0.10 per cluster hour | PASS | PASS | WEAK. Pluto and kubent already do Kubernetes deprecation scanning | Crowded and needs reading INVAR does not do. |
| H5 Audit substrate | PASS | PASS on date (EU Product Liability Directive applies from 9 December 2026) | PASS | PASS | PARTIAL. Overlaps excluded receipt transparency | The directive creates liability, not an attestation mandate. No sourced WTP for compliance receipts (prior brief). |
| H6 (mine) Removal clock on reads | PASS. Annotate each lock binding with the producer's declared removal version | PASS (same dates as H3) | PASS | PASS | PASS | A report version of H3 plus H4. Kept as an output column of H2, not a product. |
| H7 (mine) Verify agent-written migrations | PASS | PASS | PASS | PASS | FAIL. Judging the coping file at the PR is "enforce declared contracts at the PR" reworded | Excluded. Under H2 it happens for free, because the existing judge reads the generated file. |

## 4. The 14-day falsifiable test

**Setup (day 0).** Release `invar compat` with one prepared input: the Java agent 2.x to 3.0 database and code rename set, plus the GenAI renames (for example `gen_ai.system` to `gen_ai.provider.name`). Every generated file carries a header with its receipt hash, so adoption can be counted mechanically, with no survey.

**Seed set.** Run it against public repositories already shown to read retired names: Pydantic Logfire (PR #2561 added reads of `db.query.text`, `db.namespace` and `db.system.name` after DB spans lost their messages), Laminar (PR #2603, where an AI agent's fix still reads the deprecated `gen_ai.system` first), Sentry (issue #104512, where the query selector reads only `db.system`) and SigNoz (issue #8406). Then extend to any repository returned by a code search for the retired keys.

**What counts as one external event.** (a) An external repository merges a commit containing an INVAR-generated coping artifact, identified by its receipt header. (b) An external repository runs `invar compat` in CI. (c) A third party runs `invar verify-receipt` on a compat receipt.

**KILL NUMBER, pre-committed.** Fewer than 3 external events, from at least 2 independent organizations, by day 14 after release kills H2 as a product direction. A second, separate kill for the commercial version: if fewer than 1 of 10 platform teams preparing the Java agent 3.0 cutover agrees to a paid or letter-of-intent pilot by day 14, the free artifact may continue, but H2 is recorded as having no willingness to pay. That would make it the fourth brief to find none.

**Pre-committed confounder.** If Java agent 3.0 has not shipped by day 0, the test still runs on the 2.32.0 release candidate and the GenAI renames. A slip is not grounds to extend the window.

## 5. The steelman, answered or conceded

**Steelman.** The problem is real but not ours. OpenTelemetry already ships a Collector schema processor that translates telemetry between convention versions. The Java agent offers dual emission (`/dup`) so consumers never break on the day of change. The project's own blog gives users a free AI prompt that writes the mapping. Renames are the easy minority of breaks, while merges, value changes and unit changes are not in a rename table. Accurate cross-version checkers already existed (Zhang, Yang, Jin, Sethi, Rodrigues, Lu and Yuan, "Understanding and Detecting Software Upgrade Failures in Distributed Systems," SOSP 2021, DOI 10.1145/3477132.3483577, which found that about two thirds of 123 real upgrade failures came from interaction between two versions), and this program's base rate says they went unused, though I found no outside source for that last point. And three briefs found no one paying for anything near this.

**Answers.**
- *Schema processor.* ANSWERED IN PART. It rewrites data in flight. It does not touch the consumer's code, queries, dashboards or alert rules, which is where Logfire, Laminar, Sentry and SigNoz broke. ClickHouse's engineering resource "OpenTelemetry semantic conventions explained" says the processor "exists in opentelemetry-collector-contrib but remains in development" and that "the dup-mode rollout is the reliable path"; the processor's own README lists its stability as "development" for traces, metrics and logs. If teams standardize on Collector-side translation, the OpenTelemetry market for H2 shrinks to dashboards and queries. I concede that risk.
- *Dual emission.* ANSWERED. Dual emit postpones coping. It does not remove it, and 3.0 retires the old modes for database and code conventions. PR #20373 removes legacy code telemetry "without compatibility shims".
- *The free AI prompt.* ANSWERED, and this is the core argument. The producer recommending an LLM to write the mapping is the clearest dated sign that the coping layer is moving to agents (E7). Laminar PR #2603 shows what that yields: a working fix that still prefers the deprecated name, with nothing checking it against the declared diff. H2 is deterministic, restricted to fields the lock proves are read, and signs a list of what it did not translate. An LLM prompt has none of those three properties.
- *Renames are the easy part.* CONCEDED IN SCOPE. The generator covers declared renames only. Everything else becomes NOT COVERED with a reason code (merge, value change, unit change, conditional rename), for example `code.namespace` plus `code.function` into `code.function.name`, `mssql` into `microsoft.sql_server`, and milliseconds into seconds. This is E8 applied: the honest list is part of the product, not a defect.
- *Zhang et al. and the unused-checker base rate.* ANSWERED IN PART. Those checkers produced findings someone then had to act on. H2 produces the action itself, a file that drops into an existing pipeline, so adoption costs one merge, not a workflow change. It is also not a blocking gate, so the GOV.UK pattern of disabling failing gates does not apply.
- *No willingness to pay.* CONCEDED. The paid market for migrations is real: Moderne's press release of 11 February 2025 announced "$30 million in Series B funding led by Acrew Capital with participation from Morgan Stanley, Amex Ventures, and TIAA Ventures" to automate code migration on OpenRewrite, and Refresh Miami reported that "in total, Moderne has raised $50 million". But that is evidence of a budget for migration in general, not for this artifact. The 14-day test is built to settle this.

## 6. Dated evidence used

| Claim | Named origin, date | Status |
|---|---|---|
| Java agent 2.32.0 is the 3.0 release candidate; 3.0 "targeted for October 2026"; database and code conventions become stable defaults; preview post includes an AI migration prompt | OpenTelemetry blog, 6 October 2026 | FACT |
| Renames in 3.0 include `db.system` to `db.system.name`, value `mssql` to `microsoft.sql_server`, `db.client.connections.max` to `db.client.connection.limit`, milliseconds to seconds, `code.namespace` and `code.function` into `code.function.name` | OpenTelemetry blog, 6 October 2026 | FACT |
| Legacy code telemetry removed "without compatibility shims"; stable database conventions made "the default and only mode" | opentelemetry-java-instrumentation PRs #20373 (3 October 2026) and #20379 | FACT |
| The 3.0 announcement slipped: "the release is being pushed out a month" | opentelemetry.io PR #10833 comment, 21 July 2026 | FACT, and a live instance of E3 |
| No 3.0.0 release found as of 28 October 2026; v3.0.0-tagged issues still opened 20 to 21 October; milestone has no due date | Subagent review of the project issue tracker | FACT (absence of evidence, not proof) |
| GenAI conventions moved to a separate repository in semantic-conventions v1.42.0: "All gen_ai.* attributes, metrics, events, and spans ... are deprecated in this repository and have moved to the OpenTelemetry GenAI semantic conventions repository" | open-telemetry/semantic-conventions release v1.42.0, breaking change #3696, 12 June 2026 | FACT (primary) |
| Collector schema processor exists but "remains in development" | ClickHouse engineering resource, July 2026 | FACT (secondary) |
| Downstream consumers shipped incomplete mappings: Logfire #2561, Laminar #2603 (AI-agent-authored), Sentry #104512, SigNoz #8406 | GitHub, 2025 to 2026, exact dates not captured | FACT |
| Kubernetes 1.37 released 26 August 2026; 1.38 planned 16 December 2026 | Kubernetes blog, 31 July 2026; SIG Release schedule | FACT |
| MCP 2026-07-28 spec adopts a 12-month deprecation window and a deprecated-features registry; Roots, Sampling and Logging removal-eligible on or after 2027-07-28 | MCP specification changelog, 28 July 2026 | FACT |
| More than 40% of GitHub pull requests started with an agent's first draft by September 2026, up from 17% in March | GitHub Octoverse 2026 | FACT |
| EKS extended support costs $0.60 per cluster hour, "inclusive of the $0.10 per cluster per hour you pay for Amazon EKS clusters", a $0.50 surcharge (about $365 more per cluster per month), billed "since April 2024" | AWS Amazon EKS pricing page, as quoted by akalcloud.ai (2026); LiveWyer | FACT (secondary) |
| EU Product Liability Directive 2024/2853 covers software from 9 December 2026 | European Commission transition pathways page | FACT |
| Moderne $30 million Series B led by Acrew Capital, 11 February 2025; $50 million raised in total | Moderne press release, 11 February 2025; Refresh Miami | FACT |
| H2 escapes the detection base rate because its output is an artifact, not a finding | This report | INFERENCE |
| Platform teams will pay for generated coping code | None found | GUESS, unsourced |

## Caveats

**Self-check.** I read the exhaust before the market: the signals table and synthesis came first, and the external searches only tested them. The winner is absent from the known backlog. It is not a consumer lock, not a reader, not outreach or benchmarks or receipt transparency, not PR enforcement, not the judge subcommand or Action, and not the B2 or B1 tracks. Its one overlap is that the existing judge will read its output, which is reuse, not repetition. It passes all five constraints, with the compatibility argument tied to locks, receipts, judge and backtest. The operator can start the morning after reading this, with one dependency: E1 implies INVAR already parses OpenTelemetry schema files. If that parsing exists only as a harvest script, week 1 of the slice must harden it, and that is the main schedule risk.

**Overlap with B2.** The first H2 adopters partly overlap the B2 adopter class (platform engineers). Running both 14-day tests at the same time could blur attribution. The receipt-hash header keeps H2 events countable on their own.

**Where I did not look hard.** I did not look at non-OpenTelemetry coping layers (Terraform provider majors, Kubernetes Helm charts) as H2 inputs, at Grafana dashboard JSON formats as an output target, or at how commercial vendors (Datadog, Elastic, Honeycomb) handle the 3.0 cutover internally.

**Claims I wanted to make but could not source.** Any named price or budget for migration labor on telemetry renames. The exact 3.0.0 release date, which may land days after this report. Exact filing dates for the Logfire, Laminar, Sentry and SigNoz items. And whether the 12 old names in E1 overlap the 3.0 removal set, which INVAR can check from its own corpus faster than I can from outside.