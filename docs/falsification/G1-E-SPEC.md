# G1-E: a literal emission became opaque (RECALL-ATTACK-1 phase 3)

Status: r0, FROZEN with its fixtures before any implementation, for
founder ruling. Parent: G1-SPEC.md as ruled 2026-10-25 (G1-D1: G1-E plus
the r1-068 `url` isolation; G1-D3: the seven emission call forms are
frozen, additions are new frozen entries). Baseline: fb85ae9 (request side
landed, native 472 of 472). Measurements: all 273 producer texts and all
122 consumer texts of the corpus, fetched 2026-10-25, scanned with a
throwaway counter whose patterns are exactly the grammar below. Fixtures:
tests/g1e_test.rs; hashes in G1-E-SPEC.frozen.

## A. What r1-107 shows

The linkwarden tags handler emitted `.json({ response: tags?.response })`
on its GET path. The head builds `const { statusCode, ...data } = tags;`
and emits `.json(data)`. The key `response` still appears in five head
literal emissions (the error paths), so no rule keyed on a field's
disappearance can see the change: the field is still "declared". What
changed is one EMISSION SITE, from a literal the file states to an
expression it does not. The consumer reads `response`
(`tagsStatsData?.response`, recognized by the existing read patterns).
Today: silent HOLDS.

## B. Grammar

E1, emission sites. On a non-comment line (the shared comment rule: a
line whose first non-space characters are `//`, `#`, `*` or `/*`), an
occurrence of one of the seven frozen call forms:
`.json(`, `.send(`, `jsonify(`, `JsonResponse(`, `json.dumps(`,
`Response(`, `send_result(`.
A form not beginning with `.` matches only as a whole name: the character
before it is not an identifier character (`JsonResponse(` is never a
`Response(` site; `web.Response(` is).

E2, the payload argument. From the character after `(`, the argument list
is scanned to its matching `)`, across lines, string-aware (a quote opens
a literal that closes at the next unescaped matching quote), splitting on
commas at bracket depth zero. The payload is the first argument, except
for `send_result(`, whose payload is the second (`send_result(msg_id,
payload)`). An unterminated list is no site.

E3, classification of the payload (trimmed):
- LITERAL: it begins with `{`. Its keys are the depth-one entries that
  are a string literal or an identifier immediately followed (after
  spaces) by `:` and not `::`. Spread and shorthand entries contribute no
  key.
- OPAQUE: it is exactly one identifier, and not one of `null`, `None`,
  `true`, `false`, `True`, `False`, `undefined`.
- Anything else (a member access, a call, an array, a string): not
  counted, either way.

## C. Rule

G1-E. For each bound producer pair (producer and base; code companion and
its base, when declared) and each call form F, with L and O the counts of
LITERAL and OPAQUE emission sites of F:
1. L(head) < L(base) (a literal emission was lost);
2. O(head) > O(base) (an opaque emission was gained);
3. some key of a base LITERAL emission of F is read by the consumer (the
   existing recognized reads, `broad_reads`, matched by name key);
4. the contract has no residue, from any rule.
- Outcome: stated NOT EVALUATED (exit 4), never VIOLATED (the F1
  discipline: the head file no longer states what that site emits, so the
  read can be neither confirmed nor convicted from it). One statement per
  pair and form.
- Statement, exact form: `'<F>' emission became opaque in <head>: literal
  <Lb> -> <Lh>, opaque <Ob> -> <Oh> (first opaque at <head>:<line>,
  '<ident>'); base stated '<K>'[, '<K>'...] literally (<base>:<line>),
  read in <consumer>; whether head still emits it|them is not evaluated
  [not-evaluated:opaque-emission]`, where the keys are the read keys of
  the base literal emissions of F (sorted), <base>:<line> is the first
  base literal emission of F holding one of them, and <head>:<line> is
  the first head OPAQUE emission of F. The statement names counts rather
  than claiming which head site replaced which base site: the grammar
  cannot pair them.
- Mode: change-scoped only. Strict mode and lock mode are untouched.

Measured, all 122 cases: 68 literal and 63 opaque emission sites across
the 273 producer texts. Opaque sites occur in five cases (r1-026, r1-053,
r1-056, r1-069, r1-107); in four of them the opaque counts are equal base
and head. The rule fires on exactly one case, r1-107 (`.json(`, literal
6 -> 5, opaque 0 -> 1, key `response`), a held-out break. Zero controls,
zero other cases.

## D. r1-068 `url`, isolated: not an E1 defect

