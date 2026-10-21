# INIT-DISCOVERY-SLICE8: receivers reached through a property typed as the shape

Status: RATIFIED (S8-D0 to S8-D3, with the founder's id and scope rulings
of the same day), implemented through Gates 0 to 5 together with
LOCK-SLICE2; STAGE3-EVAL-008 met every gate (sections J and K). Sections 0
to I are the draft r1 text, unchanged. Parent: the founder's direction of
2026-10-21 (a measurement-first specification for the R6 lines SLICE7
leaves). Baseline: db401f4.

## 0. Evidence, measured before this draft (throwaway probe)

After SLICE7, R6 ("reads fields of S without importing it by name") is the
largest abstention class: 3,094 lines in the ten pinned repositories,
3,065 of them in the six cloned ones (freeCodeCamp's 29 were not probed).
The R6 probe of 2026-10-20 estimated about 7 percent of these real, from a
seeded sample of 30 with 2 TRUE, both reached through a property typed as
the shape in another file. This draft measures that route directly.

Probe (docs/evidence/slice8/propprobe.py, text only, run on the SLICE7 output):
for each R6 line, does the consumer read two distinctive fields of S
through `r.p.f`, where r has a written type T in the file and T's
declaration has member p of type S (PROP1), or one hop more, `r.a.p.f`
through a named or inline object type (PROP2), or `this.p.f` for a class
property p: S (THIS)? Aliases of the property value, array elements
(for-of, the array callbacks, indexing) and array receivers count; writes
do not; T is found by name and used only when exactly one declaration
carries that name in the repository.

| Class | Lines |
|---|---|
| YIELD (two or more distinctive reads through such a path) | 20 (PROP1 17, PROP2 3) |
| a path exists, fewer than two reads | 43 (21 of them through a property typed `S \| undefined` or `S \| null`) |
| no such path | 3,002 (97.9 percent) |

By repository: Understand-Anything 7 (GraphEdge 3, GraphNode, KnowledgeMeta,
DomainMeta, ProjectMeta), pi 13 (FauxProviderHandle and its state
FauxProviderState in durable tests 7, ProviderAuth 3, ModelAuth 2,
AssistantMessage 1), excalidraw, playwright and puppeteer 0.

Precision: 20 of 20 TRUE, two blind agents, agreement 20 of 20, rubric D
adapted to the route (docs/evidence/slice8/adjudication). This is AI
adjudication, reported under that label.

Recall, the only measure available: of the 2 TRUE lines in the seeded
sample, the probe reaches 1 (`requests[1].failed?.message.errorMessage`,
an inline object type on an array element). The other (`element.crop`,
excalidraw) is typed only after a narrowing call (`isImageElement`),
which is inference and out of bounds. In-sample caveat, stated: the probe
gained array indexing and inline object types after this check; the 20
are not independent of it.

What it means: the text-visible property route is real (20 of 20) and
small (0.6 percent of R6). The earlier "about 200 real" estimate rested
on 2 of 30; this route recovers 20, and the rest of R6 is mostly
coincidental field names, with the real remainder behind narrowing,
generics or untyped callbacks that text cannot type. A slice here buys
about 20 shapes in two repositories.

Probe approximations the engine would not keep: T is resolved by name
across the repository (the engine would resolve it through the consumer's
own imports, as it resolves shapes today); members are split by a
depth-counting reader, not a parser. A prototype on the engine must
re-measure both.

Bounds, stated before any code:

B1 Restriction of SLICE5 kept: a read counts only through a receiver tied
to S; this slice adds one way to tie it.
B2 The route: a receiver with a written type T in the consumer file, T
declared in that file or imported by name and resolved by the existing
resolver, T's declaration in that one file having member p whose written
type is exactly S, Readonly<S>, S[] or Array<S> (the SLICE7 forms; no
unions, S7-F10), S resolved to the R6 shape: declared in T's file or
imported there by name from S's package. The read is `r.p.f`, `r?.p?.f`,
`r.p!.f`, an alias `y = r.p` then `y.f`, or an element of an array p.
B3 Depth one hop (S8-D1). Names, not scopes, as SLICE5.
B4 Applied only to a shape that does not qualify without it, as SLICE7.
B5 Zero new dependencies, codes or exit changes; the evidence and status
text of S8-D2 only.

## B. Numbered requirements

R-S8-1 A receiver tied by B2 counts exactly as a declared receiver.
R-S8-2 Its E4 names the property and its declaration: `{S} not imported
by name; resolved via property {T}.{p} declared in {file}:{line}, {T}
{imported at {consumer}:{line} from {spec:?} | declared at {file}:{line}}`.
R-S8-3 A status line naming the route (S8-D2).
R-S8-4 Determinism: two runs byte-identical.

## C. Countermodels

CM-S8-1 a member type with a union ties S. CM-S8-2 a type T declared twice
(or unresolved) ties anything. CM-S8-3 a member type naming another S
(same name, other package) ties this S. CM-S8-4 a write `r.p.f = x`
counts. CM-S8-5 `r.p` where r's type is inferred (no written type) ties.
CM-S8-6 two hops tie when D1 rules one. CM-S8-7 any existing proposal or
abstention changes a byte, other than the R6 lines it converts.

## D. Fixtures (declared outcomes, before any code)

S8-F1 `r: T`, `T.p: S`, two reads `r.p.f`: proposal, E4 of R-S8-2. S8-F2
alias `const y = r.p`: proposal. S8-F3 `p: S[]`, for-of element reads:
proposal. S8-F4 `r: T[]`, `r[0].p.f`: proposal. S8-F5 `p: S | undefined`:
R6. S8-F6 T declared twice: R6. S8-F7 T's S is a different package's S:
R6. S8-F8 one read only: R6. S8-F9 a write: R6. S8-F10 r untyped: R6.
S8-F11 two hops: R6 under one hop (S8-D1). S8-N1 a SLICE7 proposal
byte-identical. S8-M1 two runs byte-identical.

## E. Mutation set

1 the route removed (F1). 2 unions accepted (F5). 3 duplicate T accepted
(F6). 4 S matched by name only (F7). 5 writes counted (F9). 6 aliases not
followed (F2). 7 array elements not followed (F3, F4). 8 depth not bounded
(F11). 9 E4 property omitted (F1).

## F. Gates, then STAGE3-EVAL-008

Gates 0 to 5. STAGE3-EVAL-008 pre-registration (proposed): a prototype on
the engine first; proposals and R6 conversions as it measures, expected
near +17 (one hop) or +20 (two hops), concentrated in pi and
Understand-Anything; every other repository and existing proposal
byte-identical; M-S-3 by blind adjudication of every new shape, floor 60
percent; M-S-4 0; M-S-5 and M-S-6 over the new TRUE contracts.

## H. Decision points for ruling

S8-D0 Build or record (recommended: build only if the founder values 20
real contracts in two repositories over the added code; the measurement
says the route is precise but small, and the cheaper honest alternative is
to record the finding and leave R6 as it is).
S8-D1 Depth: one hop (recommended: 17 of 20, one rule, no type chains)
versus two (20; the 3 extra are FauxProviderHandle.state, twice, and one
inline object type).
S8-D2 Status line for a proposal containing such a shape: `PROPOSED
(heuristic; receiver reached through a property declared as the shape;
field writes excluded; types are not inferred)`, and a combined line for
mixed blocks in the SLICE7 manner.
S8-D3 The init header gains "or read through a property whose declared
type is the shape", in the S7-D6 manner (the header must stay true).
S8-D4 STAGE3-EVAL-008 as section F, with a prototype on the engine first,
since the probe resolves T by name and the engine would not.

## I. Stop conditions

Stop and report on: any existing proposal changing beyond S8-D2 and D3;
any conversion outside the probe's 20 that the prototype cannot explain;
any fixture collision; any need for a parser or dependency.

## J. Rulings of 2026-10-21 (recorded)

S8-D0 BUILD. S8-D1 two hops (`r.p.f` and `r.a.p.f`). S8-D2 and S8-D3
RATIFIED with the accurate wording (founder's answer, same day): status
`PROPOSED (heuristic; receiver reached through a property declared as the
shape, at most two hops; field writes excluded; types are not inferred)`;
a block mixing this with any other kind of shape carries `PROPOSED
(heuristic; counted reads are on receivers declared as the shape in this
file, assigned from an imported function's declared return type, or
reached through a property declared as the shape, at most two hops; field
writes excluded; types are not inferred)`; the init header gains "or read
through a property, at most two hops from a receiver typed in that file,
whose declared type is the shape". The ruling's own text ("accessed
through a 2-hop property path on an imported typed shape") was not used:
the shape is the thing not imported, T may be declared in the consumer
file, and most paths are one hop.
Also ruled during the gates (founder's answers): ids stay fixed, a pair
every shape of which is reached only through a property path takes its
`#n` after every other pair with the same file names (S8-F-03); uniform
scope, as SLICE7 S7-D1 (S8-F-04); the lock defect found at Gate 5 is
fixed as LOCK-SLICE2 and lands with this slice (S8-F-05).

## K. Results

Gate 0 PASS: 284 crate and script files identical at db401f4 (Mac and
sandbox, e060a539cd97).
Gate 1 PASS: tests/init_slice8_test.rs frozen before engine code
(8e5e4142e436): F1, F1b (a type declared in the consumer file, rescuing
a named shape), F2, F3, F3b, F3c, F4, F5, F5b, F6, F6b, F7, F8, F9, F10,
F11, F11b, F11c, F12, F13 (golden body), F16, H1 (the header), N1 (golden
body). On the SLICE7 binary 14 failed as declared; the refusals, the
goldens and the controls passed.
Gate 2 PASS: codepairs.rs (a per-run type index; member types read as
written; paths of at most two hops, the middle hop a named or inline
object type; reads through the path, its aliases and its elements), init.rs
(status lines, E4, header, ids), explain.rs (the R6 and RECV texts).
Gate 3 PASS: 18 of 18 mutants caught from scratch (scripts/slice8).
Gate 4 PASS: 397 of 397 tests, 0 compiler warnings, release binary
502689914b48 (SLICE8 and LOCK-SLICE2).
Gate 5 PASS: STAGE3-EVAL-008 (docs/evidence/stage3/eval008/LEDGER.md).

Corrections to this draft, found during the gates and stated:

S8-F-01 Fixture F5b as frozen read `j.result!.exit_code`. init's field
reader does not count `!.` as a read, so that workspace had no R6 line to
convert; the fixture was corrected to `j.result?.exit_code`, and the
engine follows `?.` between hops, not `!.`, matching the field reader.
S8-F-02 Gate 3, first run: 16 of 18. Mutant 15 (`?.` between hops not
followed) survived F5b, whose `?.` is after the last hop; mutant 16 (an
alias of a longer expression tied) survived F16, whose alias reads were
also direct reads. F5c and F16b, appended after the freeze below a marker,
catch them.
S8-F-03 (stop condition, ruled) The first engine run renamed one existing
proposal on pi: `code:types.ts->bug-report.ts` became `#2` because a new
pair sorted before it. Ruled: property-only pairs number last; F17 and
mutant 18 pin it; re-measured, 0 renamed.
S8-F-04 (deviation, ruled) Section F expected about 17 or 20 conversions.
The engine converts 27 R6 lines, all 20 of the probe plus 7 it missed
(receivers the engine already recognizes, such as the cast
`e.message as AssistantMessage`, where the probe matched annotations
only, and a `type X = Readonly<{ ... }>` body the probe's declaration
reader did not open), and rescues 7 RECV lines under the uniform rule; 15 existing
proposals gain a shape and the combined status line, every other line of
them unchanged.
S8-F-05 (lock defect, ruled: LOCK-SLICE2) excalidraw's
`type RenderPositionOffset = Readonly<{ x: number; y: number }>` locked 0
fields: LOCK-SLICE1 opened one-line `= { ... }` bodies only. The contract
held through its other shape, so this shape was unprotected without a
word. Fixed in docs/architecture/LOCK-SLICE2.md; M-S-5 25 of 25 after it.
