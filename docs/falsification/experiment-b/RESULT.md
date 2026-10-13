# Experiment B: the stranger's first run on a monorepo. RESULT: FAILS

Method and thresholds: PREDICTIONS.md, frozen before any repository was
examined. Engine: v0.1.0-alpha.28 release build, defaults, no hints.

## Measured

- Frame walk: 466 repos examined (seed 20261014) to find 30 monorepos
  (6.4%): 362 not monorepos, 74 clone errors.
- `invar init` cold on the 30: proposals in 2 of 30 (6.7%); 6 contracts
  total. Slowest init 1.8 s; all completed.
- Hand labels from the code: 1 TRUE, 5 SPURIOUS.
  - TRUE: minipuft/claude-prompts-mcp, framework.schema.json (generated
    from the code's own types by scripts/generate-framework-schemas.ts)
    to an integration test that constructs and asserts on framework
    objects. Caveat: the consumer is a test, not production code.
  - SPURIOUS: the 3 *_builder/schema.json pairings in the same repo (the
    test never touches the builder tools; shared names are domain
    coincidence); ruvnet/claude-flow's 2 pairings to a UI chart, matched
    only on `payload`, which there is the Recharts tooltip prop.
- Backtest over full history on both repos: 0 absence windows across 30
  bindings (all "never absent after introduction").
- DEFECT FOUND: on claude-prompts-mcp the first `invar check` after a
  clean init and lock exited 3. Cause: init emitted duplicate contract
  ids for producers sharing a basename (schema.json in several
  directories); the lock kept one; check reported a mismatch and blamed
  the user's lock. Fixed in v0.1.0-alpha.29 (test 50, proven to fail on
  the unfixed code).

## Against the frozen thresholds

FAILS on both conditions: 1 of 30 monorepos received a TRUE proposal
(FAILS line: fewer than 3), and zero verified absence windows.

## Predictions scored

- B-P1 (20% propose; range 5 to 35%): measured 6.7%. Inside the stated
  range, well below the central estimate.
- B-P2 (at most half TRUE): CONFIRMED (1 of 6).
- B-P3 (0 to 2 absence windows): CONFIRMED (0).
- B-P4 (init under 60 s everywhere): CONFIRMED (max 1.8 s).

## Root causes, from the data

1. Producer discovery finds schema FILES only (JSON Schema, OpenAPI).
   24 of 30 monorepos had zero; modern TypeScript monorepos share
   contracts as code (types and zod schemas in shared packages), which
   the engine can read but init never proposes.
2. Consumer discovery recognizes exactly one access pattern, the literal
   `payload.<field>` (documented in invariant.rs). Real code reads
   `data.x`, `res.body.x`, `user.x`. This is why proposals are rare and
   why the claude-flow pairing matched a chart library's `payload` prop.
3. The two together mean init's first-run experience depends on a
   repository happening to name a variable `payload` near a schema file.

This is the weak link, measured. Not a product question; an init
engineering question with a specific, testable target.
