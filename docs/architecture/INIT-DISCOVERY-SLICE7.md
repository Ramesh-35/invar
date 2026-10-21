# INIT-DISCOVERY-SLICE7: receivers typed by an imported function's return type

Status: RATIFIED (S7-D1 to S7-D6), implemented through Gates 0 to 5 and
LANDED; STAGE3-EVAL-007 met every gate of the ruling (sections J and K). Sections
0 to I are the draft r1 text as committed in a52c7f0, unchanged. Parent:
the R6 measurement (docs/evidence/r6-probe/) and the founder's queue of
2026-10-20. Baseline: b0a8c79.

## 0. Evidence, measured on b0a8c79 before this draft (throwaway prototype)

SLICE5 counts a read only through a receiver declared as the shape in the
consumer file; it follows a local function's declared return type, never
an imported one. So a file that calls `f(...)` imported from the shape's
package, where `f` is declared to return the shape, cannot propose, and if
it does not import the shape by name it abstains R6.

Prototype rule (evidence/slice7/prototype.patch, binary of the prototype
tree): a receiver assigned `r = [await] f(...)` is declared as S when `f`
is imported by name from S's package and that package has exactly one
declaration of `f` with a declared return type, whose head is S,
`Promise<S>`, `Readonly<S>`, `S[]` or `Array<S>` (arrays bind elements as
in SLICE5). Two variants, measured on the six cloned repositories:

| Variant | Proposals | Shapes | Abstentions | Fixture collisions |
|---|---|---|---|---|
| R6 only (shapes not imported by name) | +39 (pi +36, excalidraw +2, Understand-Anything +1) | +40 | R6 -40 | 1 |
| uniform (also shapes imported by name) | +40 | +42 | R6 -40, RECV -2 | 1 |

Every converted R6 line was one of the 39 the R6 probe found, plus one the
probe missed: a cross-package relative import
(`../../ai/src/api/openai-codex-responses.ts`, receiver typed by
`getOpenAICodexWebSocketDebugStats(...)`; engine right). The uniform
variant's extra 2 shapes are pi's ToolStateChanges (agent-loop.ts, a new
pair) and SystemMessage (agent-session.ts, on an existing pair).

Precision already measured on 39 of the 40 R6 conversions (blind, two
agents, conservative merge, R6 probe): 38 TRUE (97.4 percent), agreement 39
of 39. The yield is concentrated: 31 of the 39 are one pattern in pi's
tests (`faux = registerFauxProvider(...)`, FauxProviderRegistration), all
TRUE; the other 8 are 7 TRUE, 1 NOT TRUE (a parameter `color` typed by a
different interface with the same field names: bound B4 of SLICE5).

Prototype gap, stated: it does not yet refuse `r = f(...).x` (CM-S7-5); a
scan of all 40 conversions found no binding of that form, so the counts
stand.

Fixture collision, measured: slice 1 `u01_inferred_type_abstains_r6` pins
R6 on exactly this route (`const r = exec('x')`, `exec(): Result`). Every
other test passes (342 of 343).

In-sample caveat, stated: the precision above was measured on the probe's
candidates before this rule existed, on the probe's claimed fields; the
engine's E5 lines may differ. STAGE3-EVAL-007 adjudicates the engine's own
evidence (S7-D5).

Bounds, stated before any code:

B1 Restriction of SLICE5 kept: a read counts only through a receiver tied
to S; this slice adds one way to tie it, nothing else.
B2 The route: `r = [await] f(...)` (and `[await] this.f(...)` is NOT
included), `f` imported by name from S's package, exactly one declaration
of `f` in that package with a declared return type, whose head is S (the
wrappers and arrays above). Return types are read as written; never
inferred; a function with no declared return type ties nothing.
B3 Names, not scopes, as SLICE5 B4.
B4 An R6 line whose shape gains two distinctive verified reads becomes a
proposal; otherwise R6 stays, unchanged.
B5 Zero new dependencies, codes or exit changes; the evidence and status
text of S7-D2 and S7-D3 only.

## B. Numbered requirements

R-S7-1 A receiver tied by B2 counts exactly as a declared receiver.
R-S7-2 A proposal shape reached this way has an E4 line naming the
function and its declaration (S7-D3).
R-S7-3 A proposal with such a shape carries the status line of S7-D2.
R-S7-4 Determinism: two runs byte-identical.

## C. Countermodels

CM-S7-1 a function with no declared return type ties a receiver. CM-S7-2
a function declared twice in the package ties one. CM-S7-3 a function
imported from another package ties a shape of this one. CM-S7-4 a
function returning `Box<S>` ties S. CM-S7-5 `r = f(...).x` ties r to S.
CM-S7-6 an R6 line with fewer than two verified reads becomes a proposal.
CM-S7-7 an existing proposal changes a byte (S7-D2 and S7-D3 aside).

