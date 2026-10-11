# W2: map-key abstraction (spec, frozen 2026-10-11)

Fixes the disqualifying finding of the effectiveness gate (OBSERVED-REALITY
gate record, 2026-10-11): map-keyed payloads explode path witnesses. Two
mechanisms, both deterministic, both frozen here before re-measurement.

## Mechanism 1: map detection and wildcard paths

Keys of a MAP are data; keys of a RECORD are schema. During shape
derivation, an object node is treated as a map when either:

- it has 16 or more direct keys (cardinality rule), or
- it has 2 or more direct keys and more than half are data-like, where a
  key is data-like if it is longer than 40 characters, or purely numeric,
  or contains any character outside [A-Za-z0-9_-] (so `@std/assert@0.213.1`
  and UUIDs qualify; `lockfileVersion` and `follow-redirects` do not).

A map's children all contribute under the single segment `*`:
`packages.jsr.@std/assert@0.213.1.integrity` derives as
`packages.jsr.*.integrity`. Value shapes under maps are preserved and
merged; only key identity is abstracted. Both rules evaluate per payload
from content alone; no configuration, no learning.

The stated trade, plainly: a RECORD with 16 or more fixed fields is
indistinguishable from a map by this rule and collapses too; its fields
are then witnessed collectively, not individually. That costs granularity,
never honesty (a collapsed removal still shows at the `*` path), and the
static contract side still evaluates such fields by name.

## Mechanism 2: ancestor dominance in drift reporting

When a path is absent from the fresh payload, every path beneath it is
trivially absent too. Drift reporting keeps only the SHALLOWEST missing
path: `dependencies` gone is one finding, not one per transitive package.
Applies to residue and stated lines alike; a retype never suppresses its
children (they may carry independent information).

## Compatibility

Lock format 3 is unchanged (wildcard paths are ordinary path strings).
Witness locks written by alpha.18 used literal keys; they still parse, but
fresh derivations now wildcard, so pre-W2 sample witnesses should be
re-locked (`invar lock --sample` again). Stated in INVAR-LOCK.md; alpha.18
was tagged under 24 hours ago with zero external users.

## Gate re-run (the ratified metric)

Re-run the effectiveness gate's r1-059 and r1-075 cases on the same pinned
payloads. PASS requires: both still VIOLATED, and total reported findings
(residue plus stated) reduced from 1,232 and 23,240 to at most 20 per
case. The suricata case is re-run for regression (expect unchanged:
stated, not silent). Results recorded in the gate record either way.

## Non-goals

No per-payload configuration, no key-pattern learning, no entry counts or
cardinality statistics in the lock, no change to static detection rules
(the expansion freeze stands).
