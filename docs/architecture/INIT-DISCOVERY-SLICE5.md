# INIT-DISCOVERY-SLICE5: receiver verification

Status: RATIFIED (S5-D1 to S5-D6, section J) and implemented; NOT LANDED.
STAGE3-EVAL-005 passed M-S-3 (43 of 50 out of sample) and failed M-S-5 on
a pre-existing lock defect (S5-F-01); by ruling of 2026-10-20 SLICE4, SLICE5
and LOCK-SLICE1 land together once all seven metrics pass (sections J, K).
Sections 0 to I are the draft r1 text, unchanged. Parent:
STAGE3-EVAL-004 (M-S-3 FAIL, finding S4-F-03) and ruling S4-D8 (hold
SLICE4; fix the receiver bound; land both; re-measure). Baseline: d299d74
with docs/evidence/slice4/slice4-implementation.patch applied.

## 0. Evidence, measured before this draft (throwaway prototypes)

Init counts a field read anywhere in the consumer file, off any object
(codepairs.rs `property_reads`, file-wide). Every NOT TRUE verdict of
EVAL-003 and 12 of 13 in EVAL-004 trace to that (S3-F-15); the 13th is a
write counted as a read.

Labeled set: the 87 code pairs adjudicated blind in EVAL-003 and EVAL-004
(57 TRUE, 30 NOT TRUE). Python model of the rule (evidence/slice5/model):

| Set | Pairs | TRUE kept | NOT TRUE kept | Survivors | Precision if every NOT TRUE survivor stays NOT TRUE |
|---|---|---|---|---|---|
| EVAL-003 | 58 | 32 of 41 | 8 of 17 | 40 | 80.0 percent |
| EVAL-004 (SLICE4) | 29 | 9 of 16 | 5 of 13 | 14 | 64.3 percent |
| all | 87 | 41 of 57 (72 percent) | 13 of 30 | 54 | 75.9 percent |

That bound is pessimistic: a surviving NOT TRUE pair keeps only its
verified reads, so its evidence changes and the old verdict no longer
describes it (example: faux.ts->harness.ts loses FauxModelDefinition, the
shape both adjudicators rejected, and keeps FauxProviderRegistration, the
one both called real).

Engine prototype (evidence/slice5/engine-prototype, binary 483501da0799,
on SLICE4):

| Repository | Proposals | Shapes | New RECV abstentions |
|---|---|---|---|
| Understand-Anything | 13 to 10 | 23 to 14 | 9 |
| pi | 139 to 63 | 202 to 77 | 125 |
| excalidraw | 46 to 26 | 56 to 29 | 27 |
| puppeteer | 8 to 5 | 13 to 6 | 7 |
| playwright | 0 to 0 | 0 to 0 | 0 |
| total | 206 to 104 | 294 to 126 | 168 |

