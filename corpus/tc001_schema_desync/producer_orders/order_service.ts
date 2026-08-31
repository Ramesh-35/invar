// TC-001 producer — "order-service" (imagine: repository A).
//
// This service USED TO emit `tax_amount_cents` on every order. A refactor
// replaced that flat field with a structured `tax_breakdown`. The change is
// locally correct, fully typed, and compiles cleanly. Nothing here is wrong in
// isolation — the failure only exists ACROSS the repository boundary.

export interface TaxBreakdown {
  rate_bps: number;
  tax_cents: number;
}

export interface Order {
  id: string;
  subtotal_cents: number;
  // BREAKING (cross-repo): `tax_amount_cents` was removed in favour of this.
  tax_breakdown: TaxBreakdown;
}

export function buildOrder(): Order {
  const subtotal_cents = 5000;
  const tax_cents = 400; // 8% of 5000
  return {
    id: "ord_1001",
    subtotal_cents,
    tax_breakdown: { rate_bps: 800, tax_cents },
  };
}
