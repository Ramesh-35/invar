# INIT-DISCOVERY-SLICE3: package "exports" maps, resolved to the source the package declares

Status: DRAFT r1 for ruling. Implementation NOT authorized. Parent:
STAGE3-EVAL-001 S3-F-08 (83 percent of UNRESOLVED is one declared slice 1
limit). Baseline: the clean tree at b79f87a. Evidence measured before a
line of this draft: docs/evidence/slice3/.

## 0. What the evidence says, before any design

An independent prototype of Node's exports algorithm was run over every
one of the 6,394 exports-blocked imports STAGE3-EVAL-001 recorded, at the
pinned SHAs (scripts: simulate_exports.py, simulate_consensus.py).

| Rule | Imports that reach a TypeScript source |
|---|---|
| Node exports resolution alone | 0 of 6,394 (5,925 land on .d.ts, 462 on built JS, 7 match no subpath) |
| + custom conditions `source`, `development` | 433 |
| + the package's own tsconfig rootDir/outDir back-map | 5,971 of 6,394 (93 percent), 0 ambiguous |
| both | 5,971; the custom conditions add nothing |

Per repository under the drafted rule: Understand-Anything 70 of 70, pi
1,902 of 1,902, excalidraw 3,840 of 3,859, puppeteer 128 of 128,
freeCodeCamp 31 of 355, playwright 0 of 80. Sanity: of the 5,971 resolved
files, 2,159 name the imported symbol and 3,812 are `export * from`
barrels (which init already follows one level); 0 do neither.

The lesson that shapes the slice: exports maps in real monorepos point at
BUILD OUTPUT. A faithful Node resolver alone would ship zero yield. The
yield comes from following the package's own declared compiler mapping
from output back to the source it was compiled from.

Bounds, stated before any code:

B1. Resolution answers "which source file declares this export", not
"what Node loads at runtime". The two differ only where condition
branches name different sources, and there init abstains.
B2. No guessing. A target is followed only through what the package
itself declares: its exports map and its own tsconfig compilerOptions. No
dist-to-src naming heuristics, no computed rootDir.
B3. Zero new dependencies, no regex, no AST: string and path operations
plus the serde_json already in the build.
B4. Resolving an import is not a proposal. A proposal still needs the
shape, the barrel rules and two or more distinctive field reads. 5,971 is
the measured ceiling of unblocked imports, never a promised yield;
STAGE3-EVAL-002 measures proposals.
B5. init only. check, lock, backtest, valuepairs and every machine
surface are untouched.

## 1. The algorithm, in exact zero-dependency mechanics

Applied only where codepairs.rs today returns `declares "exports"; not
supported in slice 1`.

1. Subpath: `.` for the bare package name, else `./` plus the rest of the
   specifier after the name.
2. Map normal form: a string or an array becomes `{".": it}`; an object
   with no key starting with `.` becomes `{".": object}` (conditions
   sugar); an object mixing `.` keys and condition keys is invalid.
3. Key match: an exact key wins. Otherwise among keys with exactly one
   `*`, split into prefix and suffix, a key matches iff the subpath starts
   with the prefix, ends with the suffix, and is at least as long as both
   together; the longest prefix wins, ties go to the longer key (Node's
   PATTERN_KEY_COMPARE). Keys with two or more `*` and trailing-`/` folder
   keys never match.
4. Substitution: the captured text replaces EVERY `*` in the target. A
   target that does not start with `./`, or whose capture or result holds
   a `..` or `node_modules` segment, is invalid.
5. Targets: a string is a leaf; an array is fallbacks, its first element
   that yields a valid leaf wins (array order is preserved by any JSON
   parser); `null` excludes the subpath; an object is conditions.
6. Conditions, order-free: every branch whose key is in the active set
   {types, import, require, default} is followed, recursively. serde_json
   here stores objects as sorted maps, so a package's own key order is not
   available; rather than restore it (S3-D2), init follows all active
   branches and requires them to agree (step 9).
7. Each leaf: a `.d.ts` leaf is never a source itself. Any other leaf is
   probed exactly as init already probes (`.js` to `.ts`/`.tsx`, a `.ts`
   or `.tsx` as is, extensionless to `.ts`, `.tsx`, `/index.ts`,
   `/index.tsx`).
