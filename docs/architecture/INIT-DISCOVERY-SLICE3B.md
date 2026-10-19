# INIT-DISCOVERY-SLICE3B: earned silence

Status: DRAFT r1 for ruling. Implementation NOT authorized. Parent:
STAGE3-EVAL-002 M-S-4 FAIL (findings S3-F-10 to S3-F-13). Baseline: the
clean tree at 0866583 plus the unlanded SLICE3 implementation (preserved
as docs/evidence/slice3/slice3-implementation.patch). SLICE3 does not land
without this slice (S3B-D6).

Within the HELLBREAKER hypothesis this slice makes one HELLBREAKER
Structural Verification guarantee provable: every import init examines
ends in a proposal, an abstention with a reason, or positive evidence that
it is not a shape. Nothing ends in unexplained silence.

## 0. The problem, found before it is solved

STAGE3-EVAL-002 found 4 silent drops. Three causes were named (S3-F-10
`export type *` unparsed, S3-F-11 regex literals unmasked, S3-F-12
`Found::NotShape => {}` silent). Patching the first two would solve the
instances found. The problem underneath is S3-F-12 alone: init's silence
carries no evidence, so "I found nothing" reads exactly like "there is
nothing", and the next syntax the scanner misses will drop silently again.

Measured before any design (classify_notshape.py, a census of every silent
NotShape event across the six multi-package repositories at the pinned
SHAs, with an independent classifier sharing no engine code):

| Silent NotShape events: 3,867 | Count | Silence |
|---|---|---|
| value import; the target package declares the name only as a value (function, const, let, var) | 3,381 | earned |
| the name is a class or enum (outside the v1 shape scope: interfaces and type aliases) | 268 | earned |
| type-only import of a value (a `typeof` use) | 13 | earned |
| the target package declares the name as a TYPE that init never reached | 84 | unearned |
| the target package's sources declare the name nowhere | 119 | unearned |
| declared both as a type and as a value | 2 | unearned |

205 of 3,867 silences (5.3 percent) are unearned. All 4 EVAL-002 drops are
among them: ListMarkerAdvance is declared at
packages/element/src/listMarker.ts:193, ChromeSettings at
packages/puppeteer-core/src/common/Configuration.ts:87.

Measured, and dropped from this slice: parsing `export type * from`
(S3-F-10) in a throwaway engine copy changed nothing on the evaluation set:
0 fewer NotShape events, 0 new proposals (puppeteer's chain is also deeper
than init follows). Regex-literal masking (S3-F-11) needs a division-versus
-regex context rule, a heuristic, and its silent drop is already turned
into an abstention by the rule below. Both are recorded backlog, each to
come with its own evidence.

Bounds, stated before any code:

B1. Silence must be earned. An import that resolves but finds no shape is
silent only when the target package's own sources declare the name as a
value, a class or an enum, and nowhere as an interface or type alias.
Every other case abstains with a reason.
B2. The evidence reader is deliberately plain: a word scan, over the text
init already masks (strings and comments), for `interface X`, `type X`,
`function X`, `const X`, `let X`, `var X`, `class X`, `enum X`, within the
target package's files init scanned. It uses no bracket depth, so S3-F-11's
corruption cannot blind it. It is never used to propose, only to decide
whether silence is allowed.
B3. Proposals are untouched: the 186 STAGE3-EVAL-002 proposals stay
byte-identical. Only new ABSTAIN lines appear; every existing proposal and
abstention line keeps its bytes, and the only other change is the
abstention count in the [discovery] block and the SUMMARY line. Zero new
dependencies, exit or format changes.
B4. Pre-registered prediction, recorded here before any code: on the same
pinned manifest, exactly the census's 205 new abstentions (84 declared but
unreached, 119 not declared, 2 both); every deviation is reported line by
line at Gate 5, never absorbed; the M-S-4 re-audit finds 0 genuine silent
drops.

## A. Frozen scope

A1. One change point: the `Found::NotShape` arm in codepairs.rs.
A2. Reasons carry an off-ramp: a declared-but-unreached type names its
declaration file and line, so a reader knows where to look.
A3. Out of scope, recorded: `export type *` parsing (S3-F-10, measured
zero yield), regex-literal masking (S3-F-11), barrel depth beyond the
current rule, and the M-S-4 detector's own stoplist bound (S3-F-13).

## B. Numbered requirements

