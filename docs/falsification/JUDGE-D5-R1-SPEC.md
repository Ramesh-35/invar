# JUDGE-D5 r1: a type written where none was is a narrowing, not a note (r1, FROZEN)

Ratified by the founder 2026-11-08 ("D5"). Frozen with
crates/invar-falsify/tests/judge_d5_r1_test.rs, two real SchemaStore
pairs, and the WG gate BEFORE implementation; freeze record:
JUDGE-D5-R1-SPEC.frozen. Withdraws D5 of JUDGE-SUBTYPE r2 Amendment B;
every other ruling stays frozen as written.

## Why: a false HOLDS, found by someone else's ground truth

The artifact of Baazizi, Ben Ali, Colazzo, Ghelli, Klessinger, Sartiani
and Scherzinger ("JSON Schema Inclusion through Refutational
Normalization", 2026) ships a SchemaStore dataset labelled by the complete
witness-generation algorithm of Attouche et al. (ACM TODS 2026): 76 pairs
of consecutive schema versions where a document valid at the older
version is rejected by the newer one. Each is a break whatever INVAR
says. INVAR v0.2.22 on the 76: VIOLATED 30, NOT EVALUATED 45, HOLDS 1.

The HOLDS is schema-catalog v12 to v13: `schemas[].versions` had
`additionalProperties` and no `type`, and gained `"type": "object"`.
D5 (JUDGE-SUBTYPE r2 Amendment B) called that "type made explicit, which
its own keywords already implied: a stated note, not a break". The note's
own wording said "values of other types, accepted before, are now
rejected". That sentence is the definition of VIOLATED. A node with
object keywords and no `type` accepts a string, a number, null, an
array: the object keywords simply do not apply to them. A validator
confirms it: a catalog whose `versions` is the string `"oops"` is valid
at v12 and rejected at v13. The same note stands on 17 HOLDS in INVAR's
own corpora (13 SchemaStore, 4 Iglu); devinit 2.0 (`run` gained
`"type": "array"`) is confirmed the same way. No corpus of ours could
see this, because none of them carries ground truth. D5 was ratified
with its contradiction in plain text, and shipped.

## Ruling D5 (r1)

The D5 branch is withdrawn. A field that had no `type` at the base and
has one at the head is judged by the existing type rule like any other
retype: `unspecified` to `object` (or `array`, `string`, `number`) is a
narrowing, VIOLATED, kind `retyped`, with a counterexample carrying a
value of another type that the counterexample builder checks against the
base as it always does. Nothing is added for it; the special case is
deleted.

Unchanged, and the guards below freeze them: D9 (an `enum` or `const`
at the base already limited the field to values of the new type: a
note, HOLDS); D7 (a type widened: a note, HOLDS); a type that
contradicts the implied one (r2_f16: VIOLATED, as before).

## The WG gate (permanent, pre-registered)

`docs/falsification/wg-bench/` carries the 76 pairs (SchemaStore files,
Apache-2.0, with MANIFEST.tsv naming each source file of the dataset) and
`replay.py`. The gate passes when the judge says HOLDS on none of the 76.
VIOLATED and NOT EVALUATED are both honest answers there; HOLDS never
is. It joins label-free, Iglu and the SchemaStore replay as a release
gate, and it is the first of INVAR's gates whose labels INVAR did not
produce.

## Fixtures (frozen before implementation)

- d5_f01 `{"properties":{...}}` gains `"type":"object"`: VIOLATED,
  `retyped`, pre type `unspecified`, post type `object`, counterexample
  `{"h": null}` (a value of another type; null is what the builder picks
  first), no note "type made explicit" anywhere.
- d5_f02 `{"items":{...}}` gains `"type":"array"`: VIOLATED, `retyped`.
- d5_f03 guard (D9): `{"enum":[{"a":1}]}` gains `"type":"object"`:
  HOLDS with the D9 note.
- d5_f04 guard (D7): `"type":"string"` becomes `"type":["string","null"]`:
  HOLDS with the widening note.
- d5_f05 real bytes, schema-catalog v12 to v13: VIOLATED, witness at
  `schemas[].versions`, counterexample present and confirmed by the
  harness in the Measured outcome.
- d5_f06 real bytes, devinit 2.0 cc3c8b16c: VIOLATED at `run`.
- The WG gate: 0 HOLDS on 76 (today: 1).

Amended fixture, with the reason: r2_f15 asserted the withdrawn note. It
now asserts VIOLATED `retyped` with a counterexample, and mutant R2-M6
(the implied-type check reintroduced) is killed by it in that form.

## Mutation operators

- D5R1-M1 the implied-type note reintroduced (R2-M6, re-aimed)  -> r2_f15, d5_f01
- D5R1-M2 unspecified-to-object not reported as retyped        -> d5_f01

## Pre-registered prediction

SchemaStore: the 13 HOLDS carrying the note move to VIOLATED, except
where the walk reached the node through keywords a validator ignores
(stale.json: a root `$ref` with sibling `properties`, draft-07; the
judge's counterexample will be accepted at the head by both validators,
and that pair is the first exhibit of the next ruling, REF-SIBLINGS).
The 7 NOT EVALUATED carrying the note stay NOT EVALUATED with a
VIOLATED witness added only where the walk reaches the node. Iglu: the 4
HOLDS carrying the note move to VIOLATED; the 2 VIOLATED stay. AsyncAPI:
no move. Label-free counts change (more VIOLATED, stated in the Measured
outcome), Iglu 5/3/0 may rise, SchemaStore replay PASS unchanged, the WG
gate passes with 0 HOLDS. Every new counterexample is run through the
two validators; the count that fails that check is reported, not hidden.