8. Back-map: a leaf that is not a source is mapped back through the
   package's own `tsconfig.json` and `tsconfig.build.json`, both read, each
   contributing its pairs: (outDir to rootDir) for `.js` and `.d.ts` leaves,
   (declarationDir to rootDir) for `.d.ts` leaves. rootDir must be
   declared, never computed. Paths are resolved against the tsconfig's
   directory and must stay inside the repository (they may leave the
   package; excalidraw's `rootDir: "../"` does). The leaf's path under the
   output directory is re-rooted under rootDir, then probed (`.d.ts` and
   `.js` to `.ts`/`.tsx`). tsconfig is JSONC: comments outside strings and
   trailing commas are removed by a small masking pass, then serde_json
   parses it; an unparsable tsconfig contributes nothing. `extends` is not
   followed.
9. Consensus: the distinct source files reached by all leaves and all
   back-maps. Exactly one: resolved, and the existing pipeline continues
   unchanged. Two or more: UNRESOLVED naming both, never a pick. None:
   UNRESOLVED with the first reason in sorted order, for example
   `exports target "./dist/x.js" is not a TypeScript source and no declared
   rootDir/outDir pair maps it back; not guessed`.

## A. Frozen scope

A1. One change point: the `exports` branch of codepairs.rs `resolve`.
A2. No new abstention code: every failure is UNRESOLVED with an exact
reason, so the explain registry, completions and SUMMARY grammar stay
byte-identical (S3-D6).
A3. Out of scope, measured and named: tsconfig `paths` aliases (619
OmniRoute abstentions are from valuepairs' resolver, and excalidraw's root
tsconfig declares paths to sources), the valuepairs resolver, `extends`
chains, custom conditions, and playwright's 80 (60 `.d.ts` with no
declared pair, 13 built JS, 7 unmatched subpaths).

## B. Numbered requirements

R-S3-1 The algorithm of section 1, exactly.
R-S3-2 Packages without "exports" resolve byte-identically to today.
R-S3-3 Every non-resolution is UNRESOLVED with a reason naming the
target or the failed step; never a silent drop (M-S-4 stays provable).
R-S3-4 Determinism: two runs byte-identical.
R-S3-5 Zero new dependencies, codes, exit changes or output-format
changes.

## C. Countermodels (defined before fixtures and code)

CM-S3-1 a dist target resolved by a naming guess (dist to src) with no
declared pair. CM-S3-2 the shorter of two matching pattern prefixes
chosen. CM-S3-3 only the first `*` substituted. CM-S3-4 a back-map that
ignores rootDir. CM-S3-5 rootDir computed when absent. CM-S3-6
disagreeing branches resolved to one of them. CM-S3-7 a null-excluded
subpath resolved through another key. CM-S3-8 a `..` capture or target
escaping. CM-S3-9 JSONC comments breaking the tsconfig read. CM-S3-10 a
condition outside the active set picked. CM-S3-11 a committed `.d.ts`
taken as a source. CM-S3-12 declarationDir ignored for a `.d.ts` leaf.
CM-S3-13 a package without exports changing by one byte.

## D. Fixtures (declared outcomes, before any code)