R-S3B-1 Silent iff the word scan finds the name declared as a value, class
or enum in the target package and not as an interface or type alias.
R-S3B-2 Declared as a type, unreached: abstain `{Name} from "{spec}":
declared as a type at {file}:{line}, which init could not reach from
{target}; not guessed` (first declaration in path order).
R-S3B-3 Declared nowhere: abstain `{Name} from "{spec}": not declared in
the sources of package {pkg}; not guessed`.
R-S3B-4 Declared both ways: abstain `{Name} from "{spec}": declared both as
a type and as a value in package {pkg}; not guessed`.
R-S3B-5 Determinism: two runs byte-identical.

## C. Countermodels

CM-S3B-1 a type declared three barrels deep left silent. CM-S3B-2 a type
after an unbalanced regex literal left silent. CM-S3B-3 a function import
turned into an abstention (noise). CM-S3B-4 a class import turned into an
abstention. CM-S3B-5 a declaration inside a comment or string counted as
evidence. CM-S3B-6 a `typeof` type-only import of a value abstaining.
CM-S3B-7 a name declared both ways left silent. CM-S3B-8 a name declared
nowhere left silent. CM-S3B-9 the reason naming the wrong declaration
site. CM-S3B-10 any existing line changing bytes.

## D. Fixtures (declared outcomes, before any code)

B-F1 a type three barrel levels deep: ABSTAIN naming its file:line.
B-F2 an unbalanced regex literal before the declaration (the S3-F-11
shape): ABSTAIN naming its file:line, never silence.
B-F3 a chain through `export type *`: ABSTAIN (S3-F-10 deferred, stated).
B-F4 a function import: silent, byte-identical to today.
B-F5 a class import: silent.
B-F6 a name re-exported from outside the package: ABSTAIN, not declared.
B-F7 a name declared as a const and a type, unreached: ABSTAIN, both.
B-F8 `import type { f }` used as `typeof f`: silent.
B-F9 `interface X` only inside a comment and a string: ABSTAIN, not
declared (the masking holds).
B-N1 every frozen init suite (slice 1, 2, 2B, 2B-REV1, 3) unedited and
green; a frozen fixture whose silence would become an abstention is a
stop condition, not an edit.
B-M1 two runs byte-identical.

## E. Mutation set

1 the arm reverted to silence (B-F1). 2 value evidence ignored, every
NotShape abstains (B-F4). 3 classes not earned (B-F5). 4 the scan reads
unmasked text (B-F9). 5 a type-only import never earns silence (B-F8). 6
both-ways left silent (B-F7). 7 not-declared left silent (B-F6). 8 the
declaration site omitted from the reason (B-F1). Each must die by a named
assertion failure.

## F. Gates 0 to 5, then the measurement

Gate 0 hash ledger at 0866583 plus the SLICE3 patch (Gate 4 hashes
016cf84b9a90, f77c2b8c3c66, d26bb80b56c2). Gate 1 fixtures first. Gate 2
codepairs.rs only. Gate 3 section E, all CAUGHT from scratch. Gate 4 full
suite, zero warnings. Gate 5 ledger, STATUS.md. Then STAGE3-EVAL-003 on the
same pinned manifest, every metric M-S-1 to M-S-7, with the B4 prediction
checked line by line; STAGE3-EVAL-001 and -002 are never overwritten.

## G. Path classes

Class 1: the standing protected set, including explain.rs (no new code per
S3B-D2), init.rs, check.rs, main.rs, completions, both manifests. Class 2:
tests/init_slice3b_test.rs, scripts/slice3b/. Class 3 (REQUESTED):
codepairs.rs. Class 4: none.

## H. Decision points for ruling

S3B-D1 Earned silence as the remedy (recommended: it removes the class, 205
of 205 measured) versus patching only the causes found (S3-F-10, S3-F-11),
which measured zero yield for S3-F-10 and leaves the next unseen syntax
silent.
S3B-D2 Abstention code: reuse UNRESOLVED with the exact reasons of section
B (recommended: consistent with S3-D6, explain registry and every CLI
label untouched) versus a new code UNREACHED (clearer, but an explain.rs
change).
S3B-D3 Classes and enums earn silence under the v1 shape scope.
S3B-D4 The evidence reader is the masked word scan of B2.
S3B-D5 S3-F-10 and S3-F-11 deferred to the backlog with their evidence.
S3B-D6 Landing order: SLICE3 and SLICE3B land together, SLICE3 first, in
one push; SLICE3 never lands alone.
S3B-D7 Class 3 for codepairs.rs.

## I. Stop conditions

Stop and report on: any frozen fixture whose bytes change; any proposal
changing; any deviation from the 205 that cannot be explained line by
line; any need for a parser, a new dependency or a Class 1 file.
