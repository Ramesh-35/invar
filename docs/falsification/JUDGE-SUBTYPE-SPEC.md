# JUDGE-SUBTYPE r1: shrinking the data judge's unjudged list (r1, FROZEN)

Rulings JS-D1 to JS-D4 ratified 2026-10-30. Frozen with
crates/invar-falsify/tests/judge_subtype_test.rs BEFORE implementation;
freeze record: JUDGE-SUBTYPE-SPEC.frozen. Target release: v0.2.1.

Honest framing, fixed: this shrinks the STATED unjudged list of `invar
judge` (declared data contracts). It has no relationship to `invar
check`'s 38 of 70 code-contract recall and must never be presented as
moving that number.

## Reading

r1 reads every schema as a validation contract: the set of payloads it
accepts. Narrowing that set breaks senders whose payloads were valid
before, so narrowing is VIOLATED and widening HOLDS. The opposite
reading (a schema describing what a producer emits, where WIDENING can
break readers, for example a new enum value reaching an exhaustive
match) is not judged in r1, and that limit is stated in the spec.

## Judged classes (recursive through properties, same as r0)

| Kind | Rule | Ruling |
|---|---|---|
| narrowed | a value in pre's `enum` absent from post's; or post adds `enum` where pre had none | JS-D1 |
| tightened | `minimum`, `exclusiveMinimum`, `minLength`, `minItems` raised or newly added; `maximum`, `exclusiveMaximum`, `maxLength`, `maxItems` lowered or newly added | JS-D2 |
| escalated | a name newly present in an object's `required[]` (an existing optional field promoted, or a new field added as required) | JS-D4 |
| constraint_changed | `pattern` or `format` changed or newly added on a property | JS-D3 |

JS-D3 as implemented: containment between true regular expressions is
decidable, but JSON Schema patterns are ECMA-262 regexes whose
backreferences make them non-regular, so r1 does not claim to decide
whether a pattern change narrows. Every change is VIOLATED as
`constraint_changed`, never described as a narrowing, never silent.
Removing an enum, a bound, a pattern or a format loosens and HOLDS.

## Output

Exit codes unchanged: any witness of any kind is VIOLATED, exit 1. JSON
witnesses gain `keyword`, `pre_value`, `post_value` (null for removed and
retyped). Text sentences, frozen as voice:
- narrowed: `<path> no longer accepts <values>` (or `<path> now accepts only <values>`)
- tightened: `<path> <keyword> raised from <a> to <b>` / `lowered from` / `added at <b>`
- escalated: `<path> is now required`
- constraint_changed: `<path> <keyword> was <a>, is now <b>. Whether this narrows what is accepted is not decided here`
Each ends with a next step, as in JUDGE-1.

The unjudged list after r1, printed in every run and carried as
`unjudged_constraints`: `$ref`, `allOf`, `anyOf`, `oneOf`, `items`.
`$ref` is a gap r0 had and never stated: properties behind a reference
were not followed. r1 states it.

## Amendment this forces: JUDGE-SPEC r2

JUDGE-1's frozen JSON test asserts the r0 unjudged list. r1 judges four
of those classes, so that assertion is updated under a recorded JUDGE-SPEC
r2 amendment; the r1 freeze record is kept and the r2 hashes appended.

## Fixtures (judge_subtype_test.rs)

- js_f01 enum loses a value -> VIOLATED narrowed, exit 1 (kills JS-M1)
- js_f02 enum gains a value -> HOLDS, exit 0
- js_f03 minimum raised -> VIOLATED tightened (kills JS-M2)
- js_f04 maximum raised -> HOLDS (kills JS-M2 in the other direction)
- js_f05 optional field promoted to required -> VIOLATED escalated (kills JS-M3)
- js_f06 pattern changed -> VIOLATED constraint_changed, sentence says
  "not decided here" (kills JS-M4)
- js_f07 b1-001 real bytes -> includes currentTime and duration maximum
  tightened: the corpus finding r0 stated but could not judge
- js_f08 JSON surface -> new witness fields, and unjudged_constraints is
  exactly the r1 list, without enum

## Mutation operators

- JS-M1 enum narrowing check dropped -> js_f01
- JS-M2 bounds comparison inverted -> js_f03, js_f04
- JS-M3 required diff dropped -> js_f05
- JS-M4 pattern or format change treated as HOLDS -> js_f06
