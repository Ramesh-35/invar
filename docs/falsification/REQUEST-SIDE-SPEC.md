# REQUEST-SIDE: what the consumer sends against what the producer accepts (RECALL-ATTACK-1 phase 2)

Status: r2, FROZEN (bounded repair, founder's ruling of 2026-10-25,
16:18: r1 rejected for landing because RS-M2 survived; one fixture added,
one wording fix, no rule or grammar change). r1 was FROZEN for
implementation under the founder's ruling of 2026-10-25 (RS-D1 approved
with an evidentiary clarification, RS-D2, RS-D3 and RS-D4 approved). r0
and r1 are preserved unchanged as REQUEST-SIDE-SPEC-r0.md and
REQUEST-SIDE-SPEC-r1.md (r1 freeze record: REQUEST-SIDE-SPEC-r1.frozen).
Section H records every r1 change and the rulings, section J the r2
repair; fixtures are tests/rs_test.rs (r1) and tests/rs_r2_test.rs (r2),
hashes in REQUEST-SIDE-SPEC.frozen. Parent: R-D3 (specification authorized,
literal keys only) and the founder's order ruling of 2026-10-25 (G3,
request side, G1-E, G2, S4). Baseline: 201d193. Measurements: every
producer text (base and head, 273 files) and every consumer text (122
files) of the corpus, fetched 2026-10-25, scanned with throwaway counters
whose patterns are exactly the grammars below.

## A. What the cases show

Today's engine checks what a consumer READS against what a producer
DECLARES. It already handles a request when the contract is modeled with
the SENDER as producer: r1-090 (Bitwarden client sends, vaultwarden
reads) is caught. The silent cases are modeled the other way round: the
API server is the producer and the client WRITES the key, so the break is
in what the server stopped ACCEPTING, which nothing reads today. Each case
uses a different inbound idiom:

| Case | Today | Producer inbound idiom | What changed |
|---|---|---|---|
| r1-118 | silent HOLDS | Go form accessor `String("pdfFormat", &pdfFormat, "")` | the read was removed; no `"pdfFormat"` literal remains in the head file |
| r1-060 | silent HOLDS | DRF serializer field `device_role = NestedDeviceRoleSerializer()` | the field survives as `read_only=True`; the writable field is `role` |
| r1-051 | silent HOLDS | raw body `io.ReadAll(r.Body)` | became a JSON decode into a `{rules}` struct; the consumer still sends `rules.String()` as plain text |
| r1-037 | stated (A1) | Rust request struct `MarkPostAsRead` | `post_id` moved to other request structs in the same file |

## B. The three grammars

### B1. Consumer payload extraction (outbound writes)

A consumer WRITE of key K is a literal K in one of three forms, on a
non-comment line:
- subscript assignment: `x["K"] = v` (not `==`);
- object or dict literal key: `"K": v`;
- builder setter: `.Add("K"`, `.Set("K"`, `.WriteField("K"`,
  `.SetFormData("K"`, `.SetQueryParam("K"`.

Not extracted, ever: keys held in variables, f-strings or templates,
spreads (`...x`), `**kwargs`, keys built by concatenation. An unextracted
key yields no claim of any kind from B1 alone, and is never evidence that
the consumer does NOT send the key.

String literals are tokenized left to right on each line (a quote opens a
literal that closes at the next unescaped matching quote), so a key is
always a whole literal, never a substring. K has the shape
`[A-Za-z_][A-Za-z0-9_-]*`. A dict-literal key needs a value character on
the same line after the colon (`"K": ` at line end is not extracted).

### B2. Producer inbound binding (what the server reads from a request)

A producer INBOUND READ of key K is a request accessor whose first
argument is the string literal K, on a non-comment line:
- Go: `.FormValue(`, `.PostFormValue(`, `.PostForm(`, `.Query(`,
  `.QueryParam(`, `.FormFile(`, `.DefaultPostForm(`, `.DefaultQuery(`,
  and builder-style form readers `String(`, `Bool(`, `Int(`, `Int64(`,
  `Float64(`, `Duration(`, `Strings(`, `Content(`, `MandatoryString(`,
  `MandatoryContent(`, `Custom(`;
