# G2: renames (RECALL-ATTACK-1 phase 4)

Status: r1, measurement erratum (no rule, grammar or fixture change). r0
was FROZEN with its fixtures before implementation and ruled 2026-10-25
(G2-D1, G2-D2, G2-D3 as recommended); it is preserved as G2-SPEC-r0.md
(freeze record G2-SPEC-r0.frozen). Section J records the erratum. Parent: R-D2 (founder, 2026-10-23: conviction of absence;
a rename only as a stated candidate) and the phase authorization of
2026-10-25. Baseline: 9e89f33 (G1-E landed, native 487 of 487).
Measurements: all 273 producer texts and 122 consumer texts of the
corpus, fetched 2026-10-25; the Go-binding counter is
scripts/g2/measure.py, the candidate counter an instrumented throwaway
build of the baseline engine reading its own field extraction
(docs/evidence/g2/candidates.tsv). Fixtures: tests/g2_test.rs; hashes in
G2-SPEC.frozen.

## A. What the three rename cases are, read from their real diffs

| Case | Today | What the producer diff shows | What G2 can soundly do |
|---|---|---|---|
| r1-076 | silent HOLDS | Go field `Link` (json `link_url`) became `ForgeURL` (json `forge_url`), same line, same type `string` | convict: the consumer is Go and reads the Go identifier `repo.Link` (models.go:131), which no longer exists at head under either name |
| r1-106 | silent HOLDS | `info['stream_name']` is still assigned at head, now only when the metadata holds it; `channel_name` is added | nothing: the key survives at head, so this is a change of WHEN it is emitted, not a rename |
| r1-091 | exit 1 (on `version`) | interface field `regexp: string` became `matchers: MiddlewareMatcher[]` | nothing new: already VIOLATED; the types differ, so no candidate |

The one silent rename the corpus holds (r1-076) is not missed because
renames are hard. It is missed because Go fields are keyed by their JSON
tag, and a Go consumer that imports the producer's types reads the Go
identifier. That is a binding gap with a sound conviction, not an
inference.

## B. G2-A: the Go identifier is a second name of the field

Go field extraction keys each exported struct field by its JSON tag
(`link_url`), or by its Go name when there is no tag. G2-A adds, for a Go
producer source only, the Go name as a second name of the same field:

A removed field (removed between base and head, not retyped) of a Go
producer source counts as READ when all hold:
1. the consumer is a Go source (`.go`): only Go code can name a Go
   identifier, and a JSON consumer reads the tag;
2. the field's struct type is exported (its name starts with an upper-case
   letter): another module cannot reach an unexported type;
3. the consumer reads the field's Go name exactly (the existing read
   patterns, `broad_reads`, case-sensitive equality, never `name_key`);
4. no struct at head in that producer source declares a field with that
   Go name (a field whose JSON tag changed but whose Go name survives is
   not removed for a Go reader).
- Outcome: VIOLATED, residue the field's JSON key, through the existing
  rule and witness: `<key> read at <consumer>:<line> · not in <producer>
  · naming differs, read as <GoName>`. The evidentiary standard is the
  existing one (a read field missing from the bound producer file); G2-A
  adds a name, not a rule.
- Measured, all 122 cases: removed exported Go fields read by their Go
  name in a Go consumer occur in exactly one case, r1-076 (`Repo.Link`).
  The only other Go-name match (r1-097: `Data`, `StatusCode`) is on an
  unexported struct (`response`), excluded by condition 2, and r1-097 is
  already VIOLATED on those keys. Zero controls.

## C. G2-R: a rename is a stated candidate, never a verdict

When a field is convicted as removed (residue, removal not retype), and in
the same producer pair exactly one field was added in the same scope with
the same KNOWN type, and no other removed field of that scope has that
type, the witness states the candidate:
`· candidate rename: <added key> (same scope, same type <type>), not
evaluated`, placed after `· not in <producer>`.
- The pairing must be one-to-one: one removed field, one added field
  (forward), and that added field the only fit for any removed field of
  its scope (reverse).
- The type must be known on both sides and equal; a field with no
  declared type (a Python dict key) never pairs.
- A candidate never changes a verdict, never creates a finding (it exists
  only inside the witness of an existing conviction), and is never a
  conviction of its own. INVAR does not claim the consumer should read
  the candidate; it states that the head offers exactly one same-shaped
  replacement.
- Measured, all 122 cases, with the engine's own field extraction:
  21 one-way candidates; the two-way rule leaves 10; the known-type rule
  leaves 7, in r1-010, r1-057 (two), r1-061, r1-070, r1-076, r1-112.
  Excluded by the known-type rule: r1-072, r1-106 (`m3u_profile_name ->
  channel_name`, wrong) and r1-115. Excluded by the two-way rule: r1-005,
  r1-027 (`base_branch`, `head_branch` both to `merge_base`, wrong),
  r1-038, r1-098, r1-104.