New file tests/init_slice3_test.rs, each a two-package workspace whose
consumer reads two distinctive fields of an imported shape:
S3-F1 `"exports": "./src/index.ts"`: proposal.
S3-F2 exact subpath `"./order": "./src/order.ts"`: proposal.
S3-F3 the outline's pattern, `"./features/*": {"types":
"./dist/features/*.d.ts", "import": "./dist/features/*.js"}` with
tsconfig rootDir `./src`, outDir `./dist`: resolves to
src/features/order.ts, proposal.
S3-F4 `./*` and `./features/*` both match: the longer prefix decides
(the two point at different sources, only the longer one's is a
proposal).
S3-F5 a two-star target, `"./x/*": "./src/*/*.ts"`: both substituted.
S3-F6 a `.d.ts` leaf under declarationDir maps back: proposal.
S3-F7 a dist target with no tsconfig pair: UNRESOLVED, "not guessed".
S3-F8 outDir declared, rootDir absent: UNRESOLVED.
S3-F9 `types` reaching src/a.ts and `import` reaching src/b.ts:
UNRESOLVED naming both, no proposal.
S3-F10 `"./order": null`: UNRESOLVED, excluded.
S3-F11 no subpath matches: UNRESOLVED.
S3-F12 a map mixing `.` and condition keys: UNRESOLVED, invalid map.
S3-F13 a capture with `..`: UNRESOLVED, never a file outside.
S3-F14 a tsconfig with comments and trailing commas: read, proposal.
S3-F15 only `{"browser": ...}`: UNRESOLVED, no active condition.
S3-F16 the pair only in an extended base: UNRESOLVED.
S3-N1 a committed `.d.ts` beside the map: never a source.
S3-N2 a workspace with no exports anywhere: byte-identical golden of
today's init output, recorded at Gate 1 from the unchanged engine.
S3-M1 two runs byte-identical.
Every frozen init suite (slice 1, 2, 2B, 2B-REV1) stays green, unedited,
except slice 1 u06, which is S3-D1.

## E. Mutation set (CAUGHT / SURVIVED / BUILD-FAILED; the last two are not kills)

1 the exports branch reverted to the blanket abstention (S3-F2). 2
shortest prefix wins (S3-F4). 3 only the first `*` substituted (S3-F5).
4 back-map re-roots at the package, ignoring rootDir (S3-F3). 5 rootDir
defaulted when absent (S3-F8). 6 consensus picks the first source
(S3-F9). 7 null falls through (S3-F10). 8 the traversal guard removed
(S3-F13). 9 the JSONC masking removed (S3-F14). 10 any condition accepted
(S3-F15). 11 a `.d.ts` leaf probed as a source (S3-N1). 12 declarationDir
ignored (S3-F6). Each must die by a named assertion failure.

## F. Gates 0 to 5

Gate 0 hash ledger at b79f87a. Gate 1 fixtures first: the F fixtures fail
against the unchanged engine with the slice 1 abstention, the N and M
fixtures pass as controls, the S3-N2 golden is recorded. Gate 2
implementation to codepairs.rs only. Gate 3 the section E harness, all
CAUGHT, control green, from scratch. Gate 4 full suite green, zero
warnings, zero frozen-suite edits beyond S3-D1. Gate 5 ledger here,
STATUS.md entry, then STAGE3-EVAL-002 (S3-D8).

## G. Path classes

Class 1 (untouched, hash-verified): the standing protected set, including
valuepairs.rs, check.rs, init.rs, explain.rs, main.rs, the completions and
both Cargo manifests. Class 2: tests/init_slice3_test.rs, scripts/slice3/.
Class 3 (REQUESTED, not assumed): codepairs.rs, a protected file; and
tests/init_slice1_test.rs u06 only, per S3-D1. Class 4: none.

## H. Decision points for ruling

S3-D1 The frozen fixture u06 asserts the exact abstention this slice
retires (`package a declares "exports"; not supported in slice 1`, for the
map `{"./order": "./order.ts"}`, which now resolves). No design that
resolves exports keeps it passing. Option A (recommended): supersede u06
under this ruling, rewritten to assert the resolution, its original
assertion preserved verbatim in the ledger, with a standing rule: a frozen
fixture that pins a declared scope limit, whose own text names the slice
that limits it, may be superseded only by the slice that lifts that
limit, by explicit ruling. This differs from the R-L-2 case, where the
fixtures pinned a machine interface (the SUMMARY terminator), which is
never superseded. Option B: an opt-in flag (`init --exports`) leaving u06
and default behavior untouched; it keeps the suite frozen but withholds
the yield from every default run.
S3-D2 Order-free consensus over all active branches (recommended) versus
Node's object key order, which would need serde_json's preserve_order
(a new dependency, indexmap, and a reorder of every JSON surface the engine
emits; rejected).
S3-D3 Active conditions {types, import, require, default}, no custom
conditions (measured: source and development add nothing beyond the
back-map).
S3-D4 The tsconfig back-map in scope, reading the package's own
tsconfig.json and tsconfig.build.json; `extends` not followed. It is the
whole yield: 0 to 5,971.
S3-D5 rootDir required, never computed; declarationDir honored for
`.d.ts` leaves.
S3-D6 No new abstention code; disagreement and every failure are
UNRESOLVED with exact reasons.
S3-D7 Class 3 for codepairs.rs.
S3-D8 STAGE3-EVAL-002: re-run phase 1 on the same pinned manifest with
the same thresholds, M-S-1 to M-S-7; every new proposal adjudicated by you
under rubric D; the predicted ceiling (UNRESOLVED-exports falls from 6,394
to 423 or fewer) is recorded before the run; STAGE3-EVAL-001 is never
overwritten.

