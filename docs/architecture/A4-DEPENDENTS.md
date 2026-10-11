# A4: field-level dependents graph (spec, ratified Option A 2026-10-10)

The moat claim this serves (MOAT.md, A4): committed locks state who reads
which field, so a producer can ask "who breaks if I remove this?" before
removing it, from real declarations, not from a crawler or a registry.

## Scope (Option A, local-first)

`invar dependents <QUERY> --locks <dir>` answers from invar.lock files on
local disk only. The caller aggregates the lockfiles however they like (a
monorepo, a directory of clones, a tarball of locks from teammates). INVAR
reads them; it never fetches them.

- Zero network: the command performs no HTTP or git calls, has no resolver,
  and takes no token. This is a guarantee, not a default.
- Discovery: every file named `invar.lock` under `--locks <dir>`, any
  depth. A consumer's identity is its lockfile path relative to that
  directory (the one thing the caller controls completely).
- A lockfile that does not parse is reported (stderr and the output's
  parse_errors); it never silently vanishes from the answer. An answer with
  unreadable lockfiles exits 4 (answered, with stated blind spots); a
  directory yielding no parseable lockfile at all is an error (exit 3).

## Query semantics

QUERY is `Type.field` or a bare `field`:

- `Type.field`: bindings whose scope and field both match by naming form
  (the engine's rule since alpha.4: case and `_`/`-` are insignificant, so
  `BatteryState.power_now` matches `batterystate.powerNow`).
- bare `field`: bindings with a matching field in any scope. Spec producers
  bind fields without a type scope, so their bindings only match bare-field
  queries; when a scoped query finds nothing but unscoped bindings match
  the field part, the answer says so instead of showing a bare zero.
- `--producer <substr>`: additionally require the contract's producer ref
  to contain the substring (plain substring, case-sensitive, as written in
  the lock).

One row per (lockfile, contract, binding) match: consumer (relative path),
contract id, producer ref, bound `Type.field` as displayed, recorded type,
t_v (since). Rows are sorted (consumer, contract id, binding display) so
output is deterministic for identical inputs.

## Output

- Default: a plain table, one row per dependent, then
  `N dependent binding(s) in M of K lockfile(s)`.
- `--format json`: canonical JSON (sorted keys), fields `query`,
  `producer_filter`, `dependents` (the rows), `lockfiles_scanned`,
  `lockfiles_matched`, `parse_errors` (paths only). No timestamps, no
  absolute paths: output for identical inputs is byte-identical.
- Zero matches is an ANSWER, exit 0 with an empty list; the caller decides
  what it means. Unreadable lockfiles alongside an answer: exit 4. Errors
  (unreadable directory, no parseable lockfiles, bad flags) exit 3, R2
  convention. This command never exits 1: it states who depends on a field;
  it holds no verdict.

## Gate (from MOAT.md, unchanged)

A producer uses the answer to change or delay a removal in a real
repository. Until then A4 is plumbing, recorded as such.

## Non-goals

No crawling, no registry, no server, no transitive closure (a lock binds a
consumer to a producer; chains are a later question), no writing, no
network. Option B (fetching consumer locks by `github:` ref) is a separate
future ratification.
