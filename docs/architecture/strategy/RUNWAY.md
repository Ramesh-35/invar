# The Runway Strategy: Own the Local Verification Loop

Status: ACCEPTED 2026-10-12 (founder dispatch of the same day, accepted
with the corrections of record below; the corrections are binding).
Read with strategy/DECLARATION-GRAPH.md (which this complements and does
not supersede) and docs/COMPATIBILITY-COVENANT.md (ratified the same day).

## Position in the strategy stack

The locked survival doctrine (become the format, neutrality, the honest
record) is defense: it makes INVAR hard to remove. This document is
offense: it makes INVAR the thing people and agents build on. CUDA needed
both: an unforgiving, fast local execution engine AND twenty years of
kept compatibility promises. One without the other does not compound.
Nothing here demotes SARIF, the lock formats, or the record; SARIF was
always a transport, shipped because it was small, and it remains exactly
that.

## The runway

A human or an AI agent edits code. The engine returns a deterministic
verdict with a byte-verifiable receipt, fast enough to live inside the
loop rather than at its end. The receipt feeds a git hook, an agent's
context window, or a CI gate. That loop, local, instant, deterministic,
is the product. Everything public (index, attestations, corpus) is the
compounding record of that loop running.

## Move 1 (next build): agent-native MCP surface

AI coding agents make probabilistic edits across schema and field
boundaries and cannot know from reasoning alone whether an edit breaks an
unstated field expectation. INVAR's receipt is deterministic, citable,
line-addressed evidence an agent can consume and act on.

What we build: first-class MCP tools over the existing engine (check,
dependents, coverage), with stdout/JSON contracts treated as a primary
interface for agent runtimes, under the frozen exit codes.

Binding corrections to the dispatch language:
- INVAR does not stop hallucination and we never say it does. It gives an
  agent deterministic evidence about declared field dependencies, with
  the published miss rate attached. The covenant's promise 6 applies.
- "Agents will refuse structural refactors without INVAR" is a labeled
  hypothesis about the future, not a fact. The falsifier: if agent
  runtimes given the MCP surface do not consult it, the hypothesis dies.

## Move 2 (standing budget): latency as a design constraint

If verification is instant it runs on every save; if it is slow it runs
on push, and the loop is lost. Accepted as a per-release measured budget,
not a covenant promise.

Measured baseline (founder's machine, 2026-10-11, pinned demo, three
checks): 1.1 ms, 6.1 ms, 0.4 ms. No general sub-10ms claim exists for
arbitrary repos; the gate repos are the honest testbed, and any number we
publish is measured on named hardware against named repos. The dispatch's
"sub-50ms kernel" and "sub-10ms micro-kernel" figures are targets, not
achievements, until measured.

## Move 3 (mostly shipped): the local dependents primitive

Correction of record: this is not future work. `invar dependents` shipped
under A4 Option A, ratified 2026-10-10: local invar.lock scanning under
a caller-supplied directory, zero network by construction (no resolver,
no transport, no token), naming-form query matching, parse errors stated
rather than dropped, exit 4 when the answer has stated blind spots.

The genuinely open increment, queued for ratification as A4.1: producer-
side awareness in `invar check` itself, so a producer editing a contract
surface is shown local sibling dependents without running a second
command. Scope question to settle at ratification: how check discovers
the sibling lock directory without violating zero-config init.

## Ordering (this adjusts the drafted sequence)

Accepted principle from the dispatch: INVAR must be radically valuable in
isolation before any public graph compounds; a public registry with ten
publishers is worth nothing to the eleventh. Therefore:

1. MCP agent surface (Move 1).
2. A4.1 producer-side dependents awareness, if ratified.
3. The nightly public JSONL index (DEPENDENTS-GRAPH.md, still DRAFT)
   moves explicitly after both. The spec is unchanged; only its place in
   the queue is.

The attestation layer (strategy draft of 2026-10-12, not yet ratified)
waits on the F1-F6 research findings and does not enter this queue until
the founder rules on it.

## Ruling of 2026-10-12, after the F1-F6 falsification pass

The founder's research pass (docs/falsification/
attestation-fpass-2026-10-12.md) came back the same day and was
adjudicated as sound. Rulings:

