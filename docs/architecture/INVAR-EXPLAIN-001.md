# INVAR-EXPLAIN-001: the code registry and `invar explain <CODE>`

Status: DRAFT r1 for ruling. Implementation NOT authorized. Parents: ruling of
2026-10-15 20:05 (V2, `invar explain`, ratified for backlog) and the directive of
2026-10-18 ordering this draft. Prepared to the 86427 and slice 2 standard:
numbered requirements with falsifiable acceptance, countermodels, fixtures with
declared outcomes before code, mutation set, gates 0 to 5.

## 0. Purpose and honest bounds

Every abstention the engine prints is a refusal with a reason. Today the reason
text is the only explanation, and the authoritative definitions live in frozen
spec documents a user of the binary never sees. `invar explain <CODE>` prints,
for any code the engine can emit, its definition, the exact condition that fires
it, and the sanctioned next action, from one registry inside the binary that the
tests also read, so the documentation cannot drift from the engine. This is the
diagnostic surface that keeps strict abstention from feeling like unexplained
friction.

Bounds, stated before any code:

B1. explain is documentation, not evaluation. It reads no repository, no
manifest, no lock and no environment; its output depends only on the binary and
the argument. It can never return a verdict about the user's code.
B2. The registry describes THIS binary's rules. It is version-pinned by
construction and never claims what another version does.
B3. Class 1 files cannot be edited, so emission sites cannot be refactored to
reference the registry. The registry is therefore a second copy of each code
string, and the no-drift guarantee comes from a consistency test that extracts
emission sites from the source text at test time (R-X-6), not from shared
constants. This is stated plainly rather than hidden.
B4. Zero tokens, zero network, no clock, sub-millisecond.

Inventory at the time of drafting (tree 8861f63): 19 distinct abstention codes
are emitted across codepairs.rs, valuepairs.rs, strings.rs and init.rs (H2,
H2-ENTRY, H2-MEMBER, SLICE-SCOPE, UNRESOLVED, UNREADABLE, UNREPRESENTABLE,
BARREL2, R6, AMBIG-PACKAGE, AMBIG-SHAPE, AMBIG-EGRESS, AMBIG-RECEIVER,
UNSUPPORTED-DYNAMIC, UNSUPPORTED-STRING-DYNAMIC, UNSUPPORTED-ORM-SOURCE,
NON-DISTINCTIVE-KEY, VERSION-FAMILY, TSC-GOVERNED), plus the 5 exit codes of the
frozen ISA. The 7 proof codes of INVAR-PROOF-001 join the registry in the same
change that lands the proof engine, never before.

## A. Scope

In scope: the registry; the `explain` subcommand; the drift test; fixtures and
mutations. Out of scope, named so nobody assumes them: any change to emission
behavior or reason text; any MCP or JSON-RPC surface; machine-readable output
(decision D-E4 defers `--json`); any new dependency; any Class 1 edit.

## B. Numbered requirements

R-X-1 Registry. One Rust table, each entry: code, kind (abstention, error or
exit), scope (the slice or command that emits it), condition (one sentence, the
exact firing condition as the frozen specs define it), action (the sanctioned
next step for a user who hits it). Sorted by code; a duplicate code is a test
failure (CM-X-5).

R-X-2 Entry lookup. `invar explain <CODE>` with a known code prints exactly five
lines (code and kind; scope; condition; action; the spec document that defines
it) and returns exit 0. Code matching is exact and case-sensitive, as printed by
the engine.

R-X-3 Index. `invar explain` with no argument prints one line per entry (code
and condition), sorted by code, and returns exit 0.

R-X-4 Exit numerals. `invar explain 0` through `invar explain 4` print the
frozen ISA meaning of that exit code (0 HOLDS, 1 VIOLATED, 2 ledger break, 3
error, 4 skipped or not evaluated). Any other numeral is an unknown code.

R-X-5 Unknown code. Exit 3, printing "unknown code <arg>" and the sorted list of
known codes. No fuzzy matching, no suggestion heuristics: deterministic output
over cleverness.

R-X-6 Drift test, both directions, in the ordinary test suite: (a) every string
that appears at an emission site in src/ (extracted at test time by scanning the
crate sources for the literal `code: "<NAME>"` pattern and the documented exit
sites, read from CARGO_MANIFEST_DIR) has a registry entry; (b) every registry
entry of kind abstention appears at at least one emission site. The test reads
sources and never edits them; an emission form the scan cannot extract is a stop
condition, not a reason to weaken the scan.

