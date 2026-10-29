# COMPAT-1: invar compat, the coping layer compiled from the declared diff (r0 DRAFT, NOT FROZEN)

H2 (docs/research/BRIEF4-REPORT-coping-layer.md, founder elevation in
docs/research/H2-ELEVATION-three-breakthroughs.md). Gated in BACKLOG.md:
no code before B2 clears. This draft exists so ratification and fixture
freezing are the only steps between a fired gate and the first commit.

## Job, one sentence

Turn a producer's declared change into the exact coping code one
consumer needs, plus a signed statement of every change it could not
translate, each carrying an executable falsification test harness
instead of a bare refusal.

## Interface (draft)

    invar compat --renames <declared-diff> --lock invar.lock
                 --emit <target>[,<target>] --out <dir> [--receipt <path>]

- `--renames`: the producer's declared rename ledger. r0 input family:
  OpenTelemetry semantic-conventions schema files (attribute_map blocks),
  because the B1 corpus supplies pinned bytes and the Java agent 3.0
  cutover is the dated forcing event.
- `--lock`: the consumer's committed invar.lock; ONLY fields the lock
  proves are read produce output. No lock, no generation: stated.
- Exit codes: 0 generated (including a generation whose every item is a
  harness), 3 error, 4 nothing bound by the lock intersects the diff
  (stated, never silent).

## Outputs, three files per run (founder's Breakthrough 1 and 3)

1. The coping artifact for 1-to-1 bound renames, per `--emit` target.
2. The falsification harness for every bound change that is NOT 1-to-1
   (merge, value change, unit change, conditional rename), as unit-test
   fixtures asserting the input-to-output transformation the hand-written
   or agent-written bridge must satisfy. A NOT COVERED without a harness
   does not exist in this design.
3. The signed receipt: every generated artifact's hash, every harness,
   every reason code, verifiable with `invar verify-receipt`.

## Decision points for ratification (C-D1 to C-D5)

- C-D1 r0 emit targets: OTTL and JSON first; TypeScript and Go in r1?
  Or all four in r0 as Brief 4 names them?
- C-D2 harness language: the consumer repository's own primary language
  (detected), with one portable fallback. Which fallback?
- C-D3 the agent surface (Breakthrough 2): `--format json` of the same
  run for MCP and hook consumption, r0 or r1?
- C-D4 does `invar judge` gain awareness of generated artifacts (the
  harness runs under judge in CI), or does the existing test runner own
  that? Brief 4 assumes judge reads the generated file unchanged.
- C-D5 fixture source: b1-003/004/005 pinned bytes for renames; the
  Java agent 3.0 set (db.system to db.system.name, mssql to
  microsoft.sql_server, ms to s, code.namespace plus code.function to
  code.function.name) for the harness cases. Freeze both?

## Pre-registered kill numbers (Brief 4, carried verbatim)

Fewer than 3 external events (merged INVAR-generated artifact, CI run,
or third-party receipt verification) from at least 2 independent
organizations by day 14 after release kills H2 as a product direction.
Fewer than 1 of 10 platform teams on the Java agent 3.0 cutover agreeing
to a paid or letter-of-intent pilot by day 14 records H2 as having no
willingness to pay.
