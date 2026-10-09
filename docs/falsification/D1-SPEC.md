# D1: detection rules (spec frozen before batch 6 is sourced)

Written 2026-10-09 from batches 1 to 5 only (all now tuning data). Batch 6 is
sourced after this file is hashed. Rules added later are post-hoc and do not
count toward the batch 6 score.

## Rules

1. **TypeScript `Omit` in a new type.** In a producer TypeScript file, a type
   alias `type T = Omit<..., "a" | "b">` (also inside `Modify<...>` or across
   lines) lists fields T does not have. If T exists at head but not at base,
   each omitted field that existed in the producer at base is treated as
   removed in T. A consumer read of it is residue (VIOLATED), witnessed as
   `not in T (Omit)`. Change-scoped mode only.
2. **Python unannotated class fields.** In a Python class body, `name =
   Callee(...)` whose callee's last dotted segment ends in `Field`,
   `Serializer`, `Column` or `ForeignKey` declares field `name` of that
   callee's type (Django models, Django REST Framework serializers,
   marshmallow, SQLAlchemy). ALL_CAPS names are constants, not fields.

## Measure

- Primary: detection (CAUGHT) and silent misses on held-out batch 6, frozen
  alpha.8 engine versus the D1 engine (A1 stays on in both).
- Guard: controls must not turn VIOLATED.
- Tuning effect on batches 1 to 5 is reported but does not count.

## Not addressed (no general rule found in batches 1 to 5)

A field wrapped into a new envelope (r1-053), a request body format change
(r1-051), a default output format switch (r1-059), moved fields that keep
their name in untyped JavaScript (r1-044).
