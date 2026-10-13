# Experiment B: the stranger's first run on a monorepo (frozen before any data)

Recorded 2026-10-13, before any repository in the sample was examined.
Question: when a stranger runs INVAR cold, with defaults, on a monorepo
that holds both sides of a contract, does it produce a problem-finding
moment? This tests the weak link (Rumelt's crux), not the engine.

## Frame and sample

- Frame: the 24,335 unique GitHub repos from the official MCP registry
  crawl of 2026-10-13 (Experiment A's frame). Disclosed bias: these are
  MCP-adjacent repos, skewing newer, smaller and TypeScript-heavy. No
  general GitHub frame is reachable from the engineer's environment.
- Sample: repos shuffled with random.Random(20261014), depth-1 cloned in
  that order until 30 MONOREPOS are found. Every examined repo is logged
  with its classification, so the monorepo rate is reported.
- Monorepo: at HEAD, any of: pnpm-workspace.yaml; package.json with a
  "workspaces" key; lerna.json; nx.json; turbo.json; go.work; a root
  Cargo.toml with a [workspace] table; rush.json.

## Procedure per monorepo (a stranger's defaults, no hints)

1. `invar init` at the repo root (alpha.28 release build), timed.
2. If it proposes at least one contract: `invar lock`, then `invar check`
   and `invar backtest` (default depth 50), on a copy with history
   (blobless unshallow fetch).
3. Each proposed contract is labeled by hand, from the code: TRUE (the
   consumer file actually reads fields of that producer) or SPURIOUS.
4. Each backtest absence window is verified by hand against the git
   diff before it counts (Experiment A's rule).

## Predictions

- B-P1: init proposes at least one contract in 20% of monorepos
  (plausible range 5 to 35%). The earlier 10-repo audit of single repos
  proposed almost nothing, and monorepos only partly help.
- B-P2: at most half of all proposed contracts are TRUE.
- B-P3: backtest surfaces a verified absence window in 0 to 2 of 30.
- B-P4: init completes in under 60 seconds wall-clock on every monorepo
  (clone time excluded).

## Thresholds (frozen)

- WORKS: at least 10 of 30 monorepos get at least one TRUE proposal,
  AND at least 3 show a verified absence window.
- FAILS: fewer than 3 of 30 get a TRUE proposal, OR zero verified
  absence windows.
- Anything between is PARTIAL, and is reported as exactly that.

Falsified predictions are reported as falsified.
