# INIT-DISCOVERY: init finds contracts where modern code keeps them

Status: DRAFT, rulings R1 to R6 and H1 to H2 recorded (2026-10-13;
R1-R3 directive 48216, R4-R6 directive 58317, H1-H2 directive 64182).
Implementation NOT authorized.
Ground truth: docs/falsification/experiment-c/. Motivated by
Experiment B (docs/falsification/experiment-b/), which measured the
first-run experience on 30 seeded public monorepos: 1 TRUE proposal in
30, because init finds producers only as schema FILES (24 of 30 had
none) and consumers only via the literal `payload.<field>` pattern.

## Goal

On a workspace that shares types across packages, `invar init` proposes
the contracts that actually exist, with evidence a stranger can check in
seconds. Every actionable proposal must expose the evidence supporting
it and must not claim a stronger evidentiary status than the engine has
established. Unsupported relationships must be explicitly abstained from
rather than presented as actionable proposals. Traceable evidence is not
proof: a proposal can satisfy every rule here and still be false, which
the 70% precision target explicitly allows (directive 73814).

## Design

1. Workspace map. Packages come from the markers Experiment B froze
   (pnpm-workspace.yaml, package.json "workspaces", lerna.json, nx.json,
   turbo.json, go.work, Cargo [workspace], rush.json), each with its
   import name (package.json "name", Go module path, crate name). A
   repository is a multi-package workspace only when its markers
   establish at least 2 qualifying packages (H2, directive 64182;
   clarified by directive 73814; evidence in "Workspace evidence"
   below). A marker that names a workspace mechanism is a declaration,
   not proof of membership: the ground-truth audit found 5 of Experiment
   B's 30 "monorepos" carried a marker holding settings only or no
   members. Otherwise init abstains from cross-package discovery and
   states why (step 7). This rule governs init's behavior only; it never
   removes a repository from the frozen evaluation corpus or changes its
   label.
