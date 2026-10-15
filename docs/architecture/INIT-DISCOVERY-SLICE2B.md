# INIT-DISCOVERY-SLICE2B: string-coupling contracts (K3)

Status: FROZEN as revision 1, ratified by the founder's ruling of 2026-10-15
18:35 (D1 to D7, section L), which also granted the lock.rs, check.rs and
main.rs expansions (Class 3, additive, `kind = "strings"` only) and authorized
gates 0 and 1. Changes after this freeze require an explicit revision and
renewed review. Prepared to the 86427 and slice 2 standard: numbered
requirements with falsifiable acceptance, abstention contract, countermodels
and fixtures with declared outcomes, mutation set, and gates 0 to 5, all
frozen before any Rust is written. Parent: docs/architecture/EMITTED-SHAPE-DISCOVERY.md
(K3, section 2) and the frozen INIT-DISCOVERY-SLICE2.md (landed e27fd8a). No
evaluation or general-detection claim is made or authorized here.

## 0. Purpose and honest bound

Slice 2 covers the emitted-payload class. Slice 2b covers the other verified
real break from the Stage 2 record: a string field list indexing a store whose
schema renames or drops a field (strapi e3961fb2: producer `attributes` keys
include `lastUsed` renamed to `lastUsedAt`; consumer `SELECT_FIELDS` string
list names `lastUsed`; merge skew reintroduced the stale string, fixed four
days later). Two verified breaks in 32 repositories is the whole evidence
base; strapi is a DEVELOPMENT case (S2-D5) and is excluded from any Stage 3
corpus. K3 is the noisiest kind (a string list has no type, no import and
often no receiver), so every design choice below trades coverage for
precision, and abstention is the default outcome.

## A. Frozen scope and architecture

A1. Pattern extraction only. No AST, no new dependencies (R1). Reuses
valuepairs.rs masking, depth-1 key extraction and statement spans.

A2. Producer sources (the only ones): (a) depth-1 member keys of
`export const X = z.object({...})` (existing K2 extractor); (b) depth-1 keys
of an `attributes: { ... }` object literal inside a content-type or entity
schema literal, where each member value is an object literal containing a
`type` key. Anything else (Prisma `.prisma` files, TypeORM or Sequelize
decorators, SQL DDL) is OUT OF SCOPE for 2b and abstains with
UNSUPPORTED-ORM-SOURCE only when a file of that kind is the sole candidate;
otherwise it is silent. Rationale: those need new parsers; deferring them is a
scope decision for ruling, not a finding.

A3. Consumer sources: an array literal of string literals, bound to a const
or passed directly at a call or property position. Template literals with
interpolation, concatenation, spreads and computed elements make the list
dynamic (section C).

A4. Scope: same package, or an explicit import path (relative or workspace
package name, one barrel level, as slice 2) between the list's file and the
shape's file, or both files in the same directory. Anything else is not
paired.

A5. Query-position evidence is required. A list qualifies only if it is
(i) bound to a name matching the frozen set `{select, fields, columns,
populate, attributes}` as a case-insensitive word segment (`SELECT_FIELDS`,
`userColumns`), or (ii) passed as the argument or value at a frozen call or
key marker: `.select(`, `.populate(`, `fields:`, `select:`, `columns:`,
`populate:`, `attributes:`. (Ruling D2: `pick` and `projection` from the
draft are removed.) A list with neither is not a candidate
(no abstention noise; counted in the scan summary only).

A6. ISA invariance: one new manifest `kind` value, `strings`, with evidence
keys `list`, `shape`, `overlap`, `scope`. No lock grammar change, five exit
codes untouched. Changes to lock.rs and check.rs are a NAMED REQUESTED
EXPANSION (Class 3), identical in form to the slice 2 grant, not assumed.

## B. Numbered requirements

