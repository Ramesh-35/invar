# Architecture Overview (Phase 1)

## Authorized vs deferred

Only `crates/invar-falsify` is authorized to build in Phase 1. Everything else
is a placeholder without a build system (DR-003). This is deliberate: the
instrument must exist and run before any substrate is written, so that the
substrate is built against evidence rather than speculation.

```
invar-falsify
├── manifest.rs   corpus types; loads/validates corpus_manifest.json
├── config.rs     hand parser for the flat composite.toml (no `toml` crate)
├── evaluator.rs  runs each composite leg; classifies a case (the skeptic)
├── ledger.rs     bi-temporal, BLAKE3-chained lineage over SQLite (WAL)
├── report.rs     renders run_results.json + EPISTEMIC_REPORT.md
└── main.rs       clap CLI: run | validate | report | verify
```

## The NEXTLOGIC integration boundary

**There is no runtime coupling between INVAR and NEXTLOGIC.** INVAR reads only
its own corpus and configuration. It does not import NEXTLOGIC source, call
NEXTLOGIC services, or share a database. A future consumer interface, if any,
will be explicit and versioned (DR-001). This boundary is an architectural
invariant, not an aspiration.

## Bi-temporal semantic invariants (defined BEFORE storage)

The ledger's meaning is fixed before its schema, so storage serves semantics and
not the other way around (DR-005):

- **Valid time `t_v`** — when a fact was true in the modelled world.
- **Transaction time `t_r`** — when we recorded it.
- These are **distinct columns** and are never collapsed. A fixture verdict
  produced at record time honestly sets `t_v == t_r` (via `append_now`); a fact
  with an independent event time uses `append(t_v, …)`.
- **State identity:** `state_hash = BLAKE3(content)`.
- **Chain identity:** `lineage_hash = BLAKE3(prev_lineage ++ state_hash ++ context)`
  where `context = "{kind}|{subject}|{t_v}|{t_r}"`.
- **Verification is recomputation.** `verify()` re-walks rows in id order and
  recomputes both hashes; it never trusts a stored "valid" flag. A silent edit
  to any committed row is caught as a `StateMismatch` (or `LineageBroken`) at the
  offending id.

## Determinism

Given the same manifest, composite config, and tool availability, a run produces
the same verdicts. The run artifact (`run_results.json`) is hashed with BLAKE3
to give each run a replicability fingerprint printed in the report.

## Dependency discipline

`clap`, `serde`, `serde_json`, `blake3`, `rusqlite` (bundled), `thiserror`,
`anyhow`. No `toml` crate — the composite format is a small, explicit,
hand-parsed subset. New primitives (tree-sitter, LSP, HKDF, Merkle-DAG, CUSUM)
are introduced only when a specific falsifiable requirement needs them, never
because they appear in a specification.
