# INIT-DISCOVERY slice 1: contract, implementation, evidence

Status: IMPLEMENTED (directive 86427, 2026-10-14). Detection capability
NOT ESTABLISHED. The Experiment C evaluation has not been run and is not
part of slice 1. Parent spec: INIT-DISCOVERY.md. Evaluation authority:
docs/falsification/experiment-c/ (unchanged).

## What slice 1 does

`invar init` proposes a contract when a TypeScript shape exported by one
qualifying workspace package is imported BY NAME in a file of another,
and that file reads at least 2 of the shape's DIRECT members by property
access, after the frozen stoplist. Each proposal states:
`status: PROPOSED (heuristic; receiver of field reads not verified)`.

A proposal is a candidate relationship. Traceable evidence is not proof:
the reads may be on another value (fixture N10 pins this), and the
precision target of 70% allows false proposals.

Out of slice 1: zod extraction; Go, Rust, Python; lerna, rush, go.work,
Cargo markers (SLICE-SCOPE abstention); receiver analysis; any change to
check, lock or backtest.

## Behavior change of `invar init` (D3)

Before slice 1, init's only proposals were schema-file pairings. Now
schema files (JSON Schema, OpenAPI-JSON components) are inventory only
(`INVENTORY schema ...`) and never proposed. `check`, `lock` and
`backtest` behave exactly as before on any invar.toml.

## Output (C1): three classes plus a summary, all in invar.toml

- Proposal: comment lines `# PROPOSED <id>`, the status line, E1 to E6,
  `# category: ...`, `# lock: ...`, then `[[invariant]]` with
  `kind = "schema"`, id, producer (the declaring file, never a barrel),
  consumer. The only `[[invariant]]` blocks are proposals.
- Abstention: `# ABSTAIN <code> <location> <reason>`, sorted, never
  capped. Not a negative finding.
- Inventory: `# INVENTORY <kind> <location>  <note>`, sorted, display
  capped at 40 (count exact).
- Last line: `# SUMMARY packages=N candidates=N proposals=N abstentions=N
  inventory=N status=COMPLETE|TRUNCATED|ABSTAINED` (also printed).

Evidence per proposal: E1 producer package (package.json, marker entry);
E2 shape declaration file:line; E3 consumer package; E4 import file:line,
specifier, resolved file, barrel file:line if one was followed; E5
fields read (first file:line each) and stoplisted reads; E6 the pair.

Identity: one proposal per (declaring file, consumer file); base id
`code:<producer basename>-><consumer basename>`, alpha.29 `#n` suffix on
collision in byte order of (producer, consumer).

Failure levels: run errors keep today's behavior (C2: exit 1, `error:`,
no file written); repository abstentions (H2, SLICE-SCOPE) and item
abstentions exit 0 with the summary present.

## Abstention codes

| Code | When |
|---|---|
| H2 | marker unparsable, or fewer than 2 qualifying packages (repository level) |
| SLICE-SCOPE | only lerna.json, rush.json, go.work or Cargo [workspace] present |
| H2-ENTRY | a marker entry with unsupported glob syntax or leaving the root |
| H2-MEMBER | a declared member with invalid package.json, no "name", or under a skipped directory (build/, dist/, ...) |
| R5 | named import resolves to an indexed alias into generated types |
| R6 | file imports from package P and reads 2 or more distinctive fields of an exported shape of P it does not import by name |
| UNSUPPORTED-SHAPE (C3) | named import resolves to a type alias that is not an object body (z.infer, Pick, Omit, unions, intersections, plain aliases) |
| BARREL2 | the shape is reachable only through more than one barrel level |
| AMBIG-SHAPE | the import resolves to more than one declaration |
| AMBIG-PACKAGE | two qualifying packages share the imported name |
| UNRESOLVED (C4) | no file for the specifier; package "exports" map; entry not a TS source (dist/index.js) |
| UNREADABLE (C5) | a .ts/.tsx file in a qualifying package that is not UTF-8 or exceeds 1 MB |
| UNREPRESENTABLE | the pair's paths cannot be written in flat TOML |

## Membership (H2 rows 1 and 2) and resolution (C4)

