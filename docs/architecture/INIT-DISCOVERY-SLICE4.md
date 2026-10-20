# INIT-DISCOVERY-SLICE4: bounded barrel chains

Status: LANDED (2026-10-20) together with INIT-DISCOVERY-SLICE5 and
LOCK-SLICE1, after STAGE3-EVAL-005 closed on all seven metrics. Before
that: STAGE3-EVAL-004 failed M-S-3 (16 of 29), and by ruling S4-D8 the
engine was held as docs/evidence/slice4/slice4-implementation.patch until
the receiver-verification slice could land with it (sections J and K).
Sections 0 to I are the draft r1 text, unchanged. Parent:
STAGE3-EVAL-003 (BARREL2 and unreached-type abstentions) and the founder's
choice of 2026-10-19 (barrel depth before tsconfig paths, on measured
yield). Baseline: the clean tree at aa03983.

## 0. Evidence, measured before this draft (throwaway prototypes)

Two candidate next slices were prototyped on the STAGE3 pinned repositories
(docs/evidence/next-slice/).

| Candidate | New proposals | Other effect |
|---|---|---|
| tsconfig `paths` in egress discovery | 0 (OmniRoute 6 to 6) | 619 generic abstentions become 79 specific ones; 4 newly reachable silences are unearned, so it needs earned silence in egress discovery too |
| barrel chains to depth 5, plus `export type *` | +29 (pi 113 to 139, puppeteer 5 to 8) | 118 BARREL2 and 13 unreached-type abstentions resolved; 0 ambiguous; deterministic |

