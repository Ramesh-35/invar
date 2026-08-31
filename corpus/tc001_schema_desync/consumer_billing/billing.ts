// TC-001 consumer — "billing" (imagine: repository B).
//
// This service still reads the REMOVED `tax_amount_cents` field. Because it
// reads an optional field with a `|| 0` fallback, the code compiles, runs, and
// returns HTTP 200 — while silently under-billing by the tax amount. There is
// no exception, no type error, no crash: the invariant "invoice total includes
// tax" is violated purely by a schema drift the compiler cannot see.

interface LegacyOrder {
  id: string;
  subtotal_cents: number;
  // No longer emitted by the producer (see ../producer_orders/order_service.ts).
  tax_amount_cents?: number;
}

export function invoiceTotalCents(order: LegacyOrder): number {
  const tax = order.tax_amount_cents || 0; // silently 0 after the producer refactor
  return order.subtotal_cents + tax; // 5000 instead of the correct 5400
}

// Expected: 5400 (5000 subtotal + 400 tax). Actual: 5000. Silent under-billing.
