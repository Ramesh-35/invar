# RECALL-ATTACK-1: the misses, taken as a set

Status: RULED (founder, 2026-10-23): R-D1 ratified with a witness
requirement, R-D2 modified (conviction of absence, rename only as a
stated candidate), R-D3 specification only, R-D4 order G3 then G1 then G2
then S4 with phase isolation, R-D5 founder-sourced batch 9 with audit
before sealing. Phase 1 (G3) landed 2026-10-25 (docs/evidence/g3). Phase 2 (request side) landed 2026-10-25 (docs/evidence/request-side). Draft
r1 text follows unchanged. Parent: the founder's mandate of 2026-10-23
(attack the missed held-out breaks; preserve precision; pre-registered
gates). Engine measured: the aed8988 release binary (bca55be8506e) on the
sealed held-out set (docs/falsification/HELD-OUT-MANIFEST.txt, batches 4 to
8, 70 real breaks and 15 controls), full corpus re-run 2026-10-23, raw rows
in docs/evidence/recall/recall-2026-10-23.tsv.

## A. The baseline, re-measured today

| Outcome | Sealed record (frozen engines, 38/70) | Current engine |
|---|---|---|
| CAUGHT | 38 | 42 |
| PARTIAL | counted in misses | 1 |
| Stated NOT EVALUATED (exit 4, visible) | part of the 32 | 12 |
| Silent miss (HOLDS on a real break) | part of the 32 | 15 |

Two rungs, never conflated (docs/falsification/METHODOLOGY.md): the sealed
record stays 38 of 70, scored on engines frozen before each batch. Today's
42 of 70 is a tuning-rung figure: the corpus has been visible during
development since it was sealed, so no claim of improved recall may be made
from it. The only way this arc can move the published number is a fresh,
sealed batch 9, scored on an engine frozen before its cases are sourced.

The mandate's "32 missed" is the sealed-record complement. Today's
attackable set is 28 cases: 16 where the engine says HOLDS or catches only
part of the residue (the dangerous ones: a false green), and 12 where it
already refuses honestly (exit 4, stated) but does not convict.

Controls today: 17 CLEAN, 2 stated NOT EVALUATED, 1 SKIPPED, and the one
pre-existing FALSE_POSITIVE (r1-065, identical in every engine since
alpha.10). Nothing in this arc may add a false positive.

## B. Taxonomy of the 28, from the case sources and the engine's own output

### S1. Restructure: every field name survives, its position does not (6)

The producer moved the same names under a new wrapper, key or shape, so
name-presence extraction still finds them and the check holds.

| Case | What happened |
|---|---|
| r1-044 | webpack-bundle-tracker moved file info from `chunks` to `assets`; names intact |
| r1-053 | paperless-ngx paginated /api/tasks; `status` now lives per-row under results[] |
| r1-068 (PARTIAL) | HA otbr/info became keyed by extended address; flat fields now nested |
| r1-107 | linkwarden `{response:[...]}` became `{data:{tags:[...]}}` |
| r1-110 | NetBox selection custom fields: scalar became `{value,label}`; names survive |
| r1-116 | evcc forecast rates: keyed objects became positional arrays; `start`/`end` gone from rates but present elsewhere |

Mechanism gap: the lock binds `Type.field`, but presence is credited when
the name exists anywhere in the bound source, not within the bound scope.
Candidate rule family G1 (scope-strict presence): a locked binding whose
field no longer exists UNDER ITS LOCKED SCOPE is at least stated (A1), and
residue when the consumer's read is through that scope. E1 (envelope) was
this family's first, too-narrow attempt; G1 subsumes it.

### S2. Rename where the engine stays silent instead of convicting (4)

| Case | What happened |
|---|---|
| r1-060 | NetBox writable `device_role` renamed to `role`; consumer sends the old name |
| r1-076 | Woodpecker `link_url` renamed `forge_url`; Go consumer reads `repo.Link` |
| r1-091 | Next.js middleware manifest `regexp` replaced by `matchers` |
| r1-106 | Dispatcharr `stream_name` replaced by `channel_name` |

Mechanism gap, two halves: (a) the consumer's read form is not one the
reads scanner recognizes (a Go field access, a form key, a template), so no
binding exists and the hold is vacuous over that field; (b) where a binding
exists, the old name often still appears somewhere in the producer file
(docs, migrations, other types), re-triggering the S1 gap. Candidate family
G2: reads-scanner coverage measured per case first; plus rename-in-scope
(locked field gone from its scope, exactly one new sibling added) convicts
with a rename witness instead of abstaining.

### S3. The value and default-switch family, F1's unfinished work (4 silent + 6 stated)

