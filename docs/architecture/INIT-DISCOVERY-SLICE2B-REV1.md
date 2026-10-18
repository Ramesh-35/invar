# INIT-DISCOVERY-SLICE2B-REV1: K3 precision revisions (fixture-first)

Status: DRAFT r1 for ruling. Not frozen. No engine code is authorized until the
decision points in section K are ruled and this file is frozen. The frozen
INIT-DISCOVERY-SLICE2B.md (r1, 3d3f6b8) is not edited; this revision amends it by
reference only.

Authorization: ruling of 2026-10-15 20:00 ("SLICE 2b REVISION AUTHORIZED").
Evidence base: the K3 development probes (EXPOSURE-LOG, commit 431b9aa).

## 0. Purpose and honest bounds

The probes found three defects that are removable by construction, with no
threshold tuning:

- P3: a list the TypeScript compiler already checks against `keyof T` adds nothing
  to a lock (3 of 26 proposals).
- P4: test-support directories were labeled production (`testUtils`) and shown as
  production usage.
- P5: six proposals from one strapi test file are one coupling shown six times.

Bounds, stated before any code:

B1. This revision removes noise. It does not raise recall. The strapi break pair
(P1) stays missed because D1 stays frozen.
B2. It does not touch the wrong-receiver class (9 of 26). The probe shows that class
is a scope question, not addressed here (section M, V1).
B3. Every rule below is judged on synthetic fixtures. The three probe repositories
are development cases; re-running them after implementation is a regression record,
not an evaluation, and no precision claim follows from it.
B4. Declared predictions (section I) are written before any run and may be falsified.

## A. Scope

In scope, exactly three remedies:

(a) TSC-GOVERNED: abstain on lists that a type annotation ties to `keyof T`.
(b) Test-path categories: `testUtils/`, `__mocks__/`, `fixtures/` become category `test`.
(c) Same-file duplicate collapse: one proposal per (list file, shape) with the other
list sites printed as evidence.

Out of scope, named so nobody assumes them: D1 (threshold, frozen); the `server/`
category gap (labeled unknown; not ratified); any change to check semantics R-K3-7;
any new producer or ORM source; scope tiering (section M, proposal only).

## B. Numbered requirements

R-R1-1 TSC-GOVERNED. A K3 candidate that would otherwise become a proposal (all of
R-K3-1 to R-K3-5 pass, unique receiver, query position, not dynamic, not a version
family) is replaced by `ABSTAIN TSC-GOVERNED` when its declaration carries a
recognized keyof constraint. Recognized forms, matched on masked text (comments and
string contents cannot match):
- trailing, after the list's closing bracket and an optional `as const`:
  `satisfies <TYPE>`, any list form (`const:`, `prop:`, `call:` lists);
- annotation on a `const` list: `const NAME: <TYPE> = [...]`.
`<TYPE>` is one of: `(keyof X)[]`, `readonly (keyof X)[]`, `Array<keyof X>`,
`ReadonlyArray<keyof X>`, where `X` is an identifier path or `typeof IDENT`.
Everything else is unrecognized and leaves behavior exactly as in 2b. A cast
(`as (keyof X)[]`) is NOT recognized: a cast is an assertion, not a check.
X need not be the matched shape: the compiler checks the list against X, so a lock
adds no protection the compiler lacks for X, and an unrelated X is the conservative
direction (an abstention, never a wrong pairing). The reason names X and the form.
Check is unaffected: a contract already locked keeps its R-K3-7 semantics even if
its list later gains a keyof constraint. Counterexamples: CM-R1-1 to CM-R1-6.

R-R1-2 Precedence. TSC-GOVERNED is decided last, only for would-be proposals. A list
that currently abstains NON-DISTINCTIVE-KEY, AMBIG-RECEIVER, VERSION-FAMILY,
UNSUPPORTED-STRING-DYNAMIC or UNSUPPORTED-ORM-SOURCE keeps that exact code. This
keeps every 2b abstention line byte-identical.

R-R1-3 Test-support categories. `category()` returns `test` when any directory
segment equals `testUtils`, `__mocks__` or `fixtures` (exact, case-sensitive segment
equality; the existing test checks keep their position first, so `src/testUtils/x.ts`
is `test`, not `production`). `is_test_path` (K1 eligibility) is NOT changed. A
`test` proposal is still emitted and labeled; labeling never suppresses (R-K3-6).
Counterexamples: CM-R1-7 to CM-R1-9.

R-R1-4 Same-file collapse. Group K3 proposals by (list file, shape file, shape name).
For a group of size n > 1:
- the representative is the list with the largest |O|; ties go to the lowest list
  line, then the lexicographically smallest listat;
