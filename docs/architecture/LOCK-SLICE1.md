# LOCK-SLICE1: no vacuous HOLDS; one-line TypeScript members

Status: IMPLEMENTED and LANDED together with INIT-DISCOVERY-SLICE4 and
SLICE5; STAGE3-EVAL-005 closed on all seven metrics (section K).
Authorized by the founder's ruling of 2026-10-20 after
STAGE3-EVAL-005 (M-S-5 FAIL, finding S5-F-01), with its two rules stated
in that ruling; this document records the bounds, fixtures and gates
before code. Lands together with INIT-DISCOVERY-SLICE4 and SLICE5 once all
seven STAGE3 metrics pass. Baseline: 3406683 plus the SLICE4 and SLICE5
evidence patches. Sections 0 to I are as recorded before code; what changed
after is in K.

## 0. Evidence, measured before this document

The failure: pi packages/chord/src/output.ts:7 declares
`export type OutputLimits = { readonly maxBytes: number; readonly maxLines: number; readonly retain: "head" | "tail" };`
on one line. surface.rs `ts_fields` reads one member per line, starting at
the line's first identifier, so it finds no member of OutputLimits;
`invar lock` writes the contract with 0 bindings; `invar check` compares
nothing and prints HOLDS. That HOLDS is vacuous. main has the defect
today: the same pair is proposed in STAGE3-EVAL-003 and -004.

Rule 2 prototype on every TypeScript file of the six cloned repositories
(17,795 files; evidence/lock-slice1/): the surface gains 2,344 member
fields in 856 files; 0 fields are removed and 0 change type. Every one of
40 sampled additions is a real member of its declaration (keyword-named
members such as `type`, `function` and `readonly` included); nested
object members are not flattened into the outer type.

Bounds:

