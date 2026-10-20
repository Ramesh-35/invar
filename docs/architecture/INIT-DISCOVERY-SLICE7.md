# INIT-DISCOVERY-SLICE7: receivers typed by an imported function's return type

Status: DRAFT r1 for ruling. Implementation NOT authorized. Parent: the R6
measurement (docs/evidence/r6-probe/) and the founder's queue of
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