- exactly one manifest block and one lock entry are emitted, from the representative;
  the lock binds the representative's overlap only;
- one extra evidence line `S7 also: <listat>@<line> |O|=<k>; ...` lists the other
  sites sorted by line; n = 1 prints no S7 line, so the output is byte-identical to 2b;
- groups never span files, shapes or kinds. TSC-governed lists leave before grouping.
- SUMMARY `proposals=` counts blocks after collapse.
Honest bound printed in the S7 line's trailing note: "other sites not locked; add a
block by hand to lock them". Counterexamples: CM-R1-10 to CM-R1-13.

R-R1-5 Independence and determinism. Where no list has a keyof constraint, no path
segment is one of the three new names, and no group has n > 1, init output, manifest
and lock are byte-identical to the landed 2b engine on every fixture and on F01.
Repeated runs are byte-identical.

R-R1-6 Budget. No new file pass; the keyof check reads text already masked by `Text`;
scan time on the probe trees stays within 10 percent of the 2b measurement.

## C. Abstention contract (codes defined before code)

| Code | Fires when |
|---|---|
| TSC-GOVERNED | R-R1-1: a would-be K3 proposal whose list carries a recognized `keyof` constraint |

Printed as `ABSTAIN TSC-GOVERNED <path>:<line> list <label> is constrained by <form> keyof <X>; the compiler checks its names against <X>, so a lock adds nothing; not proposed`.
It is an init line only. Init has no exit code for abstention, and the ISA is
untouched: `check` still returns exit 4 only for an unextractable locked contract.

## D. Countermodels (defined before fixtures and code)

