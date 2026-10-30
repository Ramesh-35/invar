# INVAR

INVAR checks cross-repository contracts: the fields your code reads from
another repository's API. You declare which producer (an OpenAPI/JSON spec or
the producer's source) a consumer file depends on; INVAR compares the
producer's previous and current versions, finds the fields that were removed,
retyped, renamed or re-enveloped, and reports the ones your code actually
reads, with file and line. It is one static binary, local-first, no server,
no account, and it never edits your code.

The honest part is the point: a skip is never a pass. Anything INVAR could
not evaluate (an unsupported language, a read pattern it cannot parse, a
changed default it cannot trace) is stated, with a reason, and the exit code
says so.

## Sixty seconds

Watch it catch three real breaks first (real incidents, pinned bytes,
measured wall time, run fully offline through the stock engine):

```sh
git clone https://github.com/Ramesh-35/invar && cd invar
cargo run --release -p invar-falsify -- demo
```

No Rust toolchain: download a signed release binary instead (verify it:
[docs/reference/RELEASE-SIGNING.md](docs/reference/RELEASE-SIGNING.md))
and run `invar demo` from the clone. Then, on your own repository:

```sh
invar init          # scans the workspace, proposes invar.toml (a proposal: review it)
invar check         # evaluates the contracts; local files only
invar check --ci    # also fetches remote producers (github:OWNER/REPO@REF:PATH)
```

Zero proposals from `invar init` is the expected result inside a single
repository; INVAR checks contracts BETWEEN repositories, and init says
so and names the next step.

`invar lock` records the exact fields you depend on in `invar.lock`, with
hashes and dates, so future checks compare against what you accepted, and
waivers (capped at 90 days) are explicit and owned. Commit it.

## Exit codes (frozen)

| Code | Meaning |
|---|---|
| 0 | every contract holds |
| 1 | a contract is VIOLATED (a field your code reads changed), and nothing else ever exits 1 |
| 2 | the local evidence ledger failed verification |
| 3 | error, including lock drift for `invar lock`: not a verdict, never silently a pass |
| 4 | something was skipped or not evaluated, stated with reasons |

Exit 4 is non-zero, so a plain `invar check` step fails a CI job by
default; the Action's `fail_on_violation` input gives report-only mode.
Every not-evaluated statement carries a stable reason code
([docs/reference/REASON-CODES.md](docs/reference/REASON-CODES.md)), and
every run ends with the same fixed summary line and the measured record,
even when every count is zero.

## In CI (GitHub Action, hardened)

```yaml
- uses: Ramesh-35/invar@COMMIT_SHA   # pin by SHA; see docs/reference/ACTION-HARDENING.md
  with:
    config: invar.toml
```

Pinned to a release commit, the Action downloads that release's binary and
verifies its checksum and SSH signature before running; pinned to any other
commit, it builds from that exact source; pinned by tag, it warns. Details
and the threat model: [docs/reference/ACTION-HARDENING.md](docs/reference/ACTION-HARDENING.md).

## Evidence receipts

`invar check --receipt r.json` writes a canonical, byte-deterministic record
of the evaluation: every input content-addressed by BLAKE3, the verdict, the
engine version. `invar verify-receipt r.json` re-runs it and must reproduce
the same bytes (exit 0), offline if you have the artifacts
(`--artifacts dir`), with optional SSH signature verification. Receipts from
Linux and macOS are byte-identical, verified in CI for every corpus case.
A receipt proves the evaluation reproduces; it does not prove your code is
correct, and it does not prove the producer is honest.

`invar dependents Type.field --locks dir/` answers "who breaks if I remove
this?" from committed lockfiles. Local directory in, deterministic list out,
no network. `invar check --locks dir/` (or `[dependents] locks = "dir"` in
invar.toml) states, after every check, which sibling lockfiles bind this
repository's producer files; it never changes the exit code.

`invar backtest` replays your locked fields over the producer's own git
history and shows, by commit and date, when each one was absent. It
replays today's extraction over past contents; it is not a replayed
verdict, and it says so in its output.

`invar coverage` states, per contract, what INVAR evaluates here and what
it does not. `invar check --sarif out.sarif` exports results for GitHub
code scanning.

## For AI coding agents

One settings entry makes INVAR part of the agent's edit loop:

```sh
invar hook claude-code --settings   # prints the Claude Code PostToolUse entry; installs nothing
```

After every edit the hook evaluates this repository's contracts; a
VIOLATED contract's witness (contract, field, consumer file and line) is
fed straight back into the agent's context, and nothing else ever is: a
hold, a skip or an error never claims safety and never thrashes the loop.
A directory without invar.toml is silent, so one global entry is safe
everywhere.

`invar mcp` is a read-only MCP server for coding agents (tools: check,
contract, dependents, coverage); it never writes files. Started with
`--lock-signers`/`--lock-principal`, every check and contract result
states whether invar.lock's signature verified in that call
(`lock_state`, `lock_verified`), and a lock that does not verify is an
error, never a verdict. Setup:
[docs/reference/MCP-INTEGRATION.md](docs/reference/MCP-INTEGRATION.md).

## Tell a producer who it breaks, before merge

In a repository that PRODUCES an interface, point INVAR at a directory of
consumers' signed lockfiles:

```sh
invar dependents --locks index/ --repo OWNER/NAME --lock-signers signers
```

