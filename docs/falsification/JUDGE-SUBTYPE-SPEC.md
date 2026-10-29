# JUDGE-SUBTYPE r1: shrinking the data judge's unjudged list (r1 DRAFT, NOT FROZEN)

Target release: v0.2.1, during the B2 window; judge is a shipped
surface, so deepening its judged set is active-track work, not gated
capability. Honest framing, fixed: this shrinks the STATED unjudged
list of `invar judge` (data contracts). It has no relationship to
`invar check`'s 38 of 70 code-contract recall and must never be
presented as moving that number.

## New judged classes (each a containment check, deterministic)

- Enum narrowing: a variant present in pre and absent in post is
  VIOLATED, kind `narrowed`, witness lists the removed variants.
  Additions are HOLDS.
- Bounds tightening: `minimum`/`minLength`/`minItems` raised, or
  `maximum`/`maxLength`/`maxItems` lowered, is VIOLATED, kind
  `tightened`, witness carries both values. Loosening is HOLDS.
- Required escalation: a property newly added to `required[]` is
  VIOLATED, kind `escalated`. (Direction question below.)
- Pattern and format: regex containment is undecidable in general, so
  r1 does NOT claim tighten/loosen analysis. Any change to `pattern`
  or `format` on a bound property is reported as kind
  `constraint_changed` with both values, never silently passed and
  never overclaimed as a subtype proof.

The unjudged list shrinks accordingly; what remains in it after r1
(composition via oneOf/anyOf/allOf, tuple `items`) stays printed in
every output, same doctrine.

## Decision points (JS-D1 to JS-D4)

- JS-D1 required escalation direction: a schema can describe what a
  producer emits or what it accepts; escalation breaks senders in one
  reading and receivers in the other. r1 draft: VIOLATED either way,
  with the witness naming the ambiguity, or a --role flag? To rule.
- JS-D2 pattern/format: confirm `constraint_changed` as its own kind
  (neither removed nor retyped), exit 1.
- JS-D3 fixture sources: synthetic minimal cases per class, plus real
  family bytes where the corpus provides them (iglu schemas carry
  minimum/maximum in the already-pinned b1-002 family).
- JS-D4 mutation operators: one per validator (JS-M1 enum check
  dropped, JS-M2 bounds comparison inverted, JS-M3 required diff
  dropped, JS-M4 pattern change treated as HOLDS), each with a named
  killer fixture, appended to the replayable mutation script.

## Constraints carried unchanged

Deterministic, no LLM in any verdict, fixed exit codes, voice rules
frozen in JUDGE-SPEC r1 apply to every new witness sentence, latency
measured per release and never promised.