R-X-7 Determinism. Output is byte-identical across runs, working directories,
locale environment variables and clock states, because nothing outside the
binary and the argument is read.

R-X-8 ISA invariance. No new exit code and no change to any existing code's
meaning. explain itself returns 0 or 3 only, the authoring-command convention.

R-X-9 Independence. No output of init, lock, check or fix changes by one byte.
Budget: the subcommand answers in under 1 ms; the drift test adds no new
dependency (std only).

## C. Countermodels (defined before fixtures and code)

CM-X-1 unknown code exiting 0. Declared: exit 3.
CM-X-2 a code added to the engine without a registry entry. Declared: drift test
fails naming the code.
CM-X-3 a registry entry whose code no emission site produces. Declared: drift
test fails naming the entry.
CM-X-4 explain run in a directory with a malformed invar.toml and in an empty
directory. Declared: byte-identical output (B1).
CM-X-5 two registry entries for one code. Declared: test failure.
CM-X-6 output varying under LC_ALL or TZ changes. Declared: byte-identical.
CM-X-7 `invar explain 2` must state that exit 2 is the ledger break and is never
returned by proof verification (the ISA split ruling), verbatim enough that the
fixture can assert it.

## D. Fixtures (declared outcomes, before any code)

X-F1 `explain TSC-GOVERNED` prints the five-line entry, exit 0; condition names
both emitters (K2 type-governed consumption, K3 keyof-constrained lists).
X-F2 `explain` with no argument prints the sorted index; every inventoried code
appears exactly once; 24 lines at the current inventory (19 plus 5).
X-F3 unknown code (`explain NOT-A-CODE`): exit 3, output lists the known codes.
X-F4 `explain 0`..`explain 4` each exit 0 with the ISA meaning; `explain 5` is
unknown (exit 3); CM-X-7 asserted on `explain 2`.
X-F5 two runs byte-identical; run in an empty dir and in a dir with malformed
invar.toml: byte-identical (CM-X-4).
X-F6 the drift test passes on the landed tree and fails when either direction is
broken (asserted by the mutation harness, not by editing sources in a fixture).
X-F7 metamorphic: LC_ALL=C, LC_ALL=en_US.UTF-8 and TZ=UTC+9 leave every explain
output byte-identical (CM-X-6).
X-M1 every slice 1, 2, 2b and REV1 fixture and F01 output byte-identical (R-X-9).

## E. Mutation set (CAUGHT / SURVIVED / BUILD-FAILED; the last two are not kills)

1 registry entry removed (drift direction a). 2 unknown code exits 0. 3 an
emission site's code replaced by one not in the registry (drift direction a
catches on the new code, direction b on the orphaned entry). 4 index left
unsorted. 5 explain reads invar.toml when present. 6 duplicate entry accepted.
7 exit numerals rejected. 8 `explain 2` text changed to allow proof verification
to return 2. 9 drift scan narrowed to one source file. Each must die by a named
assertion failure.

## F. Gates 0 to 5

Gate 0 hash ledger (the GATE0-PROOF-LEDGER.md baseline of tree 8861f63 covers the
same protected set; explain adds no protected file). Gate 1 fixtures written
first as failing tests. Gate 2 implementation to allowed paths. Gate 3 mutation
harness all CAUGHT with control green. Gate 4 full suite green, zero warnings,
byte-identity of every existing output. Gate 5 docs: evidence ledger appended
here, STATUS.md entry.

## G. Path classes

Class 1 (untouched, hash-verified): the GATE0-PROOF-LEDGER.md Class 1 set, plus
lock.rs, check.rs, valuepairs.rs, strings.rs, init.rs (explain does not need
them). Class 2: tests/explain_test.rs, scripts/explain/. Class 3 (REQUESTED, not
assumed): main.rs (subcommand dispatch only). Class 4: src/explain.rs (new,
self-contained: registry plus printing), lib.rs (module registration).

## H. Decision points for ruling

