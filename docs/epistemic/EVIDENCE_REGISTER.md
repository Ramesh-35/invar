# INVAR evidence register

Status: DRAFT r1, for ruling (founder mandate of 2026-10-23: classify every
capability by its true evidence class). Audited tree: e09eff7. Every claim
below is classified by the strongest evidence that exists in this
repository today, not by the strongest evidence that could exist. A class
is a ceiling: citing a Class 2 item as proven, or a Class 4 heuristic as
analysis, is a ledger violation.

The classes:

- Class 1, mechanically established: a machine-checked proof or
  model-checking artifact, reproducible from pinned inputs, connected to
  the implementation.
- Class 2, test-supported and adversarially falsified: frozen fixtures,
  killed mutants, golden outputs, corpus replays.
- Class 3, runtime-enforced safeguard: the binary refuses or fails closed
  at run time; tested, but the guarantee is the refusal path itself.
- Class 4, implemented heuristic: text patterns and extractors that state
  their own scope; output labeled PROPOSED or stated as replay.
- Class 5, unestablished or open: claimed nowhere, or desired and not yet
  evidenced. Listing here is the claim's only honest home.

## 0. The headline, stated first

INVAR has ZERO Class 1 items. Nothing in this repository is mechanically
proven: no proof-assistant artifact, no model checker, no verified parser.
The strongest evidence class anywhere is Class 2. Every place the word
"proof" appears (invar.proof, proof.rs, "the lock is a proof artifact") it
names a file format or a recorded evaluation, not a mathematical proof, and
the register below classifies those items where their evidence puts them.
The Rust compiler's memory- and type-safety hold for the implementation
language but establish nothing about INVAR's own properties, so they are
not listed as Class 1 items.

## 1. Verdict semantics (the exit-code contract)

| Claim | Class | Evidence | Not established |
|---|---|---|---|
| Exit codes mean 0 HOLDS, 1 VIOLATED, 2 ledger break, 3 error (never a verdict), 4 skipped or not evaluated | 2 | tests/explain_test.rs bidirectional source-scan (registry vs emission sites); fixtures across every arc; action.yml states the same table | No proof that every future code path respects the table; the scan covers today's emission sites |
| A skip is never a pass; 0-binding locks are NOT EVALUATED, never HOLDS | 2 | LOCK-SLICE1 fixtures and mutants; unsupported-language fixtures | |
| An expired waiver fails the check (exit 1) until renewed or removed | 2+3 | check fixtures; waiver cap validated at parse (exit 3 past 90 days) | |
| A reused HOLDS equals a fresh evaluation on unchanged inputs | 2 | CHECK-INCR-1: F1 to F20, mutants E01 to E22 caught 22/0, 600-commit replay with zero disagreements (docs/evidence/incr) | The input-completeness argument (every read passes one function) is code-reading checked by tests, NOT a proof; concurrency between two simultaneous checks is untested |
| explain reads nothing but its argument (documentation, never evaluation) | 2 | explain_test.rs; module charter B1 to B3 | |

## 2. Discovery (`invar init`)

| Claim | Class | Evidence | Not established |
|---|---|---|---|
| Proposals are heuristic text patterns over masked source, labeled PROPOSED, deterministic across runs | 4 (scope stated in every output header) | slices 1 to 8 fixtures and mutant batteries; byte-identical double runs in every init test | It is NOT an AST or a type checker; inference is excluded by design |
| Two-blind-agent adjudication with conservative merge | 4 | STAGE3-EVAL ledgers (eval001 to eval008) | Adjudication quality itself has no mechanical measure |
| Abstentions are stated refusals, not negative findings | 2 | ABSTAIN fixtures in init slices | Completeness of discovery: what init never proposes is unmeasured except via the EVAL corpus |

## 3. Lock and check (the core loop)

