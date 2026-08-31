# Falsification Methodology

## The gate

INVAR's category claim is treated as **false until falsified in the other
direction**. The instrument tries to catch every corpus failure with the
composite baseline. A case only supports INVAR's thesis when:

1. every **required** composite leg actually **executed** (PASS or FAIL), and
2. no executed leg **isolated** the designated failure (no finding matched the
   case's `catch_signature`), and
3. the result is reproducible and attributable.

If all three hold, the case is `PROVEN_RESIDUE` — and on FIXTURE evidence, that
is a **candidate** only. If any required leg did not execute, the case is
`UNKNOWN`. If an executed leg isolated the failure, the case is
`CAUGHT_BY_COMPOSITE`.

A case is **never** promoted to residue because its `failure_type` sounds like
something an invariant substrate "should" own. Residue is earned by a
fully-exercised composite failing, never assigned by association.

## Required-legs table

The required leg(s) define what "the composite fully ran" means for a case. A
missing required leg forces `UNKNOWN`.

| `failure_type` | Required leg(s) | Rationale |
|---|---|---|
| `AST_SCHEMA_DESYNC` | `semgrep` | Static/source analysis is the fair composite tool for a source-shaped defect. |
| `CONCURRENCY_INVARIANT` | `opentelemetry` | Only observable at runtime under interleaving. |
| `RUNTIME_STATE_LEAK` | `opentelemetry` | Only observable at runtime across requests. |
| `TEMPORAL_DRIFT` | `opentelemetry` | Only observable at runtime across clocks. |

Consequence offline: the OpenTelemetry leg cannot execute without a live
collector and workload, so TC-002/003/004 resolve `UNKNOWN` — honestly, because
we never gave the composite its fair chance, not because it failed.

## Tool availability is a first-class outcome

If Semgrep is not installed, TC-001 and TC-005 resolve `UNKNOWN` (their required
`semgrep` leg is `SKIPPED`). That is a **tool-availability gap**, reported as
such — not forced into a verdict. Install Semgrep (`pip install semgrep`) to let
the static leg genuinely execute.

## Promotion path (Phase 2)

A FIXTURE residue candidate is promoted toward evidence only by:

1. authoring a minimal cross-repository differential probe that demonstrates the
   INVAR primitive isolating the failure the composite missed, **and**
2. sourcing at least one `REAL` cross-repository incident exhibiting the same
   shape, **and**
3. showing the result is reproducible and attributable to that primitive.

Until then, the honest ceiling is `CATEGORY_RESIDUE_CANDIDATE`.
