# INIT-DISCOVERY ground-truth audit: definition (frozen before any repo is opened)

Frozen 2026-10-13, before any of the 30 repositories was re-opened for
this audit. Corpus: the 30 monorepos of Experiment B (seed 20261014,
docs/falsification/experiment-b/b1-summary.tsv), unchanged.

## Qualifying cross-package contract (the unit of ground truth)

A relationship (P, S, F, C) qualifies when ALL hold:
1. P and C are distinct packages of the same workspace, as declared by
   the frozen markers (pnpm-workspace.yaml, package.json "workspaces",
   lerna.json, nx.json, turbo.json, go.work, Cargo [workspace],
   rush.json), resolved to directories holding package.json, go.mod or
   Cargo.toml.
2. S is a named data shape exported by P: a TypeScript interface or
   object type alias, a zod object schema, a Go exported struct, a Rust
   pub struct, or a Python class with typed attributes.
3. A non-generated source file F in C imports S by name, either through
   P's package name or a relative path into P (directly, or through one
   barrel re-export).
4. F reads, by property access on values of type S, at least 2 of S's
   fields that are NOT on the stoplist. Verified by a human reading F,
   not by pattern match.

## Stoplist (frozen; identical to INIT-DISCOVERY step 4)

id, name, type, description, data, value, payload, key, label, title,
status, error, message, result, items, url, path, version, created_at,
updated_at. Matching is by the engine's naming form (case and _/-
insignificant).

## Labels

- Per relationship: TRUE (all 4 hold), or the first condition it fails.
- Per repository: POSITIVE (at least one TRUE relationship), NEGATIVE
  (no relationship satisfies 1 to 3, or every candidate fails 4),
  AMBIGUOUS (a candidate exists but 4 cannot be determined confidently;
  the reason is recorded). A NEGATIVE repository is "no ground-truth
  opportunity" and can never count as an engine false negative.
- Test files: F may be a test; the relationship is TRUE and labeled
  "test" (ruling R2).

## Procedure

1. Re-clone each repository at depth 1 (same default branch HEAD as
   available today; the commit is recorded).
2. A script lists packages and EVERY cross-package import statement,
   with no field filtering (to keep the audit independent of the
   engine's heuristic).
3. For each import of a named shape, the engineer reads P's declaration
   and F's usage and labels it. Repositories are labeled POSITIVE as soon
   as one TRUE relationship is verified; NEGATIVE requires every
   shape-import candidate to be reviewed.
4. Ambiguous cases are resolved by this definition as written; the
   definition is not changed during the audit.

## Limitation stated up front

Condition 4 mirrors the engine's own contract concept (consumers read
fields). That alignment is deliberate, because INIT-DISCOVERY can only
claim contracts of that kind; it means the audit measures discovery of
field-reading contracts, not every possible coupling between packages.