R-K3-1 Distinctive overlap. Let L be the list's string elements after
removing the frozen stoplist (the 20 words in codepairs.rs STOPLIST, name-key
normalised) and S the shape's depth-1 keys after the same removal. The
overlap O = L intersect S (name-key normalised). A pair is a candidate only
if ALL hold: (a) |O| >= 3; (b) |O| / |L| >= 0.5 (the list is mostly this
shape); (c) |O| / |S| is NOT used (a list may select a subset). The
threshold 3 is stricter than slice 1's 2 because string lists carry no type
or receiver; the strapi development probe overlapped on lastUsed plus three
others (|O| = 4). Counterexample: CM-K3-5.

R-K3-2 Unique receiver. O must be a subset of exactly one shape within scope
(A4). If two or more shapes each satisfy R-K3-1 with the same or overlapping
O, abstain AMBIG-RECEIVER. Counterexample: CM-K3-2.

R-K3-3 Query-position required (A5). Counterexample: CM-K3-4, CM-K3-9.

R-K3-4 Dynamic lists abstain. Any non-literal element makes the list
unrepresentable (section C); the engine never pairs a partial list.
Counterexample: CM-K3-3.

R-K3-5 Version families. A shape inside a version family (`v/<N>` path or
`...V<N>` names, slice 2 detector) is never paired. Counterexample: CM-K3-7.

R-K3-6 Category labels. Each proposal carries the slice 1 category
(production, test, fixture, example, doc) from the list's path; test and
fixture lists are labeled and never presented as production usage.
Counterexample: CM-K3-6.

R-K3-7 Check semantics ("no stale reference"). The lock binds O (evidence
only, as slice 1 and 2). At check, for each locked name n, using freshly
extracted L and S:
- n in L and n not in S: VIOLATED (exit 1). This covers both the producer
  rename and the merge-skew reintroduction of a stale string.
- n not in L (removed or renamed on the consumer): HOLDS for n.
- n in S and n not in L: HOLDS (a list may narrow).
- n in L and n in S: HOLDS.
If the list or the shape can no longer be extracted (became dynamic, file
gone): SKIPPED, exit 4, never a false HOLDS and never a guess. A consistent
rename of both sides therefore HOLDS; adopting it is `invar lock`.
Counterexample/fixture: K3-F3, K3-F4, K3-F5, K3-U2.

R-K3-8 Evidence and refusal. Every proposal prints E-lines (list file:line,
shape file:line, O) and a status line "PROPOSED (heuristic; runtime store
binding not verified)". It never claims the list actually reaches the store.

R-K3-9 Independence. K3 runs regardless of slice 1's H2 threshold and does not
suppress or alter slice 1 or slice 2 output on any repository where it
proposes nothing (byte-identical output on F01 and on all slice 2 fixtures).

R-K3-10 Budget. Scan cost stays within the slice 2 envelope (no new full-tree
pass beyond reusing the existing file walk; hoppscotch-class tree under 1s).

## C. Abstention contract (codes defined before code)

| Code | Fires when |
|---|---|
| AMBIG-RECEIVER | R-K3-2: more than one shape qualifies, or the list is passed to a generic helper whose parameter is not a query-position marker, or the shape is reachable only across unrelated packages |
| NON-DISTINCTIVE-KEY | a query-position list exists but |O| < 3 or |O|/|L| < 0.5 after the stoplist (names the shape and the count) |
| UNSUPPORTED-STRING-DYNAMIC | the list has concatenation, template interpolation, spread, computed or non-string elements, or is built by push, map, filter or Object.keys |
| VERSION-FAMILY | R-K3-5 (reuses the slice 2 code) |
| UNSUPPORTED-ORM-SOURCE | A2: the only candidate shape source is a `.prisma`, decorator or DDL declaration |

Additions to the slice 2 code set are UNSUPPORTED-ORM-SOURCE and NON-DISTINCTIVE-KEY
(reuse of the user-named codes AMBIG-RECEIVER and UNSUPPORTED-STRING-DYNAMIC is
within the 13003 pattern). All abstentions print as
`ABSTAIN <CODE> <path>:<line> <reason>`, sorted, deterministic.

## D. Countermodels (defined before fixtures and code)

