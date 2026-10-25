# G3: selector values beyond literals, and the default site (RECALL-ATTACK-1 phase 1)

Status: DRAFT r2, revised under the founder's conditional approval of
2026-10-23 (A-1 site identity, A-2 dot-path accepted, P6 site association
corrected). Frozen (hashed in G3-SPEC.frozen) before any implementation;
fixtures in tests/g3_test.rs are frozen with it. Implementation remains
unauthorized until r2 is reviewed.
Parent: F1 (docs/falsification/F1-SPEC.md, ruled: a default switch is
stated, never convicted) and the founder's RECALL-ATTACK-1 phase 1 ruling
of 2026-10-23. Everything F1 ruled stands; G3 only widens what counts as a
selector occurrence.

## 1. The exact current grammar (surface.rs, default_selectors and selector_value, at aed8988)

A selector occurrence is (name, value), per line, comment-opening lines
skipped, name case-insensitive and required to contain `version` or
`format`. Values (`selector_value`): an integer, `true`, `false`, or a
quoted string whose content is an integer or `v`+integer; normalized by
stripping quotes and the leading v. Four name-binding patterns: assignment
(`name = v`, `name: v`, `name: Type = v`), `defaultValue:`/`=` on the same
line, call argument (`name(v`, version-names only), quoted key (`"name", v`
or `"name": v`). The switch rule (`selector_switches`): group by name; fire
when the head's value set gains a value absent at base, for names present
on both sides. Outcome (F1, unchanged by G3): with no residue, every bound
field of that source becomes a stated NOT EVALUATED site
(`[not-evaluated:default-switch]`), exit 4; with residue, nothing is added;
never VIOLATED.

## 2. What the evidence demands (r1-113, producer bytes fetched 2026-10-23)

Cargo's change is NOT a value-set gain. At base, `ResolveVersion::V4`
already exists (`max_stable()`); what changed is the value at the default
site:

```text
base  fn default() -> ResolveVersion {        head  fn default() -> ResolveVersion {
          ResolveVersion::V3                            ResolveVersion::V4
```

So two additions are needed, and the second is the one that fires here:
a variant-shaped value, and a default-site occurrence whose name keys the
site, so its value set is {3} at base and {4} at head.

## 3. Additions

### V5, variant value

`TypeIdent::Variant` or `TypeIdent.Variant` (A-2: both separators, ruled
2026-10-23), where TypeIdent is a whole identifier containing `version` or
`format` (case-insensitive) and Variant is a whole identifier that is `V`
or `v` followed by digits, or digits alone; whole-identifier means bounded
on both sides by non-identifier characters, recognized by the scanner's
tokenizer, never by substring. Normalized value: the digits. A variant not
of that shape (`ResolveVersion::Latest`, `FormatVersion.parse`,
`formatVersion.v2x`) is NOT a value; G3 makes no claim about it (section
6). The two separators are recognized as syntax only; no semantic
equivalence between languages is implied, and the inherited outcome stays
a stated NOT EVALUATED either way.

### P5, variant occurrence

Every V5 literal on a non-comment line yields (name = lowercased
TypeIdent, value). It feeds the existing set-difference rule unchanged.
P5 alone cannot catch r1-113 (V4 exists at base) and is included because
the set rule is already ruled and variant values are the same mechanism in
enum form; it fires only when a type's variant set truly gains a value.

### P6, default-site occurrence (r2: site identity and association)

Trigger: a non-comment line containing a whole identifier that contains
`default` (case-insensitive). The trigger identifier itself is the site
identifier.

Association window: the trigger line, then up to two following non-comment
lines, extended one line at a time ONLY while the preceding line of the
chain, after trailing whitespace is trimmed, ends with a continuation
token: `{`, `(`, `:`, `=`, `,`, `->` or `=>`. A line that closes its
statement (`;`, a closed brace, anything else) ends the window, so a
completed unrelated declaration never lends its neighbor a value
(fixture g3_f17). The window length and token set are stated constants of
the grammar, chosen once from the r1-113 evidence shape (a one-line body
under a signature ending `{`), never adapted per case.

Occurrence: the FIRST V5 literal in the window yields
(key = `<lowercased trigger identifier> <lowercased TypeIdent>`, value).
A trigger line with no V5 in its window yields nothing
(`#[derive(Default)]`, `default:` config keys, prose).

Site identity (A-1, ruled 2026-10-23): the key is the trigger identifier
plus the type, the strongest identity this line scanner can establish
deterministically; type-only keying is rejected because two sites of one
type would merge and a change at one could be hidden by the other
(fixture g3_f13 constructs exactly that hiding under type-only keying).
When one file side still has two or more DISTINCT trigger lines whose
identifiers and types are identical (two `fn default()` for the same type
in two impl blocks), the scanner cannot distinguish them: the key's value
sets are the union, the merge is DISCLOSED, never silent, and the witness
appends `(k same-named default sites; site attribution not established)`
with k the larger side's count (fixture g3_f14). Known limitation, stated:
within one open window the grammar cannot prove the V5 belongs to the
trigger's declaration; the bounds are the continuation gate, first-V5-only,
the disclosure above, and the outcome ceiling (a stated NOT EVALUATED,
never a conviction). A value swap between two same-keyed sites that leaves
the union unchanged does not fire; that is the set rule's existing
semantics, inherited from F1, stated here.