## D. Fixtures (declared outcomes, before any code)

S7-F1 `import { exec } from 'a/run'`, `exec(): Result`, two reads through
`r = exec(...)`: proposal of Result with the E4 of S7-D3 (the u01
workspace; S7-D4). S7-F2 `Promise<Result>` with `await`: proposal. S7-F3
`Result[]` and a `for of` element: proposal. S7-F4 `exec()` with no return
type: R6, unchanged. S7-F5 `exec` declared twice in the package: R6.
S7-F6 `exec(): Box<Result>`: R6. S7-F7 `r = exec().inner`: R6. S7-F8 one
verified read only: R6. S7-F9 (uniform only, S7-D1) Result also imported
by name, receiver only through `exec()`: proposal instead of RECV. S7-N1 a
SLICE5 proposal byte-identical. S7-M1 two runs byte-identical.

## E. Mutation set

1 the route removed (S7-F1). 2 no declared return type still ties
(S7-F4). 3 duplicate declarations not refused (S7-F5). 4 a non-head
occurrence of S ties (S7-F6). 5 a property access on the call result ties
(S7-F7). 6 arrays not bound to elements (S7-F3). 7 `await` not followed
(S7-F2). 8 the E4 function name omitted (S7-F1).

## F. Gates, then STAGE3-EVAL-007

Gates 0 to 5. STAGE3-EVAL-007: proposals 113 to 152 (R6 only) or 153
(uniform); R6 -40; every other repository and every existing proposal
byte-identical except the status line rule of S7-D2; M-S-3 by blind
adjudication of every new shape (S7-D5); M-S-4 0; M-S-5 and M-S-6 over the
new TRUE contracts.

## H. Decision points for ruling

S7-D1 Uniform (recommended: one evidence rule for every shape; whether the
consumer also imports the shape by name should not change what counts as
a declared receiver; +1 proposal, +2 shapes, 2 RECV recovered) versus R6
only (+39, exactly the measured set).
S7-D2 Status line. A shape reached through an imported return type is not
"declared as the shape in this file". Recommended: proposals containing
such a shape carry `PROPOSED (heuristic; each counted read is on a
receiver declared as the shape in this file or returned by a function the
shape's package declares to return it; field writes excluded; types are
not inferred)`; every other proposal keeps today's line byte for byte.
Alternative: one widened line for all 113, which changes every proposal.
S7-D3 E4 for such a shape: `E4 {S} reached through the return type of {f}
({decl file}:{line}), imported at {consumer}:{line} from {spec:?}`.
S7-D4 Slice 1 u01 pins R6 on exactly this route: supersede under the
standing rule of S3-D1, original kept verbatim; its workspace becomes
S7-F1. The R6 explain text gains "and no receiver is tied to it by an
imported function's declared return type"; the code is unchanged.
S7-D5 STAGE3-EVAL-007 precision: blind dual adjudication of every new
shape the engine proposes (40 or 42), floor 60 percent, with the R6 probe
verdicts reported beside it; precision also reported without the faux
pattern, which is 31 of the 39.

## I. Stop conditions

Stop and report on: any existing proposal changing beyond S7-D2; any
conversion outside the measured set; any fixture collision beyond u01; any
need for a parser or dependency.

## J. Rulings of 2026-10-21 (recorded)

S7-D1 uniform scope, APPROVED: one rule for every shape, applied only to a
shape that does not qualify without it, so no existing evidence moves.
S7-D2 APPROVED with a narrower text: a proposal whose shapes are all
reached through the route carries `PROPOSED (heuristic; receiver assigned
from imported function return type; field writes excluded; types are not
inferred)`; every other proposal keeps today's line byte for byte. A
proposal that mixes declared and routed shapes (founder, same day) gains
the route shape in its own block and carries `PROPOSED (heuristic;
counted reads are on receivers declared as the shape in this file or
assigned from an imported function's declared return type; field writes
excluded; types are not inferred)`.
S7-D3 APPROVED: E4 names the function and its declaration, `{S} not
imported by name; resolved via return type of {f}() declared in
{file}:{line}, {f} imported at {consumer}:{line} from {spec:?}`; a shape
also imported by name keeps its E4 and gains `; receiver resolved via
return type of {f}() declared in {file}:{line}`. Abstain if ambiguous.
S7-D4 APPROVED: slice 1 u01 superseded, the original kept verbatim, its
error codes kept; the R6 explain text gains the slice 7 clause.
S7-D5 APPROVED: blind adjudication of every new shape from the frozen
release binary, reported for all shapes, for the faux pattern and
excluding it, with denominators; 60 percent floor, a hard gate.
Also ruled: fixtures frozen before implementation; the property-chaining
gap closed; 100 percent of non-equivalent mutants caught, equivalents
recorded; byte preservation; determinism on 10 repositories; no tuning
to the expected count; no cross-file property types.
S7-D6 (founder, 2026-10-21, on finding S7-F-04) APPROVED for SLICE7: the
header init writes at the top of every invar.toml must not misstate how a
shape was reached. It now reads "a TypeScript shape exported by one
workspace package, with at least 2 of its direct fields read in a file of
another, each on a receiver declared as that shape in that file or
assigned from a function imported from the shape's package whose declared
return type is the shape (field writes excluded; types are not
inferred)". The ruled clause is added verbatim; "imported by name in
another" is dropped with it, because it is false for the same 40 shapes.

