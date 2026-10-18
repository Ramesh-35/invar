# INVAR-PROOF-001: cross-repository egress attestation (`invar.proof`)

Status: REVISION 3. Rulings D1 to D9 RATIFIED (2026-10-15 20:00); open items I1 to
I3 RULED (2026-10-18). Revision 2 was a syntax alignment: it applied the ratified D4
binding syntax (`[[proof.import]]`) and D6 file layout, and recorded the ratified
engine fingerprint. Revision 3 records the I1 to I3 rulings and the executed Gate 0
baseline; no normative requirement changed. Gate 0 (hash baseline) is executed under
the 2026-10-18 ruling; Gates 1 to 5 await the execution directive. Prepared
to the 86427 and slice 2 standard: numbered requirements with falsifiable
acceptance, abstention contract, fixtures with declared outcomes before code,
mutation set, gates 0 to 5. Parents: INIT-DISCOVERY-SLICE2.md (K1 egress locks,
landed e27fd8a) and INIT-DISCOVERY-SLICE2B.md (landed 34f1e52, probes 431b9aa).
No capability, adoption or evaluation claim is made. There is no real-repository
evidence for proofs; this document is design only.

Changes from revision 1: the consumer binding is a `[[proof.import]]` table with
`contract_id`, `anchor`, `accepted_epoch`, `accepted_digest` (D4) plus the two keys
`proof` and `fields` that B4 requires and the ruling did not list (section K, item
I1); proof files live at `invar.proof` or `.invar/proofs/<name>.proof` (D6); a
`rule_fingerprint` is added to the proof (section K, ruling of the volunteered items);
a missing proof file gets its own code; section K records the rulings and three open
items. Nothing else changed.

## 0. What a proof is, and the honest bounds

A provider repository that has locked an egress contract (K1) can emit a small
file, `invar.proof`, stating: "at this source state, this boundary emits at
least these direct fields." A consumer repository declares which provider
contract and which fields it relies on, and `invar check` verifies the declared
fields against the proof offline, with no network, in one deterministic pass.

Four bounds are stated up front and are not negotiable by wording:

B1. Offline freshness is relative. A consumer with no network cannot know the
provider has moved on. Freshness can only be judged against state the consumer
itself holds (an accepted epoch it recorded). A proof can therefore never prove
"the provider still emits this today". Everything this spec calls staleness is
staleness relative to the consumer's recorded acceptance, nothing more.

B2. Integrity, not authenticity, at level 0. A content digest shows a proof was
not edited after emission. It does not show who emitted it. Authenticity needs a
signature (level 1), which needs a new dependency and a key-distribution answer;
that is ruling D1, not assumed.

B3. Keys only. Like K1, a proof attests depth-1 field names, not types, values or
serialization. It inherits K1's disclaimer: runtime serialization is not verified.

B4. Explicit bindings only. The K3 probes (431b9aa) measured 35% wrong-receiver
noise from name overlap. Nothing in this spec discovers pairings. A consumer
states its receiver binding by hand; the engine never infers one.

## A. Frozen-at-ruling scope

A1. Pattern extraction and no new dependency at level 0. Digest is blake3, already
in the tree. Level 1 (signatures) and RFC 3161 timestamps are deferred (D1, D8).

A2. Supported contract kind in v1: `egress` only. Proofs of `strings` or `schema`
contracts abstain PROOF-UNSUPPORTED-KIND.

A3. Zero network, always. Neither emit nor check opens a socket. Git is invoked
only as a local subprocess on the provider side at emit time, exactly as
`backtest` already does; the consumer check invokes nothing.

A4. ISA invariance. No change to the five exit codes or their meaning. Authoring
commands use 0 and 3 only. `check` uses 0, 1, 3, 4 for proofs; exit 2 keeps its
meaning (ledger break) and is never returned by proof verification. No new
lock-file grammar; consumer acceptance state lives in invar.toml (D2).

