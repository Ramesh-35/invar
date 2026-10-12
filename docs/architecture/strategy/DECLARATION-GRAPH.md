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
