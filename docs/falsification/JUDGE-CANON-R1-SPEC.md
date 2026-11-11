# JUDGE-CANON r1: literals in normal form, and annotations never a verdict (r1, FROZEN)

Ratified by the founder 2026-11-08. Frozen with
crates/invar-falsify/tests/judge_canon_r1_test.rs and two real SchemaStore
changes BEFORE implementation; freeze record: JUDGE-CANON-R1-SPEC.frozen.
Extends JUDGE-CANON r0; every earlier ruling stays frozen as written,
except the two fixtures amended below, each with its reason.

## Why

After v0.2.19, 488 of SchemaStore's changes are NOT EVALUATED. Measured
by what differs at each stated pointer, three spellings of one meaning
recur that the r0 normal form does not see:

- `{"enum": ["x"]}` written `{"const": "x"}` (22 changes, every
  prettierrc option among them);
- `{"enum": [false, true, "x"]}` written as the `anyOf` of
  `{"type": "boolean"}` and `{"type": "string", "const": "x"}` (the
  htmlhint family);
- a node that carries `anyOf`, `oneOf`, `allOf` or `$ref` and differs
  only in annotations (`description`, `markdownDescription`,
  `x-intellij-html-description`): r0 kept the r2 behaviour and reported
  it unjudged, although the diff ignores the same annotations everywhere
  else.

Each is an equality of accepted sets, not an inclusion.

## Ruling CANON (r1)

The normal form of r0 gains three steps, applied at every node after its
children are in normal form and its annotations and unknown keywords are
set aside:

1. Literals. `const: v` becomes `enum: [v]` (with an `enum` present, the
   enum keeps only `v` if it lists it). A node that is exactly
   `{"type": "boolean"}` becomes `{"enum": [false, true]}`; exactly
   `{"type": "null"}` becomes `{"enum": [null]}`.
2. A type constraint beside an enum is folded into it. When a node has an
   `enum` and either a `type` (one name or a list) or an `anyOf`/`oneOf`
   whose branches are each exactly one `type` name, the enum keeps only
   the values of those types (an integral number has both types
   `integer` and `number`), and the type constraint is dropped. The enum
   is then sorted by its values' canonical JSON and made distinct.
3. Union of literals. A node that is exactly an `anyOf` whose branches
   are each exactly an `enum` becomes the one `enum` of their union; the
   same for a `oneOf` whose branch enums are pairwise disjoint (a value
   in two branches is rejected by `oneOf`, so that is not a union).

Then r0's type-list rewrite. Two nodes equal in this normal form are
covered: no witness, no unchecked pointer, and a note at the node where
a rewrite of step 1 to 3 changed the spelling (`  noted: equivalent
after rewriting literals (const, enum, boolean, null) at <pointer>`),
r0's note where its rewrite did. A difference only in annotations is
covered silently, as the diff already treats it: no note, no verdict.
The D1 comparison (a node carrying a combinator or `$ref`) uses the same
normal form and the same rule; r0's exception for annotation-only
differences is withdrawn.

Not covered, deliberately: an enum narrowed or widened (judged as
before, VIOLATED or HOLDS by the existing enum rule); a branch that is a
type other than `boolean` or `null` beside literals (`{"type":
"integer"}` admits infinitely many values and is not a literal); a
`oneOf` whose branch enums overlap; a `const` or `enum` inside a branch
that also carries other keywords; anything an earlier ruling withholds
(a draft migration whose head uses `const` under draft-04 stays with
JUDGE-DRAFT r0: c1_f10 shows the nodes covered and the root still
withheld).

## Fixtures (frozen before implementation)

- c1_f01 `enum` of one value to `const`: HOLDS, note at `/properties/a`.
- c1_f02 redundant `type` dropped beside `enum`, order changed: HOLDS.
- c1_f03 enum to `anyOf` of boolean and a typed const: HOLDS, note.
- c1_f04 guard: an enum narrowed is still VIOLATED, counterexample kept.
- c1_f05 guard: `{"type": "integer"}` beside a literal is not covered:
  NOT EVALUATED.
- c1_f06 `oneOf` of disjoint enums to one enum: HOLDS.
- c1_f07 guard: `oneOf` of overlapping enums is not the union: NOT
  EVALUATED.
- c1_f08 a combinator node differing only in annotations: HOLDS, no note.
- c1_f09 a value the type never admitted is dropped: `{"type":
  "string", "enum": ["a", null]}` to `{"const": "a"}`: HOLDS.
