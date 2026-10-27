# B1: the Program B kill gate (declared data contracts on real history)

Status: r0 RATIFIED 2026-10-27 (B1-D1 to D3 as recommended), frozen
before any searching or measurement. Clock: opened 2026-10-27, search cap
2026-11-01. Parent: docs/PROGRAM-LEDGER.md track B1. This gate decides
whether Program B (the breaking-change firewall for declared data
contracts) earns a single line of product code.

## A. What the gate measures, and why this and not the engine

On declared contracts the engine's verification is deterministic by
construction, so the risk is not detection. The risk is the premise
itself: that enough teams DECLARE data contracts in public, that those
contracts suffer REAL breaking changes, and that the evidence is pinnable.
If the public record cannot produce the cases, the market claim rests on
vendor marketing, and Program B dies here, in days.

## B. Admission rules (one case = one historical breaking change)

A case is admissible only if all five hold, every item pinned by commit:
1. Public repository (license permitting study).
2. A DECLARED contract artifact at the pre-change commit: an ODCS YAML, a
   dbt model contract (`contract: enforced: true` with typed columns), an
   Avro or JSON Schema for a shared table or event, or a
   datacontract-cli file. Prose docs and column lists without enforcement
   semantics do not count.
3. A later commit or merged pull request that breaks it: a promised
   column or field removed or retyped (renames count as remove plus add).
4. A named consumer OUTSIDE the contract's own project boundary (a
   dashboard config, a consuming service, a downstream repository or team's
   model), or impact documented across a team boundary (issue, incident,
   revert). Founder ruling, 2026-10-27: an intra-project dbt break that
   dbt's own build-time enforcement would catch is a refactor, not a case,
   and counts for nothing; INVAR's only valid terrain is the
   cross-repository, cross-team boundary.
5. Both sides' bytes recoverable at pinned commits.

Controls: for every admitted break, one compatible change to the same
contract (column added, description changed) is recorded as a control.

## C. Pre-registered thresholds (the kill rule)

Search effort is capped at 5 days. At the cap:
- FEWER THAN 10 admissible break cases, or FEWER THAN 3 with documented
  downstream impact (4. second form): Program B is recorded dead in the
  ledger with the counts, before any product code.
- 10 or more, with 3 or more impact-documented: B2 (the contract reader
  slice, frozen fixtures first) is authorized, and these cases become its
  corpus, admitted under the same rules as corpus-real (frozen spec,
  pinned bytes, held-out split once the set is large enough).

Secondary record, not a kill condition: how many candidate repositories
declare contracts at all, by kind (ODCS, dbt enforced, Avro/JSON Schema),
as the base-rate evidence for the market claim.

## D. Method

- Discovery: web and code search over public repositories and the
  standard bodies' example lists (Bitol/ODCS, datacontract-cli users, dbt
  projects with enforced contracts, schema registries in monorepos such
  as public data-team repositories).
- Verification: clone at pinned commits; extract the declared promise and
  the post-change artifact; record the broken field, the consumer, and
  the impact evidence. All scripted where possible, human-checked always.
- Everything lands in a worksheet with one row per candidate, admitted or
  rejected WITH the rejection reason, so the search itself is auditable.

## E. Decision points

- B1-D1 (contract kinds in scope). ODCS, dbt enforced contracts, Avro and
  JSON Schema for shared tables and events, datacontract-cli files
  (recommended: these have enforcement semantics). Or ODCS and dbt only
  (purer, but likely too few public cases).
- B1-D2 (consumer requirement). A named consumer OR documented impact
  (recommended: matches how corpus-real admitted cases). Or documented
  impact only (stricter, risks starving the gate).
- B1-D3 (who searches). The sandbox searches and the Mac clones and pins
  (recommended: the sandbox's network cannot clone arbitrary repositories;
  the Mac can, with a provided script). Or Mac-only.

## F. Bounds

No engine change, no product code, no outreach in B1. The gate's output
is counts and a worksheet, recorded in the ledger either way. The 5-day
cap is absolute: an extension is a founder ruling, not a drift.
