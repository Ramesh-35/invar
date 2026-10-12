# W3: constant-value witnesses (DRAFT, awaiting ratification; NOT built)

The five-time-recurring default-switch class is now measured in scope
and missed: the r1-113 Cargo.lock v3->v4 transition is shape-identical
(same keys at every level, verified on real pinned payloads), so shape
witnesses cannot see it; the selector is a VALUE. Catching it needs a
verdict-semantics extension, which only the founder ratifies.

## The mechanism (proposed)

Per locked path whose value is a short scalar (integer, boolean, or
string of at most 32 characters), the sample accumulates one more
witness: whether the value has been IDENTICAL across every observation.
Values are never stored (the frozen never-values privacy rule stands);
the lock stores only a salted fingerprint: 16 hex of
blake3(contract || path || canonical JSON value). Identical values
share a fingerprint; a changed value changes it; nothing about the
value is recoverable from the lock.

Drift, for a fresh payload: a path whose value was constant across
observations >= 3 and whose fresh value's fingerprint differs is
reported as `constant value changed` with the path (the values
themselves are not printed from the lock side; the fresh side MAY be
printed, founder's call, since it is already in the user's own file).
First distinct value during locking permanently marks the path
free-valued; free-valued paths never accuse.

## Open points the ratification must decide

1. Accused or stated: VIOLATED residue (waivers apply), or
   [not-evaluated:value-switch] stated-only for one release before
   promotion. The cautious path is stated-first, measured on the gate
   trio (r1-059 deno version, r1-075 lockfileVersion, r1-113 version),
   promoted only if clean E1-style probes show zero false accusations
   on value-constant paths across clean producer changes.
2. Lock format bumps to 4 (older engines must fail closed on the new
   per-field token, per the fail-closed doctrine; engine reads 1..4
   forever, writes lowest-expressing).
3. The 32-character scalar bound and the N>=3 threshold (mirrors
   witness-count-low).

## Prediction, recorded before any build

All three gate selectors (deno "version", npm "lockfileVersion", cargo
"version") were value-constant across their base observations and
changed at head, so W3 as specified would state or accuse all three.
That prediction is falsifiable and will be measured, not assumed.