D-E1 Command name `explain` and the five-line entry format of R-X-2.
D-E2 Exit numerals accepted as arguments (recommended: yes, R-X-4).
D-E3 Unknown-code behavior: exit 3 plus the sorted known-code list (recommended)
versus a nearest-name suggestion (rejected as cleverness; a wrong suggestion in a
trust tool costs more than no suggestion).
D-E4 `--json` emitting the registry machine-readably: serde_json is already in
the tree, so no new dependency, but it creates a second output contract to
freeze. Recommended: DEFERRED to the MCP surface ruling; v1 is text only.
D-E5 Class 3 grant for main.rs.
D-E6 Whether the registry's condition sentences are normative copies of the spec
documents (recommended) or may paraphrase. Recommended: each condition line ends
with the defining document's name, and Gate 5 includes a read-through check of
every line against its spec.

## I. Stop conditions

Stop and report on: any need to edit a Class 1 file to expose or rename a code;
any emission form the drift scan cannot extract textually; any new dependency;
any output that depends on repository state, locale or clock; any pressure to
make explain evaluate rather than document.

## J. Rulings of 2026-10-18 (recorded)

D-E1 to D-E6 RATIFIED AND FROZEN as drafted, with two flagged interpretations
ratified in the same message: explain returns exit 0 or 3 only (exit 1 is the
VIOLATED verdict and explain issues no verdicts), and the error marker prints
in the house hyphen style as UNKNOWN-EXPLAIN-CODE.

## K. Implementation Evidence Ledger (appended at Gate 5; sections 0 to J are unchanged)

Status: IMPLEMENTED against the ratified rulings. explain is documentation,
never evaluation; nothing here claims anything about any repository.

### K1. Gate results

| Gate | Result |
|---|---|
| 0 baseline | the clean tree at the proof landing (2c06442); Class 1, Class 3 and landed-engine hashes re-verified byte-identical after implementation |
| 1 fixtures first | 7 fixtures written before src/explain.rs existed; 6 failed via the unrecognized subcommand (clap exit 2) and X-F5 passed trivially (clap's error is itself state-independent), kept as an invariance control |
| 2 implementation | src/explain.rs (registry of 33 entries plus lookup, new, self-contained), main.rs (subcommand dispatch, granted D-E5), lib.rs (registration); completions/invar.{bash,zsh,fish} amended (K3 below) |
| 3 mutations | 9 of 9 CAUGHT by named assertion, control green, none SURVIVED or BUILD-FAILED; re-run in full after the completions amendment with the same result |
| 4 regression | 229 of 229 tests green, zero warnings; every slice byte-identity suite unchanged; explain output asserted byte-identical across repository states, locales and time zones |
| 5 documentation | this section and the STATUS.md entry |

### K2. Registry facts and divergences from the draft (stated, not hidden)

- The index holds 33 entries, not the draft's 24: the proof engine landed
  first (per the ratified sequencing), adding 8 proof codes, not the 7 the
  draft counted (PROOF-NOT-FOUND joined in spec r2); UNKNOWN-EXPLAIN-CODE is
  itself an entry, so explain can describe its own refusal.
- The 33: 19 engine abstention codes, 8 proof codes, UNKNOWN-EXPLAIN-CODE,
  and the 5 exit numerals.
- The registry is a tuple table on purpose: the drift scan's patterns
  (`code: "` and `abstain("`) can never match the registry itself, so the
  no-drift test cannot be satisfied by its own second copy.
- The drift test is black-box (R-X-6 strengthened): it extracts emission
  sites from the crate sources and compares them against the BINARY's own
  index output in both directions, links no library symbol, and asserts its
  own scan breadth (at least five files, provenance from at least three),
  so narrowing the scan is itself a caught mutation (09).

### K3. Corrections and catches during the arc

- The existing guard test (bare_invocation_and_completions_stay_in_step)
  failed at Gate 4 because the hand-written shell completions lacked the new
  subcommand. The three completion scripts were amended. The same guard
  checks subcommands only, so the proof arc's missing flags (--emit-proof,
  --out on lock; --proof on check) had passed silently; they were added in
  the same amendment and are recorded here as a proof-arc omission caught
  late. The completion files were not in any declared path class; they are
  CLI surface adjacent to main.rs and the change is content-only.
- Per ruling D-E6, each condition line was read against its defining
  document or emission site before freezing; TSC-GOVERNED's entry names both
  of its emitters (K2 typed consumption, K3 keyof lists).