- Python: `request.<data|POST|GET|args|form|json|query_params>[` and
  `.get(` on those;
- JavaScript and TypeScript: `req.body`, `req.query`, `req.params`
  followed by a literal subscript or call argument (`req.body["K"]`), or
  by a literal member (`req.body.K`, K an identifier).

Comment rule for B1, B2 and B3 (shared with F1 and G3, unchanged): a line
whose first non-space characters are `//`, `#`, `*` or `/*` is skipped
entirely. A trailing comment after code on the same line is NOT masked;
that is a stated limitation.

Request structs and DTOs (Rust, Go, TypeScript classes) are already read
by the existing field extraction and are not re-specified here.

### B3. Read-only declaration (DRF)

A serializer field line `name = SomeSerializer(...)`: optional
indentation, an identifier, `=`, a bare identifier immediately followed by
`(`. Its arguments, up to the first `)` on the line, either contain
`read_only=True` (spaces allowed around `=`) or do not.

## C. Rules

RS-1, inbound key removed. All four must hold, over the producer sources
the contract binds (the producer and, when declared, its code companion):
1. a base producer source has a B2 inbound read of K;
2. no head producer source has a B2 inbound read of K;
3. no head producer source contains K as a whole string literal on any
   non-comment line (a surviving literal in a form the grammar does not
   read is an extraction boundary, never proof of absence: RS-1 then makes
   no claim at all);
4. the consumer has a B1 write of K.
- Outcome: VIOLATED (exit 1), residue K. The contract binding is the
  `[[invariant]]` block itself (one producer, its companion, one
  consumer); nothing outside that block is ever paired.
- Witness, exact form: `K · not in <producer> at head: no inbound read
  and no literal of K remain (inbound read at <base source>:<base line>
  at base); written at <consumer>:<line>[, <line>...]`, where <producer>
  is the head producer source's artifact name and <base source> the name
  of the base source holding the read (the same path at another revision
  in the corpus). The consumer lines
  live in the message, so a SARIF result for RS-1 carries no separate
  location (stated limitation, no struct change).
- Evidentiary standard: the same as the existing response-side rule,
  which convicts a read field missing from the bound producer file.
- Measured, all 122 cases: fires on exactly one, r1-118, on exactly
  `pdfFormat` (its expected residue). Zero controls, zero tuning cases.

RS-2, became read-only. A serializer field is declared at base WITHOUT
`read_only=True` and at head WITH it (B3), in the same producer source,
and the consumer mentions its name on a non-comment line (whole word). A
field that is new at head and read-only does not qualify: it gained
nothing.
- Outcome: stated NOT EVALUATED (exit 4), never VIOLATED, because the
  consumer's mention could be a read, which a read-only field still
  serves. Statement: `'F' became read_only in <producer> (<producer>:<head
  line>); consumer mentions it at <consumer>:<line>; whether it writes it
  is not evaluated [not-evaluated:read-only-field]`.
- Measured: fires on exactly one, r1-060, on exactly `device_role`.

RS-0, fail closed. RS-1 conditions 1 to 3 hold for K, the consumer has
no B1 write of K, and the consumer has an opaque payload construct on a
non-comment line: a variable subscript assignment `x[k] =` (k an
identifier, not `==`), `**name`, a spread `...name`, `json=name` or
`data=name` (the words `json` and `data` whole, spaces allowed around
`=`), or `SetBody(name` / `SetBody(&name`.
- Outcome: stated NOT EVALUATED (exit 4), because the consumer may send
  K without naming it. Statement: `inbound read of 'K' removed from
  <producer> (read at <base source>:<base line> at base); consumer payload is
  opaque at <consumer>:<line>, so whether it sends 'K' is not evaluated
  [not-evaluated:opaque-payload]`.
- Measured: fires on zero corpus cases. The boundary is real but
  untested by the corpus, so its evidence will be synthetic fixtures
  only, and the spec says so.

A statement never overrides a conviction, and RS-2 and RS-0 add nothing
to a contract that already has residue, from any rule (the F1
discipline, unchanged). RS-1 may add residue to a contract that already
has residue; a K already in the residue is not added twice.

Mode: change-scoped only (a `producer_base` is declared), like F1 and G3.
Lock mode and strict mode are untouched.