L1 (rule 1) check never reports HOLDS for a contract whose lock entry
binds no field. It reports NOT EVALUATED (exit 4, already in the frozen
exit-code contract), with a reason that names the cause and the remedy.
A contract with at least one binding is unaffected.
L2 (rule 2) lock's TypeScript reader also reads members written on the
declaration's own line: after the first `{` outside `<...>`, `(...)` and
`[...]`, up to its matching `}` or the end of the line, split at `;` and
`,` outside brackets. Each segment is read exactly as a member line is
read today. Nothing else in `ts_fields` changes.
L3 Additive. L2 only adds (scope, field) entries that did not exist; it
never removes or retypes one. A lock written before this slice therefore
cannot become VIOLATED through it, because check reports residue only for
a bound field that disappeared.
L4 No change to init output (init's code-pair reader is separate), to the
exit-code contract, to any outcome tag, or to any other language reader.
Zero new dependencies.

## A. Frozen scope

A1 lock.rs or check.rs: the zero-binding rule (L1). A2 surface.rs: the
same-line member reader (L2). Both are Class 1 files, changed by this
ruling only within L1 and L2.

## B. Requirements

R-L-1 A locked contract with 0 bindings and no residue is NOT EVALUATED
with: `the lock binds no field of this contract (0 bindings), so there is
nothing to compare and a HOLDS would be vacuous; make the producer's
fields readable and run `invar lock --update``.
R-L-2 One-line members are read (L2) on `type`, `interface` and `class`
declaration lines.
R-L-3 Determinism: two runs byte-identical.

## C. Countermodels

CM-L-1 a zero-binding contract reported HOLDS. CM-L-2 a contract with
bindings reported NOT EVALUATED by L1. CM-L-3 a one-line member missed.
CM-L-4 a member of a nested object type (`a: { p: number }`) read as a
member of the outer type. CM-L-5 a comma inside generic arguments
(`Map<string, number>`) splitting a member. CM-L-6 a multi-line
declaration whose bindings change. CM-L-7 a removed one-line member not
reported VIOLATED after lock.

## D. Fixtures (declared before code)

L-F1 one-line `type` with two read members: lock binds 2, check HOLDS
with the durability line, exit 0. L-F2 one-line `interface`: lock binds 2.
L-F3 one-line type with nested object and generic members: lock binds
exactly the outer members the consumer reads, never the nested one. L-F4
L-F1 locked, then a member removed from the one-line type: check
VIOLATED, exit 1, naming it. L-F5 a producer whose shape lock cannot read
(`Pick<...>` of a type in another file): lock binds 0; check NOT
EVALUATED, exit 4, the R-L-1 reason, never HOLDS. L-N1 a multi-line
interface: lock file and check output byte-identical to the baseline
(golden at Gate 1). L-M1 two runs byte-identical.

## E. Mutation set

1 L1 removed (L-F5). 2 L1 applied to every contract (L-F1). 3 same-line
members not read (L-F1). 4 nested braces not tracked, so a nested member
leaks (L-F3). 5 generic angle brackets not tracked, so a comma splits a
member (L-F3). 6 the reason text missing its remedy (L-F5).

## F. Gates

Gate 0 hash ledger. Gate 1 fixtures first. Gate 2 the change. Gate 3
section E from scratch. Gate 4 full suite, zero warnings. Then the
STAGE3-EVAL-005 round trip (M-S-5) rerun on all 88 TRUE contracts with the
new binary, and the other six metrics confirmed unchanged (init output
byte-identical, so their verdicts carry over).

## I. Stop conditions

Stop and report on: any frozen fixture changing; any field removed or
retyped by L2 on the corpus; any zero-binding contract among the 88 after
L2; any init output byte changing.

## J. Rulings of 2026-10-20 (recorded)

The authorization (after STAGE3-EVAL-005): hold SLICE4 and SLICE5; rule 1,
check must never return HOLDS on a zero-binding contract (fail loud); rule
2, lock reads multi-member, single-line type declarations; re-verify the
round trip on all 88 TRUE contracts; land the three slices atomically once
all seven metrics report a genuine, non-vacuous PASS.
L-D1 (raised at Gate 2; stop condition I fired) L-F3 as frozen expected the
type text `Map<string, number>`, but the member reader cuts every type at
its first comma (4,348 fields of the corpus already read that way), and
making it whole would retype existing bindings, which L3 forbids. RULED:
correct the fixture to the reader's existing form, original assertion kept
verbatim; whole type text is its own later slice with a lock-format
migration.

## K. Results

Gate 0 PASS: the crate at 3406683 is the c406834 crate (ledger
153ada8d09c8); the sandbox base is that crate plus the SLICE4 and SLICE5
patches (Gate 4 of SLICE5: 321 of 321).
Gate 1 PASS: tests/lock_slice1_test.rs frozen before code; L-F1 to L-F5
failed on the baseline as declared, L-N1 (goldens recorded on the
baseline) passed.
Gate 2 PASS: check.rs (L1, at the point every outcome is assembled, so it
covers every contract kind) and surface.rs (L2); L-D1 applied.
Gate 3 PASS: 6 of 6 section E mutants caught from scratch, after one
correction (finding L-F-01).
Gate 4 PASS: 328 of 328 tests, 0 compiler warnings, release binary
26824a08805e.
Then: STAGE3-EVAL-005 re-measured on 26824a08805e; all seven PASS
(docs/evidence/stage3/eval005/LEDGER.md, the closing section).

L-F-01 (Gate 3, survivor) Mutant 05 as declared (angle brackets not
tracked in the member split) SURVIVED: a comma inside `<...>` there is
harmless, because the piece after it has no `name:` and is dropped, and
the type text is cut at that comma anyway. The hypothesis in section E was
wrong, not the fixture. Resolution: the dead angle tracking was removed
from the member split (the reader's output is identical on all 17,795
corpus files before and after), and mutant 05 was retargeted at the scan
that finds the opening brace, where angle tracking does matter
(`type X<T extends { a: number }> = { ... }`); fixture L-F6 was added for
it and catches it. Fixture added after Gate 1, recorded here.

L-F-02 (process, sandbox only) Removing a throwaway corpus tool from
crates/invar-falsify/examples/ also removed the tracked file
examples/real_incident.rs from the sandbox tree after Gate 1. It was
caught while assembling this landing, restored from aa03983, and Gate 4
rerun with it in place (328 of 328, 0 warnings, same binary hash). It
never reached a committed patch or the Mac.

Original kept verbatim (L-D1):

    vec!["Limits.byKey Map<string, number>".to_string(), "Limits.inner { pending: number }".to_string(), "Limits.maxBytes number".to_string()]
