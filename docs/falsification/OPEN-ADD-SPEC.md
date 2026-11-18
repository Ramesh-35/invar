# OPEN-ADD r0: a property added to an open object

Ruled by the founder on 16 November 2026. Amends JUDGE-REACH Amendment A, test r_f14.

## Direction

`invar judge` checks one compatibility direction: every instance valid under the base contract must stay valid under the head. Nothing else is claimed.

## The rule

A property the head declares under `properties` that the base did not declare, on an object node:

| Base object at that node | Result |
|---|---|
| Closed (`additionalProperties: false` or `unevaluatedProperties: false`), no `patternProperties` | Widening. Not a witness (unchanged from JUDGE-REACH). |
| Has `patternProperties` | Unchecked, NOT EVALUATED (unchanged): a pattern may already constrain the key. |
| Open, and the added schema is `true`, `{}` or annotations only (`title`, `description`, `$comment`, `examples`, `default`, `deprecated`, `readOnly`, `writeOnly`) | Every value is still accepted. Not a witness. |
| Open, anything else | Candidate break, kind `added_constrained`. |

A candidate is VIOLATED only with a confirmed counterexample: COUNTEREXAMPLE tries one value of each JSON type at that key in a minimal base instance, and prints one only when its checker says the base accepts it (including the base's `additionalProperties` schema, when one is given) and the head rejects it. If no candidate is confirmed, or the checker meets a keyword it does not read, the path is listed in `undecided_paths`, with the reason in `undecided_changes`, and the verdict is NOT EVALUATED (`undecided_constraint`). It is never HOLDS by default and never VIOLATED without a counterexample.

## r1: compatibility scope (ruled 17 November 2026)

The verdict stays VIOLATED. A formally valid counterexample is not, by itself, evidence that any real caller breaks, so every result now says which part of the base's accepted instances each witness breaks, and counts the two kinds apart. These fields are added; no existing field, value or exit code changes.

Per witness:

| Field | Values |
|---|---|
| `compatibility_scope` | `DECLARED_INPUT`: an instance using only what the base declared (or omitting a key it now requires) is rejected. `UNDECLARED_PROPERTY_SPACE`: only an instance that sent a key the base never declared, accepted because the object was open, is rejected (kind `added_constrained`). |
| `witness_basis` | `OLD_ACCEPTS_NEW_REJECTS` when the counterexample is printed (valid at base, rejected at head); `SCHEMA_CHANGE` when the witness stands on the schema difference alone. |
| `previously_declared` | whether the witness path is declared in the base contract (`true`/`false`). A newly required property that was never declared is `DECLARED_INPUT` with `previously_declared: false`: instances that omit it were valid. |
| `observed_caller_usage` | always `UNKNOWN`: the judge reads schemas, never callers. |
| `impact_assessment` | always `NOT_ESTABLISHED`, for the same reason. |

Per result, `findings`: `declared_usage_violations` and `undeclared_property_restrictions` (witness counts) and `category`: `DECLARED_USAGE_VIOLATION` if any witness is `DECLARED_INPUT`, else `UNDECLARED_PROPERTY_RESTRICTION` if any is `UNDECLARED_PROPERTY_SPACE`, else `null`. A `--contracts` summary counts contracts by category. Text output adds one line under the VIOLATED headline when any witness is undeclared-only: `declared usage: N; undeclared properties only: M (... whether any caller does is not observed)`.

There is no severity score. Formal correctness, observed usage and operational impact are separate dimensions; only the first is computed.

The receipt `result` carries the same fields, so it remains exactly the `--format json` document. Receipts reproduce only under the engine version that wrote them, as before.

## What it does not claim

- It does not generalise to every optional-property addition. A closed base, a base pattern, an assertion-free added schema, and an added schema that accepts every value tried are all handled differently, as above.
- When the base's `additionalProperties` already restricts values to what the head declares, no break exists, but the judge reports NOT EVALUATED (no counterexample found), not HOLDS. Proving absence is not attempted.
- Dialect: COUNTEREXAMPLE's checker is the authority on validity; constructs it does not read make the result NOT EVALUATED.

## Evidence at ruling time

- 25 adversarial cases (`scripts/openadd/adversarial.py`; run `python3 scripts/openadd/adversarial.py target/release/invar`, needs `pip install jsonschema`): every printed counterexample independently confirmed with python-jsonschema 4.26 under the dialect named by `$schema` (valid under base, invalid under head); 0 failures.
- Rust tests: `r_f14_open_object_gains_constrained_property` (amended) and `oa_added_property_by_openness` (10 cases) in `judge_reach_test.rs`.

- r1 corpus gate: the 220 committed `inputSchema` changes in github/github-mcp-server at `67d4dc4`: 125 HOLDS, 12 NOT EVALUATED, 83 VIOLATED, of which 16 `DECLARED_USAGE_VIOLATION` and 67 `UNDECLARED_PROPERTY_RESTRICTION`; 121 printed counterexamples, all valid at base and rejected at head by python-jsonschema; every earlier key and value unchanged; exit codes unchanged.

## Amendment record

r_f14 before this ruling: exit 4, `unchecked: ["/properties/k"]`. After: exit 1, VIOLATED, witness kind `added_constrained` with a counterexample, `unchecked: []`. RE-M6 (an added property silently covered as a pass) stays killed.
