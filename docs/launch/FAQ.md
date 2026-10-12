# Launch FAQ (honest answers, prepared cold)

**Why not just diff OpenAPI specs (oasdiff etc.)?**
When both sides keep honest specs, do that too; they compose. INVAR
exists for the majority case we measured: producers whose contract
lives in code (serializers, pb.go, TS interfaces) and consumers that
read JSON in another repo entirely. The corpus baselines record where
spec diffing was applicable and what it caught; for most of our real
cases it was not applicable at all.

**Why only Go, Python, JS/TS, Rust?**
Because those are the languages our extraction demonstrably works on.
C, C++, C#, PHP, Swift, Java, Kotlin, Ruby are SKIPPED with a stated
reason and exit 4, never silently passed. Scope grows only with
held-out evidence, which is slower than claiming, deliberately.

**Why should I trust your corpus?**
You should not have to. Every case pins public commits with BLAKE3
hashes; the held-out set was sealed before the repo went public; the
scorer is a dependency-free script; CDLA-licensed JSONL export means
you can rescore everything, including against your own tool.

**GraphQL? gRPC? Events?**
GraphQL: not evaluated, stated. Protobuf: the generated .pb.go
surface works today (a real catch in the demo is exactly that);
.proto parsing is not claimed. Event payloads: the event kind checks
top-level keys; observed-reality witnesses go deeper on any JSON
payload you can sample.

**How is this different from consumer-driven contract testing
(Pact)?**
Pact needs both sides to adopt a framework and run brokers. INVAR
needs the consumer alone, works on repos that never heard of it, and
evaluates statically in milliseconds. It also catches less than a
full contract-test suite would; 38 of 70 is the measured floor of the
static approach, which is why the lock, witnesses and receipts exist
around it.

**When is v1?**
When criteria we have not yet earned are met and published, not when
marketing wants it. v0.1.0 stays alpha until then; the DOI-minted
releases are archival, not stability claims.