## I. Stop conditions

Stop and report on: any Class 1 file needing a change; any frozen fixture
other than u06 failing; any new dependency, code, exit or output-format
change; any resolution that needs a guess the package did not declare;
any byte change for a package without exports.

## J. Rulings of 2026-10-19 (recorded)

S3-D1 to S3-D8 RATIFIED as drafted (14:16). S3-D1 Option A: u06
superseded, its original assertion preserved in K3, and the standing rule
adopted: a frozen fixture pinning a declared scope limit by name may be
updated or replaced only by the slice that explicitly lifts that limit.
Class 3 granted for codepairs.rs. Implementation GO under baseline 0866583.

## K. Implementation Evidence Ledger (appended at Gate 5; sections 0 to J are unchanged)

Status: IMPLEMENTED. codepairs.rs is the only engine file changed; the
explain registry, completions, SUMMARY grammar and every other Class 1
file are byte-identical.

### K1. Gate results

| Gate | Result |
|---|---|
| 0 baseline | sandbox equal to the Mac at 0866583, 42-file ledger digest a7e9b200ef01517f on both sides |
| 1 fixtures first | 19 fixtures written before the change; 18 of them and the superseded u06 failed with exactly `package a declares "exports"; not supported in slice 1`; S3-N2 (byte-exact golden of a workspace without exports, recorded from the unchanged engine) passed |
| 2 implementation | codepairs.rs only: the section 1 algorithm, one back-map table per exports package read once |
| 3 mutations | PASS: 12 of 12 CAUGHT by named assertion on the first run, control green, none SURVIVED, BUILD-FAILED or APPLY-FAILED (scripts/slice3/mutate.sh) |
| 4 regression | 284 of 284 tests green, zero warnings; frozen suites unedited except u06 under S3-D1 |
| 5 documentation | sections J and K, STATUS.md entry, STAGE3-EVAL-002 ledger |
| landing | STAGE3-EVAL-002 found 4 silent drops (M-S-4 FAIL), so this slice did not land alone; it lands atomically with INIT-DISCOVERY-SLICE3B (ruling S3B-D6) after STAGE3-EVAL-003 passed. Its 12 mutants were re-run on the final combined tree: 12 of 12 CAUGHT, control green |

### K2. What the gates taught (stated, not hidden)

- Section D called S3-N1 and S3-M1 controls that pass on the unchanged
  engine. They cannot: both assert SLICE3 behavior (the new reason text,
  a back-mapped proposal), so at Gate 1 they failed with the slice 1
  abstention like the F fixtures. Only S3-N2 is a pure invariance control.
  The draft's wording was wrong, not the fixtures.
- The pre-registered ceiling held to the unit: STAGE3-EVAL-002 measured
  423 exports-related UNRESOLVED (from 6,394), exactly the predicted
  bound, because the drafted rule and the pre-draft simulator agree.
- Correct by its own gates, the slice made a pre-existing slice 1 silence
  reachable at scale (S3-F-12): imports that now resolve could fall into a
  silent NotShape. STAGE3-EVAL-002 caught it (M-S-4 FAIL, 4 drops); the
  slice was held, and INIT-DISCOVERY-SLICE3B (earned silence) is the remedy.

### K3. The superseded fixture (S3-D1), original assertion verbatim

tests/init_slice1_test.rs u06_exports_map_unresolved, for the map
`{"./order":"./order.ts"}`:

    &["ABSTAIN UNRESOLVED b/src/x.ts:1 Order from \"a/order\": package a declares \"exports\"; not supported in slice 1"],
    "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE",

Now u06_exports_map_resolved_by_slice3: the same map resolves to
a/order.ts and proposes the pair.
