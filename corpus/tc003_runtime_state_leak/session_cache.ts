// TC-003 — RUNTIME_STATE_LEAK (minimal stub).
//
// Required composite leg: OpenTelemetry (runtime observation). Expected to
// resolve UNKNOWN offline.
//
// The defect: request-scoped state is stored in a module-level singleton, so
// one request's user context can leak into another request under concurrency
// or reuse. Purely static analysis sees an ordinary module variable.

interface UserContext {
  userId: string;
}

// Module-level (process-wide) — should have been request-scoped.
let currentUser: UserContext | null = null;

export function beginRequest(user: UserContext): void {
  currentUser = user;
}

export function whoAmI(): string {
  // Under interleaved requests this can return a different request's user.
  return currentUser?.userId ?? "anonymous";
}