CM-K3-1 Receiver collision: a list of fields for a different store sharing
names with a shape (`['email','phone','lastLogin','role']` against an
unrelated `Account` shape). Outcome per R-K3-1/2.
CM-K3-2 Two qualifying shapes (`User` and `Account` both own email, phone,
lastLogin). AMBIG-RECEIVER, zero proposals.
CM-K3-3 Dynamic strings: `['last' + 'Used', ...]`, `` [`${p}Used`] ``,
`[...BASE_FIELDS, 'lastUsed']`, `Object.keys(x)`. UNSUPPORTED-STRING-DYNAMIC.
CM-K3-4 ORM builder wrapper: `buildQuery(FIELDS)` and `repo.find(FIELDS)` where
the helper is generic. Not a query marker: AMBIG-RECEIVER if otherwise
qualifying, silent if not.
CM-K3-5 Non-distinctive collisions: a list that is entirely or mostly stoplisted
(`['id','name','status','created_at','lastUsed']`) overlapping a shape in
`lastUsed` alone, or overlap of exactly 2. NON-DISTINCTIVE-KEY.
CM-K3-6 Test and fixture lists: same pair under `__tests__`/`fixtures`: proposed
only with category test/fixture, never production.
CM-K3-7 Version family: a shape in `v1/` and `v2/` with the list beside one of
them. VERSION-FAMILY, zero alarms (hoppscotch-class control).
CM-K3-8 Cross-package, no import: shape in package A, same-named list in package
B with no import path. Not paired (AMBIG-RECEIVER).
CM-K3-9 Display column lists: UI table `columns: ['lastUsed','owner','role','plan']`
matching a shape. Marker `columns:` makes it a candidate by A5; this is the
accepted residual noise class. It is labeled in the proposal, and the mutation
set and Stage 3 protocol must measure it. It is NOT claimed solved.
CM-K3-10 Partial-subset benign list: a list naming 3 of a shape's 8 fields.
HOLDS after a rename of one of the other 5 fields (additive/narrow tolerance).

## E. Fixture families (declared outcomes, before execution)

F (positive): K3-F1 strapi-shape synthetic (attributes literal with 6 keys and a
same-package SELECT_FIELDS naming 4 distinct plus 2 stoplisted): one `strings`
proposal, O of size 4, category production. K3-F2 zod-shape variant of the same.
K3-F3 init then rename in shape: `check` exit 1 naming the stale name.
K3-F4 merge-skew variant (shape renamed, list re-adds old string): exit 1.
K3-F5 consistent rename of both sides: exit 0.
N (negative controls, must produce zero proposals and zero alarms): K3-N1 CM-K3-1,
K3-N2 CM-K3-5, K3-N3 CM-K3-4, K3-N4 CM-K3-7, K3-N5 CM-K3-8, K3-N6 stoplist-only list.
U (abstention exactness, each asserts the exact code and line): K3-U1 CM-K3-2,
K3-U2 list turned dynamic after lock (check exit 4, never HOLDS), K3-U3 CM-K3-3,
K3-U4 orm-source-only, K3-U5 CM-K3-9 (proposal present, column-marker label printed).
M (shared): K3-M1 slice 1 and slice 2 fixtures byte-identical output; K3-M2 F01
output and lock identical; K3-M3 repeated runs byte-identical (determinism).

## F. Mutation set (harness reports CAUGHT / SURVIVED / BUILD-FAILED)

M1 overlap threshold 3 lowered to 2. M2 coverage ratio check removed. M3 stoplist
not applied to L. M4 unique-receiver check removed. M5 query-position check
removed. M6 dynamic element treated as skipped instead of abstaining the list.
M7 version-family check removed. M8 check ignores n in L and not in S (never
alarms). M9 check alarms on n not in L (over-alarms on benign removal). M10
unextractable returns HOLDS. M11 lock binds the whole shape instead of O.
M12 test and fixture category dropped. Each must be killed by a named assertion
failure; BUILD-FAILED and APPLY-FAILED are not kills.

## G. Gates 0 to 5