Every dropped shape is exactly one RECV abstention: 0 silent, 0 new
shapes or proposals, no other abstention line changed. Model against
engine: 1 difference (`filteredModels: Model<Api>[]`, a generic array the
model's regex missed; engine right). Of the 104 survivors, 41 were
adjudicated TRUE, 13 NOT TRUE, and 50 were never adjudicated (pi 38,
excalidraw 12).

Fixture impact, measured: 12 of 307 tests change, all for one reason, a
consumer reading through `o: any`: slice 1 n10 (which pins this bound by
name), the four S4-D7 re-pins of SLICE3B, and seven SLICE4 fixtures. The
other 295 pass unchanged.

Rejected, measured: scope-aware checking (a shadowed name stops counting)
needs a parser; done on text it cannot tell a call argument `f(message)`
from a parameter and lost true reads (TRUE kept 39 to 32). Name reuse is
declared instead (B4).

In-sample caveat, stated: forms E1 and E2 below were added after reading
the TRUE pairs the first cut lost. The labeled set is therefore not a
clean test of this rule; the registered test is the 50 survivors no one
has judged (S5-D5).

Bounds, stated before any code:

B1 Restriction only. A read counts for shape S only through a receiver
declared as S in the consumer file. The slice never adds a proposal, a
shape or a read.
B2 Declared as S means exactly one of: an annotation `r: S`, `r?: S`,
with `readonly`, `Readonly<>`, `Partial<>`, `Required<>`, generic
arguments `S<...>`, or an array `S[]`, `Array<S>`, `ReadonlyArray<S>`; a
cast `r = ... as S`; an explicit type argument `r = f<S>(` or
`[r, ...] = f<S>(`; a call `r = [await] [this.]f(` to a function in the
same file whose declared return type is `S` or `Promise<S>`; an element of
a receiver declared as an array of S (`for (const r of a)`, `a.map((r`
and the other array methods, `a[i].f`). `this.r.f` counts for a field r
declared S. No inference beyond these.
B3 A write never counts: `r.f =`, compound assignment, `++`, `--`.
B4 Names, not scopes: a name declared as S anywhere in the file verifies
every bare read through that name (no parser). Declared countermodel,
seen once in the labeled set (agent.test.ts, `message`).
B5 A shape with at least two distinctive reads file-wide but fewer than
two on declared receivers abstains RECV, naming both counts. Never silent.
B6 Zero new dependencies. One new abstention code (RECV). Code-pair
discovery only; R6 and egress/value discovery unchanged.

## A. Frozen scope

A1 One change point: the object-shape branch of codepairs.rs counts
receiver-verified reads, plus a receiver module (tokens, declarations,
reads). A2 RECV in the explain registry and the drift list (S5-D1). A3 The
proposal status line and the init header state what is now verified
(S5-D4).

## B. Numbered requirements

R-S5-1 A read counts for S only through a receiver declared as S (B2).
R-S5-2 Writes never count (B3).
R-S5-3 A shape that passes file-wide but not on declared receivers emits
`RECV {file}:{line} {S}: {n} of its fields are read in this file, {k} on a
receiver declared as {S}; the rest are read off values init cannot tie to
{S}; not guessed`.
R-S5-4 Restriction: on any input, proposals, shapes and E5 reads are a
subset of the SLICE4 result.
R-S5-5 Determinism: two runs byte-identical.

## C. Countermodels

CM-S5-1 an untyped receiver (`other: any`) counted. CM-S5-2 a write
counted. CM-S5-3 `x: Foo<S>` (S as a type argument of another type)
treated as declaring x as S. CM-S5-4 `S.member` or `S["k"]` treated as S.
CM-S5-5 an element of an array not declared as S counted. CM-S5-6 a
dropped shape left silent. CM-S5-7 a proposal or shape that SLICE4 did not
produce. CM-S5-8 `a.b.f` counted through a nested receiver b. CM-S5-9 a
cast to another type counted. CM-S5-10 a function returning another type
counted through E2.

## D. Fixtures (declared outcomes, before any code)

S5-F1 `(o: Order)` reads two fields: proposal (slice 1 shape, unchanged).
S5-F2 `(o: Order, other: any)` reading through `other`: RECV, exact
reason (replaces n10, S5-D3). S5-F3 two fields only written
(`o.total_minor = 1`): RECV. S5-F4 `const o = x as Order`: proposal.
S5-F5 `const [o, setO] = useState<Order | null>(...)`: proposal.
S5-F6 a local `function load(): Promise<Order>` and
`const o = await load()`: proposal. S5-F7 `for (const o of list)` with
`list: Order[]`: proposal. S5-F8 `list.map((o) => ...)` with
`list: Array<Order>`: proposal; `list[0].total_minor`: counted. S5-F9
`x: Box<Order>` reading `x.total_minor`: RECV. S5-F10 `this.order.f` for a
field `order: Order`: proposal. S5-N1 every surviving proposal of the
slice 1 to 4 suites byte-identical except the status line (S5-D4). S5-M1
two runs byte-identical. S5-R1 restriction: on the SLICE4 fixture
workspaces, no proposal, shape or read outside the SLICE4 result.

## E. Mutation set

1 the receiver check bypassed (S5-F2). 2 writes counted (S5-F3). 3 casts
not recognized (S5-F4). 4 type arguments not recognized (S5-F5). 5 return
types not recognized (S5-F6). 6 array elements not recognized (S5-F7). 7
`Foo<S>` counted as S (S5-F9). 8 `this.` fields not recognized (S5-F10).
9 RECV not emitted, dropped shape silent (S5-F2). 10 the RECV reason
missing a count (S5-F2).

## F. Gates 0 to 5, then the measurement

Gate 0 hash ledger at d299d74 plus the SLICE4 patch. Gate 1 fixtures
first. Gate 2 the receiver module and the object-shape branch. Gate 3
section E from scratch, plus the SLICE4 and SLICE3B harnesses rerun.
Gate 4 full suite, zero warnings. Gate 5 ledger. Then STAGE3-EVAL-005 on
SLICE4 plus SLICE5 together, same pinned manifest.

## G. Path classes

Class 2: tests/init_slice5_test.rs, scripts/slice5/. Class 3 (REQUESTED):
codepairs.rs; explain.rs (RECV entry, S5-D1); the drift list; the status
and header text (S5-D4); the 12 fixtures of S5-D3.

## H. Decision points for ruling

S5-D1 A new abstention code RECV (recommended: it names exactly this
refusal) versus folding the refusal into R6 (no new code, but R6 means a
shape read without being imported, a different fact).
S5-D2 Land SLICE4 and SLICE5 together and measure them together as
STAGE3-EVAL-005 (recommended; S4-D8 asked for exactly this) versus SLICE5
alone first.
S5-D3 The 12 colliding fixtures. Slice 1 n10 pins this bound by name:
superseded under the standing rule of S3-D1, original kept verbatim, its
workspace becoming S5-F2. The seven SLICE4 fixtures and the four S4-D7
re-pins read through `o: any`, which says nothing about chains or
silence; they change to `o: {Name}`, originals kept verbatim, so each
still tests what it was written for. They were frozen at Gate 1 of a
slice that never landed.
S5-D4 The proposal status line `PROPOSED (heuristic; receiver of field
reads not verified)` and the init header sentence `The receiver of those
reads is NOT verified` become `PROPOSED (heuristic; each counted read is
on a receiver declared as the shape in this file; types are not
inferred)` and the matching header sentence. Every proposal changes one
line; nothing else in it moves.
S5-D5 STAGE3-EVAL-005 precision: the registered test is the out-of-sample
set, every surviving proposal never adjudicated before (50 by the
prototype), two blind agents, conservative merge, floor 60 percent. The
54 previously adjudicated survivors are re-adjudicated on their new
evidence and reported, not tested. Retention of adjudicated TRUE pairs is
reported beside it (prototype: 41 of 57).
S5-D6 The cost, stated for ruling: about a quarter of the adjudicated
true contracts (16 of 57 in the prototype) become RECV abstentions,
because their receiver's type is inferred (a store field, a return value
from another file, a type guard), which init cannot see without a
parser. Each stays visible, with its counts. Accept, or ask for a wider
B2.