## D. Unsupported, stated

- r1-051. The body changed from raw text to a JSON object, and the break
  is a REQUIRED key the consumer never sends. Seeing it needs route
  association: the consumer posts to `/filtering/set_rules` while the
  producer registers `/control/filtering/set_rules`, so even the route
  match would be a heuristic. No rule is proposed; it stays a silent miss.
- Inbound reads through framework binding (struct tags on a decoded
  request, form-model classes, validation schemas) other than the
  existing field extraction.
- Any key not written as a literal.

## E. Pre-registered corpus movement

Exactly two cases move: r1-118 from silent HOLDS to VIOLATED on
`pdfFormat` (a tuning-rung catch), and r1-060 from silent HOLDS to stated
NOT EVALUATED. Every other case, including all 15 held-out controls, must
stay byte-identical; any other movement stops the phase. No recall claim:
the corpus is not held out.

## F. Decision points

- RS-D1 (RS-1 outcome). VIOLATED, as above (recommended: the same
  evidentiary standard the engine already applies to response-side reads,
  and measured exact on the only case it fires on), or a stated
  NOT EVALUATED like G3.
- RS-D2 (scope). RS-1, RS-2 and RS-0 together (recommended: RS-0 is the
  fail-closed boundary the founder required, and costs nothing on the
  corpus), or RS-1 alone.
- RS-D3 (grammars). The accessor list in B2 and the write forms in B1 are
  the measured sets, frozen; additions need their own measurement.
- RS-D4 (r1-051). Confirm it stays unsupported rather than adding route
  association, whose own matching would be heuristic.

## G. Bounds, unchanged

No conviction without a witness naming both the consumer write site and
the producer read site; statements never override convictions; exit
codes and the PRESENT/ABSENT/UNRESOLVED interface untouched; frozen spec
and fixtures before code; mutation operators with declared killers;
full-corpus byte comparison; native verification; tuning-rung numbers
are never published as recall.

## H. r1 rulings and changes (founder, 2026-10-25)

Rulings: RS-D1 approved with an evidentiary clarification (conviction
rests on the recognized inbound binding, not on a token's disappearance;
witness names the consumer write line, the producer base read line, the
absence at head, and the contract binding); RS-D2 approved (RS-1, RS-2,
RS-0; RS-0's evidence is synthetic fixtures only and is never presented
as corpus coverage); RS-D3 approved (B1, B2, B3 frozen; no additions);
RS-D4 approved (r1-051 unsupported, no route association).

pdfFormat inventory (docs/evidence/request-side/r1-118-pdfformat-
inventory.tsv): the r1-118 head producer holds 21 case-insensitive
occurrences of `pdfformat` on 15 lines, all identifiers (`PdfFormats`,
`pdfFormats`, `FormDataChromiumPdfFormats`, `ErrPdfFormatNotSupported`)
or comments. None is the string literal `"pdfFormat"` and none is a B2
inbound read. The base's only inbound read is line 142,
`String("pdfFormat", &pdfFormat, "")`.

r1 changes from r0, none widening a grammar:
- RS-1 is stated as four conditions over all bound producer sources
  (producer and companion), with the exact witness form; the literal
  guard (condition 3) is kept and its reason stated.
- The string-literal tokenization, key shape, same-line value rule and
  comment rule are written down (they were implicit in the r0 counters).
- B2's JavaScript member form (`req.body.K`), present in r0's text but
  not in r0's counter, is implemented as written; the corpus gate covers
  it.
- RS-2 requires the field at base without `read_only=True` (r0's counter
  also counted a field new at head; narrower, r1-060 still qualifies).
- RS-0's `json` and `data` are whole words (r0's counter matched any
  suffix such as `metadata =`; narrower, the corpus count stays zero).
- Statement texts and reason codes are fixed; RS-1 witness lines live in
  the message (SARIF location limitation).

## I. Fixtures and pre-registered mutation operators (frozen with r1)

Fixtures: tests/rs_test.rs, 22 tests. On the baseline engine (201d193)
10 FAIL, the new behavior (rs_f01, f06, f07, f08, f08b, f11, f12, f15,
f16, f20), and 12 pass, the guards (rs_f02, f03, f04, f05, f08c, f09a,
f09b, f10, f13, f14, f17, f19). RS-0's fixtures (rs_f06, f07, f08, f08b,
f08c) are SYNTHETIC; no corpus case exercises RS-0.

