# JUDGE-R3 r0: three false accusations, three rules corrected (r0, FROZEN)

Founder ruling 2026-11-03: targeted judge fixes (option B); a VIOLATED
finding with no counterexample keeps its verdict and states why no
payload was built (v0.2.7 behaviour, unchanged); the independent check
of printed payloads is a release gate. Frozen with
crates/invar-falsify/tests/judge_r3_test.rs, the amended cx_f12, the
relabelled MANIFEST.tsv and label-free.py BEFORE implementation; freeze
record: JUDGE-R3-SPEC.frozen.

## Why

COUNTEREXAMPLE r0 built no payload for three commits labelled real, and
hand checks with Python jsonschema showed none exists: each was a false
accusation, posted publicly in SchemaStore#6587 and corrected there.
Rules are claims; payloads are proof. Each wrong rule is corrected here.

## Rulings

J1 (patternProperties). A property removed from a closed object is
VIOLATED only when the post object has no `patternProperties`. With
`patternProperties` present, a pattern may still admit the key, and this
judge does not evaluate regular expressions: the path is listed in
`not_evaluated_paths` (as D1), never accused. (4836277f2, pyproject
`dependency-groups.dev`.)

J2 (enum added over a finite domain). When a field gains an `enum` and
had none, and its base values form a finite set, the judge compares that
set with the new enum, as an enum narrowing: lost values are VIOLATED
`narrowed` ("no longer accepts ..."), and no lost value is no witness
plus a note. Finite: `type` exactly `boolean` (true, false), or exactly
`integer` with a lower and an upper bound (inclusive or exclusive), at
most 256 values. Any other base domain keeps r2 behaviour (VIOLATED
"now accepts only"). (0091e807a, poe `verbosity`: integer -1..1 to enum
-2..2.) The note: `enum added: <path> now lists every value valid before (<values>).`

J3 (required inside allOf). The names an object requires are its own
`required` plus the `required` of each direct member of its `allOf`
(members are conjunctive). A name required at base either way is not
newly required at post; a name newly required at post either way is
escalated. (b514ab55c, mkdocs `site_name`.)

Unchanged: everything else, including D1 for fields under combinators.

## Fixtures (judge_r3_test.rs)

- r3_f01 J1: removal from a closed object whose post has
  `patternProperties`: exit 4, `unjudged_construct`, path listed, no
  `removed:` line (kills R3-M1).
- r3_f02 J1 guard: the same removal without `patternProperties`:
  VIOLATED.
- r3_f03 J2: integer -1..1 to enum [-2..2]: HOLDS, the note exactly
  (kills R3-M2).
- r3_f04 J2: integer 0..5 to enum [0,1,2]: VIOLATED narrowed, "no longer
  accepts 3, 4, 5", counterexample `{"f":3}`.
- r3_f05 J2: boolean to enum [true]: VIOLATED narrowed, "no longer
  accepts false".
- r3_f06 J2 guard: string to enum ["a"]: VIOLATED "now accepts only".
- r3_f07 J3: `site_name` required inside base `allOf`, at root at post:
  HOLDS (kills R3-M3).
- r3_f08 J3: newly required inside post `allOf`: escalated (kills R3-M4).
- r3_f09 J3 guard: required nowhere at base, at root at post: escalated.

## Amendment to a frozen fixture (stated, old hash kept)

cx_f12 used J1's exact case to show a claim the counterexample checker
could not confirm. J1 removes the claim, so the fixture keeps its
purpose (CX-M2 killer: a real VIOLATED claim, the post check unknown)
with a closed post object that carries `propertyNames` instead: the
removed key is truly rejected (`additionalProperties: false`), and the
checker meets `propertyNames` first. Expected: null, reason
`propertyNames is not checked by this judge`. Old hash in
COUNTEREXAMPLE-SPEC.frozen; new in JUDGE-R3-SPEC.frozen.

## Corpus relabel (MANIFEST.tsv, stated)

4836277f2, 0091e807a, b514ab55c: real to false_alarm (evidence:
COUNTEREXAMPLE-SPEC.md Amendment A, hand-checks.py). Real 17, false
alarms 57. replay.sh acceptance unchanged: every real commit VIOLATED,
every stated one NOT EVALUATED, no false alarm VIOLATED, no holds commit
newly VIOLATED.

## Label-free gate (label-free.py)

Runs `invar judge --contracts` on every commit of the corpus (479) and
checks every printed payload with Python jsonschema, without reading the
labels. Pre-registered:

- every printed payload is valid at base and invalid at post (any
  failure fails the release);
- reported as measured, no threshold: VIOLATED commits, confirmed (at
  least one checked payload), unconfirmed (listed, for review by hand).

## Mutation operators

- R3-M1 patternProperties check dropped: r3_f01.
- R3-M2 finite domain not compared: r3_f03.
- R3-M3 allOf members ignored at base: r3_f07.
- R3-M4 allOf members ignored at post: r3_f08.
