# INIT-DISCOVERY: init finds contracts where modern code keeps them

Status: DRAFT for founder ratification (2026-10-13). Motivated by
Experiment B (docs/falsification/experiment-b/), which measured the
first-run experience on 30 seeded public monorepos: 1 TRUE proposal in
30, because init finds producers only as schema FILES (24 of 30 had
none) and consumers only via the literal `payload.<field>` pattern.

## Goal (one sentence)

On a workspace that shares types across packages, `invar init` proposes
the contracts that actually exist, with evidence a stranger can check in
seconds, and proposes nothing it cannot back.

## Design

1. Workspace map. Packages come from the markers Experiment B froze
   (pnpm-workspace.yaml, package.json "workspaces", lerna.json, nx.json,
   turbo.json, go.work, Cargo [workspace], rush.json), each with its
   import name (package.json "name", Go module path, crate name).
2. Code producers. In each package, exported shapes become producer
   candidates, read by the extraction the checker already uses
   (surface::code_fields): TypeScript `export interface` / `export type
   X = {`, Go exported structs, Rust `pub struct`, Python classes.
   NEW WORK: zod `export const XSchema = z.object({...})` extraction
   (the shape Experiment A's research extractor handled), added to
   surface.rs so the checker and init read it identically.
3. Import-graph consumers. A consumer candidate is a file in a DIFFERENT
   package that imports a producer's exported name, resolved by: a
   workspace package import (`from '@scope/shared'`), or a relative path
   into the producer package. Barrel re-exports (`export * from`) are
   followed one level, stated. Variable names play no part.
4. Evidence rule. Within each consumer, field reads come from
   surface::broad_reads, intersected with the imported type's fields. A
   proposal needs at least 2 overlapping fields that are NOT on the
   generic stoplist (id, name, type, description, data, value, payload,
   key, label, title, status, error, message, result, items, url, path,
   version, created_at, updated_at). This rule alone would have rejected
   all 5 spurious proposals in Experiment B.
5. Legibility. Each proposal in invar.toml carries a comment line with
   its evidence: the import line, the overlapping fields, and "test" when
   the consumer is a test file.
6. Unchanged paths. Schema-file discovery remains, now also subject to
   the evidence rule (step 4). Cross-repository contracts are untouched.
   Ids follow alpha.29's uniqueness rule.

## Hard constraints

Deterministic (same tree, same manifest, byte for byte); zero network;
no model calls; zero new dependencies unless ruling R1 says otherwise.
Init stays a proposal tool: it writes invar.toml only, never a lock. Its
latency is a measured budget per release, not a promise.

## Honest scope (stated in init's output)

Pattern matching, not type checking: no generics, aliases, intersection
types, or inference; broad_reads is not scoped to the imported type's
variables (the evidence rule compensates, imperfectly). A missed
contract is a stated limit; a spurious one is the failure to minimize.

## Rulings needed

- R1. Pattern-based extraction with zero new dependencies (recommended;
  consistent with surface.rs today), or a real parser (tree-sitter or
  swc), which revokes the zero-dependency doctrine for init.
- R2. Test files as consumers: include and label (recommended;
  Experiment B's only TRUE proposal was a test), or exclude by default.
- R3. The evidence rule: minimum 2 distinctive fields with the stoplist
  above (recommended), or another threshold.

## Acceptance: Experiment B re-run (thresholds frozen at ratification)

Same seed (20261014), same 30 monorepos, same procedure. Step zero,
before any build: hand-audit the 30 for GROUND TRUTH (how many actually
share types across packages), so a low score separates "engine misses"
from "nothing to find". Targets to freeze then, proposed now: TRUE
proposals in at least 60% of the ground-truth repos, and precision of at
least 70% across all proposals. Predictions recorded before the build.
