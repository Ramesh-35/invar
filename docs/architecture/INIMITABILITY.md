# Inimitability Constraints (founder directive, 2026-10-07)

> We do not compete on feature parity. INVAR wins only where a rival would have
> to rebuild its core engine to match it. Each constraint below is binding on
> future work, and each carries its true status today. A constraint is a
> design obligation, not a claim that it already holds.

Governing laws: [M1 Adaptive Rivalry](laws/M1.md),
[M2 Structural Evolution](laws/M2.md), [Simplicity](SIMPLICITY.md). Adequacy
stays NOT ESTABLISHED (docs/epistemic/MATHEMATICAL-ADEQUACY.md) until real
incidents establish it; nothing here changes that.

Status vocabulary: **HOLDS** (verified, evidence named), **PARTIAL** (some of
it holds; the gap is named), **TARGET** (not built).

## I. Bitemporal differential residue

Obligation: evaluation keeps transaction time (`t_r`, when an artifact or
verdict was recorded) distinct from valid time (`t_v`, when it was true in the
world), and every result isolates the residue `R = Δ(producer, consumer)`
with BLAKE3 provenance.

| Part | Status | Evidence / gap |
|---|---|---|
| Bitemporal ledger (`t_v`/`t_r`, lineage hash, as-of replay) | HOLDS (synthetic) | `ledger.rs`; tests 4-8 |
| Temporal admissibility in the evaluator (`admit_temporal`) | HOLDS (synthetic) | test 13 |
| Residue isolated as a set in `invar check` | HOLDS | REAL-INCIDENT-001, runner runs 2026-10-07 |
| BLAKE3 provenance for remote artifacts | HOLDS | runner runs; local files carry no hash yet (PARTIAL for local) |
| `invar check` is bitemporal | TARGET | `check` evaluates artifacts as they are now; it writes no ledger entry and has no as-of. "Bitemporal causality" is not evaluated by `check` today |

## II. Three contract surfaces

Obligation: schema, event and lock-order contracts are first-class in every
layer (CLI, resolver, CI runner, PR report). "Irreducible" is a hypothesis to
be tested against real incidents, not an established fact.

| Layer | schema | event | lock_order |
|---|---|---|---|
| Engine (`check_contract`) | HOLDS | HOLDS (synthetic, test 19) | HOLDS (synthetic, test 20) |
| `invar check` dispatch and remote resolution | HOLDS | HOLDS (synthetic) | HOLDS (synthetic) |
| PR report | HOLDS | HOLDS (same renderer) | HOLDS (same renderer) |
| `invar init` discovery | PARTIAL (JSON Schema, OpenAPI) | TARGET (AsyncAPI inventoried only) | TARGET (no trace source) |
| Real-world evidence | one constructed incident | none | none |

## III. Zero-friction mechanics (Simplicity)

| Obligation | Status | Evidence / gap |
|---|---|---|
| No new crate dependencies; `--locked` always passes | HOLDS | CI and release use `--locked` |
| Single binary under 15 MB | HOLDS | about 4.1 MB unstripped (glibc build); release tarballs 1.5-1.8 MB |
| No runtime dependencies | PARTIAL | musl binary is static (release check), but `--ci` remote fetch shells out to `curl` (founder decision 2026-10-06) |
| Verification in under 10 s with no hand-written config | TARGET | `invar init` exists and emits a runnable manifest, but its proposals are heuristic and need review; time-to-first-check is unmeasured |

## IV. PR action as the acquisition loop

| Obligation | Status | Evidence / gap |
|---|---|---|
| On violation, post a residue table; refresh the same comment, never spam | HOLDS | live on `invar-action-smoke` PR #1, 2026-10-07/08: one comment posted, then refreshed to ALL HOLD and back to VIOLATED, never duplicated (STATUS.md) |
| Ids and residue always escaped inline code (no mentions, links, HTML; pipes escaped) | HOLDS | `md_code`, test 23 |
| Errors never shown as verdicts | HOLDS | exit 3, runner run 37590676958 |

## What this directive does not license

- No "moat" or "cannot be copied" claim in user-facing text until M1 is met:
  INVAR out-detects a fairly strengthened rival on real incidents.
- No constraint above may be marked HOLDS without named evidence.
