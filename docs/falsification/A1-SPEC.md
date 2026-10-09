# A1: stated blind spots (spec frozen before batch 5 is read)

Written 2026-10-09 from batches 1 to 4 only. Batch 5 cases are not read until
this file is hashed and recorded. Any rule added after that is disclosed as
post-hoc and does not count toward the batch 5 score.

## Problem

A real break INVAR does not see is reported as HOLDS. On batch 4 that was
4 of 10. A silent HOLDS is worse than "not evaluated": it invites trust.

## Rules (change-scoped and lock modes only; strict mode unchanged)

1. **Mentioned, not read.** For every changed field (removed, retyped, or
   moved, below), if the consumer text contains the field's name (naming form,
   as a whole identifier or a quoted string) at a site that is not one of
   INVAR's recognised reads, the contract records an UNEVALUATED site:
   `field mentioned at file:line, not evaluated (pattern not read)`.
   This covers destructuring, keys the consumer writes, and any idiom INVAR
   does not parse, without a rule per idiom.
2. **Moved.** A field present in a type at base that is absent from that type
   at head but present in another type at head is "moved" (today it counts as
   unchanged). A moved field the consumer reads is UNEVALUATED:
   `field moved from A to B; consumer read at file:line not evaluated`.
3. **Nullability is part of a type.** Spec types include nullability
   (`nullable: true`, `"null"` in a type list); a change in nullability is a
   type change, reported as residue like any other.
4. **Outcome.** Residue still means VIOLATED (exit 1). A contract with no
   residue but at least one UNEVALUATED site is NOT FULLY EVALUATED (exit 4,
   as for SKIPPED), never HOLDS. The report lists each site.

## Measure

- Primary: silent miss rate on held-out batch 5 (real breaks reported HOLDS),
  frozen alpha.5 engine versus the A1 engine.
- Guard: controls must stay CLEAN or become NOT FULLY EVALUATED; a control
  turned VIOLATED is a false positive and counts against A1.
- Cost: share of all contracts reported NOT FULLY EVALUATED. If A1 turns most
  contracts into "not evaluated", it has traded silence for noise; report it.

## Non-goals

No new parser, no new dependency, no per-language idiom rules in A1.
