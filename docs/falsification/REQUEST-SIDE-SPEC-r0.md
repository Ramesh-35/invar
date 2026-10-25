# REQUEST-SIDE: what the consumer sends against what the producer accepts (RECALL-ATTACK-1 phase 2)

Status: DRAFT r0, evidence, rules and decision points, for founder ruling.
No fixtures, no implementation. Parent: R-D3 (specification authorized,
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
key yields no claim of any kind from B1 alone.

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
- JavaScript and TypeScript: `req.body`, `req.query`, `req.params` with a
  literal member or subscript.

Request structs and DTOs (Rust, Go, TypeScript classes) are already read
by the existing field extraction and are not re-specified here.

### B3. Read-only declaration (DRF)

A serializer field line `name = SomeSerializer(...)` whose arguments
contain `read_only=True`.

## C. Rules

RS-1, inbound key removed. The producer base has an inbound read of K
(B2); K occurs nowhere as a string literal in the head producer file; the
consumer writes K (B1).
- Outcome: VIOLATED, with the witness `K written at <consumer:line>;
  read at <producer> base, no longer read anywhere in <producer> head`.
- Evidentiary standard: the same as the existing response-side rule,
  which convicts a read field missing from the bound producer file.
- Measured, all 122 cases: fires on exactly one, r1-118, on exactly
  `pdfFormat` (its expected residue). Zero controls, zero tuning cases.

RS-2, became read-only. A serializer field gains `read_only=True`
between base and head (B3), and the consumer mentions its name.
- Outcome: stated NOT EVALUATED (exit 4), never VIOLATED, because the
  consumer's mention could be a read, which a read-only field still
  serves.
- Measured: fires on exactly one, r1-060, on exactly `device_role`.

RS-0, fail closed. The producer base has an inbound read of K that is
gone from the head file (RS-1's producer half), the consumer has no
literal write of K, and its payload is built opaquely (a variable
subscript `x[k] =`, `**kwargs`, a spread, `json=payload`, `data=payload`,
`SetBody(x)`).
- Outcome: stated NOT EVALUATED, because the consumer may send K without
  naming it.
- Measured: fires on zero corpus cases. The boundary is real but
  untested by the corpus, so its evidence will be synthetic fixtures
  only, and the spec says so.

A statement never overrides a conviction, and nothing is added to a
contract that already has residue (the F1 discipline, unchanged).

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