Pre-registration proposed for ruling (from the engine prototype, before
any slice code): code-pair proposals 206 to 104, shapes 294 to 126, RECV
168, 0 silent, strings proposals unchanged (9), M-S-3 at least 60 percent
on the out-of-sample set.

## I. Stop conditions

Stop and report on: any proposal, shape or read outside the SLICE4 result;
any dropped shape without a RECV line; any fixture collision beyond the
12 of S5-D3; any need for a parser or a new dependency.

## J. Rulings of 2026-10-20 (recorded)

S5-D1 RATIFIED: the code is RECV (confirmed against the example name
UNRESOLVED-RECV-UNTYPED, which was not adopted).
S5-D2 RATIFIED: SLICE4 and SLICE5 land together and are measured together
as STAGE3-EVAL-005; SLICE4 stays unlanded until it passes all seven.
S5-D3 RATIFIED as drafted: n10 superseded under the standing rule of S3-D1;
the seven SLICE4 fixtures and the four S4-D7 re-pins read through a
receiver declared as the shape; originals kept verbatim. (The ruling text
placed all 12 in tests/init_slice1_test.rs; they are in the slice 1,
SLICE3B and SLICE4 suites, as section 0 measured.)
S5-D4 RATIFIED after one clarification: the ruling first proposed
"receiver type annotation verified in local scope; field writes excluded",
which the engine cannot back (receivers are names, not scopes, B4; casts,
type arguments and return types count, not only annotations). The pinned
line is the accurate form: `PROPOSED (heuristic; each counted read is on a
receiver declared as the shape in this file; field writes excluded; types
are not inferred)`.
S5-D5 RATIFIED: M-S-3 is tested on the 50 never-adjudicated survivors.
S5-D6 RATIFIED: the 16 inferred-receiver contracts become RECV, visible.
Pre-registered by the founder: code-pair proposals about 104, RECV +168,
M-S-3 at least 60 percent out of sample, M-S-4 0, M-S-5 100 percent, 10 of
10 exit 0.
After STAGE3-EVAL-005 (M-S-5 FAIL, S5-F-01): hold all three slices; fix
lock and check in LOCK-SLICE1 (check never reports HOLDS on a zero-binding
contract; lock reads one-line type members); land SLICE4, SLICE5 and
LOCK-SLICE1 together once all seven metrics pass.

## K. Results

Gate 0 PASS: 56 of 56 crate hashes identical at c406834 (ledger
153ada8d09c8, Mac and sandbox). The SLICE4 patch applied cleanly; its
rebuilt binary is 2823f90b0161, the EVAL-004 binary, byte for byte.
Gate 1 PASS: fixtures frozen before engine code: init_slice5_test.rs
(46911ff46b0f; S5-F1 to F10, R1, R2, N1 with the golden recorded on the
SLICE4 binary, M1) and the S5-D3 and S5-D4 changes in the slice 1, 3,
3B and 4 suites and the explain registry count (34 entries); 27 tests
failed on the SLICE4 baseline as declared.
Gate 2 PASS: the receiver module and the object-shape branch in
codepairs.rs, RECV in explain.rs, the S5-D4 text in init.rs; 3 source
files; the frozen fixture files byte-unchanged.
Gate 3 PASS: 10 of 10 section E mutants caught from scratch; the SLICE4
harness 11 of 11 and the SLICE3B harness 9 of 9 on the combined engine.
Gate 4 PASS: 321 of 321 tests, 0 compiler warnings.
Gate 5: STAGE3-EVAL-005 M-S-3 PASS (86.0 percent out of sample), M-S-5
FAIL 87 of 88 (docs/evidence/stage3/eval005/LEDGER.md).

Original kept verbatim (S5-D3):

    fn n10_receiver_not_verified_is_pinned()
    expect(&o, &[order_pair("b/src/x.ts", "production")], &[], &[], "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE");
    assert!(o.proposals[0].evidence.contains(&STATUS.to_string()));