Gate 0 hash ledger of Class 1 files plus the new landed slice 2 files, before any
edit. Gate 1 implementation to the allowed paths only. Gate 2 all fixtures F/N/U/M
pass. Gate 3 mutation harness all CAUGHT with control green. Gate 4 full suite
green (target: 125 plus the new tests), zero warnings, F01 and slice 2 outputs
byte-identical. Gate 5 documentation: Section K-style evidence ledger appended to
this file, STATUS.md entry, EXPOSURE-LOG probe records. Failure at any gate stops
the arc for ruling.

## H. Path classes

Class 1 (untouched, hash-verified): surface.rs, backtest.rs, invariant.rs, config.rs,
yaml.rs, codepairs.rs, both Cargo.toml files, Cargo.lock, the five Experiment C
artifacts. Class 2 (verification artifacts): tests/init_slice2b_test.rs,
scripts/slice2b/. Class 3 (REQUESTED expansions, not assumed): lock.rs, check.rs,
main.rs (the egress `fix --suggest` skip extended to `strings`). Class 4 (proposed
implementation): src/stringpairs.rs (new, self-contained), init.rs, lib.rs. valuepairs.rs
is read-only reused through its public functions; any needed visibility change is a
named Class 3 item.

## I. Corpus and evaluation boundaries

Strapi (e3961fb2~1 and fix e54e5f7d) and formbricks are DEVELOPMENT cases, excluded
from any Stage 3 corpus. K3 calibration uses synthetic fixtures only. Real-repository
runs during implementation are read-only development probes recorded in EXPOSURE-LOG.md,
never evaluation results. A Stage 3 protocol in the Stage 2 mold is a separate future
ruling; the 2b evidence base supports no detection or noise-rate claim.

## J. Decision points for ruling

D1 Overlap threshold |O| >= 3 and coverage >= 0.5 (proposed; alternatives 2 and 0.6).
D2 Query-position marker and name sets in A5 (frozen as listed or amended).
D3 Kind name `strings` and evidence keys.
D4 Producer sources limited to zod and `attributes` literals; ORM sources deferred
(UNSUPPORTED-ORM-SOURCE).
D5 Check semantics R-K3-7 ("no stale reference", additions and narrowing hold).
D6 Class 3 expansions for lock.rs, check.rs, main.rs.
D7 Acceptance of CM-K3-9 residual noise as measured-not-solved.

## K. Stop conditions

Stop and report on: a countermodel that is unrepresentable by pattern extraction;
any slice 1 or slice 2 output change; a Class 1 hash change; a mutation kill that is
a build or load failure; the unique-receiver rule failing on a development probe in a
way fixtures cannot reproduce; any need to widen scope past A2 or A4; any need for an
AST or new dependency.

## L. Rulings of 2026-10-15 18:35 (D1 to D7), recorded verbatim in effect

- D1 RATIFIED: |O| >= 3 and |O|/|L| >= 0.5.
- D2 RATIFIED: marker set {select, fields, columns, populate, attributes}. A list
  outside a query position is not touched. Interpretation recorded for this
  freeze: a non-query list that would otherwise satisfy R-K3-1 and R-K3-2 emits
  AMBIG-RECEIVER, one that fails R-K3-1 only by the distinctive count emits
  NON-DISTINCTIVE-KEY when it is in a query position, and a non-query list that
  qualifies on nothing is silent (a repository holds thousands of string arrays;
  an abstention per array would be the noise this slice exists to avoid).
- D3 RATIFIED: `kind = "strings"`.
- D4 RATIFIED: zod and `attributes` literals only; no ORM parsers.
- D5 RATIFIED: no-stale-reference semantics, VIOLATED exit 1.
- D6 GRANTED: additive lock.rs, check.rs, main.rs handling of `kind = "strings"`
  only; Class 1 paths stay byte-locked.
- D7 RATIFIED: display column lists are residual noise, measured and labeled,
  never papered over with heuristics.
- Acceptance bound carried from the freeze message: K3-N* controls and the
  hoppscotch-class control (K3-N4) must produce zero alarms.

## M. Implementation Evidence Ledger (appended at Gate 5; sections A to L above are unchanged)

