# INVAR Platform

**INVAR** is the *Executable Invariant Control Substrate* (EICS) — an independent
platform whose category claim is **not yet proven** and is currently under
falsification.

> INVAR is a standalone product/platform. **INVAR is not NEXTLOGIC.** NEXTLOGIC is
> a separate enterprise decision-intelligence platform that may one day *consume*
> INVAR through explicit interfaces, but INVAR's source, history, build, tests,
> and release lifecycle are fully independent. See [`INVAR.md`](INVAR.md) and
> [`docs/decisions/DR-001-repository-independence.md`](docs/decisions/DR-001-repository-independence.md).

## Epistemic status

This repository is in **Phase 1: Falsification**. Before any production runtime
claim is made, INVAR must establish whether an *irreducible category residue*
exists beyond a realistic composite baseline of **Semgrep + OPA + OpenTelemetry
+ Property Testing**. See [`docs/epistemic/STATUS.md`](docs/epistemic/STATUS.md).

Every forward-looking statement in this repository is labelled:

- `[PROVEN]` — demonstrated *in this repository*, reproducibly.
- `[HYPOTHESIS]` — a testable proposition, not yet tested.
- `[TARGET]` — a desired outcome, no evidence yet.
- `[UNVERIFIED]` — asserted elsewhere, not established here.
- `[STRATEGIC HYPOTHESIS]` — a business/positioning bet, not a technical fact.

Nothing in this repository claims performance, security, correctness, or
category superiority as fact.

## What exists today

| Component | State |
|---|---|
| `crates/invar-falsify` | **Authorized** — the Phase-1 falsification instrument (`invar-falsify 0.1.0-alpha`). |
| `corpus/` | Five **FIXTURE**-tier failure cases (TC-001 … TC-005). |
| `rules/` | Local, experimental Semgrep + OPA instruments (not enterprise detection). |
| `crates/invar-core`, `crates/invar-lsp`, `crates/invar-cusum`, `packages/invar-isv` | **DEFERRED** — placeholders only, no build system, not workspace members. |

## Quick start

```sh
# Validate the corpus manifest
cargo run -p invar-falsify -- validate --manifest corpus_manifest.json

# Run the falsification harness (writes runs/latest/{run_results.json,EPISTEMIC_REPORT.md,ledger.db})
cargo run -p invar-falsify -- run \
  --manifest corpus_manifest.json --composite composite.toml --output runs/latest

# Re-verify the bi-temporal ledger chain
cargo run -p invar-falsify -- verify --ledger runs/latest/ledger.db
```

The static (Semgrep) leg only *executes* when Semgrep is installed
(`pip install semgrep`). When a tool is absent, its leg is reported `SKIPPED`,
which is **never** the same as PASS — cases that depend on a skipped leg are
classified `UNKNOWN`, not residue.

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT),
at your option. See [`LICENSE`](LICENSE).
