# JSON-1: structured verdicts for agents (AGENT-1 P1)

Status: r0, FROZEN with its fixtures before any implementation, for
founder ruling. Parent: docs/architecture/AGENT-1.md as ruled 2026-10-25
(AG-D3: P2, P1, P4, P3) and the P1 authorization of 2026-10-26 (residue
schema; abstentions as `{reason, text}`; plain-text output byte-identical;
spec and fixtures frozen first). Baseline: the S2 tree (sandbox binary
d2c449b14aa1, 537 of 537), which lands on main before P1. Fixtures:
tests/json_test.rs; hashes in JSON-SPEC.frozen. Measurement:
scripts/json/measure.py over the full-corpus capture of that binary.

## A. What JSON-1 is, and what it is not

An agent calling the MCP `check` tool today receives each conviction as
strings (`"read_at": ["c.go:4"]`, `"missing_from": "<prose>"`) and each
NOT EVALUATED statement as one string with its reason code inside
brackets, when it has one. To act, it must parse INVAR's prose. JSON-1
adds the same facts as fields, so an agent branches on codes and reads
locations as data.

JSON-1 is additive. Every key the `check` result carries today keeps its
name and value. Nothing else changes:
- the text output of `invar check` (byte-identical, measured on the
  full corpus);
- receipts (format 1, a signed canonical body; a schema change there is
  a new receipt format and its own spec);
- SARIF, `coverage --format json`, `dependents --format json`;
- every verdict and exit code.

`invar check` has no JSON output today; the agent surface is the MCP
`check` tool, and JSON-1 is defined there. A `--format json` for the CLI,
built on the same function, is a later, separate decision.

## B. Schema