Barrel depth alone gives pi +26; `export type *` alone gave 0 (SLICE3B
census); together they give puppeteer +3, whose chains run through
`export type *` four and five levels deep. Depth at which each declaration
was found: pi 158 at 1 (today's rule), 38 at 2, 5 at 3; puppeteer 6 at 4,
2 at 5; no chain reached the limit at 5. Five is the smallest limit that
covers every observed chain.

Bounds, stated before any code:

B1. A chain is followed only through re-exports the files themselves
declare (`export * from`, `export { A as B } from`, and now
`export type * from`); never through a guess. `export * as ns` and
`export type * as ns` are not followed, as today.
B2. Exactly one distinct declaring file resolves. Two or more is
AMBIG-SHAPE; a chain still going at depth 5 is BARREL2. Every path is
cycle-guarded. Order-independent: the same file set gives the same answer
whatever order the re-exports appear in.
B3. Code-pair discovery (codepairs.rs) only. Egress and value discovery
(valuepairs) keep their one-level rule; its measured BARREL2 count on the
evaluation set is 0 today, and it changes only with tsconfig paths, the
next slice.
B4. Resolving a chain is not a proposal: the shape, the two-distinctive-
field rule and earned silence (SLICE3B) all apply unchanged. +29 is the
measured yield of the prototype, not a promise; its precision is measured
by STAGE3-EVAL-004. pi, where most of the yield lands, had the lowest
precision in EVAL-003 (9 of 20, S3-F-15).
B5. Zero new dependencies, codes, exit or format changes beyond the E4
chain rendering of S4-D4 and the BARREL2 explain text of S4-D3.

## A. Frozen scope

A1. One engine change point: `find_shape` in codepairs.rs (bounded,
cycle-guarded traversal replacing the one-level search and its separate
BARREL2 probe), plus `export type * from` recognized as a star re-export.
A2. The E4 evidence line of a proposal found through more than one barrel
names the whole chain (S4-D4); single-level proposals keep their bytes.

## B. Numbered requirements

R-S4-1 Traverse re-exports from the import's resolved entry, depth limit
5, each (file, name) visited once; a named re-export follows its original
name.
R-S4-2 One distinct declaring file: resolved; several: AMBIG-SHAPE naming
them in path order; none with a chain cut at the limit: BARREL2 with the
reason `{Name} from "{spec}" is re-exported through more than 5 barrel
levels (from {entry})`; none otherwise: the SLICE3B earned-silence rule.
R-S4-3 `export type * from "x"` is a star re-export; `export type * as ns`
is not followed.
R-S4-4 E4 for a multi-level chain: `... resolved to {entry}, re-exported
through {f1}:{l1} -> {f2}:{l2} -> ... from {decl file}`.
R-S4-5 Determinism: two runs byte-identical; result independent of
re-export order.

## C. Countermodels

CM-S4-1 a depth-2 chain still abstaining BARREL2. CM-S4-2 a depth-6 chain
resolved (the limit not enforced). CM-S4-3 a cycle looping or crashing.
CM-S4-4 a diamond (two routes to one declaration) called ambiguous.
CM-S4-5 two different declarations resolved to the first one found.
CM-S4-6 a renamed re-export (`export { A as B }`) followed by the wrong
name. CM-S4-7 `export type *` left unparsed. CM-S4-8 a namespace re-export
followed. CM-S4-9 a cut chain left silent instead of BARREL2. CM-S4-10 a
single-level proposal changing a byte.

## D. Fixtures (declared outcomes, before any code)

S4-F1 depth 2 (the u02 workspace): proposal. S4-F2 depth 5: proposal.
S4-F3 depth 6: BARREL2 with the new reason. S4-F4 a two-file cycle with no
declaration: no hang, the SLICE3B rule decides (declared nowhere:
UNRESOLVED). S4-F5 a diamond to one declaration: proposal. S4-F6 two
different declarations reachable: AMBIG-SHAPE naming both. S4-F7
`export { Order as Ord } from` two levels down, imported as `Ord`:
proposal on `Order`. S4-F8 an `export type *` chain: proposal. S4-F9
`export * as ns from` on the only route: not followed. S4-F10 the E4 chain
line, exact. S4-N1 a single-level proposal byte-identical to today (golden
recorded at Gate 1). S4-M1 two runs byte-identical, and the same result
with the re-export lines of a barrel reordered.
Every frozen suite unedited except slice 1 u02 (S4-D1).

## E. Mutation set

1 depth limit 1 (S4-F1). 2 depth limit unenforced (S4-F3). 3 the visited
set keyed on file only, dropping renamed routes (S4-F7). 4 a diamond
counted twice (S4-F5). 5 ambiguity resolved to the first (S4-F6). 6 the
original name of a named re-export ignored (S4-F7). 7 `export type *`
unparsed (S4-F8). 8 namespace re-export followed (S4-F9). 9 a cut chain
returned as NotShape (S4-F3). 10 the chain omitted from E4 (S4-F10).
A mutant that removes the cycle guard would hang the harness rather than
fail an assertion; S4-F4 runs under a wall-clock limit inside the test so
that mutant fails by assertion (mutant 11).

## F. Gates 0 to 5, then the measurement

Gate 0 hash ledger at aa03983. Gate 1 fixtures first. Gate 2 codepairs.rs
(and explain.rs text per S4-D3). Gate 3 section E from scratch. Gate 4
full suite, zero warnings. Gate 5 ledger, STATUS.md. Then STAGE3-EVAL-004
on the same pinned manifest: every metric, the +29 prediction recorded
here before any code, and blind adjudication of the new proposals.

## G. Path classes

Class 1: the standing protected set. Class 2: tests/init_slice4_test.rs,
scripts/slice4/. Class 3 (REQUESTED): codepairs.rs; explain.rs for the
BARREL2 text only (S4-D3); tests/init_slice1_test.rs u02 only (S4-D1).

## H. Decision points for ruling

S4-D1 Slice 1 u02 pins the one-level limit ("re-exported through more than
one barrel level") on a two-level chain that this slice resolves.
Supersede it under the standing rule of S3-D1, original assertion kept
verbatim in the ledger.
S4-D2 Depth limit 5: the smallest limit covering every chain observed on
the evaluation set (deepest: 5, in puppeteer).
S4-D3 The BARREL2 explain entry currently says "resolution follows exactly
one level", which becomes false for code-pair discovery. Proposed text:
"a symbol is re-exported through more barrel levels than init follows
(five in code-pair discovery, one in egress and value discovery)". This
edits explain.rs text only; the code list and the drift test are
unchanged.
S4-D4 E4 names the full chain for multi-level proposals (recommended:
evidence a reviewer can follow) versus first hop only (byte-closer to
today, but hides how the declaration was reached).
S4-D5 Code-pair discovery only; egress and value discovery stay at one
level until the tsconfig paths slice, which needs its own earned silence.
S4-D6 STAGE3-EVAL-004 adjudication: the founder, or two blind independent
agents with the conservative merge used in EVAL-003.

## I. Stop conditions

Stop and report on: any frozen fixture other than u02 failing; any
single-level proposal changing; any ambiguity the prototype did not show;
any need for a parser, a new dependency or a Class 1 change beyond
S4-D3.

## J. Rulings of 2026-10-19 (recorded)

S4-D1 to S4-D6 RATIFIED as drafted: u02 superseded under the standing rule
of S3-D1 with its original assertion kept verbatim (section K); depth limit
exactly 5; the BARREL2 explain text as proposed, code list and drift test
unchanged; E4 names the full chain; code-pair discovery only; STAGE3-EVAL-004
adjudicated by two blind independent agents, TRUE only when both say TRUE.
Implementation AUTHORIZED, Gates 0 to 5 then STAGE3-EVAL-004.

Pre-registered by the founder before any engine code (STAGE3-EVAL-004):
+29 proposals (pi 113 to 139, puppeteer 5 to 8); BARREL2 abstentions -118;
unreached-type abstentions -13; 0 new AMBIG-SHAPE; every cycle guarded, no
loop or timeout; M-S-3 precision at least 60 percent on the delta; M-S-4
silent drops 0 (hard gate).

S4-D7 (raised at Gate 4; stop condition I fired). Section D
claimed every frozen suite unedited except u02. That was wrong: four
SLICE3B fixtures pin the limits this slice lifts, and each now proposes.

| Fixture | Pins | Under SLICE4 |
|---|---|---|
| b_f1_a_type_three_barrels_deep_abstains_with_its_site | depth 1 (unreached site at depth 3) | proposes |
| b_f3_an_export_type_star_chain_abstains | `export type *` unparsed (S3B-D5) | proposes |
| b_f7_a_name_declared_both_ways_abstains | depth 1 (both-ways reason at depth 3) | proposes, as the same file does one level down today |
| b_m1_two_runs_are_byte_identical | LOUD summary on the depth-3 chain | proposes (determinism half unchanged) |

Proposed: supersede all four under the standing rule of S3-D1, originals
kept verbatim; b_f1 and b_f7 keep their reasons pinned on a route init
still does not follow (a namespace re-export at the third level), so the
SLICE3B mutants 01, 06 and 08 keep a killer; their `# catch:` lines name
the renamed tests. b_m1 keeps its name.

## J (continued). Rulings of 2026-10-20 (recorded)

S4-D1 to S4-D6 restated unchanged, with the same pre-registration.
S4-D7 RATIFIED as proposed: the four SLICE3B fixtures superseded under the
standing rule of S3-D1, originals kept verbatim (section K).
S4-D8 (after STAGE3-EVAL-004) Hold the engine; record EVAL-004 as FAIL,
docs only, as EVAL-002 was; draft a receiver-verification slice (S3-F-15
and writes counted as reads, S4-F-03), land it with SLICE4, and re-measure
the pair as STAGE3-EVAL-005.

## K. Results

Gate 0 PASS: 56 of 56 crate hashes identical, Mac and sandbox, at aa03983.
Gate 1 PASS: tests/init_slice4_test.rs frozen at 33fb361c7040 before any
engine code (12 fixtures: S4-F1 to S4-F10, S4-N1 golden, S4-M1); 9 failed
on the baseline as declared, S4-F9, S4-N1 and S4-M1 passed as declared.
Gate 2 PASS: find_shape replaced by a bounded, cycle-guarded traversal
(depth 5, each (file, name) visited once, renamed re-exports followed by
original name, one distinct declaring file or AMBIG-SHAPE, a chain cut at
the limit BARREL2 with the new reason); `export type * from` is a star
re-export; E4 names the chain; explain.rs BARREL2 text per S4-D3.
Gate 3 PASS: 11 of 11 section E mutants caught from scratch; the SLICE3B
harness, rerun on the new engine, still catches 9 of 9.
Gate 4 PASS: 307 of 307 tests pass with S4-D1 and S4-D7 applied, 0
compiler warnings (rerun 2026-10-20).
Gate 5: STAGE3-EVAL-004 FAIL on M-S-3; every other metric PASS
(docs/evidence/stage3/eval004/LEDGER.md).

Originals kept verbatim (S4-D1, S4-D7):

    fn u02_barrel_depth_two_abstains()
    &["ABSTAIN BARREL2 b/src/x.ts:1 Order from \"a\" is re-exported through more than one barrel level (from a/index.ts)"],
    "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE",

    fn b_f1_a_type_three_barrels_deep_abstains_with_its_site()
    loud(&case("f1", &files, "{ Order }"), "Order", &unreached("a/l3.ts:1"));

    fn b_f3_an_export_type_star_chain_abstains()
    // S3-F-10 is deferred (S3B-D5): the chain stays unparsed, and stated.
    loud(&case("f3", &files, "{ Order }"), "Order", &unreached("a/o.ts:1"));

    fn b_f7_a_name_declared_both_ways_abstains()
    loud(&case("f7", &files, "{ Order }"), "Order", "declared both as a type and as a value in package a; not guessed");

    b_m1_two_runs_are_byte_identical:
    assert_eq!(o.summary, LOUD, "{}", o.raw);

What the measurement says: the slice did what it claimed (+29 exactly,
every pre-registered count met, 0 silent drops) and the proposals it adds
are no worse built than the ones before; it reaches more shapes in the
repository where init's declared bound, "receiver of field reads not
verified", costs the most. Widening reach before that bound is fixed
lowers precision where it lands. Fix the bound, then widen.