## B. Artifact format (`invar.proof`, flat TOML, deterministic)

```
[proof]
format = "1"
level = "0"                 # 0 = digest only; 1 reserved for signatures
rule_fingerprint = "<64 hex>"   # blake3 over the engine's rule table (R-P-10)

[producer]
repo = "acme/orders-api"    # declared name; NOT verified
commit = "<40 hex>"         # HEAD at emit time (the commit before the one adding the proof)
source_blob = "<40 hex>"    # git blob OID of the shape file at emit time
boundary = "app/api/v1/orders/route.ts"
shape_file = "lib/schemas.ts"
shape = "OrderResponse"
contract = "egress:OrderResponse"
anchor = "GET /v1/orders"   # the provider's declared receiver anchor, required

[fields]
names = "currency_code id nested total_minor"     # sorted, space separated

[chain]
epoch = "7"                 # strictly increasing integer, +1 per emit
parent_digest = "<hex or none>"

[attestation]
digest = "<blake3 hex over the canonical bytes of every section above>"
generated_at = "2026-10-15T12:00:00Z"   # informational, NOT in the digest, not used by default
```

R-P-1 Canonical form. The digest covers the exact bytes of sections `proof`,
`producer`, `fields`, `chain`, emitted in the fixed key order above with LF line
endings and sorted `names`. Re-emitting an unchanged contract at the same source
state yields a byte-identical file except `generated_at`, which is preserved from
the previous proof when nothing else changed (the `recorded_at` rule of invar.lock).
Counterexample fixture: PRF-M1.

R-P-2 Self-reference is avoided by binding to `source_blob` and the pre-proof
`commit`, never to a tree that contains the proof file itself.

R-P-2a Layout (D6). A provider writes its proof to `invar.proof` at the repository
root (default) or to `.invar/proofs/<name>.proof`, where `<name>` matches
`[A-Za-z0-9_-]+`. `--out` outside these two forms is exit 3. One file holds exactly
one contract. A provider with several contracts uses `.invar/proofs/`.

R-P-10 Rule fingerprint. `rule_fingerprint` is the blake3 hex digest of the engine's
rule table for the proof's kind, serialized as sorted `key=value` lines: the engine
version, the response markers, the stoplist, MIN_FIELDS, and the proof format and
level. It is inside the digest. Its only effect is traceability: `check --proof`
prints the fingerprint, and a fingerprint that differs from the verifying engine's is
a printed NOTE, never an alarm and never a verdict change. (The ruling named SHA-256;
the tree has blake3 and no SHA-256 crate, and a new dependency is a stop condition,
so blake3 is used. See section K, item I2.) Fixtures: PRF-M5, PRF-M6.

## C. Provider side: `invar lock --emit-proof <contract-id> [--out invar.proof]`