- Of the 7, three sit on a field convicted as REMOVED and so appear in
  output: r1-010 `FILESIZE -> TOTAL_FILESIZE_BYTES`, r1-070 `list_id ->
  project_id`, r1-076 `link_url -> forge_url`. Each matches its case's
  recorded title. r1-057's `pipeline -> tasks` sits on a field convicted
  as RETYPED (`Option<Pipeline> -> Option<Spanned<Pipeline>>`, another
  scope), so the rule, which annotates removals only, prints nothing; the
  other three pair fields no consumer reads.

## D. Unsupported, stated

- A Go rename that keeps the JSON tag (`Link` to `ForgeURL`, tag
  `link_url` unchanged): a real break for Go readers, but the field is not
  removed in the JSON identity the engine tracks; no rule (zero corpus
  cases, measured).
- r1-106: conditional emission of a surviving key.
- Renames across scopes, files or types, and many-to-one restructures
  (r1-027, r1-098): no candidate, by design.

## E. Pre-registered corpus movement

- r1-076: silent HOLDS to VIOLATED on `["link_url"]`, witness `read as
  Link` with candidate `forge_url`.
- r1-010, r1-070: verdict, exit code and residue unchanged; the witness
  of the renamed field gains the candidate text.
- Every other case, including all 15 held-out controls, byte-identical.
  Any other movement stops the phase.
- r1-076 and r1-070 are held-out breaks examined during
  specification: tuning rung, no recall claim; the sealed record stays 38
  of 70.

## F. Fixtures (frozen with r0)

tests/g2_test.rs, 14 tests. On the baseline engine (9e89f33) 2 FAIL, the
new behavior (g2_f01, a silent HOLDS; g2_f07, a conviction without the
candidate), and 12 pass, the guards: g2_f02, f03, f04, f05 (G2-A
boundaries), f08, f09, f10, f11, f12, f13 (G2-R ambiguity and
non-findings), f14, f15 (strict mode, determinism). Every G2-R guard is a
real conviction (exit 1) asserted to carry no candidate, so a guard can
only fail by naming a candidate it must not.

## G. Pre-registered mutation operators

| Operator | Change | Property broken | Declared killer |
|---|---|---|---|
| G2-M1 | G2-A disabled (Go name never added) | r1-076 shape stays silent | g2_f01 |
| G2-M2 | Go-consumer requirement removed | a JSON consumer reading `.Link` convicts | g2_f02 |
| G2-M3 | exported-type requirement removed | an unexported struct is treated as reachable | g2_f03 |
| G2-M4 | Go-name survival check removed | a tag-only change convicts Go readers | g2_f04 |
| G2-M5 | Go name matched by name_key | `repo.link` matches `Link` | g2_f05 |
| G2-M6 | candidate annotation dropped | a unique replacement is not stated | g2_f07 |
| G2-M7 | reverse uniqueness removed | two removals share one candidate | g2_f08 |
| G2-M8 | forward uniqueness removed | one of two additions is picked | g2_f09 |
| G2-M9 | type equality removed | a different type is a candidate | g2_f10 |
| G2-M10 | known-type requirement removed | untyped keys pair | g2_f11 |
| G2-M11 | scope equality removed | a field in another type is a candidate | g2_f12 |

Kill rule unchanged (KILLED, SURVIVED, EQUIVALENT argued in writing, or
UNAPPLIED; a build or harness failure is never a kill; --no-fail-fast).

## H. Decision points

- G2-D1 (G2-A). Approve the Go identifier as a second name under the four
  conditions, convicting through the existing rule (recommended: no new
  evidentiary standard, measured exact, one case).
- G2-D2 (G2-R). Approve the stated candidate inside the existing
  witness, one-to-one, same scope, same known type (recommended: every
  printed candidate matches its recorded rename; it never changes a
  verdict), or defer G2-R and land G2-A alone.
- G2-D3 (r1-106). Confirm it as unsupported (a surviving key emitted
  under a new condition), with no rule.

## I. Bounds, unchanged

No conviction without a witness; a candidate is never a verdict;
statements never override convictions; exit codes and the
PRESENT/ABSENT/UNRESOLVED interface untouched; zero new dependencies;
frozen spec and fixtures before code; mutation operators with declared
killers; full-corpus byte comparison; native verification; tuning-rung
numbers are never published as recall.

## J. r1 erratum (2026-10-25, found by the implementation's corpus gate)

- Measurement. r0 counted four printed candidates by intersecting the
  candidate set with the residue NAMES, without checking the kind of
  conviction. r1-057's `pipeline` is convicted as a retype, which the
  frozen rule (section C: "convicted as removed ... removal not retype")
  never annotates. Three candidates print, not four. The implementation
  follows the frozen rule; the pre-registered movement of r0 named one
  case too many, and r1-057 stays byte-identical. Founder ruling G2-D2
  accepted "all four"; the fourth was this measurement error.
- Rendering. An existing fixture (harness_test, change-scoped markdown)
  showed the r0 limitation in practice: the candidate inside the `not in`
  code span broke the asserted span. The implementation gives the
  candidate's producer, name and type each their own code span; plain
  text output is unchanged, the existing fixture holds unmodified, and the
  limitation is removed from section D.