Status: IMPLEMENTED against the frozen revision 1. Detection capability on real repositories remains NOT ESTABLISHED. Nothing here is an evaluation result, and no real-repository probe was run for slice 2b: K3 was calibrated on synthetic fixtures only (strapi stays a development case, S2-D5).

### M1. Gate results

| Gate | Result |
|---|---|
| 0 baseline hash ledger | PASS (Class 1 files, Cargo files and the five Experiment C artifacts byte-identical to GATE0-LEDGER.md) |
| 1 implementation | landed: valuepairs/strings.rs (new child module), valuepairs.rs (one `pub mod strings;` line), init.rs, lock.rs, check.rs, main.rs |
| 2 fixtures | PASS: 18 of 18 in init_slice2b_test.rs (F1 to F7, N1 to N6, U1 to U5) |
| 3 mutations | PASS: 12 of 12 CAUGHT by assertion failure, control green, none SURVIVED, BUILD-FAILED or APPLY-FAILED |
| 4 regression | PASS after ruling: 149 of 149 (16 lib, 50, 40, 25 slice 2, 18 slice 2b), zero warnings; slice 2 harness 14 of 14 still CAUGHT |
| 5 documentation | this section, STATUS.md entry, EXPOSURE-LOG.md record |

### M2. Mutation table

01-threshold-lowered, 02-coverage-ratio-removed, 03-stoplist-ignored, 04-receiver-uniqueness-removed, 05-query-position-removed, 06-dynamic-silent, 07-version-family-removed, 08-check-never-alarms, 09-check-alarms-on-removal, 10-dynamic-list-holds, 11-lock-binds-whole-shape, 12-test-category-dropped. Harness: scripts/slice2b/mutate.sh, patches in scripts/slice2b/mutations/. In the first run mutation 03 SURVIVED because fixture N6 could not distinguish it; N6 was strengthened (shape now owns the generic names) and the whole harness re-run.

### M3. Findings and rulings

- F2B-1 Gate 4 conflict: the frozen slice 2 fixture cm6 expected zero proposals on a repository that is an exact K3 positive. RULED 2026-10-15: amendment granted. cm6 is now `cm6_string_field_list_is_a_k3_positive_since_slice_2b` and asserts one `strings` block (shape OrderIn, list const:SELECT_FIELDS, overlap currency_code total_minor unread_flag); slice 2's block parser accepts kind `strings`. K3 was not narrowed to exclude `.js` consumers. The `# catch:` header of slice 2 mutation patch 13 was updated to the renamed test (patch hunk unchanged); both harnesses re-run on the final tree: slice 2 14 of 14 CAUGHT, slice 2b 12 of 12 CAUGHT.
- Interpretations RATIFIED: (1) attribute shapes are named `<decl>@attributes` (`module_exports@attributes`), because a dot breaks the lock's Type.field grammar; (2) UNSUPPORTED-STRING-DYNAMIC fires only when the list's literal part alone qualifies against an in-scope shape; (3) NON-DISTINCTIVE-KEY fires only on a near miss (3 raw names or 2 distinctive); (4) ORM detection is `.prisma` and `.sql` whole-word containment only, no parser, decorators not detected; (5) module location valuepairs/strings.rs instead of stringpairs.rs.
- Performance probe: 3200 synthetic files, full `invar init` 0.74s.
- Residual (D7, unchanged): a same-package display list with 3 or more shared distinctive names is indistinguishable from a query list; measured, not solved.
- Not implemented, by design: the 32-bit Murmur3 namespace hashing, strict_projection, break-glass and other items in the red-team directive of 2026-10-15 are backlog, not part of frozen revision 1.

### M4. Protected hashes (first 12 hex), verified byte-identical after the change

surface.rs 71b521ce7528, backtest.rs df9ef6e87de4, invariant.rs 49972028d11b, config.rs d30f5e4bf6b3, yaml.rs 1254fde8cdd6, codepairs.rs 004a26dec58e, crates/invar-falsify/Cargo.toml a8023ba46533, root Cargo.toml 7c5f4cee5d13, Cargo.lock 81bcb0ff4e12. Experiment C artifacts unchanged (full hashes in GATE0-LEDGER.md).
