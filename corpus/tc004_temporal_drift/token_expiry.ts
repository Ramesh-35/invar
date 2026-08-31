// TC-004 — TEMPORAL_DRIFT (minimal stub).
//
// Required composite leg: OpenTelemetry (runtime observation). Expected to
// resolve UNKNOWN offline.
//
// The defect: an expiry check mixes two clocks — a token minted against one
// service's wall clock is validated against another's. When the clocks drift,
// a still-valid token is rejected (or an expired one accepted). No static
// signature captures the cross-service temporal relationship.

export interface Token {
  issuedAtMs: number; // producer clock
  ttlMs: number;
}

export function isExpired(token: Token, validatorNowMs: number): boolean {
  // validatorNowMs comes from a DIFFERENT host's clock than issuedAtMs.
  return validatorNowMs - token.issuedAtMs > token.ttlMs;
}
