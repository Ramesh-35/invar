# The "CUDA Thesis" (Strategic Positioning): RETIRED 2026-10-08

> **RETIRED.** Founder directive 2026-10-08, on the October 2026 research
> report: CUDA's moat came from hardware margins, closed tuned libraries,
> compatibility guarantees and an installed base. A contract checker over open
> formats has none of these. The thesis does not hold and must not be cited.
> Kept below unchanged as history. Current strategy: docs/architecture/strategy/DECLARATION-GRAPH.md, whose Substrate doctrine section (2026-10-12) explicitly supersedes this retirement with a narrower, different mechanism.

> **Classification: `[STRATEGIC HYPOTHESIS]`.** This entire document is a
> positioning bet about *how INVAR could matter if the category residue is real*.
> None of it is technical evidence. It must not be cited as proof of anything.

## The analogy

`[STRATEGIC HYPOTHESIS]` Just as CUDA turned the GPU from a fixed-function
graphics device into a general substrate for parallel compute — creating a
category rather than competing in an existing one — INVAR could turn scattered,
single-scope checks (static, policy, runtime, property) into a single
programmable **invariant substrate**. The bet is that the value is in the
*substrate and its lock-in*, not in any individual check.

## Claims that are NOT established

- `[UNVERIFIED]` That existing tooling has "**exactly five fundamental flaws**".
  The number five is rhetorical. There is no enumeration here that has been
  tested to be exhaustive, mutually exclusive, or fundamental. Treat any "five
  flaws" framing as marketing scaffolding, not analysis.
- `[UNVERIFIED]` Every *specific* technical claim implied by the analogy — that
  invariants compose like CUDA kernels, that there is a natural "instruction
  set" for invariants, that adoption follows a developer-platform flywheel.
  Each is a separate hypothesis requiring its own evidence.
- `[UNVERIFIED]` Any performance or scale figure attached to the analogy. No
  benchmark exists.

## A required correction carried into strategy

`[PROVEN]` (definitional) **HKDF is a key derivation function, not a
zero-knowledge proof system.** Any strategic narrative that describes HKDF as
providing zero-knowledge proofs is wrong on the facts. HKDF derives keys from
input keying material; it proves nothing in zero knowledge. Cryptographic
positioning claims must pass through a threat model first (DR-006).

## How to use this document

Use it to align *positioning and sequencing*, never to justify a technical
assertion. If the falsification instrument fails to find a durable residue, the
CUDA analogy collapses with it — there is no substrate to platform-ize if there
is no category to substrate.
