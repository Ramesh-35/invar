# G1: shape changes at the emission site (RECALL-ATTACK-1 phase 2)

Status: DRAFT r0, evidence and decision points only, for founder ruling.
No fixtures, no implementation. Baseline: 201d193 (G3 landed; native
449 of 449). Corpus outcomes: docs/evidence/g3/corpus-before-after.tsv
(today's engine). Producer texts: all 273 base and head files fetched
2026-10-25.

## A. The measurement that changes the plan

RECALL-ATTACK-1 r1 grouped six held-out misses as one mechanism, S1
"restructure: every field name survives, its position does not", and the
founder ruled G1 the strategic target on that basis. Reading each case's
real producer diff shows the grouping was by SYMPTOM. The causes are six
different mechanisms, one case each:

| Case | Today | What actually changed in the producer | Line-grammar reach |
|---|---|---|---|
| r1-044 | silent HOLDS | fields set on a `fileInfo` builder variable moved from `output.chunks[...]` to `output.assets[...]` | none: needs dataflow from a variable into a container |
| r1-053 | silent HOLDS | one added line: `pagination_class = StandardPagination`; the envelope is Django REST Framework behavior | only by framework knowledge |
| r1-068 | exit 1, 4 of 5 fields | fields nested under a dict keyed by `extended_address` | yes: E1 already convicts 4; `url` is missed (cause not yet isolated) |
| r1-107 | silent HOLDS | literal `.json({ response: ... })` became opaque `.json(data)` built from a rest spread | yes: the emission went from literal to opaque |
| r1-110 | silent HOLDS | a scalar reassigned to `{'value', 'label'}` inside one branch | weak: a value-shape change on one code path |
| r1-116 | exit 1, wrong field | `Grid api.Rates` became `[][]float64`; `start`/`end` are members of `api.Rate`, declared in another file | partial: the type change is visible, the lost members are not |

Of the six, four are silent today (r1-044, r1-053, r1-107, r1-110); two
already fail the check (exit 1), with partial or different fields.

## B. What recurs

Among the 12 held-out breaks that are silent today (exit 0):

| Mechanism | Silent cases | Also in the corpus |
|---|---|---|
| request side: the consumer SENDS a field the producer stopped accepting | r1-051, r1-060, r1-118 (3) | r1-037 stated (A1), r1-090 caught |
| rename | r1-076, r1-106 (2) | r1-091 exit 1, partial |
| the six S1 mechanisms above | 1 each (4 silent) | |
| other single cases | r1-079, r1-095, r1-096 | |

Request side is the only mechanism with three silent recurrences of the
SAME cause. It is the R-D3 scope (specification authorized, not yet
written).

## C. What G1 can soundly claim, measured

G1-E, opaque emission. At an emission call (`.json(`, `.send(`,
`jsonify(`, `JsonResponse(`, `json.dumps(`, `Response(`,
`send_result(`), the base passes an object literal whose keys include a
key the consumer reads, and the head passes an opaque expression (an
identifier, a spread result) at that site. The head file no longer states
the emitted keys, so the read can be neither confirmed nor convicted from
it.
- Outcome: stated NOT EVALUATED (exit 4), never VIOLATED, nothing added
  when residue exists (the F1 discipline).
- Measured on all 122 corpus cases with a throwaway counter (literal
  emissions fell AND opaque emissions rose, base to head): it fires on
  exactly one, r1-107, a held-out break; zero controls, zero tuning
  cases.
- Corpus value: one silent HOLDS becomes a stated refusal. No recall
  claim: tuning rung, and a single case.

G1-T, declared container type change. A field's declared type changes
from a named type to a non-member type (`api.Rates` to `[][]float64`).
Consumer reads of names that are not declared in the head file are
stated as not evaluated, because their declaring type is elsewhere.
- Corpus value: r1-116 already fails (exit 1, on `temperature`). This
  would add a correct witness for `start` and `end`, but not change the
  outcome, because a statement never overrides a conviction.

E1 `url` gap (r1-068): an E1 defect or limitation, to be isolated before
any rule is written. If it is a defect, the fix belongs to E1's own
spec, not G1.

Unsupported, stated: builder-variable dataflow (r1-044), framework
semantics (r1-053), branch-local value shape (r1-110). A line grammar
cannot see these without inventing structure. No rule is proposed for
them, and they stay silent misses in the record.

## D. Decision points

- G1-D1 (scope). (i) G1-E only, plus isolating r1-068's `url`
  (recommended: smallest, sound, measured precise on the whole corpus);
  (ii) G1-E and G1-T; (iii) defer G1 entirely.
- G1-D2 (order). R-D4 ruled G3, G1, G2, S4. The measurement says the
  largest recurring silent mechanism is request side (3 silent, 1 stated,
  1 caught). Recommended: specify request side (R-D3) next, under its own
  frozen spec, with G1-E as a small parallel phase or after it.
  Alternatively keep the ruled order.
- G1-D3 (emission-call list). The seven call forms in C are the measured
  set. Any addition is a new frozen entry, never adaptive.
- G1-D4 (claims). The RECALL-ATTACK-1 r1 statement "6 of 16 silent
  misses share S1" is withdrawn as a mechanism claim and recorded as a
  symptom grouping, in that spec and here.

## E. Bounds, unchanged from G3

No conviction without a witness; statements never override convictions;
exit codes and the PRESENT/ABSENT/UNRESOLVED interface untouched; frozen
spec and fixtures before code; mutation operators with declared killers;
full-corpus byte comparison; native verification; tuning-rung numbers are
never published as recall.