Every signed consumer lock naming this repository is evaluated against
the working tree, and each bound field now absent or retyped is named,
per consumer. Unsigned or unverified locks are stated, never read. In the
Action (`dependents_repo`, `dependents_locks`, `dependents_signers`,
`check: 'false'` for pure producers) the report lands as an advisory PR
comment, and on pull requests the index and trust root are read from the
BASE branch, never from the pull request, so a change cannot choose its
own judge. Watch that live: a pull request that deleted the entire index
and trust root in its own diff was still held against main's signed locks
([the attack PR](https://github.com/Ramesh-35/invar-demo-api/pull/2), and
[the honest break](https://github.com/Ramesh-35/invar-demo-api/pull/1)).
A lock records the fields a consumer depends on, not the lines that read
them, and every report says so.

## Judge a declared data contract, at the moment of change

Code is not the only place promises live. A JSON Schema that other teams
read is a declared contract, and `invar judge` holds a change to it
against the version everyone already trusts:

```sh
invar judge --contract schemas/media_player.json --base origin/main
```

The PRE contract is read from the base ref, never from the working tree,
so a change cannot choose its own baseline. A schema is read as a
validation contract: narrowing what it accepts is a break. The verdict
speaks in sentences. On the real Snowplow media_player 1-0-0 to 2-0-0
change (case b1-001 of our pinned corpus), excerpted:

```
VIOLATED: schemas/media_player.json breaks 10 promises made at origin/main.
  tightened: currentTime maximum lowered from 9007199254740991 to 2147483647.
  Values valid before can now be rejected; restore the bound, or version the change.
  removed: percentProgress was integer|null (line 47 at origin/main). Consumers
  reading this field will break; keep the field, or ship this as a new major
  version and migrate them.
  ...
judged: property presence and type, enum, numeric, length and item-count
bounds, required, pattern and format, recursively. not judged: $ref, allOf,
anyOf, oneOf, items (stated, never skipped).
```

In the Action, one input:

```yaml
- uses: Ramesh-35/invar@COMMIT_SHA
  with:
    check: 'false'
    judge_contract: schemas/media_player.json
```

HOLDS passes. VIOLATED fails the job and the verdict lands in the step
summary. A brand-new contract passes, stated as `new_contract`: nothing
was promised before. An unparsable contract or an unreadable base ref
fails: a lost baseline is never a pass. `--format json` carries the same
facts for machines, including `unjudged_constraints`, and `--receipt`
writes the canonical record.

The judged set is stated and narrow, on purpose: property presence and
`type`, enum narrowing, tightened bounds, newly required fields, and
changes to `pattern` or `format` (flagged as changed; whether a regex
change narrows what is accepted is not decided, and the verdict says so),
recursed with dotted-path witnesses. Not judged yet, and printed in every
output: `$ref`, `allOf`, `anyOf`, `oneOf`, `items`. Specs, fixtures and
the mutation record:
[docs/falsification/JUDGE-SPEC.md](docs/falsification/JUDGE-SPEC.md),
[docs/falsification/JUDGE-SUBTYPE-SPEC.md](docs/falsification/JUDGE-SUBTYPE-SPEC.md).
`invar judge` ships from v0.2.0; the subtype checks (enum, bounds,
required, pattern, format) from v0.2.1.

## What INVAR does not do

No semantic checks: INVAR evaluates structure (a field's presence, type,
name, envelope), never what values mean. No runtime behavior: nothing is
executed. Payload ingestion covers JSON and a stated
TOML subset (YAML and binary payloads are stated as not evaluated,
exit 4). And it misses real breaks:

Measured on a corpus of 122 real cross-repository break cases and controls
(sourced from public incidents, pinned commits, scored on held-out batches
the rules never saw): INVAR currently catches 38 of 70 held-out in-scope
real breaks. That is a 46% miss rate, the detection-reliability hypothesis
is recorded as FALSIFIED against our own 20% gate, and every release note
says so. What INVAR adds even where it misses: what it did not evaluate is
stated instead of silently passed, the lock records exactly what you
accepted and when, and the receipts make every verdict reproducible. The
full, unvarnished record: [docs/epistemic/STATUS.md](docs/epistemic/STATUS.md),
corpus and per-batch scores in [docs/falsification/](docs/falsification/).

What we promise never to break, in writing: [docs/COMPATIBILITY-COVENANT.md](docs/COMPATIBILITY-COVENANT.md).

Unsupported languages (C, C++, C#, PHP, Swift, Java, Kotlin, Ruby) are
SKIPPED, stated, exit 4; never a pass.

## Research harness

This repository also contains the falsification harness that produced the
record above (`invar run`, `validate`, `report`, `verify`, `audit`,
`replay`, the corpus under `corpus-real/`, and the runner in `scripts/`).
It exists to attack INVAR's own claims; batches are sourced after each
rule's spec is frozen and hashed. To add a real case, see the admission
rules in [docs/falsification/R1-CORPUS.md](docs/falsification/R1-CORPUS.md).

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or
[MIT](LICENSE-MIT), at your option. See [`LICENSE`](LICENSE). Corpus
data is separately licensed under
[CDLA-Permissive-2.0](LICENSE-DATA); see
[docs/reference/CORPUS-DATA.md](docs/reference/CORPUS-DATA.md). Files
under `demo/` keep their upstream licenses ([demo/NOTICE](demo/NOTICE)).