| Operator | Change | Property broken | Declared killer |
|---|---|---|---|
| RS-M1 | B2 builder-word readers removed | the r1-118 read is not recognized | rs_f01 |
| RS-M2 | RS-1 condition 2 removed (head B2 reads ignored) | a surviving accessor no longer prevents conviction | rs_f02b (r2; r1 declared rs_f02, see J) |
| RS-M3 | RS-1 condition 3 removed (literal guard) | a surviving literal no longer blocks conviction | rs_f09b |
| RS-M4 | comment skip removed from B2 | a commented read becomes a binding | rs_f04 |
| RS-M5 | comment skip removed from B1 | a commented write becomes a write | rs_f05 |
| RS-M6 | B1 subscript form no longer requires `=` | a read becomes a write | rs_f03 |
| RS-M7 | RS-1 condition 4 removed (consumer write) | conviction without the consumer side | rs_f10 |
| RS-M8 | code companion head not consulted | a read moved to the companion is missed | rs_f17 |
| RS-M9 | B2 JavaScript member form removed | `req.body.K` not recognized | rs_f15 |
| RS-M10 | B1 builder-setter forms removed | `.WriteField("K"` not recognized | rs_f16 |
| RS-M11 | RS-2 convicts instead of stating | a mention alone convicts | rs_f12 |
| RS-M12 | RS-2 base requirement dropped | a field new at head counts as gaining read_only | rs_f13 |
| RS-M13 | RS-2 and RS-0 added despite residue | a statement rides on a conviction | rs_f11 |
| RS-M14 | RS-0 opaque detection removed | an opaque payload is silent | rs_f06 |
| RS-M15 | `data`/`json` whole-word requirement dropped | `metadata = x` becomes opaque | rs_f08c |
| RS-M16 | RS-1 witness omits the base read line | the evidence is incomplete | rs_f01 |

Kill rule (unchanged from G3): a kill is a failing assertion in the
fixtures, reported per operator with the declared killer's result and the
incidental kills separately; KILLED, SURVIVED, EQUIVALENT (argued in
writing) or UNAPPLIED; a build failure or harness failure is never a kill.

Stated limitation, no fixture: RS-1 does not add a K that is already in
the residue. No fixture isolates that de-duplication (it needs the same K
both convicted response-side and removed request-side), so it carries no
operator and is claimed only by code reading.


## J. r2 repair (founder, 2026-10-25, 16:18)

Finding. The r1 mutation run (docs/evidence/request-side/mutate-r1.log)
killed 15 of 16 operators by their declared killers; RS-M2 SURVIVED. It is
NOT equivalent: rs_f02 keeps the head read as `r.FormValue("pdfFormat")`,
a line that also holds the literal `"pdfFormat"`, so condition 3 blocks
conviction with or without condition 2. The two conditions coincide for
every B2 form that takes a string-literal argument, and differ for one:
the JavaScript member form `req.body.K` reads K with no literal. A
throwaway workspace whose head still reads `req.body.pdfFormat` gives HOLDS
on the r1 engine and a false VIOLATED on the RS-M2 mutant
(docs/evidence/request-side/rs-m2-survivor-probe.txt). The implementation
was right; the fixture set missed the path.

Repair, no rule or grammar change:
- Added fixture rs_f02b (tests/rs_r2_test.rs): base and head both read
  `req.body.pdfFormat`, no literal anywhere in the head, the consumer
  writes the key; must HOLD. It is RS-M2's declared killer. rs_test.rs is
  unchanged byte-for-byte; rs_f02 stays as a guard of the literal-bearing
  accessor case.
- Wording: the RS-0 statement names `<base source>:<base line>`, as RS-1's
  witness does and as frozen fixture rs_f06 requires (`base.go:6`). r1's
  text said `<producer>:<base line>`; the r1 implementation already
  followed the fixture, and in the corpus base and head are the same path,
  so no output changes.
- The mutation harness runs both fixture files with --no-fail-fast.