Syntactic recognition is not semantic relevance: a P6 occurrence proves
only that a default-named site's selector-typed variant text changed. G3
therefore inherits F1's outcome verbatim: stated NOT FULLY EVALUATED with
the existing witness line naming the selector and `OLD -> NEW`, exit 4,
never VIOLATED, nothing added when residue exists, change-scoped and lock
modes only (no `producer_base`, no G3).

## 4. Conditions

- HOLDS: unchanged from today unless a switch (existing patterns, P5 or
  P6) is detected.
- Stated NOT EVALUATED: a detected switch on a residue-free contract, with
  the F1 witness format, reason code `[not-evaluated:default-switch]`.
- VIOLATED: never produced or altered by G3.
- UNEVALUATED is never produced for: names on one side only, value
  removals, non-numeric variants, values behind calls or across files.

## 5. Expected corpus movement, pre-registered

Exactly one case: r1-113 moves from silent HOLDS to stated NOT EVALUATED.
r1-059, r1-075, r1-099, r1-100 already fire (existing patterns) and must
be byte-identical. r1-056 and r1-094 are A1 statements, untouched. r1-095,
r1-096 and r1-051 are NOT selector cases and are assigned to later phases;
G3 does not claim them. Zero control flips of any kind; zero changes on
every other corpus case; EVAL-007/008 round-trips byte-identical. Any
other movement, in either direction, stops the phase for review.

## 6. Unsupported, stated