| Claim | Class | Evidence | Not established |
|---|---|---|---|
| The lock records the consumer's dependency slice bound to declaring types; check reports residue with witnesses | 2 | harness_test.rs, lock_slice and init_slice check fixtures; EVAL-008 round-trips | |
| Surface extraction (spec fields, code fields, zod shapes, one-line Readonly bodies) | 4 | per-slice fixtures; corpus equivalence 18,206 files (CHECK-PERF-1) | Extractors read a stated subset of TypeScript/JSON/YAML; everything outside it abstains or is invisible |
| Consumer-read detection (broad_reads, 4 keys) | 4 | fixtures; 2,037,643-read corpus equivalence | A read outside the four patterns is not seen; A1 states changed-but-unread bindings instead |
| Memoized analyses are byte-identical to unmemoized (CHECK-PERF-1) | 2 | P-F1 to P-F6, P-N1; 9/9 mutants; byte-identical check on 3 repos | |
| Lock formats 1 to 4 read forever; unknown formats fail closed | 3 | parse fixtures | |
| held/held_input records: only HOLDS, never under waiver/sample/receipt/--sample | 2 | CHECK-INCR-1 F8 to F10, F19; E07 to E09, E19, E21 | |
| Default-switch statements (F1 plus G3): a selector value or default-site variant that changes between base and head makes a residue-free contract stated NOT EVALUATED, never VIOLATED | 4 (textual grammar, stated in G3-SPEC r2/r3), outcome Class 2 | F1 tuning record; G3: 24 fixtures, 18 operators (17 killed by declared fixtures, 1 argued equivalent), 122-case corpus byte comparison (one movement, r1-113), native 449/449 | Which output path the consumer actually receives; values defined in other files; ownership of a value inside an open window beyond the stated grammar; held-out recall effect (tuning rung only) |
| Request side (RS-1): a literal inbound read removed between base and head, with no surviving head literal, that the consumer writes as a literal, is VIOLATED; RS-2 (became read_only) and RS-0 (opaque payload) are stated NOT EVALUATED | 4 (textual grammars B1 to B3, frozen in REQUEST-SIDE-SPEC r2), outcome Class 2 | 23 fixtures (RS-0's synthetic), 16 operators all killed by declared fixtures, 122-case corpus byte comparison (two movements, r1-118 and r1-060), native 472/472 | Keys not written as literals; framework binding beyond the existing field extraction; route association (r1-051); trailing comments; held-out recall effect (tuning rung only) |
| Opaque emission (G1-E): a literal emission of a frozen call form lost while an opaque one is gained, with a base literal key the consumer reads, makes a residue-free contract stated NOT EVALUATED, never VIOLATED | 4 (textual grammar E1 to E3, frozen in G1-E-SPEC r0), outcome Class 2 | 15 fixtures, 12 operators all killed by declared fixtures, 122-case corpus byte comparison (one movement, r1-107), native 487/487 | Which head site replaced which base site; member-access or call payloads; forms outside the frozen seven; cross-file object flow; held-out recall effect (tuning rung only) |
| Go identifier binding (G2-A): a removed exported Go field read by its exact Go name in a Go consumer is VIOLATED through the existing removed-field rule | 4 (textual grammar, G2-SPEC r1 section B), outcome Class 2 | 14 fixtures, 11 operators all killed by declared fixtures, 122-case corpus byte comparison (r1-076), native 501/501 | Go renames that keep the JSON tag; reads through embedded or aliased types; held-out recall effect (tuning rung only) |
| Rename candidates (G2-R): a convicted removed field with exactly one same-scope, same-known-type replacement is annotated as a candidate, never a verdict | 4 (one-to-one pairing over extracted fields), stated only | same fixtures and operators; 3 printed candidates in the corpus, each matching its recorded rename | Whether the consumer should read the candidate; renames across scopes, files or types; untyped fields |
| Self-audit (AGENT-1 P2): every schema conviction is re-checked against the bytes it cites; a witness that fails is exit 3, never VIOLATED | 2 (deterministic check of the engine's own output against pinned input bytes) | 13 fixtures, 9 operators all killed by declared fixtures, 2 defect injections each trapping 65 of 65 corpus convictions, 122-case corpus byte comparison (no movement), native 514/514 | Whether a cited read is semantically a read of that field (r1-116 passes); non-schema contract kinds |
| Reuse authenticity (SECURITY S1): a reused HOLDS comes only from a signature-verified invar.lock or a cache record keyed under the user's own key; a forged record is evaluated | 2 (keyed MAC and signature check, deterministic) | 9 fixtures, 6 operators all killed by declared fixtures, forgery probe (pre-S1 exit 0, S1 exit 1), 122-case corpus byte comparison, round-trip verdict identity | An attacker who can read the user's home; the trust in the signers file the operator passes |
| Terminal neutralization and bounded walk (SECURITY S2): no repository control or bidirectional-override character reaches a terminal raw and no value forges a verdict line; the lock walk follows no link and is bounded; MCP dependents reads inside the configuration directory | 2 (deterministic escaping and bounds) | 14 fixtures frozen before code, 9 operators all killed by declared fixtures, injection and loop probes, 122-case corpus byte comparison, round trips | JSON and SARIF (read by programs); a newline in a non-verdict value starts a new line (neutralized of terminal escapes, not escaped) |

## 4. Evidence and trust boundary

| Claim | Class | Evidence | Not established |
|---|---|---|---|
| Receipts bind input bytes + engine to outcomes; in-toto export wraps the body verbatim; SSH verify rejects tampered payloads and wrong namespaces | 2 | harness_test.rs receipt and a21 fixtures | Receipts are not countersigned or timestamped by a third party |
| Under --ci, cache never read or written; lock records reused only after --lock-signers verified | 2+3 | F11, F12; E10, E11 | An UNSIGNED lock is not tamper-evident: a PR can hand-edit held_input hashes; this is exactly why I-D4 exists |
| Fork-PR text is rendered as escaped code in reports (no pings, no links) | 3 | action report fixtures with hostile ids | |
| Remote fetches: size-capped, curl subprocess, failure is exit 3 never a verdict | 3 | resolver fixtures | TLS and GitHub trust are assumed, not analyzed |
| Engine identity invalidates reuse across builds (version + BLAKE3 of sources, Cargo.lock, rustc) | 2 | F7; E06 | A binary patched AFTER build keeps its id; release provenance and reproducible builds do not exist in this repository |

## 5. Temporal and corpus machinery

| Claim | Class | Evidence | Not established |
|---|---|---|---|
| Ledger: bi-temporal, BLAKE3-chained rows over SQLite; tamper breaks the chain (exit 2) | 2 | ledger tamper fixtures (out-of-band row edit caught) | Chain security assumes the attacker cannot recompute the whole chain; no anchoring outside the file |
| Backtest replays today's extraction over past bytes; stated as replay, never replayed verdicts | 4 (coda states scope in every output) | backtest fixtures and goldens; HERM-1 bt_h1 (goldens survive a hostile host git config) | DEFECT FOUND AND FIXED (HERM-1, 2026-10-23): the goldens had been recorded in an environment whose git config signs every commit (commit.gpgsign, SSH), so they encoded that environment's signing key and failed on any unsigned machine, the founder's included; proven by raw commit objects (a gpgsig header on the recording side) and by the founder's SHAs matching a config-free build exactly. Fix: every test git call is isolated from host global and system config, and the goldens are re-recorded to the configuration-free SHAs. The engine itself is never isolated: it reads real repositories and honors their owners' configuration |
| Proof imports (invar.proof): canonical bytes, tamper detected, precedence 3>2>1>4>0 | 2 | proof_test.rs; unit tests | "proof" here means a signed, recorded evaluation, not a mathematical proof |
| Observed reality: locked samples, staleness stated, drift is residue | 2 | sample fixtures | Sample quality depends on what the operator recorded |
| Gate: check before, hash-watch, re-check after, drift stated | 3 | gate_test.rs | TOCTOU between the final check and the command's own effects is stated, not closed |

## 6. Measured performance and recall (never SLAs)

| Claim | Class | Evidence | Not established |
|---|---|---|---|
| 38 of 70 held-out real cross-repository breaks caught (batches 4 to 8) | 2 (measured, printed with every check) | docs/epistemic/STATUS.md | The other 32 were MISSED; recall is 54% on that corpus and is the single most honest number INVAR prints |
| Mac latency, founder's machine: at 0abe91e fresh check 15.15 ms, first MCP call 13.87 ms, warm MCP 2.12 ms; at e09eff7, three consecutive runs, fresh check 3.60 to 3.73 ms, first call 10.63 to 11.36 ms, warm 2.11 to 2.15 ms (reuse cut the fresh check about 4x, warm unchanged; sandbox agrees: 72 to 100 ms to 6.4 to 7.2 ms on the bench workspace) | 2 (measured there, those days) | founder's bench runs of 2026-10-23; sandbox triple runs | Any other machine, workload or day; one set of 27 to 29 ms readings taken immediately after a compile is classified transient machine state, resolved by the stable triple, and is not a figure |
| Reuse cuts a typical commit's check cost 73 to 95% (instructions), zero verdict changes | 2 | docs/evidence/incr/*.tsv; skipcost/callgrind runs | |

## 7. Open problems (Class 5, the honest backlog)

- No Class 1 item exists. Candidate first target, per the founder's
  standard: the reuse soundness condition (a reused HOLDS equals fresh
  evaluation) as a mechanically checked statement over a model of
  evaluate_reusing, with the input-capture completeness as its stated
  assumption.
- Release provenance: no reproducible build, no artifact signing, no tie
  from a shipped binary to a source revision.
- Concurrency: two `invar check` processes in one workspace interleave on
  .invar/cache (atomic rename means last-writer-wins, never corruption,
  but this is reasoned, not tested).
- Detection completeness: 32 of 70 held-out breaks missed; per-category
  misses are recorded in the eval ledgers but no arc has attacked them as
  a set.
- Languages: C, C++, C#, Go-adjacent files et al. are SKIPPED by stated
  policy; "supported" means the TypeScript/JSON/YAML subset the extractors
  read.
- The MCP server (tools: check, contract, coverage) inherits every class
  above; it adds no verification of its own and its warm-path equality to
  fresh processes is Class 2 (mcpchange.py, P fixtures).

## 8. Rule of use

A sentence about INVAR may claim at most the class this register records
for it. Upgrades happen by landing evidence and amending this file in the
same commit, never by wording.
