# E1: fields moved into a new envelope (spec frozen before batch 7 is sourced)

Written 2026-10-09 from batches 1 to 6 only (all tuning data). Batch 7 is
sourced after this file is hashed. Rules added later are post-hoc and do not
count toward the batch 7 score.

## Problem

A producer wraps fields in a new object: `{"message": m}` becomes
`{"type": "log_message", "log_message": {"message": m}}`, or a flat result
becomes `{extended_address: {"url": u, ...}}`. Every key still exists, so
INVAR reports nothing. Seen in r1-068, r1-069 (batch 6); r1-053 is out of
scope (the envelope comes from a framework setting, not a literal).

## Rules (change-scoped mode, producer code in Python, JS/TS or Go)

1. **Key paths.** For payloads built as dict/object/map literals, each key is
   recorded with the path of literal keys that enclose it (`log_message.message`).
   A container opened under a non-literal key (a variable, in Python and Go;
   a computed `[key]` in JS/TS) contributes `*` to the path.
2. **Relocated.** A key whose head paths share none of its base paths, and
   that sits deeper at head than at base, has been relocated from the base
   path to the head path.
3. **Consumer adapted?** If the consumer mentions any new literal path segment
   (a segment in the head path that is not in the base path), it is treated
   as already reading the new envelope: no finding. Otherwise, a relocated key
   the consumer reads is residue (VIOLATED), witnessed as
   `moved from <base path> to <head path>`. A new segment `*` counts as
   unmentioned.

## Measure

- Primary: detection (CAUGHT) and silent misses on held-out batch 7, released
  engine (alpha.9) versus the E1 engine.
- Guard: controls must not turn VIOLATED; report any new extra residue.

## Not addressed

Envelopes added by framework configuration (pagination classes), default
format switches, keys renamed to runtime values (r1-071), spec-only nesting.