Non-numeric variants; selector values produced by calls or held in other
files; cfg/feature-gated or macro-generated defaults; default sites more
than two non-comment lines from their value. All stay exactly as today
(fail-closed means no claim is manufactured; a real break in these shapes
remains a silent miss in scoring, which is the register's business to
record, not the grammar's to hide).

## 7. Fixtures (tests/g3_test.rs, frozen with this spec)

| Fixture | Property | On baseline |
|---|---|---|
| g3_f01 | r1-113 shape: default site V3 to V4, V4 present at base elsewhere; states, exit 4 | FAILS |
| g3_f02 | pure variant-set gain (P5): head gains FormatVersion::V2; states | FAILS |
| g3_f03 | non-numeric variant default change (::Latest): stays HOLDS | passes |
| g3_f04 | default site unchanged, code moved lines: stays HOLDS | passes |
| g3_f05 | derive(Default) and default: keys with no V5 in window: HOLDS | passes |
| g3_f06 | variant change only on comment lines: HOLDS | passes |
| g3_f07 | head LOSES a variant value, gains none: HOLDS | passes |
| g3_f08 | two selector types, one default changes: the witness names that type only; states | FAILS |
| g3_f09 | residue plus a variant default switch in one JS producer: VIOLATED, no default-switch line added | passes |
| g3_f10 | existing F1 quoted-int pattern golden: still fires identically | passes |
| g3_f11 | strict mode (no producer_base) byte-frozen: no switch analysis runs, and the baseline's existing conviction on a .rs producer (it extracts no spec fields there) is asserted verbatim so G3 cannot alter strict mode in passing | passes |
| g3_f12 | exact witness text for g3_f01 (`default selector 'default resolveversion' changed 3 -> 4 ...`) | FAILS |
| g3_f13 | two same-type default sites with distinct identifiers (`default`, `default_for_rust_version`); one changes; the witness names that site only. Type-only keying would merge {3,4} to {4} and hide the switch entirely | FAILS |
| g3_f14 | two same-type sites BOTH named `default`; one changes; fires with the ambiguity disclosure `(2 same-named default sites; site attribution not established)` | FAILS |
| g3_f15 | dot-path positive: a Python-shaped default site `FormatVersion.V1` to `.V2` under a `def default_format():` trigger (window opened by `:`); states | FAILS |
| g3_f16 | dot-path negatives: `FormatVersion.parse` to `.encode` and a `formatVersion.v2x` identifier change stay HOLDS | passes |
| g3_f17 | a default declaration that closes its statement (`const defaultTimeout = 5;`) followed by an unrelated selector-typed variant change whose values all exist at base: no association, HOLDS | passes |

The seven FAILS are the new behavior; the ten passes are frozen boundary
guards that must survive implementation unchanged.

## 8. Mutation operators (declared domain: P5, P6, V5 and their keying)

| Mutant | Change | Property broken | Killed by |
|---|---|---|---|
| G3-M1 | V5 extraction removed | variant values never seen | g3_f02 |
| G3-M2 | P6 window set to same-line only | default-site value on the next line missed | g3_f01 |
| G3-M3 | P6 fires without the default-word requirement | every variant line becomes a site | g3_f04 |
| G3-M4 | TypeIdent is_sel requirement dropped | non-selector enums fire | g3_f05 |
| G3-M5 | non-numeric variants accepted as values | ::Latest manufactures a claim | g3_f03 |
| G3-M6 | P6 key omits the type ident | two types collapse to one selector | g3_f08 |
| G3-M7 | switch fires on value removal | losses treated as switches | g3_f07 |
| G3-M8 | comment-line skip removed for the P6 window | commented variants fire | g3_f06 |
| G3-M9 | G3 occurrences fire despite residue | F1's adds-nothing rule broken | g3_f09 (JS, residue and switch in one source) |
| G3-M10 | witness renders NEW -> OLD | evidence misstates direction | g3_f12 |
| G3-M11 | P6 key drops the trigger identifier (type-only merge) | a change at one site hidden by another site's values | g3_f13 |
| G3-M12 | the continuation gate removed (window crosses `;`) | a closed declaration lends its neighbor a value | g3_f17 |
| G3-M13 | same-key multiplicity not disclosed | silent merge of indistinguishable sites | g3_f14 |
| G3-M14 | V5 dot-path matched by substring instead of whole identifiers | non-variant members and suffixed identifiers fire | g3_f16 |

Pass bar: every non-equivalent mutant killed by a failing behavioral
assertion in the frozen fixtures; a compilation failure, harness failure
or unrelated panic is not a kill and is reported as NOT A VALID KILL with
the mutant re-worked; any claimed-equivalent mutant argued in writing in
the kill log, never counted as a kill.

## 9. Files the implementation would touch (phase-isolated; NOT yet authorized)

- crates/invar-falsify/src/surface.rs: V5, P5 and P6 (the P6 window makes
  `default_selectors` a small two-pass over lines; the switch tuple gains
  the optional site-multiplicity disclosure).
- tests/g3_test.rs: the frozen fixtures.
- OPEN SCOPE QUESTION for the implementation ruling: the A-1 ambiguity
  disclosure must reach the witness line, which check.rs renders. Either
  (i) one mechanical line in check.rs threads the disclosure into the
  existing witness format (recommended; no other check.rs behavior), or
  (ii) surface.rs folds the disclosure into the selector name string,
  keeping check.rs untouched but printing the note inside the name's
  quotes. g3_f14 is frozen to (i)'s rendering; a ruling for (ii) re-freezes
  that one fixture's expected text. No change to main.rs, lock.rs, mcp.rs,
  strict mode, or any output beyond the registered witness instances.

## 9a. Baseline facts recorded while freezing the fixtures

- Change-scoped field extraction does not read Rust struct fields (removing
  a `pub` field from a .rs struct gives HOLDS), so g3_f09 provokes residue
  in a JS producer; the selector patterns are textual and fire in any
  language.
- Strict mode on a .rs producer extracts no spec fields and convicts the
  consumer's read today; g3_f11 freezes that byte-for-byte. Both facts are
  G1/G2 material, out of G3's scope.

## 10. Reproduction

- Baseline: release binary from aed8988 (bca55be8506e), sandbox Linux
  x86_64.
- Corpus: `env -u GITHUB_TOKEN -u GH_TOKEN bash scripts/r1-run.sh <invar>`
  (the unset matters: an injected placeholder token routes fetches to the
  authenticated endpoint, which 403s); raw rows
  docs/evidence/recall/recall-2026-10-23.tsv; held-out breaks today:
  42 CAUGHT, 1 PARTIAL, 12 stated, 15 silent of 70.
- Fixtures: `cargo test --release --test g3_test`; on the baseline,
  g3_f01, g3_f02, g3_f08 and g3_f12 fail and the other eight pass.

## 11. Ruled ambiguities (2026-10-23)

- A-1: RULED. The key is the trigger's own identifier plus the type
  (section 3, P6); type-only keying rejected; indistinguishable same-named
  sites merge with disclosure, never silently. Exact key format:
  `<lowercased trigger identifier> <lowercased type identifier>`, e.g.
  `default resolveversion`, `default_for_rust_version resolveversion`.
- A-2: RULED. Dot-path accepted under the whole-identifier lexical
  constraints of V5, with direct positive (g3_f15) and negative (g3_f16)
  fixtures; `::` and `.` are each exercised independently (g3_f01/f02 use
  `::`).

## 12. Native verification gate

The Linux baseline and fixture results never substitute for the native
macOS release run. Phase completion requires the founder-captured
`cargo test --release` output on the Mac, with actual counts, skips and
failures; the expected native 425 of 425 is UNVERIFIED until that command
has completed there, and is not claimed before.