r1-068's expected residue lists five fields; E1 convicts four
(active_dataset_tlvs, border_agent_id, channel, extended_address) and not
`url`. Isolation (docs/evidence/g1e/r1-068-url-isolation.txt): the bound
consumer, thread-config-panel.ts, never reads `url`. The field is
declared in the consumer-side type `OTBRInfo` (src/data/otbr.ts:8) and
read in a DIFFERENT file, dialog-thread-dataset.ts:52
(`otbrInfo.url`), to which the panel passes the whole object
(thread-config-panel.ts:340, `showThreadDatasetDialog(this, { network,
otbrInfo: this._otbrInfo })`).

E1 is correct for the contract as bound: it reports exactly the moved
fields the bound consumer reads. The miss is a binding limitation (a read
reached through an object passed to another file), the same class as the
builder-variable dataflow of r1-044. No rule is proposed. The corpus is
sealed, so the case's binding is not changed; r1-068 stays PARTIAL in the
record, with the cause now stated.

## E. Unsupported, stated

- Which head site replaced which base site (counts only).
- Opaque payloads that are member accesses or calls (`.json(tags.data)`,
  `.json(build())`): not counted, so a literal turned into one of them is
  silent, as today.
- Emission through a call form outside the frozen seven, or through a
  variable holding the response object.
- Comments inside a multi-line argument list are not masked.
- Values of the opaque identifier (what `data` holds): never evaluated.

## F. Pre-registered corpus movement

Exactly one case moves: r1-107, from silent HOLDS to stated NOT
EVALUATED. Every other case, including all 15 held-out controls, stays
byte-identical; any other movement stops the phase. No recall claim:
r1-107 was examined during specification (tuning rung), so the sealed
record stays 38 of 70.

## G. Fixtures (frozen with r0)

tests/g1e_test.rs, 15 tests. On the baseline engine (fb85ae9) 5 FAIL,
the new behavior (g1e_f01, f07, f10, f11, f15; each fails as a silent
HOLDS, exit 0), and 10 pass, the guards (g1e_f02, f03, f04, f05, f06,
f08, f09, f12, f13, f14).

## H. Pre-registered mutation operators

| Operator | Change | Property broken | Declared killer |
|---|---|---|---|
| G1E-M1 | condition 1 removed (literal loss) | an added opaque emission alone fires | g1e_f02 |
| G1E-M2 | condition 2 removed (opaque gain) | a lost literal alone fires | g1e_f03 |
| G1E-M3 | condition 3 removed (consumer read) | fires on keys nobody reads | g1e_f04 |
| G1E-M4 | comment skip removed from E1 | a commented emission counts | g1e_f05 |
| G1E-M5 | statement added despite residue | a statement rides on a conviction | g1e_f06 |
| G1E-M6 | payload always the first argument | `send_result(` payload missed | g1e_f07 |
| G1E-M7 | OPAQUE accepts any non-literal expression | a member access becomes opaque | g1e_f08 |
| G1E-M8 | keyword exclusion removed | `null` becomes opaque | g1e_f09 |
| G1E-M9 | whole-name boundary removed | `JsonResponse(` also counts as `Response(` | g1e_f10 |
| G1E-M10 | argument scan limited to the call line | a multi-line literal loses its keys | g1e_f11 |
| G1E-M11 | keys taken at any depth | a nested key counts as emitted | g1e_f12 |
| G1E-M12 | code companion pair not scanned | a companion switch is missed | g1e_f15 |

Kill rule unchanged: a kill is a failing assertion in the frozen
fixtures, reported per operator with the declared killer's result and the
incidental kills separately; KILLED, SURVIVED, EQUIVALENT (argued in
writing) or UNAPPLIED; a build or harness failure is never a kill. The
harness runs with --no-fail-fast.

## I. Decision points

- G1E-D1 (rule). Approve G1-E as specified: counts per form, statement
  only, residue-free contracts only (recommended: measured exact on the
  corpus, and it never convicts).
- G1E-D2 (r1-068). Accept the isolation: binding limitation, not an E1
  defect, no rule, the case stays PARTIAL with its cause stated
  (recommended), or direct further work on cross-file object flow, which
  would be a new mechanism with its own spec.
- G1E-D3 (payload position). `send_result(` takes its payload from the
  second argument, every other form from the first (recommended: the
  measured Home Assistant signature), or exclude `send_result(` from the
  OPAQUE count.

## J. Bounds, unchanged

No conviction from G1-E; statements never override convictions; exit
codes and the PRESENT/ABSENT/UNRESOLVED interface untouched; zero new
dependencies; frozen spec and fixtures before code; mutation operators
with declared killers; full-corpus byte comparison; native verification;
tuning-rung numbers are never published as recall.