2. Code producers. In each package, exported shapes become producer
   candidates, read by the extraction the checker already uses
   (surface::code_fields): TypeScript `export interface` / `export type
   X = {` / `export type X = Readonly<{` (R4; the audit's lemma fixture
   is mandatory, with positive and boundary tests), Go exported structs,
   Rust `pub struct`, Python classes. Indexed aliases into generated
   types (`type X = components["schemas"]["X"]`) are out of v1 (R5):
   init abstains and states the case, never fabricating a shape.
   NEW WORK: zod `export const XSchema = z.object({...})` extraction
   (the shape Experiment A's research extractor handled), added to
   surface.rs so the checker and init read it identically.
3. Import-graph consumers. A consumer candidate is a file in a DIFFERENT
   package that imports a producer's exported name, resolved by: a
   workspace package import (`from '@scope/shared'`), or a relative path
   into the producer package. Barrel re-exports (`export * from`) are
   followed one level, stated. Variable names play no part. The shape
   must be imported BY NAME (R6): consumers that read an inferred type
   without importing it are out of v1 scope.
4. Evidence rule. Within each consumer, field reads come from
   surface::broad_reads, intersected with the imported type's fields. A
   proposal needs at least 2 overlapping fields that are NOT on the
   generic stoplist (id, name, type, description, data, value, payload,
   key, label, title, status, error, message, result, items, url, path,
   version, created_at, updated_at). Correction of record: this rule
   does NOT alone reject Experiment B's 5 spurious proposals. It rejects
   the 2 Recharts pairings (their only shared field, payload, is
   stoplisted). The 3 builder-schema pairings share 3 to 7 distinctive
   fields and pass it; they are spurious because nothing imports them,
   which step 3 enforces for code producers only; schema-file pairings
   are therefore excluded from actionable proposals (step 6, H1).
5. Legibility. Each proposal in invar.toml carries a comment line with
   its evidence: the producer declaration and package, the consumer
   import and its location, the qualifying shared fields, any fields the
   stoplist rejected, and the consumer's category: production, test,
   example, tooling, or unknown (R2: tests included and labeled; test
   evidence is never presented as production usage).
6. Schema files are INVENTORY ONLY in v1 (H1, directive 64182).
   Schema-file producers have no import to require, so nothing in this
   design can reject a coincidental pairing such as Experiment B's 3
   builder-schema false positives (they share 3 to 7 distinctive fields;
   the field rule passes them). Therefore init lists discovered schema
   files in a separate inventory section and never emits them as
   actionable contract proposals. Inventory entries never count as
   proposals in evaluation, and a schema file excluded here must not
   re-enter through another producer classification. This path is OUT
   OF ACTIONABLE PROPOSAL SCOPE for v1, not solved. The 3 builder-schema
   cases (minipuft/claude-prompts-mcp, *_builder/schema.json) are
   regression fixtures. Cross-repository contracts are untouched. Ids
   follow alpha.29's uniqueness rule.
7. Outcome classes (directive 73814). Init's output keeps three classes
   in separate sections, and each item gets exactly one class, once:
   - Actionable proposal: a contract written to invar.toml, asserting a
     candidate relationship, with the step 5 evidence. Created only by
     the code-producer path (steps 2 to 4).
   - Explicit abstention: a stated decision not to propose, with a reason
     code and location: H2 (fewer than 2 qualifying packages, or
     unsupported marker evidence), R5 (indexed alias into generated
     types), R6 (shape read but not imported by name), or unsupported
     syntax. Never written to invar.toml.
   - Inventory-only observation: a file or structure recorded with no
     asserted relationship (schema files, H1). Never written to
     invar.toml.
   No later step may turn an abstention or an inventory item into a
   proposal, and a field match is never sufficient on its own: a
   proposal also needs the step 3 named import (R3, R6).

## Workspace evidence (H2)

A qualifying package is a distinct directory that (a) is named as a
member by an explicit entry in a membership marker below, after globs
and exclusions are resolved, and (b) holds the manifest that marker
requires, with an import name. Nothing else makes a package: not an
undeclared directory with a manifest, not an orchestration file, not a
default a tool would apply when the entry is absent. Markers combine by
union of qualifying directories.

| Marker | Observable evidence that qualifies a member | Root directory |
|---|---|---|
| package.json "workspaces" | array of globs, or object with a `packages` array; match holds package.json with "name" | not counted |
| pnpm-workspace.yaml | `packages:` list of globs, `!` exclusions applied; match holds package.json with "name" | not counted (pnpm includes it implicitly; implicit is not declared) |
| lerna.json | explicit "packages" array; match holds package.json with "name" | not counted; no "packages" key is no evidence |
| go.work | `use` directives, single or block; directory holds go.mod with a module path | counted only via `use .` |
| rush.json | "projects" entries; each projectFolder holds package.json with "name" | not counted |
| Cargo.toml [workspace] | `members` globs minus `exclude`; match holds Cargo.toml with [package] | counted when the same Cargo.toml declares [package] (the root package) |
| nx.json, turbo.json | never membership evidence; they orchestrate | n/a |

Abstain, stating the reason, rather than infer membership: glob syntax
beyond `*`, `**`, `?` and leading `!`; Nx projects defined only by
project.json; Cargo members implied only by path dependencies; a marker
that cannot be parsed.

Boundary cases:
- minipuft/claude-prompts-mcp: package.json "workspaces": ["cli"] gives
  1 qualifying package (server/ holds a package.json but is undeclared):
  not a multi-package workspace; init abstains (H2).
- pnpm-workspace.yaml holding settings only, no `packages:`: 0 members.
- Cargo [package] plus [workspace] with no members: 1 package (the root).
- A glob matching directories without the required manifest: those
  directories do not count.
- Two entries resolving to the same directory: 1 package.
- turbo.json or nx.json beside package.json "workspaces": decided on the
  package.json evidence alone.

## Hard constraints

Deterministic (same tree, same manifest, byte for byte); zero network;
no model calls; zero new dependencies unless ruling R1 says otherwise.
Init stays a proposal tool: it writes invar.toml only, never a lock. Its
latency is a measured budget per release, not a promise.

## Honest scope (stated in init's output)

Pattern matching, not type checking: object type aliases only in the
forms listed in step 2; no generics resolution, indexed or intersection
aliases, or inference; broad_reads is not scoped to the imported type's
variables (the evidence rule compensates, imperfectly). A missed
contract is a stated limit; a spurious one is the failure to minimize.

## Rulings (recorded)

- R1 APPROVED: pattern-based extraction, zero new dependencies, no AST
  parser, no model calls. Not to be described as AST-aware. A parser
  dependency would need its own architectural ruling.
- R2 APPROVED: test consumers included and labeled; no default test
  exclusion; ambiguous classification kept only if all other evidence
  holds, labeled unknown.
- R3 APPROVED: at least 2 shared fields surviving the frozen stoplist,
  plus producer-import evidence; field overlap alone is never enough.
- R4 APPROVED: `export type X = Readonly<{...}>` producers.
- R5 OUT OF V1: indexed aliases into generated types; abstain and state.
- R6 OUT OF V1: inferred-type consumers; named import required.
- H1 APPROVED (option a): schema files are inventory-only in v1.
- H2 APPROVED: a repository is a multi-package workspace only when its
  markers establish at least 2 qualifying packages ("Workspace
  evidence"; clarified directive 73814); init behavior only, evaluation
  frame unchanged.
- Outcome classes (directive 73814): abstentions and inventory-only
  observations are not proposals (step 7).

## Acceptance

Frozen in docs/falsification/experiment-c/EVALUATION-PROTOCOL.md
(corpus, labels, definition, stoplist, metrics and thresholds). Not to
be changed after engine results are seen; errors are recorded as
protocol deviations.