## K. Results

Gate 0 PASS: every crate file identical to b0a8c79 (Mac and sandbox).
Gate 1 PASS: tests/init_slice7_test.rs frozen before engine code
(9b1864b7538e): S7-F1 to F4b, F5 to F11, F12 (uniform RECV rescue), F13
(mixed block), F14 (golden: the route never adds reads to a shape that
already qualifies), F15 (a shape name declared twice in the package), F16
(`this.f()` not followed), N1 (golden). Slice 1 u01 superseded as
`u01_return_type_receiver_proposes_by_slice7` (3957506278f3).
Gate 2 PASS: codepairs.rs, init.rs and explain.rs only. Per package: an
index of function declarations with their written return type, and of
shape names. A receiver `r = [await] f(...)` is tied to S only when f is
imported by name from S's package, declared there exactly once, returns
exactly S, Readonly<S>, Promise<S> (awaited), S[] or Array<S>, S is
declared once by that name in the package, and the call ends the
expression (no `.`, `?.`, `[`, `!`, `??` or `||` after it). Stop
conditions checked on the six cloned repositories: none hit.
Gate 3 PASS: 15 of 15 mutants caught from scratch (scripts/slice7), the
section E eight plus seven for the bounds added at Gate 1.
Gate 4 PASS: 364 of 364 tests, 0 compiler warnings, release binary
029c0c286ecf; changed files exactly the five above plus the new test file.
After S7-D6: 364 of 364, 0 warnings, release binary 681fc1463275; init.rs
changes by the header sentence only, and the goldens of slices 3 to 7 by
that sentence only, each with its original lines kept verbatim (S5-D4
precedent). Reverting those lines in tests/init_slice7_test.rs gives back
the Gate 1 bytes, 9b1864b7538e, above the Gate 3 marker.
Gate 5 PASS: STAGE3-EVAL-007 (docs/evidence/stage3/eval007/LEDGER.md):
152 proposals; precision 39 of 40 new shapes TRUE (97.5 percent),
agreement 40 of 40; 0 silent drops; 39 of 39 TRUE contracts round-trip.
Closing re-measurement on 681fc1463275: 10 of 10 exit 0, deterministic;
each output differs from EVAL-007 in exactly the header sentence (3 lines
to 4) and nothing else (eval007/closing-s7d6).

Corrections to this draft, found during the gates and stated:

S7-F-01 Section 0 and section F forecast 153 proposals and 42 shapes for
the uniform variant; the engine gives 152 and 40. The prototype accepted
a return type written `S | undefined`; the ruled forms do not. The two
prototype conversions that used it are gone, and nothing else moved.
S7-F-02 Gate 3, first run: 3 of 15 mutants survived, and each was a fault
in the test set, not the engine. Mutant 03 (duplicate declarations not
refused) survived S7-F6 because the duplicate's file sorted before the
real declaration and returned another type, so the first declaration
happened to tie nothing; S7-F6b puts a duplicate returning the shape
after it. Mutant 14 (functions imported from any package) had no fixture
at all: section D declared none for CM-S7-3; S7-F17 is that fixture.
Mutant 15 as first written did not change the engine's behaviour (an
equivalent mutant, written wrongly: it moved a variable the route never
read); it was rewritten to follow `this.f(...)` and is caught by S7-F16.
S7-F6b and S7-F17 were appended after the Gate 1 freeze, below a marker;
the bytes above it are the frozen file (9b1864b7538e).
S7-F-03 The R6 probe's README calls the faux pattern one function and one
shape (`registerFauxProvider`, FauxProviderRegistration). Its own TSV, and
the engine, show two: `fauxProvider()` (FauxProviderHandle, 27) and
`registerFauxProvider()` (FauxProviderRegistration, 5).
S7-F-04 (resolved by S7-D6) The header that `invar init` writes at the top
of every invar.toml says each proposal's shape is "imported by name" and
each read is "on a receiver declared as that shape in the consumer file".
For the 40 route shapes that is no longer true; their status and E4 lines
say so, the header does not. Changing the header changes every non-empty
init output, which S7-D2 did not authorize; ruled as S7-D6 above.
