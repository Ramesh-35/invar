# JUDGE-CC r0: an undecided constraint change is NOT EVALUATED, never VIOLATED (r0)

Drafted 2026-11-09, frozen with the six `cc_` fixtures in
crates/invar-falsify/tests/judge_cc_test.rs BEFORE implementation; freeze
record: JUDGE-CC-SPEC.frozen. Amends JUDGE-SUBTYPE r1 (JS-D3).

## Why

JS-D3 makes every `pattern` or `format` change (and every non-numeric
bound change) VIOLATED with the sentence "whether this narrows what is
accepted is not decided here". That is a verdict and its own retraction in
one line. Measured on SchemaStore (PROG-0 population work, 2026-11-09):
27 of the judge's 67 VIOLATED changes rest only on such witnesses where
the new constraint accepts at least everything the old one did (22 SARIF
`format: uri` -> `uri-reference`; claude-code-settings twice,
openapi-overlay, openapi-arazzo, partial-inwards: alternatives added, a
lookahead or prefix made optional). Two more of the 29
constraint-change-only VIOLATED are real narrowings (pyproject: a pattern
added; linutil-tabs: `[^ ]` -> `\S`). The engine cannot tell these apart,
and A3 receipts must never seal a verdict the engine itself calls
undecided. ECMA-262 pattern containment is not decided by this judge in
either direction, so the honest verdict is the one that says so.

## Ruling CC (r0)

A change this judge detects but does not decide — `pattern` changed,
added or not (removal stays a loosening, HOLDS); `format` likewise; a
non-numeric bound changed — is stated, not accused:

- the witness kind `constraint_changed` is retired; such changes become
  stated findings (`undecided_constraint`) carrying path, keyword, old
  and new value;
- verdict: VIOLATED only when a decided witness remains. A change whose
  only findings are undecided constraints is NOT EVALUATED, reason
  `undecided_constraint`, exit 4: it still blocks a gate, and the text
  says review the change or version it;
- precedence among NOT EVALUATED reasons: ref_siblings, then
  undecided_constraint, then unjudged_construct, then unchecked_change;
- JSON gains `undecided_paths` (sorted paths) and `undecided_changes`
  (one sentence each, "PATH KEYWORD was A, is now B"); text prints one
  "not evaluated:" line per change;
- the SARIF rule `constraint_changed` is removed (nothing can fire it;
  nothing has been published that depends on it);
- `invar witness` on such a pair writes nothing and states the verdict,
  as it already does for any NOT EVALUATED pair.

Decided witnesses are untouched: removed, retyped, narrowed (enum),
tightened (numeric, length, item-count), escalated keep VIOLATED and
their counterexamples.

## Fixtures (frozen before implementation; 2 of 6 pass today, the guards)

- cc_f01 pattern changed, nothing else: NOT EVALUATED, exit 4, both
  values stated, no witness (today: VIOLATED).
- cc_f02 format uri -> uri-reference: NOT EVALUATED (today: VIOLATED).
- cc_f03 guard: enum narrowing stays VIOLATED (passes today).
- cc_f04 removal + pattern change: VIOLATED on the removal, the pattern
  change stated beside it (today: the stated line fails).
- cc_f05 guard: pattern removed stays HOLDS (passes today).
- cc_f06 pattern added: NOT EVALUATED, "was absent, is now" (today:
  VIOLATED).

## Amendments to earlier fixtures, declared at freeze time

- js_f06 (JUDGE-SUBTYPE r1): asserts exit 4 and the undecided sentence
  instead of VIOLATED; still kills JS-M4 (under that mutant the change
  vanishes entirely: HOLDS, no sentence).
- fp_f09 (JUDGE-FILEPAIR r0): asserts NOT EVALUATED with
  `undecided_paths: ["f"]` instead of a constraint_changed witness.
- w_f04 (WITNESS r0): the pair's verdict is now NOT EVALUATED; asserts
  0 documents written and no witness lines; still kills W0-M2 via w_f01.
  If W0-M2's killer no longer fails, W0-M2 is re-anchored and recorded.
- judge_sarif_test: the static rule list drops constraint_changed.

## Mutation operators

- CC-M1 the undecided branch dropped from the verdict (falls to HOLDS) -> cc_f01
- CC-M2 undecided changes left in the witness list (stays VIOLATED)    -> cc_f01
- CC-M3 enum narrowing also demoted to undecided                       -> cc_f03

## Pre-registered prediction

On SchemaStore (819 changes): VIOLATED 67 -> 38; the 29 changes whose
witnesses are all constraint_changed (list frozen in cc-predicted.txt)
move to NOT EVALUATED; no change anywhere becomes HOLDS; HOLDS stays 270.
Iglu and AsyncAPI: no VIOLATED built on constraint_changed alone is
expected (measured, not assumed); no HOLDS gained. WG gate (76 pairs):
still 0 HOLDS — some VIOLATED may become NOT EVALUATED and the counts are
recorded. The test-document harness and the Rust fixtures corpus do not
read these kinds and do not move.