Silent: r1-113 (Cargo lockfile v4 default; `version` present, value set
grew), r1-095 (exitstatus became IntEnum), r1-096 (`enabled` 0/1 became
boolean against a `=== 1` comparison), r1-051 (endpoint began requiring a
JSON body; request side). Stated already (exit 4, F1 or A1 fired): r1-056,
r1-059, r1-075, r1-094, r1-099, r1-100.

Mechanism gap: F1's frozen value grammar is integers, booleans and quoted
integers; r1-113's selector value is an enum variant (`ResolveVersion::V4`)
and r1-095's a typed enum. STATUS already names the widening as the next
frozen spec. Candidate family G3: widen the selector-value grammar to
identifiers and enum variants; conviction stays what F1 ruled (NOT FULLY
EVALUATED, never a guessed VIOLATED).

### S4. Names the engine cannot statically see (2 silent + 6 stated)

Silent: r1-079 (Prometheus metric names queried as strings by opencost),
r1-118 (a multipart form field sent by the consumer, removed by Gotenberg).
Stated: r1-037 (field moved between request types, A1 already states it),
r1-038, r1-050, r1-071 (response key renamed per repo name), r1-092,
r1-105.

Mechanism gap: the dependency is carried in dynamic strings (metric names,
form fields, map keys) or request payloads, outside the read grammar.
Honest treatment per case: either a narrow, evidence-first extension
(form-field and metric-name literals are still string literals in the
consumer), or an explicit abstention so the false green at least becomes a
stated refusal. A silent HOLDS converted to a stated NOT EVALUATED is a
win for honesty even where conviction is impossible.

## C. Pre-registered pass gates (tuning rung)

- G-T1: of the 16 silent cases, at least 8 become CAUGHT, and none remains
  a silent HOLDS: every unconverted one must at least state its blind spot
  (exit 4). A false green surviving the arc fails the arc.
- G-T2: of the 12 stated cases, at least 6 become CAUGHT.
- G-T3: zero regressions across all 123 corpus cases: no CAUGHT case
  degrades, no control flips to FALSE_POSITIVE, r1-065 stays the only
  false positive, and EVAL-007/008 round-trips stay byte-identical where
  no rule applies.
- G-T4: full suite green, 0 warnings; 100 percent of non-equivalent
  mutants caught for every new rule, one mutant battery per rule family.
- G-T5: each rule family lands under its own frozen spec
  (docs/falsification/G1..G3-SPEC.md pattern), spec hash recorded before
  implementation, exactly as A1/D1/E1/F1 were run.

## D. The held-out gate (the only one that moves the published number)

- G-H1: after the rule families land and freeze, a batch 9 of 15 fresh
  real breaks plus 3 controls is sourced under the existing admission
  rules, sealed, and scored on the frozen engine. The published recall
  record updates to the combined batches 4 to 9 score, whatever it is.
- G-H2: the register's recall row cites both rungs explicitly; the
  measured line printed by `invar check` changes only on G-H1's result.

## E. Decision points for ruling

- R-D1 (scope of conviction, S1): when a locked field leaves its locked
  scope but survives elsewhere, is the verdict VIOLATED with a moved-field
  witness when the consumer's read is scope-qualified, and stated NOT
  EVALUATED otherwise (recommended), or always stated NOT EVALUATED?
- R-D2 (rename conviction, S2): may the engine convict a rename (locked
  field gone from scope, exactly one added sibling) as VIOLATED with a
  rename witness (recommended: the consumer's read is dead either way), or
  does rename stay an A1 statement?
- R-D3 (request-side reads, S2/S4): extend the reads grammar to consumer
  REQUEST payloads (form fields, JSON bodies the consumer sends:
  r1-051, r1-060, r1-118, r1-037)? Recommended yes, literal keys only,
  since a sent field the producer no longer accepts is the same break in
  the other direction.
- R-D4 (order): G3 first (grammar widening over an already-ruled rule,
  smallest risk), then G1, then G2, then the S4 residue honesty pass
  (recommended), or by bucket size (G1 first)?
- R-D5 (batch 9 sourcing): sourced by me under the admission rules with
  every candidate's provenance recorded for your audit before sealing
  (recommended), or sourced by you?

## F. Bounds

- F-B1: No conviction without a witness; abstentions never become HOLDS;
  every converted case cites the exact line evidence in its output.
- F-B2: The PRESENT/ABSENT/UNRESOLVED interface and exit codes are
  untouched; init stays heuristic and says so.
- F-B3: Nothing lands without its frozen spec, fixtures before code,
  mutants at 100 percent, and the full-corpus regression sweep (G-T3).
- F-B4: Tuning-rung numbers are never published as recall; only G-H1 moves
  the record (register rule of use).
