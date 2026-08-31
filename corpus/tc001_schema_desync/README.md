# TC-001 — AST_SCHEMA_DESYNC (cross-repository)

**Evidence tier:** FIXTURE. **catch_signature:** none.

A producer (`producer_orders/order_service.ts`) removed the flat
`tax_amount_cents` field in favour of a structured `tax_breakdown`. A consumer
in a different repository (`consumer_billing/billing.ts`) still reads
`tax_amount_cents`, falling back to `|| 0`. Both files are individually valid,
typed, and compile. The result is silent under-billing: `5000` instead of
`5400`, HTTP 200, no exception.

**Why the composite is expected to struggle (a HYPOTHESIS, not a proof):** the
defect lives in the *relationship* between two separately-valid ASTs across a
repository boundary. A single-repo static rule sees no local violation. There is
no local `catch_signature`, so if a Semgrep advisory happens to fire it must be
treated as an advisory, **not** as isolation of this failure.

This case exists to be attacked. If a fully-executed composite catches it, the
correct outcome is `CAUGHT_BY_COMPOSITE`.
