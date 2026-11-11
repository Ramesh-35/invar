# JUDGE-TESTS r3: a `$ref` to a file is read from the schema's own directory (r3)

Drafted 2026-11-08, frozen with the six `t3_` fixtures appended to
crates/invar-falsify/tests/judge_tests_test.rs BEFORE implementation;
freeze record: JUDGE-TESTS-R3-SPEC.frozen. Extends JUDGE-TESTS r2; the
Rust judge is untouched.

## Why

After r2, 70 SchemaStore changes with test documents stay `unverifiable`.
67 of them `$ref` another file: a sibling by relative path
(`eslintrc.json`, `partial-setuptools.json`), an extensionless name
(`tsconfig#/definitions/...`, `feed-1#/...`) or an absolute URL whose last
segment is a sibling (`https://www.schemastore.org/quikrun.json`). Both
validators refuse to resolve anything outside the document (r0 ruling:
"nothing outside a schema document is resolved"), so no document of those
schemas has ever been checked. The referenced file is in the same commit
of the same repository, which is the only place the harness should look.

## Ruling TESTS (r3): hermetic local resolution

A `$ref` whose target is not inside the schema document is resolved from
a siblings directory, read-only:

- the target name is the last path segment of the URI without fragment
  or query; the file is `<siblings>/<name>`, or `<siblings>/<name>.json`
  when the first does not exist and `name` has no `.json` suffix;
- only the siblings directory is read: a name with a path separator is
  reduced to its last segment, nothing above or beside the directory is
  reachable, and nothing is ever fetched from the network;
- a sibling's own `$ref`s are resolved the same way, from the same
  directory;
- the base side resolves from the base siblings directory and the head
  side from the head siblings directory. Default for each is the
  directory of that side's schema file; `--base-siblings DIR` and
  `--head-siblings DIR` override it. `judge_tests.sh` exports the base
  contract's directory from the base ref with `git archive` and passes
  it as `--base-siblings`; the head side is the working tree;
- both validators follow the same rule independently: the Python side
  through a `referencing` retrieve function, the Ajv side through
  `loadSchema` and `compileAsync`;
- a target that is not there stays an error on that side, so the
  document is `not_checked` and the change `unverifiable`, as today.

Classification in `tests.py` is unchanged. No new dependency.

## Fixtures (frozen before implementation)

- t3_f01 relative sibling `$ref`, document goes through it on both sides:
  VIOLATED, both validators, `not_checked` empty.
- t3_f02 absolute URL `$ref` whose last segment is a sibling: VIOLATED.
- t3_f03 extensionless `$ref` finds `common.json`: VIOLATED.
- t3_f04 guard: a missing sibling stays `unverifiable`, no witness.
- t3_f05 a sibling that refers to another sibling: VIOLATED.
- t3_f06 the contract unchanged, the sibling tightened between
  `--base-siblings` and `--head-siblings`: VIOLATED.

## Mutation operators

- TS3-M1 the jsonschema side resolves nothing            -> t3_f01
- TS3-M2 the Ajv side loads no sibling                   -> t3_f01
- TS3-M3 the `.json` fallback dropped, Python side         -> t3_f03
- TS3-M3b the `.json` fallback dropped, Ajv side           -> t3_f03
- TS3-M4 head siblings read from the base directory      -> t3_f06

## Pre-registered prediction

On the 70 `unverifiable` changes left after r2 (the only ones that can
move: a change Ajv compiled had no unresolved reference), each side
resolving against `src/schemas/json` at its own commit: at most 67 move
(the 3 regex-extension changes cannot), and a change moves only to
`instances_pass`, `no_base_valid_instance`, `disputed` or VIOLATED by the
same rules as before. How many land where is measured, not predicted:
no document of theirs has been checked before. Changes whose sibling
`$ref`s point outside the repository (a GitHub raw URL, `djlint.com`,
`unpkg.com`) resolve only if a file of that name happens to sit in the
directory; otherwise they stay. Nothing else in the corpus moves; the
Rust judge's gates do not move.

## Measured outcome (2026-11-08, after implementation)

6 of 6 fixtures pass; TS3-M1, M2, M3, M3b, M4 killed. On the 70 changes
`unverifiable` after r2, each side resolving against `src/schemas/json`
at its own commit (`tests-r3-moves.tsv`):

| r2 reason     | r3 reason        | changes |
|---------------|------------------|--------:|
| unverifiable  | instances_pass   | 62      |
| unverifiable  | unverifiable     | 8       |

`unverifiable` falls from 70 to 8 (113 before r2). No change became
VIOLATED or disputed: every SchemaStore test document behind a sibling
`$ref` is valid at its head in both validators. Nothing else in the
corpus can move, and the Rust judge is untouched.

The 8 left, each stated by the harness: 4 refer to a URL outside the
repository (GitHub release downloads for `declarative-automation-bundles`
and `lychee`, a raw.githubusercontent file for `circleciconfig`, unpkg
for `twee-ts.config`) and are not fetched, by design; 3 use regex
syntax Python `re` lacks (`(?<name>`, `\p{...}`: `global.json`,
`appsettings.json`, `zeropress-wxr-import-base-0.7`); 1 (`djlint`) refers
to `https://djlint.com/schema/djlint.json`, whose last segment is the
file itself under a different URI, and the Python side recurses without
end (the Ajv side does not). That self-reference under a foreign URI is
an r4 candidate (resolve a sibling to the document itself when the bytes
are the document's own); it moves one change and is not built here.
