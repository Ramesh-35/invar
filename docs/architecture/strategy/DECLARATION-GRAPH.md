# The declaration graph (strategy, LOCKED by founder 2026-10-12)

Two assumptions were attacked and the inversions ratified:

1. The pain is coordination, not only detection. The consumer who
   checks can only react; the producer who breaks has no incentive to
   run a checker. Every tool in this category dies on that asymmetry.
2. The primary reader of verdicts is shifting from humans to
   autonomous coding agents, which are confident, probabilistic and
   unverifiable; the scarcest resource of the agentic era is
   deterministic, evidence-grade ground truth an agent can cite.
   INVAR's frozen exit codes, stated blind spots, byte-reproducible
   receipts and MCP server are exactly that instrument.

## The locked thesis

Checks do not compound; declarations do. invar.lock is a
machine-readable statement, "this repository reads these exact fields
from that repository," committed, hashed, dated. Made public and
aggregable, it inverts the incentive:

- Producers (and their agents) ask "who reads the field I am about to
  remove?" and get a deterministic, receipted answer. Nobody breaks
  consumers out of malice, only blindness; INVAR sells sight at the
  moment of guilt. No tool offers field-level dependents; platform
  dependency graphs stop at the package.
- Consumers publish locks as self-defense: a published lock is how
  your producers see you before they break you. Declaring protects
  you, protection requires declaring; the graph grows itself.
- Agents adopt certainty, not tools: blast-radius answers with
  receipts are what their operators demand.

One sentence: INVAR makes invisible field dependencies into a public,
verifiable graph; checking is merely what you do with the graph
locally.

Zen constraint: minimal machinery. Locks already live in public
repos; discovery starts as code search; the first index is a
nightly-built public JSONL in a repo, zero servers, local-first.

## Survival against platform absorption (stated without comfort)

If GitHub ships native field-level dependent-break detection on its
private repo graph, it wins distribution overnight; no feature race
survives that. Tools survive platforms one way: become the format the
platform ingests, not the feature it rebuilds (SARIF and SPDX are the
precedents, in GitHub's own UI).

Priority order:
1. Become the format: the lock (field-dependency declaration) and the
   receipt (reproducible evidence) as open, specified, anchored
   formats. Nearest move: emit SARIF from `invar check` so verdicts
   appear natively in GitHub code scanning, riding the platform.
2. Own what platforms structurally will not: offline, air-gapped,
   cross-platform neutrality, auditor-grade vendor-neutral evidence.
3. Compound the reputation asset: sealed benchmark, published miss
   rate, one-command reproduction; a feature clones in a quarter, a
   record does not.
4. The honest floor, on the record: if a platform does all of it
   including the open formats and the evidence discipline,
   INVAR-the-company loses and INVAR-the-standard still wins its
   purpose. We build knowing the floor exists.

## Build sequence (ratified)

1. SARIF export from `invar check` (uses existing outcomes; no
   verdict semantics change).
2. DEPENDENTS-GRAPH spec: extending `invar dependents` from local
   directories toward published locks (frozen spec before any build,
   founder ratifies).
3. The public lock index as a nightly JSONL artifact (after the spec).

## Substrate doctrine (founder, 2026-10-12: first-class strategy)

This section REVIVES the substrate framing that CUDA-THESIS.md retired
on 2026-10-08, as an explicit supersession, not a silent flip. The
retirement held that CUDA's moat (hardware margins, closed tuned
libraries, compatibility guarantees, installed base) does not map to a
contract checker. The revived form claims only the two that do map:
compatibility guarantees (our frozen formats: exit codes never
re-meaning, locks reading 1..4 forever, receipts byte-reproducible)
and an installed base (the declaration graph: every published lock and
every agent integration compounds). Margins and closed libraries are
conceded as never applicable. CUDA-THESIS.md stays retired as written;
this doctrine is its successor with a different mechanism.

The doctrine: do not build to be integrated or acquired; build the
layer other things are unsafe without. Platforms own the UI; INVAR
owns the deterministic verification kernel and its evidence ledger.
The lock is the intermediate representation agents compile their
changes against; the receipt is the proof object that travels across
org and platform boundaries; SARIF and MCP are the sockets platforms
and agents plug into.

Language discipline for this doctrine, binding (overclaim corrections
applied to the founder's framing before adoption):
- Never "zero-hallucination" or "proves safety": INVAR deterministically
  verifies conservation of DECLARED fields and STATES what it did not
  evaluate; the held-out record (38 of 70) is part of every claim.
- "The only engine that provides it" is a positioning hypothesis,
  unverified; it is argued from the record, never asserted as fact.
- Format scope is what ships: JSON and a stated TOML subset for
  payloads, the supported languages for static surfaces; Protobuf and
  OpenAPI enter when evidence demands, not when the analogy does.

Technical priorities under the doctrine (ordered):
1. The kernel stays instant and zero-overhead: engine wall times
   remain measured and published per release, never promised.
2. First-class agent interface: the MCP server grows from read-only
   check/contract toward dependents and coverage, so an agent can ask
   blast-radius and blind-spot questions mid-edit with receipts.
3. The lock as protocol: INVAR-LOCK.md graduates toward a standalone
   versioned specification others can implement; the DEPENDENTS-GRAPH
   spec (step 2 of the locked sequence) is its first cross-repo use.
