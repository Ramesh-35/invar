# invar.lock (R2)

`invar.lock` sits beside `invar.toml` in the consumer repository and is
committed. It records the producer fields this repository reads, each bound to
the type that declares it. It replaces `producer_base`.

## Commands

| Command | Does | Exit |
|---|---|---|
| `invar lock` | Fetch each schema contract's producer (spec and/or code) and consumer; write the fields the consumer reads (the slice). If a lock exists and its bindings would change, print the changes and stop. | 0 written or current; 3 LOCK_DRIFT (would change) or error |
| `invar lock --update` | Accept a reviewed producer change: rewrite the lock. Waivers are kept. | 0; 3 error |
| `invar check` | If `invar.lock` exists, each locked contract is checked against its slice: a bound field that no longer exists in any type of its source, or whose type changed in the same type, is residue, reported as `Type.field`. | 0 hold, 1 violated or waiver expired, 3 error, 4 skipped or not evaluated |
| `invar mcp` | Read-only MCP server on stdio. Tools: `check` (as above, JSON, skips stated) and `contract` (the lock: bindings, hashes, waivers with days left). `--offline` reads local files only. | |
| `invar fix --suggest` | For each missing bound field, a unified diff of the consumer when the field's type lost exactly that field and gained exactly one other, in every source. Otherwise a reason. Never writes; refuses when `CI` is set. | 0; 3 error |

## File

```toml
[lock]
format = "1"

[[contract]]
id = "orders"
producer = "github:acme/api@main:openapi.yaml"
pointer = "/components/schemas/Order"
blake3 = "..."                    # producer bytes when locked
producer_code = "github:acme/api@main:order.go"
blake3_code = "..."
recorded_at = "2026-10-08T14:03:11Z"           # t_r: fetched and written
bind = "spec 2026-10-08 Order.amount_cents integer"
#       source  t_v (accepted)  Type.field  type ("?" if unknown)
members = "spec Order amount_cents currency note"  # all fields of a bound type

[[waiver]]
contract = "orders"
field = "Order.amount_cents"      # or the bare field name
owner = "nikhil"
reason = "moving to amount_minor, tracked in #42"
expires = "2026-11-01"            # stops applying on this day
```

Rules:
- Deterministic: contracts by id, bindings sorted. Re-locking an unchanged
  producer rewrites nothing; an unchanged binding keeps its t_v.
- The producer ref may move between lock and check (that is what is checked);
  a different repository, path or pointer is an error until `--update`.
- Exit 1 means VIOLATED and nothing else; `lock`, `mcp` and `fix` report
  errors with 3.
- A waiver may run at most 90 days: `expires` more than 90 days after today
  is an error (exit 3) in `check` and `lock`. Waivers are written or renewed
  on the day they are committed, so the cap binds at creation and renewal.
- A waived field shows as `WAIVED (N days remaining)`. From `expires`, the
  waiver is EXPIRED and the contract fails until the waiver is renewed or
  removed. A waiver naming an unknown contract is an error; one that matches
  no residue is noted as removable.
- Reads of producer fields the lock does not bind are noted, not failed:
  run `invar lock --update` to bind them.

## Compatibility promise (format 1, frozen 2026-10-09)

- Every later INVAR release reads format 1 locks and gives the same
  bindings. A new format gets a new number; format 1 is never redefined.
- New optional keys may be added to format 1 only if older INVAR versions can
  ignore them without changing a verdict; anything else is a new format.
- `invar lock` writes the newest format; it never silently rewrites an
  existing format 1 lock to another format without `--update`.

## A1: stated blind spots (alpha.6)

With a lock, a bound field that changed but that the consumer only mentions
outside a recognised read, or a bound field that moved to another type, is
reported NOT EVALUATED with its site. A contract with no residue but such a
site exits 4 (NOT FULLY EVALUATED), never 0.

## Honest scope

The slice inherits alpha.4's readers: pattern matching, not parsing.
Consumer reads are not scoped, so a field is bound in every producer type
that declares a name the consumer reads. The lock makes a check
reproducible and reviewable; it does not raise the detection rate measured
in R1 (40% missed in-scope, batch 4).