CM-R1-1 keyof of an unrelated type: list qualifies against shape S but is
constrained by `keyof Other`. Declared: TSC-GOVERNED (conservative), reason names Other.
CM-R1-2 `satisfies readonly string[]`: not keyof. Declared: proposal as in 2b.
CM-R1-3 `keyof typeof OBJ` constraint. Declared: TSC-GOVERNED.
CM-R1-4 the words `satisfies readonly (keyof T)[]` inside a comment or a string next
to a qualifying list. Declared: proposal as in 2b (masking).
CM-R1-5 a `satisfies (keyof T)[]` that belongs to a different statement later in the
file. Declared: proposal as in 2b (the check binds to this list's own closing bracket).
CM-R1-6 `as (keyof T)[]` cast. Declared: proposal as in 2b.
CM-R1-7 near-name directories `mytestUtils/`, `testUtilsExtra/`, `fixture/`,
`test-utils/`, `__fixtures__/`. Declared: category unchanged (exact segments only;
extra spellings need unseen-repo evidence first).
CM-R1-8 `fixtures.ts` as a file name, not a directory. Declared: unchanged.
CM-R1-9 `src/fixtures/seed.ts` and `src/testUtils/x.ts`. Declared: `test`.
CM-R1-10 same-file lists matching two different shapes. Declared: no collapse.
CM-R1-11 the same list text in two files with the same shape. Declared: no collapse.
CM-R1-12 a larger list appears later in the file than a smaller one. Declared: the
larger is the representative.
CM-R1-13 a site whose overlap names are not a subset of the representative's.
Declared: listed as a site with its own count; never merged into the lock.

## E. Fixture families (declared outcomes, written before any engine edit)

A (TSC-GOVERNED): A1 CM trailing `as const satisfies readonly (keyof T)[]` on a
qualifying `const` list: one TSC-GOVERNED line, zero proposals, SUMMARY proposals=0.
A2 `const X: (keyof T)[] = [...]` annotation. A3 `ReadonlyArray<keyof T>` and
`Array<keyof T>` on two lists: two lines. A4 CM-R1-3. A5 CM-R1-1 (reason names the
unrelated type). A6 a locked contract whose list later gains the constraint: `check`
unchanged (exit 0 when consistent, exit 1 when a locked name goes stale). A7 CM-R1-2.
A8 CM-R1-6. A9 CM-R1-4 and CM-R1-5. A10 a keyof-constrained list that would abstain
NON-DISTINCTIVE-KEY keeps that code (R-R1-2). A11 trailing form on a `prop:` list.
B (categories): B1 `testUtils/`, B2 `__mocks__/`, B3 `fixtures/` each label `test`.
B4 CM-R1-9 nested forms. B5 CM-R1-7 and CM-R1-8 unchanged. B6 a K1 or K2 proposal
under `src/fixtures/`: proposal set identical to 2b, label per ruling RD1. B7 a
`test` K3 proposal is still emitted.
C (collapse): C1 six same-shape lists in one file (strapi model): one block, S7 lists
five sites, lock overlap is the representative's. C2 CM-R1-12. C3 CM-R1-10. C4
CM-R1-11. C5 a stale name present only in a site: `check` HOLDS (the documented
bound). C6 n = 1: output byte-identical to 2b. C7 three lists, one governed:
one TSC-GOVERNED line plus one collapsed block with one site. C8 CM-R1-13.
M (shared): M1 all 2b fixtures, all slice 2 fixtures and F01 pass unchanged. M2
determinism (two runs byte-identical). M3 manifest and lock grammar unchanged (a 2b
lock parses and checks identically). M4 SUMMARY counts after collapse.
Total: 11 + 7 + 8 + 4 = 30 fixtures; Gate 4 target is 149 plus the fixtures that are
tests (no fixture is merged away to reach a number).

## F. Mutation set (harness reports CAUGHT / SURVIVED / BUILD-FAILED)

R1-M1 keyof detection removed. R1-M2 TSC decided before the other abstentions
(R-R1-2 broken). R1-M3 masking bypassed (comment or string text matches). R1-M4
trailing search window widened to later statements. R1-M5 `string[]` treated as
keyof. R1-M6 `as` cast treated as governed. R1-M7 check skips locked lists that gain
a constraint. R1-M8 `testUtils` dropped from the set. R1-M9 `__mocks__` dropped.
R1-M10 `fixtures` dropped. R1-M11 segment equality loosened to substring. R1-M12
`test` proposals suppressed instead of labeled. R1-M13 group key drops the shape.
R1-M14 group spans files. R1-M15 representative chosen by first seen, not by |O|.
R1-M16 lock binds the union of sites. R1-M17 collapse removed. R1-M18 S7 printed
for n = 1. Each must be killed by a named assertion failure; BUILD-FAILED and
APPLY-FAILED are not kills; a control (unmutated) run must be green.

## G. Gates 0 to 5

Gate 0 hash ledger (section H lists), plus a grep proving no existing test or fixture
contains `testUtils`, `__mocks__`, `/fixtures/` or `keyof` (so the existing suite
cannot move). Gate 1 fixtures written first as failing tests, each failing for the
declared reason, before any engine edit. Gate 2 implementation to the allowed paths
only. Gate 3 mutation harness all CAUGHT with control green. Gate 4 full suite green,
zero warnings, F01 and all 2b and slice 2 outputs byte-identical. Gate 5 docs: a
Section M ledger appended to this file, STATUS.md entry, EXPOSURE-LOG regression
record with predictions scored. Failure at any gate stops the arc for ruling.

## H. Path classes

Class 1 (untouched, hash-verified): surface.rs, backtest.rs, invariant.rs, config.rs,
yaml.rs, codepairs.rs, both Cargo.toml files, Cargo.lock, the five Experiment C
artifacts, the frozen INIT-DISCOVERY-SLICE2B.md.
Class 2 (verification artifacts): tests/init_slice2b_rev1_test.rs, scripts/slice2b/rev1/.
Class 3 (REQUESTED, not assumed): valuepairs.rs, the `category()` function only
(one dir set; K1 and K2 labels follow, see RD1).
Class 4 (implementation): valuepairs/strings.rs (keyof detection, collapse),
init.rs (S7 line only). lock.rs, check.rs and main.rs are not touched.

## I. Corpus, evaluation and declared predictions

No new calibration on the probe repositories. After Gate 4 the same three trees are
re-run read-only. Predictions, recorded now and scored later:
- formbricks: 13 proposals become 10 (the three `satisfies keyof` lists become
  TSC-GOVERNED); wrong-receiver count stays 7; strict precision stays 2 of 10.
- strapi: 13 proposals become 8 (six `fieldsToPick` sites collapse to one block with
  five sites), provided the six lists share one shape, which Gate 0 confirms first;
  the testUtils-labeled proposals change from production to test.
- hoppscotch: 0 proposals, unchanged.
Any other delta is a finding and stops the arc. A confirmed prediction shows the
revision does what it says; it is not a precision result.

## J. Decision points for ruling

RD1 Category change is shared (K1, K2 and K3 labels follow `category()`), with
`is_test_path` untouched. Recommended; the alternative (a K3-local copy) leaves two
diverging label tables in one module.
RD2 TSC-GOVERNED fires for any `keyof X`, not only the matched shape. Recommended
(abstain-safe).
RD3 The collapsed lock binds the representative only; other sites are evidence.
Recommended (a union would bind names the check does not read from that list).
RD4 Recognized type forms are exactly the four in R-R1-1 plus `typeof`. Further
forms need an unseen-repo observation.
RD5 Class 3 grant for the `category()` function in valuepairs.rs.

## K. Stop conditions

Stop and report on: any 2b, slice 2 or F01 output change outside the declared deltas;
a Class 1 hash change; a mutation kill that is a build or load failure; a fixture
that cannot fail for its declared reason at Gate 1; a prediction in section I
falsified; any pressure to widen a recognized form, a directory name or the stoplist
from the probe repositories; any need for an AST or a new dependency.

## L. Rejected

Lowering D1 to capture strapi `lastUsed`. Treating a `keyof` cast as governed.
Suppressing `test` proposals (label only). Union locks for collapsed sites. Substring
matching of directory names. Collapsing across files or shapes.

## M. Volunteered proposals (NOT part of this revision; for separate ruling)

V1 Scope-tier labels. The probe data already shows a split by link strength:
5 proposals were import-linked and 0 were false receivers (3 compiler-governed, 1
true, 1 unclear); 21 were same-package and 9 were false receivers. Five is far too
small to rule on, so this is a hypothesis: print each proposal's link strength
(`same-file`, `import`, `same-directory`, `same-package`) as an explicit tier and
measure the false-receiver rate by tier on an unseen repository set before any
behavior depends on it. It must not drop same-package proposals: the one real strapi
break pair is same-package.
V2 `invar explain <CODE>`: prints the code's definition, the exact condition that
fired, and the sanctioned next action, from one table that the tests also read, so
the documentation cannot drift from the engine.
V3 Metamorphic invariance suite: output must be byte-identical under CRLF versus LF,
a UTF-8 BOM, reordered files, injected comments, and reformatted whitespace. One
generic harness over every fixture gives a trust claim no single fixture can.
V4 Engine fingerprint: record a digest of the rule set (codes, thresholds, stoplist)
in the lock as evidence only, so a result can be tied to the exact rules that
produced it; a golden-output snapshot per released version makes any silent behavior
change a visible diff.
V5 Stage 3 evaluation: a pre-registered protocol on unseen repositories with labels
fixed before any run, publishing pooled precision and the wrong-receiver rate with
intervals. Without it, D1 and every future threshold remain unjustified by anything
but fixtures.

## N. Rulings of 2026-10-15 20:05 (recorded)

RD1 to RD5 RATIFIED AND FROZEN as recommended. Volunteered items: V1 scope tiers
ratified as telemetry only (no filtering change); V2 `invar explain <CODE>` to the
backlog; V3 metamorphic invariance suite mandated for Gate 4; V4 engine fingerprint
ratified for the proof spec; V5 Stage 3 pre-registered evaluation ratified.

Interpretations applied during implementation (for ratification):
- N1 RD4 lists `keyof typeof X` as a fifth form. It is implemented as an operand that
  every wrapper accepts, e.g. `readonly (keyof typeof X)[]`, because a bare
  `keyof typeof X` is not an array type and cannot annotate a list.
- N2 V1 (scope tier telemetry): every proposal already prints its tier as the S4 line
  (`same-file`, `import`, `same-directory`, `same-package`). A collapsed group shares
  one list file and one shape, so every site has the representative's tier; S7 carries
  no extra tier text. No output change was needed.
- N3 The code `TSC-GOVERNED` already exists from slice 2 (K2: a shape consumed through
  types). K3 reuses the name with its own reason text for the same underlying idea (the
  compiler governs it). Section C above described it as new; it is a reuse.

## O. Implementation Evidence Ledger (appended at Gate 5; sections A to N are unchanged)

Status: IMPLEMENTED against the ratified revision. Detection capability on real
repositories remains NOT ESTABLISHED. Nothing here is an evaluation result.

### O1. Gate results

| Gate | Result |
|---|---|
| 0 baseline | PASS: Class 1 files, Cargo files and the five Experiment C artifacts byte-identical; landed 2b files identical between the build tree and the repository |
| 1 fixtures first | 33 tests written before any engine edit; against the unchanged engine 19 failed for their declared reasons and 14 (negative controls and the metamorphic suite) passed |
| 2 implementation | strings.rs (keyof recognizer, TSC-GOVERNED, collapse), valuepairs.rs (one line in `category()`), init.rs (S7 line) |
| 3 mutations | PASS: 18 of 18 CAUGHT by assertion failure, control green, none SURVIVED, BUILD-FAILED or APPLY-FAILED (scripts/slice2b/rev1/mutate.sh) |
| 4 regression | PASS: 182 of 182 (16 lib, 50, 40, 25, 18 slice 2b, 33 REV1), zero warnings; slice 2 harness 14 of 14 and slice 2b harness 12 of 12 still CAUGHT |
| 5 documentation | this section, STATUS.md entry, EXPOSURE-LOG.md regression record |

### O2. Corrections made during the arc (stated, not hidden)

- Fixture r1_m4 first expected `abstentions=0`; the SUMMARY line counts slice 1's H2 line, so the correct value is 1. Fixture error, engine unchanged.
- Mutation 16 listed fixture r1_c1 as a catcher. r1_c1's representative already holds every name, so a union equals it and the fixture cannot distinguish the mutant. Mutant 16 is killed by r1_c5 and r1_c8; the `# catch:` header was corrected. Declaration error, not an engine change.
- Mutation R1-M3 of section F ("masking bypassed") is an equivalent mutant in this implementation: no scan path reads comment or string text, because the recognizer starts at the list's closing bracket and accepts only `as const` or `satisfies`. It was replaced by 03-trailing-form-ignored. The masking cases remain fixtures (a9).
- One 2b mutation patch (12-test-category-dropped) no longer applied because the one-line `category()` change and the `also` field moved its context. It was regenerated with the same mutation and the same catching test; the slice 2b harness re-ran 12 of 12.
- An earlier overlapping harness run shared a build directory with another run and gave false control failures; it was discarded and the harness re-run alone.

### O3. Metamorphic suite (ruling V3)

CRLF line endings, a UTF-8 BOM on source files, trailing comments on every line, and
reversed file creation order each leave `invar init --stdout` byte-identical on four
bases (plain, governed, collapsed, test-support). All four passed against the
UNCHANGED engine as well, so no engine change was needed for invariance. Not covered:
a BOM on package.json (invalid JSON for strict parsers), comments that shift line
numbers, Windows paths.

### O4. Regression record on the development trees (read-only; NOT an evaluation)

| Tree | Blocks before | Predicted | Actual | Notes |
|---|---|---|---|---|
| hoppscotch | 0 | 0 | 0 | control holds |
| formbricks | 13 | 10 | 8 | 3 TSC-GOVERNED lists, plus 3 `prop:fields` lists in one file collapsed to 1 (not foreseen) |
| strapi | 13 | 8 | 6 | six `fieldsToPick` lists collapsed to 1 as predicted, plus two more same-file pairs (`prop:fields` x2, `permissionFields` and `sanitizedPermissionFields`) not foreseen |

Two predictions were falsified, in the same direction and for the same reason: the
rule collapsed more same-file same-shape groups than I enumerated. The engine did what
R-R1-4 says; the predictions undercounted. Per stop condition K this is reported for
ruling and not adjusted. No precision figure is derived: the wrong-receiver class (B2)
is untouched, and by the earlier triage 5 of the 6 remaining formbricks ZContactTableData blocks are the
false-receiver pattern and 1 is unclear. The labels moved as intended: strapi had 6 test-labeled
proposals (all in reducer.test.js) and now has 2 test-labeled blocks, the collapsed
reducer.test.js block and testUtils/data.js, which was labeled production before.
Clean timings on an idle machine: hoppscotch 0.93s, formbricks 3.5s, strapi 0.87s.

### O5. Protected hashes (first 12 hex), verified unchanged after the change

surface.rs 71b521ce7528, backtest.rs df9ef6e87de4, invariant.rs 49972028d11b, config.rs d30f5e4bf6b3, yaml.rs 1254fde8cdd6, codepairs.rs 004a26dec58e, crates/invar-falsify/Cargo.toml a8023ba46533, root Cargo.toml 7c5f4cee5d13, Cargo.lock 81bcb0ff4e12; lock.rs 357d26c28503, check.rs 0dcbe09af5f8, main.rs 7b4768b0543f, lib.rs af72c0c7d1d5; Experiment C: DEFINITION.md 090d8574fb2f, EVALUATION-PROTOCOL.md 02d0c4299aa4, GROUND-TRUTH-REPORT.md 00daf71769bb, corpus.tsv 6c3982b2105f, labels.tsv 8b8d1b9002c7.

### O6. Files changed

strings.rs 07e91fae4374, valuepairs.rs 42e158a9e22f (one line), init.rs 7b8a8a80b05e (one block), tests/init_slice2b_rev1_test.rs, scripts/slice2b/rev1/ (harness and 18 patches), scripts/slice2b/mutations/12-test-category-dropped.patch (regenerated).
