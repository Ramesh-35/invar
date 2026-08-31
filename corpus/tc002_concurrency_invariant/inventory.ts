// TC-002 — CONCURRENCY_INVARIANT (minimal stub).
//
// Required composite leg: OpenTelemetry (runtime observation). Offline there is
// no workload to observe, so this case is expected to resolve UNKNOWN, not
// residue — we cannot claim the composite failed to catch something we never
// let it run against.
//
// The defect: a non-atomic check-then-act on shared stock. Under concurrency,
// two callers can both pass the `stock >= qty` check and both decrement,
// oversation the inventory below zero. Statically this looks fine.

let stock = 1;

export async function reserve(qty: number): Promise<boolean> {
  if (stock >= qty) {
    await Promise.resolve(); // yield point: another task can interleave here
    stock -= qty; // may drive stock negative under interleaving
    return true;
  }
  return false;
}

export function currentStock(): number {
  return stock;
}
