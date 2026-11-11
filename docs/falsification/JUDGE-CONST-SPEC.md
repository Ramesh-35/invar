# JUDGE-CONST r0: `const` is an enum of one value, everywhere the judge reads enums (r0, FROZEN)

Drafted 2026-11-08 for ratification. Frozen with
crates/invar-falsify/tests/judge_const_test.rs BEFORE implementation;
freeze record: JUDGE-CONST-SPEC.frozen. Every earlier ruling stays frozen
as written, except the one fixture amended below, with its reason.

## Why

Drafting JUDGE-DRAFT r1, guard fixture d1_f02 (`{"const":"x"}` at the
base, `{"enum":["x"]}` at the head) came back VIOLATED from v0.2.20:
"k now accepts only ["x"]", with "no counterexample built: no candidate
value was valid at the base and rejected now". The judge's enum rule
does not read `const`, so a base that allowed exactly one value was read
as a base that allowed anything, and the head's enum of that same value
became a narrowing. The counterexample builder does read `const`, which
is why it could find nothing to reject. A VIOLATED verdict that is false
is the one thing this judge must never emit, and it has been possible
since v0.2.15 at least. No change in SchemaStore, Iglu or AsyncAPI
history triggers it (measured: no witness in any corpus stands at a node
that carries `const`), which is why no gate caught it.

The judge is of two minds about `const`: JUDGE-D9 reads it as an enum of
one value, JUDGE-CANON r1 normalises it to one, JUDGE-REACH r0 lists it
as a keyword the judge does not read, and the enum rule ignores it.

## Ruling CONST (r0)

Wherever the judge reads a node's `enum`, a node with `const: v` and no
`enum` reads as `enum: [v]`: the narrowing rule (JUDGE-SUBTYPE r1), the
finite-domain rule J2 (JUDGE-R3), the dropped-enum rule D3 (JUDGE-SUBTYPE
r2) and the counterexample builder, which already does. `const` joins
the keywords JUDGE-REACH r0 counts as read at a reached node, so a
changed `const` is judged, not stated unchecked. Nothing else changes:
a node with both `const` and `enum` keeps its enum (CANON r1 already
intersects them in normal form; the walk reads the enum as written).

## Fixtures (frozen before implementation)

- k_f01 `{"const":"x"}` to `{"enum":["x"]}`: HOLDS, no witness, the
  CANON r1 literal note, nothing unchecked. (Today: a false VIOLATED.)
- k_f02 `{"const":"x"}` to `{"enum":["x","y"]}`: HOLDS, nothing
  unchecked (a widening).
- k_f03 `{"const":"x"}` to `{"enum":["y"]}`: VIOLATED, narrowed, with
  counterexample `{"k":"x"}`.
- k_f04 `{"enum":["x","y"]}` to `{"const":"x"}`: VIOLATED, narrowed, with
  counterexample `{"k":"y"}`. (Today: NOT EVALUATED.)
- k_f05 `{"const":"x"}` to `{"const":"y"}`: VIOLATED, narrowed, with
  counterexample `{"k":"x"}`. (Today: NOT EVALUATED.)
- k_f06 guard: `{"const":"x"}` to `{"const":"x","description":"d"}`:
  HOLDS, nothing unchecked.

Amended fixture, with the reason: JUDGE-REACH r0 r_f13 used `const` as
its example of a keyword the judge does not read at a reached node.
`const` is now read, so r_f13 uses `multipleOf`, which the judge still
does not read; what it tests (an unread keyword's change is stated,
never covered) is unchanged, and mutant RE-M5 now adds `multipleOf` to
the read list instead of `const`.

## Mutation operators

- K-R1 `const` not read as an enum by the narrowing rule   -> k_f01, k_f05
- K-R2 `const` not counted as read at a reached node        -> k_f04

## Pre-registered prediction (all three corpora at v0.2.20)

Exactly one change moves: AsyncAPI 3.0.0 to 3.1.0 (tag neutral), from NOT
EVALUATED to VIOLATED, with a narrowed witness at `asyncapi`
(`const` "3.0.0" to "3.1.0") and counterexample `{"asyncapi":"3.0.0"}`.
The corpus builder neutralises the version tag when it is an `enum`
and missed `const`; that is a harness gap to fix separately, and the
verdict on the pair as given is right. No other verdict or witness moves;
label-free 26/14/50/0, Iglu 5/3/0, replay PASS unchanged.