Result (the tool's structuredContent):
- X-1 `exit_code`: the exit code `invar check` returns for the same
  outcomes: 1 when any contract is VIOLATED, else 4 when any is skipped or
  not fully evaluated, else 0 (the existing `contracts_exit` rule).

Per contract:
- A-1 `not_evaluated_items`: one object `{reason, text}` per entry of
  `not_evaluated`, in the same order. `text` is that entry, verbatim
  (its `[not-evaluated:CODE]` tag included when it has one). Present on
  every contract, VIOLATED ones included (a statement can ride beside a
  conviction).
- S-1 `skip`: `{reason, text}` when `skipped_reason` is set, `text` being
  that string verbatim; otherwise null.

Per residue item:
- R-1 `rule`, exactly one of:
  - `lock-order`: the contract's kind is `lock_order`;
  - `request-side`: `missing_from` has the RS-1 form frozen in
    REQUEST-SIDE-SPEC and AUDIT-SPEC AU-3 (`<head> at head: no inbound read
    and no literal of K remain (inbound read at B:L at base); written at
    N:L1[, L2...]`);
  - `absent`: `missing_from` is set (the field is not in a bound producer
    source; a lock-mode item absent from one source and retyped in another
    is `absent`, its `type_change` still carried);
  - `retyped`: only `type_change` is set.
- R-2 `consumer_sites`: `{file, line, access}` objects. One per `read_at`
  entry, split at its last `:` (the engine writes `name:line`), `access`
  `"read"` (`"trace"` for `lock-order`, whose sites are trace lines). For
  `request-side`, one per written line of `written at N:L1, L2`, `access`
  `"write"`. `read_at` keeps its existing elision-free list; so does this.
- R-3 `producers`: the bound producer artifacts the witness places the
  field in. P is the artifact names of the contract's `producer`,
  `producer_base`, `producer_code` and `producer_code_base` keys. Each
  place (the part of `missing_from` before its first ` · `, split on
  `, `; and the `in` place of `type_change`) contributes the longest name
  in P that it equals or starts with, followed by a space or `:`. In order,
  without duplicates; `[]` when no place names a bound producer (never a
  guess).
- R-4 `candidate_rename`: `{key, type}` when `missing_from` carries the
  frozen G2-R form `· candidate rename: KEY (same scope, same type TYPE),
  not evaluated`; otherwise null. Still a candidate, never evaluated.

Reason codes (A-1, S-1), the first rule that applies:
1. The text's trailing `[not-evaluated:CODE]` tag: CODE. Every tagged
   statement keeps the code it already shows.
2. The untagged statement forms, enumerated (each is a fixed engine
   sentence, matched on its constant part):
   - lock-mode A1 rule 1 (`... changed; mentioned at ..., not evaluated
     (pattern not read)`): `pattern-not-read`, the code the change-scoped
     form of the same rule already carries;
   - lock-mode A1 rule 2 (`... moved to ...; consumer read at ... not
     evaluated`): `field-moved`, likewise;
   - L1 (`the lock binds no field of this contract (0 bindings), ...`):
     `no-bindings`.
3. The untagged skip forms, enumerated:
   `language not supported (`: `unsupported-language`;
   `not evaluated: no invar.lock entry`: `no-lock-entry`;
   `not evaluated: list ... is no longer statically extractable`:
   `list-not-extractable`; `not evaluated: ... is no longer statically
   extractable` (a shape): `shape-not-extractable`;
   `not evaluated: no shape ... found`: `shape-not-found`;
   `not evaluated: list ... no longer found`: `list-not-found`;
   `not evaluated: inline list ... no longer contains any locked name`:
   `list-unrecognizable`.
4. Otherwise `unclassified`: stated, never silent, and a defect to fix
   (every fixture asserts no item is `unclassified`).

The engine matches its own fixed sentences once, inside one function,
pinned by fixtures; the agent never parses prose. Section H (JSON-D1)
states the alternative.

## C. Measured before code

Full-corpus capture of the baseline binary, 122 cases
(scripts/json/measure.py, each case's bound producers read from its own
invar.toml):

| Quantity | Count |
|---|---|
| contracts: VIOLATED / NOT EVALUATED / HOLDS / SKIPPED | 65 / 18 / 33 / 6 |
| witnesses | 127 |
| rule: absent / retyped / request-side / lock-order | 108 / 18 / 1 / 0 |
| read sites shown, `name:line` form | 240 of 240 (25 elided as `+N more` in the text only; JSON lists all) |
| request-side write lines | 3 (r1-118: 329, 331, 333) |
| producers named by R-3 | 127 of 127 |
| candidate renames | 3 (r1-010, r1-070, r1-076) |
| NOT EVALUATED statements, tagged | 96 of 96 (53 of them beside a conviction) |
| skip reasons | 6, all `language not supported (`, untagged |

The untagged statement forms (rule 2) occur only in lock mode; the corpus
runs change-scoped, so they are measured by fixtures, not by the corpus.

## D. Pre-registered movement

- CLI text: byte-identical on all 122 corpus cases and in every existing
  fixture. Any text movement stops the phase.
- MCP `check` JSON: every existing key unchanged in value; only the keys of
  section B are added.
- Receipts, SARIF, coverage and dependents JSON: byte-identical.
- Verdicts and exit codes: unchanged. No recall claim; P1 adds no rule.

## E. Fixtures (frozen with r0)

tests/json_test.rs, 14 tests, each driving the real `invar mcp` server
over stdio. Every fixture also runs `consistent`: one item per
`not_evaluated` string, its text verbatim, its reason equal to the
string's own tag when it has one, never `unclassified`; a skip likewise.

| Fixture | Asserts |
|---|---|
| j_f01 | a G2 removal: existing keys unchanged; `rule` absent, read site `c.go:4`, `producers` [head.go], `candidate_rename` {forge_url, string} |
| j_f02 | a retype: `rule` retyped, `producers` from the type-change place, no candidate |
| j_f03 | RS-1: `rule` request-side, `read_at` empty, the write line `c.py:3` as `access` write |
| j_f04 | lock_order: `rule` lock-order, two trace sites, `producers` [] |
| j_f05 | lock mode: a removed bound field is `absent` in `lib/limits.ts` |
| j_f06 | a tagged statement: `{reason: pattern-not-read, text: <the string>}` |
| j_f07 | the untagged lock-mode A1 rule 1 form: `pattern-not-read` |
| j_f08 | the untagged lock-mode A1 rule 2 form: `field-moved` |
| j_f09 | 0 bindings: `no-bindings` |
| j_f10 | an unsupported-language skip: `skip` {unsupported-language, text} |
| j_f11 | an unlocked egress contract: `skip` {no-lock-entry, text} |
| j_f12 | `exit_code` 1, 4, 0, matching `result` and the CLI |
| j_f13 | a conviction with a statement beside it carries both |
| j_f14 | guard: the CLI text of the j_f01 workspace, as the baseline prints it |

On the baseline: 13 FAIL, each at a JSON-1 assertion (the new keys do not
exist), and j_f14 passes. With every JSON-1 assertion removed, all 14
pass on the baseline: every premise (statuses, existing keys, the
untagged forms, binding counts) is the engine's real behavior, measured
(docs/evidence/json/premises-baseline.log).

## F. Pre-registered mutation operators

| Operator | Change | Declared killer |
|---|---|---|
| J-M1 | the RS-1 form not recognized (request-side reads as absent) | j_f03 |
| J-M2 | kind `lock_order` not recognized | j_f04 |
| J-M3 | `type_change` ignored by R-1 (retyped reads as absent) | j_f02 |
| J-M4 | request-side write lines omitted | j_f03 |
| J-M5 | `access` always `read` | j_f04 |
| J-M6 | R-3 matches no bound name | j_f01 |
| J-M7 | the G2-R candidate not parsed | j_f01 |
| J-M8 | the trailing tag not read | j_f06 |
| J-M9 | the lock-mode A1 rule 1 form unclassified | j_f07 |
| J-M10 | the lock-mode A1 rule 2 form unclassified | j_f08 |
| J-M11 | the L1 form unclassified | j_f09 |
| J-M12 | the language skip unclassified | j_f10 |
| J-M13 | the no-lock-entry skip unclassified | j_f11 |
| J-M14 | `exit_code` ignores stated contracts (4 becomes 0) | j_f12 |
| J-M15 | `text` stripped of its tag | j_f06 |
| J-M16 | statements beside a conviction dropped | j_f13 |

Kill rule unchanged: KILLED, SURVIVED, EQUIVALENT (argued in writing) or
UNAPPLIED; a build or harness failure is never a kill; --no-fail-fast.

## G. Unsupported, stated

- Observed-sample findings (`check --sample`) are CLI-only; the MCP
  `check` tool does not run samples, so they never reach JSON-1.
- Skip reasons other than `unsupported-language` and `no-lock-entry` are
  enumerated in B but not fixtured (they need locked egress and strings
  contracts made unextractable); their mapping is code review only.
- A future statement added without a tag and without an entry in B maps to
  `unclassified`. No fixture can see a sentence that does not exist yet;
  the rule for new statements is to carry a tag (JSON-D2).
- R-3 names producers; it does not give producer line numbers.

## H. Decision points

- JSON-D1 (how the fields are built). (a) Derived in the JSON layer from
  the engine's own fixed output forms, as specified (recommended: zero
  changes to the Witness and ContractOutcome structs, the text renderer,
  receipts, or any frozen fixture; measured exact on the corpus; every
  form pinned by a fixture). (b) Carried as typed fields set where each
  witness and statement is built: no matching of sentences at all, but
  the structs change, so seven struct literals in four frozen fixture
  files (audit_test, audit_r1_test, harness_test, security_s2_test) need a
  mechanical amendment with no assertion change.
- JSON-D2 (new statements). From JSON-1 on, every new NOT EVALUATED or
  skip sentence carries a `[not-evaluated:CODE]` tag in its text
  (recommended: new text, so no byte-identity is broken), and the existing
  untagged forms stay as they are until a ruling changes their text.
- JSON-D3 (scope). The MCP `check` tool only (recommended), or also a
  `invar check --format json` CLI output built on the same function.

## I. Bounds

No verdict change; exit codes and the PRESENT/ABSENT/UNRESOLVED interface
untouched; CLI text byte-identical; receipts unchanged; additive JSON
only; zero new dependencies; frozen spec and fixtures before code;
mutation operators with declared killers; full-corpus byte comparison;
native verification.