- c1_f10 real bytes, htmlhint 8a87ed47d: every stated node covered, the
  root withheld by JUDGE-DRAFT r0 (`const` is not a draft-04 keyword):
  NOT EVALUATED, reason `unchecked_change`, unchecked exactly `[""]`,
  no unjudged path.
- c1_f11 real bytes, prettierrc a3cf4d531: HOLDS with literal notes.

Amended fixtures, with the reason:

- JUDGE-CANON r0 c_f04 used `null` as the second type. Its two spellings
  admit the same documents, and r1 proves it, so the guard now uses
  `integer`, which has more than one value; what it guards (a shared
  keyword moved inside one branch) is unchanged.
- JUDGE-SUBTYPE r2 r2_f13 changed only a `description` on a node in a
  reference cycle. r1 covers annotation-only differences, so the fixture
  now changes the reference target inside the cycle, which is what its
  name says it tests.

## Mutation operators

- CR1-M1 `const` not rewritten to `enum`                -> c1_f01
- CR1-M2 `boolean` not a literal                        -> c1_f03
- CR1-M3 `oneOf` union without the disjointness check   -> c1_f07
- CR1-M4 the type constraint not folded into the enum   -> c1_f09
- CR1-M5 D1 keeps refusing annotation-only equality     -> c1_f08

## Pre-registered prediction (SchemaStore at v0.2.19, 488 NOT EVALUATED)

Measured by the Python prototype (canon-r1-proto.py, the engine's own
ANNOTATIONS, KNOWN and reach sets read from judge.rs, `$ref` followed
where the walk follows it): exactly the 33 changes listed in the freeze
record move from NOT EVALUATED to HOLDS (21 `unchecked_change`, 12
`unjudged_construct`); no VIOLATED or HOLDS verdict or witness changes
on any corpus; label-free 26/14/50/0, Iglu 5/3/0, replay PASS unchanged.
The r0 prototype undercounted by not following `$ref`; this one follows
it, and the Measured outcome will say where the count still differs.

## Measured outcome (after implementation, 2026-11-08)

- Fixtures 11 of 11; amended c_f04 and r2_f13 pass. Full suite 837 tests.
  Mutants 95 of 95 killed (CR1-M1..M5 new). Clippy silent.
- Label-free 26/14/50/0, Iglu 5/3/0, SchemaStore replay PASS: unchanged.
- Prediction: 28 of the 33 moved ([canon-r1-moves/schemastore.tsv](canon-r1-moves/schemastore.tsv)),
  no other verdict or witness moved on SchemaStore or AsyncAPI, and one
  Iglu change the prototype never looked at moved too (snowflake_config
  1-0-1 to 1-0-2: descriptions added under an `anyOf`, covered by the
  annotation rule, HOLDS). The five that did not move, and why the judge
  is right:
  - four package.json changes: the stated node is `jspm: {"$ref": "#"}`,
    a reference to the document root, which contains `jspm`. JUDGE-SUBTYPE
    r2 cuts a reference cycle and states it; nothing in r1 touches that.
    The prototype followed the reference once and compared the roots.
  - solarxy-config: `"minimum": 0.0` became `"minimum": 0`. The same
    number, two spellings; the judge compares JSON values and these
    differ, the prototype compared Python numbers and they do not. A
    numeric normal form (an integral number written either way) is a
    candidate for a later ruling, not slipped into this one.
- Two implementation facts worth stating. Children are put in normal form
  before their parent, so by the time step 2 looks at an `anyOf` beside an
  enum, a `{"type": "null"}` branch is already `{"enum": [null]}`; step 2
  therefore accepts a branch that is a bare type or a bare enum, which is
  the ruling's intent read in that order. And a type list r0 expands may
  produce an `anyOf` of two literal branches (`["boolean", "null"]`), so
  step 3 runs once more on the expanded node; without that, fifteen HOLDS
  of v0.2.19 regressed to NOT EVALUATED in the first build, caught by the
  prediction run before anything else.
- Amendment B: with r1 covering an annotation-only node before the diff
  reads its keys, mutant RE-M2 (JUDGE-REACH r0) was no longer observable
  through r_f05. Fixture r_f05b (an annotation changed beside a judged
  change leaves no unchecked pointer) was added to judge_reach_test and
  RE-M2 now names it. r_f05 is unchanged.
