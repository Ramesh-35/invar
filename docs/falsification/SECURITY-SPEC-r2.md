# SECURITY-1: INVAR's threat model, measured (S0), and reuse authenticity (S1)

Status: r2 (written before any S2 code: one fixture added, one declared
killer corrected; section J). r1 is preserved as SECURITY-SPEC-r1.md with
SECURITY-SPEC-r1.frozen. r1 header follows.
r1: r0 (S0 measured, S1 implemented) is preserved as
SECURITY-SPEC-r0.md with SECURITY-SPEC-r0.frozen. r1 adds S2 (sections H
and I) under the ruling of 2026-10-25 (SEC-D4 to SEC-D8 ratified), frozen
with tests/security_s2_test.rs BEFORE any S2 code. r0 header follows.
r0: S0 (threat model) for founder ruling on S2 onward. S1
implemented under the ruling of 2026-10-25 (SEC-D1 S1 first, SEC-D2 an
unsigned lock is not reused locally, SEC-D3 claims re-anchored). Baseline:
the self-audit tree (a4aa356 plus AGENT-1 P2, release binary 14e0dce4e181;
its commit waits on native verification). S1 binary: 01f9ae96b7d2. Ids are
sha256 prefixes. Sandbox: Linux x86_64. Every status below comes from a
runnable probe in scripts/security/; none is asserted.

Process note, stated: S1's code was written before its fixtures were
frozen (the mandate authorized both together). The fixtures were then run
against the pre-S1 engine to prove they fail there for the right reason
(section E), and the mutation gate was run against them as frozen.

## A. What INVAR claims, re-anchored (SEC-D3)

- Network: `invar check` is local by default. `check --ci` fetches
  `github:` artifacts over HTTPS through the system `curl`, sending
  `GITHUB_TOKEN` when set. It is not "strictly local".
- Evidence class: the matching grammars are Class 4 text-pattern
  heuristics, evidence-checked (fixtures, mutation gates, corpus), never
  "mathematically grounded".
- Evasion: code written to avoid the grammars yields a silent HOLDS, not a
  NOT EVALUATED. That is a permanent limitation of a pattern engine,
  stated, never hidden. Abstention covers what INVAR sees and cannot
  decide; it does not cover what INVAR does not see.
- Agent handshakes and zero-knowledge contract proofs are research
  backlog, not plan and not claim.

## B. S0: seven threat paths, measured

| # | Path | Probe | Measured status (2026-10-25, sandbox) |
|---|---|---|---|
| 1 | Forged reuse records (verdict cache; unsigned invar.lock `held_input`) | reuse-forgery.sh | VULNERABLE before S1: both forgeries turned a real VIOLATED (exit 1) into HOLDS (exit 0) on a local run; `--ci` was safe. FIXED by S1 (section C): every forged line exits 1. |
| 2 | Pathological input (crash or stall) | robustness.py | No finding in 300 generated cases (random bytes, deep nesting, 200 KB lines, unterminated literals, NUL/ESC, invalid UTF-8, 7 languages): 0 crashes, 0 runs over 2 s, slowest 0.02 s. 199 inputs were refused as unreadable (exit 3, never a verdict), 101 were scanned to a verdict (0/1/4). Coverage is stated, not exhaustive. |
| 3 | Token follows a cross-host redirect | token-redirect.py | MITIGATED on this host by curl 8.5.0: the token reached the first host only. INVAR's production flags also forbid any non-HTTPS hop. The protection belongs to the system curl (versions before 7.58 forward the header), so the probe must run on every host, macOS included. |
| 4 | Control bytes injected into terminal output | terminal-injection.py | VULNERABLE: a field name holding ESC, BEL and CR from the repository is printed raw in the plain-text witness line (the residue list escapes it, the witness line does not). A crafted name can retitle the terminal, clear the screen, and overwrite the VIOLATED line with a carriage return. |
| 5 | Directory walk over symbolic-link loops (CLI `dependents --locks`, MCP `dependents`, and `check` with a `[dependents] locks` key) | walk-loop.sh | VULNERABLE: three self-referencing links make the walk branch exponentially; still running when killed at 20 s. A repository's own invar.toml can trigger it in a plain `invar check`, after the verdict is printed (CI or agent loop never returns). The MCP tool also accepts any absolute path. |
| 6 | Programs run by INVAR | source inventory | `invar gate` runs the command from its own argv (the operator's choice, not the repository's). `invar run` executes the program named by the composite config's `bin_path` (default `semgrep`): a hostile config executes code when an operator runs `invar run` on it. `backtest` and `proof` run `git log`, `show`, `rev-parse`, `hash-object` in the target repository; not yet measured against a hostile `.git/config`. |
| 7 | Supply chain of INVAR itself | source inventory | Release archives carry SHA-256 checksums and SSH signatures (namespace `invar-release`), verified by the Action before use; publishing refuses without the key. 7 direct dependencies, 57 crates in Cargo.lock; no dependency audit in CI; reproducibility not verified. |