Only root markers. package.json "workspaces" (array, or object with a
`packages` array) and pnpm-workspace.yaml `packages` (read with the
existing yaml.rs reader; block and flow lists both accepted, see
deviations). Globs `*`, `**`, `?`, leading `!`; `*` never matches a
dot-name; node_modules and .git never traversed; symlinks not followed.
A member qualifies with a valid package.json and a non-empty "name"; the
root never counts.

Bare `name`: package.json "types", "typings", then "main", when it names
a .ts/.tsx (not .d.ts) file; with none of them, index.ts/index.tsx;
otherwise UNRESOLVED. `name/sub` and relative specifiers probe `.ts`,
`.tsx`, `/index.ts`, `/index.tsx`; a `.js` suffix maps to `.ts`/`.tsx`.
Path aliases (`@/...`) are not candidates and are not counted: a stated
silent limit.

## Implementation

- NEW src/codepairs.rs: masking of strings and comments, bracket-depth
  shape and member extraction, import and re-export scan, membership,
  resolution, one-level barrels, property reads, stoplist, pairing,
  abstentions.
- src/init.rs: schema files to inventory; code pairs to proposals;
  rendering; summary. src/main.rs: init's printed lines only. src/lib.rs:
  module line.
- Unchanged (sha256 equal to 2f5b542): surface.rs, check.rs, lock.rs,
  backtest.rs, invariant.rs, config.rs, yaml.rs, Cargo.toml, Cargo.lock.

## Evidence (2026-10-14)

| Gate | Result | Classification |
|---|---|---|
| Fixtures (tests/init_slice1_test.rs) | 37 of 37 pass; each runs init twice, outputs byte-identical (D01); every proposal checked for E1 to E6, status and lock note | MECHANIZED |
| Mutations (scripts/slice1/mutate.sh) | control passes; 13 of 13 mutations build and are caught by their named fixtures | ADVERSARIALLY VERIFIED (on these fixtures) |
| Full suite | 90 of 90 (3 unit, 50 harness, 37 slice 1); zero warnings | MECHANIZED |
| F01, release binary | easyeda-copilot @ fe7b2a5: init exit 0 (6 proposals incl. the F01 pair with exactly its 7 fields; mcp-client.ts not paired with preview.ts), lock exit 0, check exit 0 | PRODUCTION-REACHABLE (one exposed case) |
| Detection on real repositories | not measured | NOT ESTABLISHED |

## Deviations and corrections, recorded

1. pnpm-workspace.yaml flow lists (`packages: [a, b]`) are accepted, as
   yaml.rs parses them explicitly; revision 2 said abstain. Not guessing,
   but a deviation from the written contract. Checked once in a scratch
   run (a flow list of two packages yields the expected pair); there is
   no committed fixture for it.
2. init_discovers_and_emits_runnable_manifest: its manifest now has no
   blocks, and `check`'s unchanged behavior on a manifest without blocks
   is an error ("has no [[invariant]] blocks"). Revision 2 said the
   manifest "must still load in invar check"; it cannot, by check's
   existing rule. The test asserts that error.
3. Fixture m01's expected serde message was predicted as column 16; the
   real message says column 17 (17 bytes). Prediction corrected, not the
   code.
4. A defect found by the fixtures and fixed before acceptance: R6 fired
   for a shape imported by name whose import did not resolve.

## Limitations

- D1: receivers not verified; N10 is a known false-positive pattern.
- Lock binds wider than the evidence (recorded, unchanged): on F01, the
  preview pair's 7 evidence fields became 13 lock bindings, including
  unscoped `mode`, `x` and `renderer` from nested and sibling types.
- Without a lock, `invar check` on a slice 1 proposal is a vacuous HOLDS
  (D6 a: every proposal says to lock first; check unchanged).
- A manifest with zero proposals makes `invar check` error (pre-existing).
- UNSUPPORTED-SHAPE is common: on F01, 73 of 79 abstentions, mostly
  `z.infer` aliases (zod is a non-goal).
- Masking does not model regex literals or `${}` inside template
  strings; path aliases, namespace imports and default imports never
  produce proposals.