R-P-3 Preconditions, checked in order; any failure is exit 3 (authoring error)
with the reason, and no file is written: (a) the contract exists, is kind egress
and has an invar.lock entry; (b) the shape is statically extractable; (c) the
current direct keys equal the locked set (no unlocked drift: otherwise "run
`invar lock --update`"); (d) the invar.toml block declares `anchor`; (e) `git` is
available and the working tree file equals the committed blob (no uncommitted
edits to the shape file). Fixtures: PRF-E1, PRF-E2, PRF-E3.

R-P-4 Epoch. If an `invar.proof` already exists and verifies, the new epoch is its
epoch plus 1 and `parent_digest` is its digest; if absent, epoch 1 and
`parent_digest = none`. An existing proof that fails verification is exit 3.

R-P-5 Exit 0 on success, printing the path, epoch and digest. Never exit 2.

## D. Consumer side

D-1 Binding (invar.toml, explicit, one table per imported contract):

```
[[proof.import]]
proof = ".invar/proofs/orders-api.proof"    # a file, or a directory of *.proof files
contract_id = "egress:OrderResponse"        # MUST equal the proof's producer contract
anchor = "GET /v1/orders"                   # MUST equal the proof's producer anchor
fields = "id total_minor currency_code"     # the fields this repo relies on, by hand
accepted_epoch = "7"                        # epoch last reviewed and accepted
accepted_digest = "<hex>"                   # digest last reviewed and accepted
max_age_days = "30"                         # optional, opt-in clock check (R-P-8)
```

`contract_id`, `anchor`, `accepted_epoch` and `accepted_digest` are the keys the
ruling fixed; `proof` and `fields` are required by B4 (the engine cannot find a file
or know what the consumer relies on without being told). The consumer's copy of the
provider's proof may live anywhere inside the consumer repository; only the provider
side is bound to the D6 layout. Unknown keys in the table are exit 3 (a typo in a
trust binding must not be silently ignored).

Plain `invar check` does not evaluate `[[proof.import]]` tables; only
`invar check --proof` does, so adding a binding cannot change any existing verdict
(fixture PRF-N3). Whether plain `check` should evaluate them once the feature is
stable is open item I3.

`invar init` never proposes a proof block (B4). A tool may print a template; the
human or agent writes `fields`, `anchor`, `accepted_*` deliberately.

R-P-6 Verification order for `invar check --proof`, per `[[proof.import]]`, first
failing step decides (the process exit is the worst code over all imports, ordered
3 over 1 over 4 over 0, and every import prints its own line):
0. Resolve: the `proof` path is absent, or a directory with no `*.proof` file whose
   contract equals `contract_id`: exit 4 PROOF-NOT-FOUND. Unknown binding keys: exit 3.
1. Parse and digest (exit 3 on malformed file, unknown format, or digest mismatch:
   PROOF-TAMPERED, a stated error, not a verdict).
2. Kind: the proof's contract kind other than egress: exit 4 PROOF-UNSUPPORTED-KIND.
3. Scope: `contract_id` or `anchor` differ from the proof's: exit 4
   PROOF-SCOPE-MISMATCH.
4. Ambiguity: more than one proof file resolves for the binding: exit 4
   PROOF-AMBIG-SCHEMA. (Ambiguity arises when `proof` is a directory holding several
   proofs for the same `contract_id`.)
5. Freshness anchor: `accepted_epoch` or `accepted_digest` absent: exit 4
   TEMPORAL-STALENESS-UNANCHORED (freshness cannot be established; HOLDS is not
   offered; ruling D3). Present: proof epoch lower than `accepted_epoch` is exit 4
   STALE-PROOF-COMMIT (rollback or replay of an older proof); equal epoch with a
   different digest is exit 3 PROOF-FORK; equal epoch and equal digest, or a higher
   epoch, passes this step. A higher epoch whose `parent_digest` does not chain to
   `accepted_digest` when epoch = accepted_epoch + 1 is exit 3 PROOF-FORK; a larger
   gap cannot be chained offline and is noted, not alarmed.
6. Fields: every name in `fields` present in the proof's `names` holds; any missing
   is residue and the contract is VIOLATED (exit 1), naming each missing field,
   the proof path and epoch. Extra names in the proof are noted and tolerated
   (additive tolerance, as K1).
Counterexamples: PRF-V1 to PRF-V3, PRF-U1 to PRF-U5.

R-P-7 A proof never converts a violation into a pass. There is no override in this
spec; the break-glass design (red-team directive, GAP-09) is a separate ruling and
a separate document, because it changes the meaning of an exit code path.

R-P-8 Determinism and clock. Default verification reads no clock. An optional
`max_age_days` on the `[[proof.import]]` table adds one clock-dependent step (proof `generated_at`
older than the window yields exit 4 TEMPORAL-STALENESS-UNANCHORED). It is opt-in,
it is the only non-hermetic check, and results that depend on it say so.
Fixture: PRF-U6, PRF-M2.

R-P-9 Zero network and budget. Verification of one proof completes in under 2 ms
on the release build (stop condition if exceeded on the fixture), reads only the
proof file and the manifest, and adds no dependency. Gate 4 measures both.

## E. Abstention and error contract

| Code | Exit | Fires when |
|---|---|---|
| PROOF-TAMPERED | 3 | digest mismatch, malformed file, unknown format or level, unknown binding key |
| PROOF-FORK | 3 | equal epoch with different digest, or broken parent chain at epoch + 1 |
| PROOF-NOT-FOUND | 4 | the `proof` path is absent or holds no proof for `contract_id` |
| PROOF-UNSUPPORTED-KIND | 4 | proof or binding names a kind other than egress |
| PROOF-SCOPE-MISMATCH | 4 | contract or anchor in the proof differs from the binding |
| PROOF-AMBIG-SCHEMA | 4 | the binding resolves to more than one proof for the contract |
| STALE-PROOF-COMMIT | 4 | proof epoch below the consumer's accepted epoch |
| TEMPORAL-STALENESS-UNANCHORED | 4 | no accepted epoch/digest recorded, or opt-in age window exceeded |

Codes follow the slice 2 print form `ABSTAIN <CODE> <path>:<line> <reason>`.
Exit 1 is reserved for a missing relied-on field (R-P-6 step 6) and nothing else.

## F. Fixtures (declared outcomes, before any code)

All synthetic, built in temporary directories, run through the real binary twice
(byte-identical output).

F (positive):
- PRF-F1 provider emits epoch 1; consumer binds three fields, records accepted_epoch 1 and the digest; `check --proof` exits 0.
- PRF-F2 provider adds a field and re-emits epoch 2 (chains to epoch 1); consumer with accepted_epoch 1 still exits 0 and notes the added field; after the consumer records epoch 2, exit 0.
- PRF-F3 two `[[proof.import]]` tables to two contracts in one proof directory resolve independently.
- PRF-F4 layout: a provider emits to `.invar/proofs/orders.proof` and to the root `invar.proof`; both verify; `--out` elsewhere is exit 3.

V (violations):
- PRF-V1 provider removes a relied-on field, re-locks, re-emits epoch 2; consumer exits 1 naming the field and epoch.
- PRF-V2 provider renames a field (old name gone); exit 1.
- PRF-V3 consumer relies on a field the proof never listed; exit 1.

E (authoring refusals, exit 3, no file written):
- PRF-E1 no invar.lock entry for the contract. PRF-E2 shape turned dynamic. PRF-E3 current keys differ from the locked set. PRF-E4 uncommitted edits to the shape file. PRF-E5 `anchor` undeclared.

U (abstentions and errors, exact code and exit):
- PRF-U1 anchor mismatch (PROOF-SCOPE-MISMATCH, 4). PRF-U2 two proofs for one contract id in the bound directory (PROOF-AMBIG-SCHEMA, 4). PRF-U3 epoch below accepted_epoch (STALE-PROOF-COMMIT, 4). PRF-U4 equal epoch, different digest (PROOF-FORK, 3). PRF-U5 no accepted_epoch (TEMPORAL-STALENESS-UNANCHORED, 4, never HOLDS). PRF-U6 max_age_days exceeded (4, labeled clock-dependent). PRF-U7 proof of a `strings` contract (PROOF-UNSUPPORTED-KIND, 4). PRF-U8 hand-edited field list, digest unchanged (PROOF-TAMPERED, 3). PRF-U9 higher epoch whose parent_digest does not chain at accepted_epoch + 1 (PROOF-FORK, 3). PRF-U10 `proof` path absent (PROOF-NOT-FOUND, 4). PRF-U11 an unknown key in a `[[proof.import]]` table (exit 3).

N (negative controls, zero verdict change):
- PRF-N1 repository without proof blocks: output and exit codes byte-identical to today on every slice 1, 2 and 2b fixture. PRF-N2 `invar init` on a repository holding an invar.proof proposes nothing from it (B4). PRF-N3 a repository with `[[proof.import]]` tables: plain `invar check` output and exit codes are byte-identical to the same repository without them.

M (shared): PRF-M1 re-emit at unchanged state is byte-identical (generated_at preserved); PRF-M2 default check output does not vary with the system clock (run under two faked dates); PRF-M3 `cargo tree` shows no new dependency; PRF-M4 no socket is opened (no network-capable crate in the dependency tree of the new module, asserted by a source scan test). PRF-M5 `rule_fingerprint` is identical across repeated emits and changes when one rule-table constant changes. PRF-M6 a proof whose fingerprint differs from the verifying engine's prints a NOTE and the verdict and exit code are unchanged.

## G. Mutation set (CAUGHT / SURVIVED / BUILD-FAILED; the last two are not kills)

1 digest not verified. 2 missing field treated as held. 3 extra provider field treated as violation. 4 scope check removed. 5 ambiguity check removed. 6 epoch rollback accepted. 7 fork accepted. 8 unanchored freshness returns HOLDS. 9 emit proceeds with unlocked drift. 10 any proof path returns exit 2. 11 canonical form depends on field order. 12 clock read by default. 13 emit overwrites a proof that fails verification. 14 anchor ignored at emit. 15 fingerprint mismatch changes the verdict. 16 plain `check` evaluates `[[proof.import]]`. 17 unknown binding key ignored. 18 `--out` outside the D6 layout accepted. 19 missing proof file returns HOLDS. Each must die by a named assertion failure.

## H. Gates 0 to 5

Gate 0 hash ledger of Class 1 files plus landed slices 2 and 2b (clean tree at 431b9aa or later). Gate 1 implementation to allowed paths. Gate 2 all fixtures. Gate 3 mutation harness, control green. Gate 4 full suite green, zero warnings, slice 1/2/2b outputs byte-identical, 2 ms verification and no-new-dependency checks. Gate 5 Section evidence ledger appended here, STATUS.md entry, EXPOSURE-LOG.md record. A failure at any gate stops the arc for ruling. No real-repository probe applies to proofs until an upstream/downstream pair exists that agrees to be probed; none is claimed.

## I. Path classes

Class 1 untouched and hash-verified: surface, backtest, invariant, yaml, codepairs, Cargo files, the five Experiment C artifacts. Class 2: tests/proof_test.rs, scripts/proof/. Class 3 GRANTED by ruling D9: src/proof.rs (new), lock.rs (`--emit-proof` handler), check.rs (`--proof` verification runner), config.rs (`[[proof.import]]` schema; config.rs leaves Class 1 for this change only, and its pre-change hash is recorded at Gate 0). Class 3 also GRANTED by ruling I1b (2026-10-18): main.rs, because the two CLI flags are parsed there. Class 4: lib.rs (module registration).

## J. Rejected or deferred from the red-team input, with reasons

- HMAC-SHA256 envelope: symmetric, needs a shared secret, so any holder can forge a proof; it is not an attestation. Rejected. Level 1 would be an asymmetric signature (D1).
- Git Tree OID of the provider repo: the proof file lives in that tree, so the OID changes when the proof is added. Replaced by `source_blob` plus the pre-proof commit (R-P-2).
- RFC 3161 timestamp tokens: need ASN.1 and certificate-chain parsing (new dependencies) and a TSA at sign time. Deferred (D8); clock use is opt-in (R-P-8).
- Murmur3 namespace hashing: not needed once bindings are explicit (B4).
- Break-glass downgrade of exit 1 to exit 2: a separate ruling (R-P-7).
- `strict_projection` closed-world mode: an egress (K1) change, separate document.

## K. Rulings (2026-10-15 20:00) and open items

Ratified and frozen: D1 level 0 blake3 digest only (signatures deferred to
INVAR-PROOF-002); D2 acceptance state in invar.toml, invar.lock stays a generated
contract artifact; D3 mandatory freshness anchor; D4 explicit `[[proof.import]]`
blocks; D5 egress only in v1; D6 proof at `invar.proof` or `.invar/proofs/<name>.proof`;
D7 opt-in `max_age_days`; D8 RFC 3161 deferred; D9 Class 3 grants for proof.rs,
lock.rs, check.rs and config.rs. Ruling of 20:05 on the volunteered items: the rule
fingerprint is ratified for this spec (R-P-10).

Open items I1 to I3 were RULED 2026-10-18; all are closed:
I1 APPROVED. `proof` (file location) and `fields` (the relied-on names) are locked
into the `[[proof.import]]` schema as required keys beside the four ruled ones.
I1b APPROVED. main.rs is granted Class 3 scope for parsing `--emit-proof` and
`--proof`; section I reflects the grant.
I2 APPROVED. blake3 is the standard for every INVAR cryptographic primitive; no
SHA-256 crate enters the tree (the zero-new-dependency stop condition holds). I2b
APPROVED. `rule_fingerprint` stays in proof files only; lock files keep their exact
bytes. A future lock-side fingerprint is a separate ruling with its own migration
fixtures.
I3 RULED, kept separated. Plain `invar check` never evaluates `[[proof.import]]`
tables; proof verification is strictly behind `invar check --proof`, so standard
checks stay fast, local and decoupled. The one-line NOTE floated in r2 is NOT
adopted: plain check output stays byte-identical on every repository, with or
without proof imports. Mutation 16 and fixture PRF-N3 enforce this permanently.

Gate 0 was initialized under these rulings. The hash baseline is
docs/architecture/GATE0-PROOF-LEDGER.md, taken at the clean tree of commit 8861f63
(slice 2b REV1 landed). A failure against that ledger at any later gate stops the
arc for ruling.

## L. Stop conditions

Stop and report on: any change to an exit code's meaning or a new exit code; any new dependency or network capability; a lock grammar change not ruled; a slice 1, 2 or 2b output change; a Class 1 hash change; a mutation kill that is a build or load failure; verification over 2 ms on the fixture; any need to infer a binding; any claim in docs of freshness beyond B1.

## M. Implementation Evidence Ledger (appended at Gate 5; sections 0 to L are unchanged)

Status: IMPLEMENTED against revision 3 (rulings D1 to D9, I1 to I3). What is
proven is what the fixtures and mutations below state about this engine on
synthetic repositories, nothing more: no adoption, capability or real-repository
claim is made, and no upstream/downstream pair has been probed (section H).

### M1. Gate results

| Gate | Result |
|---|---|
| 0 baseline | PASS: GATE0-PROOF-LEDGER.md (tree 8861f63); re-verified after implementation, every Class 1 and untouched Class 3 hash byte-identical |
| 1 fixtures first | 32 fixtures written before any engine code; against the unchanged engine 28 failed for their declared reasons (27 by the missing flags, which clap rejects with exit 2 today; M4 by the absent module; M6 by its declared stub) and 4 invariance controls passed (N1, N2, N3, M3) |
| 2 implementation | src/proof.rs (new, self-contained), main.rs (flag dispatch, granted I1b), lib.rs (module registration); lock.rs, check.rs and config.rs granted by D9 but NOT touched (their D9 duties fit proof.rs and the dispatch) |
| 3 mutations | PASS: 20 of 20 CAUGHT by named assertion failure, control green, none SURVIVED, BUILD-FAILED or APPLY-FAILED (scripts/proof/mutate.sh). 01 digest-not-verified, 02 missing-field-held, 03 extra-field-alarms, 04 scope-check-removed, 05 ambiguity-check-removed, 06 epoch-rollback-accepted, 07 fork-accepted, 07b chain-break-accepted, 08 unanchored-returns-holds, 09 emit-with-unlocked-drift, 10 proof-path-returns-exit-2, 11 canonical-depends-on-field-order, 12 clock-read-by-default, 13 emit-overwrites-failing-proof, 14 anchor-ignored-at-emit, 15 fingerprint-changes-verdict, 16 plain-check-evaluates-imports, 17 unknown-binding-key-ignored, 18 out-path-anywhere, 19 missing-proof-returns-holds |
| 4 regression | 219 of 219 tests green (the 182 landed tests, the 33 proof fixtures, and 4 proof unit tests including the R-P-9 budget check), zero warnings; slice 1, 2 and 2b byte-identity suites green unchanged; dependency set asserted frozen (PRF-M3); release verification measured under 2 ms per proof (budget test, 200 iterations) |
| 5 documentation | this section, STATUS.md entry, EXPOSURE-LOG.md note |

### M2. Interpretations applied (for ratification)

- P-INT-1 `[producer] repo` reads an optional `repo` key on the egress block
  ("declared name; NOT verified"); absent, it is written as "none".
- P-INT-2 R-P-4's "+1 per emit" is read as +1 per emitted CHANGE: an emit at an
  unchanged source state is a no-op that leaves the file byte-identical,
  because R-P-1 (PRF-M1) is unsatisfiable otherwise.
- P-INT-3 With `--proof` and at least one [[proof.import]] table, an invar.toml
  holding zero [[invariant]] blocks is a pure consumer repository, not an
  error; plain `check` keeps today's behavior byte for byte (PRF-N3).
- P-INT-4 Exit precedence when `--proof` rides with ordinary contracts:
  3 over 2 over 1 over 4 over 0 (exit 2 never arises from proofs; a ledger
  break on the plain side that returns early keeps its own path; the
  combination of a ledger break with proof imports is untested and out of the
  fixture set).
- P-INT-5 For a directory binding, parse errors (step 1) surface before
  contract matching, so the frozen file-binding order of R-P-6 is preserved
  and directory resolution stays deterministic (sorted file names).

### M3. Corrections and additions made during the arc (stated, not hidden)

- Fixture PRF-N2 first asserted whole-output byte-identity of `invar init`
  with and without an invar.proof file; the walk honestly counts the extra
  file in `files_scanned`. The fixture now normalizes that one counter line
  and asserts the proposal set is byte-identical and the proof file is never
  mentioned. Fixture error; the engine was right.
- Fixture PRF-E6 was added at Gate 3 so mutation 13 (emit overwrites a proof
  that fails verification) dies by a named assertion; the behavior is R-P-4's
  own sentence. Fixture count 31 to 32 (33 with the budget unit check).
- Spec mutation 7 (fork accepted) is implemented as two mutants, 07 and 07b,
  so the equal-epoch fork arm and the broken-chain arm each die
  independently: 20 mutants cover the 19 ruled ones.
- PRF-U7 is exercised through the binding side of the kind rule (a
  contract_id naming a non-egress kind), because a validly sealed non-egress
  proof cannot exist: emit refuses to author one, and a hand-built one fails
  the digest first. The proof-side arm of the same rule is implemented and
  covered by the binding-side fixture only.

### M4. The honest bounds, restated after implementation

B1 holds: freshness is judged only against the consumer's recorded
accepted_epoch and accepted_digest; nothing claims "the provider still emits
this today". B2 holds: the blake3 digest proves the file was not edited after
emission, not who emitted it; level 1 signatures remain INVAR-PROOF-002. B3
holds: depth-1 field names only, runtime serialization is not verified. B4
holds: `invar init` proposes nothing from proof files (PRF-N2) and no binding
is ever inferred.

### M5. Files changed

src/proof.rs (new), src/main.rs (Class 3, granted I1b), src/lib.rs (Class 4),
tests/proof_test.rs (Class 2), scripts/proof/ (Class 2). lock.rs, check.rs,
config.rs: granted, untouched, byte-identical to the Gate 0 baseline.