## C. S1: reuse authenticity (implemented)

Rules:
- S1-1, lock records. A local `invar check` reuses an `invar.lock` hold
  record only when `--lock-signers` verified the lock's signature in that
  run, the rule `--ci` already followed. An unsigned lock is editable by
  anyone who can edit the repository, so its records are not evidence: the
  contract is evaluated.
- S1-2, cache records. The local cache (`.invar/cache/verdicts.json`,
  format 2) carries, per record, a keyed BLAKE3 MAC over the contract id
  and the record, under a per-user key `$HOME/.invar/key` (32 bytes from
  the operating system's random source, created owner-only, never inside
  a repository). A record whose MAC does not verify is no record. Format 1
  (unkeyed) caches are never read.
- S1-3, no key, no reuse. Without a usable home or random source, nothing
  is cached and nothing is reused; every contract is evaluated.
- Unchanged: `--ci` never reads or writes the cache; `--no-reuse`,
  receipts and samples reuse nothing; exit codes and the verdict of every
  contract.

Not claimed: protection against an attacker who can read the user's home
(they hold the key, and the user's account); a signed lock is only as
trustworthy as the signers file the operator passes.

## D. Measured movement and cost

- Corpus (122 cases, `--ci`): byte-identical, as pre-registered.
- Round trips (EVAL-007/008: `lock`, then two local checks, on three real
  repositories): every contract's verdict identical in all 12 check
  outputs. check1 and check2 now differ in reuse wording: the first local
  check after `lock` evaluates instead of reusing the unsigned lock, and
  its output is byte-identical to the pre-S1 engine's `check --no-reuse`
  on all three repositories (timestamps normalized).
- Latency (p50, 15 runs): first local check after `lock` 9.5 to 63.2 ms
  (it evaluates); later checks from the keyed cache 4.5 to 13.3 ms, against
  3.7 to 8.4 ms for pre-S1 reuse (pi: 8.4 to 13.3 ms).

## E. Fixtures

New, tests/security_test.rs (9), each with its own HOME:

| Fixture | Asserts | Pre-S1 engine |
|---|---|---|
| s1_f01 | a forged keyed cache record is not reused; exit 1 | FAIL |
| s1_f02 | a forged unsigned lock record is not reused; exit 1 | FAIL |
| s1_f03 | an honest unsigned lock is evaluated, then the keyed cache is reused | FAIL |
| s1_f04 | a signed lock IS reused locally (`--lock-signers`) | pass (guard) |
| s1_f05 | a format-1 cache record copying an honest record's real engine, block and entry, with valid hashes of broken inputs, is not read; exit 1 | FAIL |
| s1_f06 | the key is created in HOME, 32 bytes, never in the repository | FAIL |
| s1_f07 | the key file is owner-only (0600) | FAIL |
| s1_f08 | without a usable HOME nothing is reused | FAIL |
| s1_f09 | a record keyed by another user does not verify | FAIL |

Revised under SEC-D2, prior files kept as docs/evidence/security/*.pre-s1.rs:
- tests/check_incr1_test.rs (CHECK-INCR-1, ruling I-D4): 11 fixtures
  asserted that a local check reuses records from an unsigned lock (f1,
  f3, f4, f5, f6, f8, f9, f15, f16, f19, f20). Their property is WHICH
  records may be reused once trusted, so the shared `invar` helper now,
  for a local `check` against a lock holding records, signs the lock with
  a per-workspace key and passes `--lock-signers`, and sets HOME to the
  workspace. No fixture body changed (30 lines, all in the helper). f12
  (`--ci`) and f17 (no records, baseline output) are untouched in effect.
- tests/gate_test.rs g_m1 (determinism of the gate's lines across runs):
  one warm-up run first, because the first run now evaluates and warms the
  keyed cache; runs two and three are compared.

## F. Mutation operators (S1)

| Operator | Change | Declared killer |
|---|---|---|
| S1-M1 | unsigned lock records reused locally (the pre-S1 rule) | s1_f02 |
| S1-M2 | cache MAC not verified on load | s1_f01 |
| S1-M3 | key read from the working directory instead of HOME | s1_f06 |
| S1-M4 | key file created readable by others | s1_f07 |
| S1-M5 | without a key, cache records read unverified | s1_f08 |
| S1-M6 | MAC computed with a fixed key, not the user's | s1_f09 |

## G. Decision points (S2 onward)

- SEC-D4 (path 4, terminal injection). Escape every control character in
  repository-derived text in plain-text output (residue, witness, notes,
  statements, ids) as `\u{..}`, as the residue list already does
  (recommended: corpus output unchanged, measured to carry no control
  bytes).
- SEC-D5 (path 5, walk). The walk does not follow symbolic links and has
  a depth and file-count bound; the MCP `dependents` tool accepts only a
  path inside the configuration's directory (recommended).
- SEC-D6 (path 6). `invar run` executes `bin_path` only when given on the
  command line or explicitly allowed by flag; measure `git` against a
  hostile `.git/config` before ruling on it.
- SEC-D7 (path 7). Add a dependency audit to CI and a reproducible-build
  check of release archives.
- SEC-D8 (path 3). Run the token probe on the founder's Mac and record the
  curl version; refuse `--ci` fetches with a curl older than 7.58.

## H. S2 rules (SEC-D4, SEC-D5)

SEC-D4, terminal neutralization, two layers:
- S2-1, values. The verdict renderer (`outcome_lines`, shared by `check`
  and `demo`) escapes every repository-derived value it prints (contract
  id and kind, residue, witness fields, statements, notes, waiver text,
  fetched references): each control character (newline, tab and CR
  included) and each bidirectional override or isolate (U+202A to U+202E,
  U+2066 to U+2069) becomes its Rust escape (`\n`, `\r`, `\u{1b}`,
  `\u{202e}`). Nothing else changes, so a value with nothing to escape
  prints byte-identically. A value cannot forge an output line.
- S2-2, sinks. Every terminal print in `main.rs`, `gate.rs` and
  `dependents.rs` (`println!`, `print!`, `eprintln!`, `eprint!`, shadowed by
  crate-local macros) neutralizes the same characters except newline and
  tab, which the engine's own layout uses. This covers every command
  (`lock`, `coverage`, `dependents`, errors), not only the verdict block.
- S2-3, markdown. The PR report's code spans escape the same characters
  (they already replaced newline, CR and tab with a space).
- Not covered: JSON and SARIF output (JSON escapes control characters;
  they are read by programs, not terminals); a newline inside a value in
  non-verdict output is neutralized as a line break, not escaped (it can
  start a new line but cannot carry a terminal escape).

SEC-D5, walk and path:
- S2-4. The lock walk (`dependents::locks_under`, used by `dependents
  --locks`, the MCP `dependents` tool and `check`'s `[dependents] locks`
  section) never follows a symbolic link (directory or file), stops with an
  error past depth 64 or past 200,000 entries visited, and states the
  bound it hit. Measured on six real repositories: at most 29,919 entries,
  depth 12; 7 symbolic links (claude-mem) are no longer followed.
- S2-5. The MCP `dependents` tool resolves `locks` (absolute or
  relative, symbolic links resolved) and refuses a path outside the
  configuration's directory: `locks path is outside the configuration
  directory`.
- `check`'s `[dependents]` section stays informational: a bound hit is a
  stated note, never a change of verdict or exit code.

SEC-D6 to SEC-D8 are separate slices after S2 (run restriction and git
measurement; dependency audit and reproducible build in CI; curl version
floor after the Mac token probe).

## I. S2 fixtures and operators (frozen before code)

tests/security_s2_test.rs, 13 fixtures. On the S1 engine (with a
throwaway stub of `locks_under` keeping today's walk, so the file builds)
12 FAIL, each on raw terminal-active bytes, an unbounded walk or the path
check, and 1 passes, the guard s2_f07 (ordinary output byte-identical).
Construction notes: the consumer's literal carries the raw characters
(INVAR does not decode escapes in literals or in invar.toml strings); a raw
newline cannot reach a witness through a source read, so s2_f02 drives the
renderer directly with a constructed outcome.

| Fixture | Asserts |
|---|---|
| s2_f01 | ESC, BEL, CR in a witness are escaped (`x\u{1b}]0;...\rfake read at c.js:1`) |
| s2_f02 | a newline in id, residue, witness, note or statement forges no line |
| s2_f03 | a raw ESC in a contract id on a HOLDS run is neutralized |
| s2_f04 | a bidirectional override is escaped (`\u{202e}`) |
| s2_f05 | `lock` output is neutralized (sink) |
| s2_f06 | `dependents` output (query and bound field) is neutralized (sink) |
| s2_f07 | guard: ordinary output byte-identical |
| s2_f08 | the markdown report carries no terminal-active character |
| s2_f09 | `dependents` over a symbolic-link loop returns within 5 s |
| s2_f10 | a repository's `[dependents] locks` over a loop cannot hang `check`; verdict unchanged |
| s2_f11 | a lock reachable only through a link is not read |
| s2_f12 | the walk errors past its depth bound and past its entry bound, and finds within bounds |
| s2_f13 | MCP `dependents` refuses an absolute path and a `../` escape outside the configuration directory, and allows one inside |

| Operator | Change | Declared killer |
|---|---|---|
| S2-M1 | `main.rs` sink macros removed | s2_f05 |
| S2-M2 | renderer value escaping removed (identity) | s2_f02 |
| S2-M3 | bidirectional characters left out of the escape set | s2_f04 |
| S2-M4 | markdown code spans not escaped | s2_f08 |
| S2-M5 | the walk follows symbolic links | s2_f11 |
| S2-M6 | depth bound removed | s2_f12 |
| S2-M7 | entry bound removed | s2_f12 |
| S2-M8 | MCP path restriction removed | s2_f13 |
| S2-M9 | `dependents.rs` sink macros removed | s2_f14 (r2; r1 declared s2_f06, see J) |

## J. r2 (before any S2 code)

Checking the operator table against the print sites, before writing code:
`dependents` output reaches the terminal through `main.rs` (`print!` of
`render_text`), so s2_f06 exercises the main.rs sink, not `dependents.rs`.
`dependents.rs` prints only `skipping <path>: <error>` for a lock it cannot
read, and that path is repository-controlled. r1's declared killer for
S2-M9 could therefore never kill it. Added s2_f14 (a lock that cannot be
read, under a directory whose name carries ESC, BEL and U+202E); it fails
on the S1 engine with raw bytes and is S2-M9's declared killer. s2_f06
stays as the guard of `dependents` output through main.rs. Fixtures: 14.
`gate.rs` is also covered by S2-2; it prints the child check's lines, which
are already neutralized, and watched paths in its drift line; it has no
separate fixture or operator in r2, stated.