1. The attestation layer is SHELVED with a named re-entry condition
   (strategy/ATTESTATION-LAYER.md). The runway is confirmed as the
   strategy: agents consume receipts and exit codes, not envelopes, and
   the incumbent's own revealed bet (SmartBear shipping most of
   PactFlow's API over MCP, April 2026) points the same way.
2. The platform threat is refined: per the pass, the credible threat to
   INVAR is not a GitHub attestation move but Copilot code review
   commentary ("this change breaks a dependent" as review prose, already
   on more than one in five GitHub reviews per GitHub's own post). Our
   answer is the deterministic, replayable receipt an agent can cite,
   which review prose cannot be.
3. The gate-ownership finding is adopted as a standing constraint: GOV.UK
   configured its working Pact gate not to block. INVAR therefore ships
   advisory-first: receipts, evidence, and the agent loop come before any
   blocking gate, and a blocking gate is a consumer's choice, never our
   default, with the published miss rate attached wherever a gate is
   configured.

Corrections to the founder's accepting dispatch, which upgraded three
findings beyond the report's text (recorded so the stronger versions do
not enter the canon):
- "GUAC cannot parse custom predicates" is the report's [INFERENCE],
  explicitly downgraded in its own zen pass (the error branch was not
  read in source). Likely, unconfirmed.
- "Attestation consumption is effectively zero" is not the finding; the
  finding is UNMEASURED everywhere examined, with weakly established
  absences (Renovate/Dependabot were not searched). Unmeasured and zero
  are different claims.
- SmartBear moved "most of" PactFlow's API to MCP (their words), not the
  entire API.
- The dispatch's phrase "zero-hallucination MCP kernel" is barred by
  covenant promise 6, ratified in the same dispatch. The claim we make:
  deterministic, replayable receipts with a published miss rate.
- `invar_init` does not join the MCP surface: the server's stated
  guarantee is that it never writes files. Init stays a CLI command; a
  write-capable MCP tool would need a separate ratification revoking
  that guarantee. The MCP extension is dependents + coverage, read-only.

## Commercial hypothesis (accepted 2026-10-12, labeled, unmeasured)

The founder's accepted model, recorded as a hypothesis because no revenue
measurement exists: the engine, CLI and MCP server stay free, permissive,
local-first forever; what is sold, later and only if demanded, is what
heterogeneous enterprises need across forges and clouds (cross-org policy
governance, historical ledger compliance, deployment tooling). Platform
disadvantages INVAR builds against, each a structural claim, not an
"unfixable" one: the multi-forge boundary, the deterministic agent
runway, and evidentiary honesty with offline determinism. Falsifier for
the third: a platform publishing its own measured miss rate would erase
that edge, and nothing prevents one from doing so.

## Move 4 (shipped alpha.27): the backtest, proof from the user's own past

The first-run problem of every verification tool: on a healthy repo the
check says HOLDS, which shows nothing, and the value only appears months
later when something breaks. The find: the user's git history already
contains their real breaks. `invar backtest` replays today's locked
bindings over the producer's past commits and shows, by commit hash and
date, when each bound field was absent from the producer's extracted
surface. The proof comes from their own repository, offline,
deterministic, zero network, one command. Honest scope in the output
coda: timelines replay today's extraction over past contents; they are
not replayed verdicts; unreadable commits are stated, never guessed;
remote producers are skipped and stated; renames are not followed (v1).
Exits 0 / 4 (stated blind spots) / 3, never a verdict.

## Corrections of record on the accepting dispatch

Recorded so the record stays ahead of the narrative:
1. SARIF was never the primary strategy. It was first in the ratified
   build order because it was smallest. No pivot occurred; the dispatch's
   framing of a pivot is rejected while its conclusion (transport, not
   strategy) was already our position.
2. W3 value witnesses are not pending. They shipped in alpha.24,
   ratified stated-first, and were measured on the ripgrep Cargo.lock
   v3 to v4 transition (r1-113: stated, as predicted). The dispatch
   lists this as a next action; it is a done action.
3. `invar dependents` (Move 3) shipped 2026-10-10 under A4. Only the
   A4.1 increment above is new.
4. The dispatch's example SHA (e46d310) is a real commit in our history:
   the reverted unreviewed script rewrite. It appears there as a
   placeholder and carries no claim; noted only so nobody later reads it
   as one.
