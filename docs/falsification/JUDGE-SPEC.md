# JUDGE-1: invar judge, the declared-data-contract verdict (r1, FROZEN)

Ratified 2026-10-28: J-D1, J-D2, J-D3, J-D4, all as drafted. Frozen with
crates/invar-falsify/tests/judge_test.rs and tests/fixtures/judge/ BEFORE
any implementation; the freeze record is JUDGE-SPEC.frozen.

B2 slice (PROGRAM-LEDGER, ratified 2026-10-28). One stateless subcommand
that judges a change to a DECLARED data contract at the moment of change.
Slice 1 contract family: self-describing JSON Schema (Iglu style), chosen
because the B1 corpus supplies pinned pre/post bytes as fixtures.

## Interface

    invar judge --contract <path> [--base <ref>] [--format text|json]

- `--contract`: path to the declared contract file in the repository.
- `--base`: git ref to read the PRE contract from, via `git archive`,
  exactly as SIB-3 reads trust: the base tree, never the PR tree. Default
  `origin/main` in CI; the POST contract is always the working tree.
- Exit codes are the engine's fixed interface: 0 HOLDS, 1 VIOLATED,
  3 error (never a verdict), 4 NOT EVALUATED (stated, never silent).
  2 stays reserved for the ledger.

## Verdict semantics

- HOLDS: every property present in pre is present in post with the same
  `type`. Additions are HOLDS. Description and metadata changes are HOLDS.
- VIOLATED: a property removed, or its `type` changed. The diff is
  RECURSIVE over nested `properties`, witnesses carry the dotted path
  (`contexts.deviceMemory`), so a nested removal can never pass silently.
- NOT EVALUATED, with reason code, when judging is impossible:
  - `new_contract`: no contract at base; nothing was promised before.
  - `unreadable_base`: base ref or file unreadable (CI must treat the
    job per B-D2 r1 discipline; locally this is exit 4 with the reason).
  - `unparsable`: pre or post bytes are not valid JSON Schema.
- Scope statement, in every receipt: judged = property presence and
  `type`, recursive. Not judged in r0, carried as the receipt field
  `unjudged_constraints[]` = ["enum", "pattern", "maximum", "required"].
  A skip is never a pass: the not-judged classes are STATED, not omitted.

## Witness payload (JSON format)

Per violation: `path` (dotted field), `kind` (removed|retyped),
`pre_type`, `post_type`, `contract` (file path), `pre_line`, `post_line`
(line of the field in each byte stream; post_line null when removed).
The receipt is the existing signed-receipt machinery, the same key
anchor as v0.1.0 release signing verifies against. SARIF: deferred,
recorded as a non-goal of r0.

## Fixtures (frozen with this spec; bytes from the B1 corpus)

- jf_f01: b1-001 pre/post (media_player 1-0-0 vs 2-0-0) -> VIOLATED,
  removed percentProgress, removed isLive. Exit 1.
- jf_f02: b1-002 pre/post (browser_context) -> VIOLATED, retyped
  deviceMemory integer|null -> number|null. Exit 1.
- jf_f03: identical pre/post -> HOLDS. Exit 0.
- jf_f04: post adds a property, removes none -> HOLDS. Exit 0.
- jf_f05: post bytes `[[[` -> NOT EVALUATED unparsable. Exit 4.
- jf_f06: no contract at base -> NOT EVALUATED new_contract. Exit 4.
- jf_f07: NESTED removal only (payload.user.zipcode) -> VIOLATED with
  dotted path witness. Exit 1.
- json surface: b1-002 with --format json -> verdict, witnesses (path,
  kind, pre_type, post_type, pre_line) and unjudged_constraints[].

## Mutation operators (each must be killed by a named fixture)

- JM-M1 drop removal detection -> killed by jf_f01.
- JM-M2 drop retype detection -> killed by jf_f02.
- JM-M3 unparsable input treated as HOLDS -> killed by jf_f05.
- JM-M4 read pre from working tree instead of base ref -> killed by
  jf_f01: the working tree holds ONLY post bytes, so the mutant compares
  post to post, answers HOLDS, and fails the exit-1 assertion.
- JM-M5 recursion removed, top-level only -> killed by jf_f07.
- JM-M6 missing base contract treated as HOLDS -> killed by jf_f06.

## Voice (frozen with the semantics; the human surface is part of the contract)

The text format speaks to a person, not a parser. Rules, each testable:
- Every VIOLATED line is a sentence naming the field (dotted path), what
  happened to it (removed, retyped), and both types when retyped. No
  codes without words.
- Every NOT EVALUATED carries its reason code AND a plain sentence; the
  pinned phrases, asserted by fixtures: new_contract -> "nothing was
  promised before"; unparsable names the file it could not read.
- No exclamation marks, no blame, no filler. The tone of a careful
  colleague who read the diff: what broke, where, what to do next.
- The JSON format carries the same facts for machines and agents, byte
  for byte stable ordering, so an AI tool in the loop reads the same
  truth a human does.

## Decision points, RATIFIED 2026-10-28

- J-D1 scope: r0 judges the contract's self-diff only (producer side,
  single player). Binding consumer reads via signed locks is JUDGE-2,
  reusing SIB-1 machinery unchanged.
- J-D2 new contract at base: NOT EVALUATED new_contract (exit 4), not
  HOLDS, because nothing can hold against no prior promise.
- J-D3 output: text and JSON in r0; SARIF deferred.
- J-D4 breaking set: presence and type, recursive, with the not-judged
  constraint classes stated in every receipt rather than silently passed.

## r2 amendment (2026-10-30), forced by JUDGE-SUBTYPE r1

JUDGE-SUBTYPE r1 judges enum, bounds, required, pattern and format, so
the r0 scope statement in this spec ("not judged in r0: enum, pattern,
maximum, required") is superseded. The unjudged list is now `$ref`,
`allOf`, `anyOf`, `oneOf`, `items`, defined in JUDGE-SUBTYPE-SPEC.md.
`$ref` enters the list because r0 never followed references and never
said so: a gap found while writing r1, stated from r2 on. The one
assertion in judge_test.rs that encoded the r0 list is updated; every
other r1 assertion is unchanged. The r1 freeze record is kept and the r2
hashes are appended to JUDGE-SPEC.frozen.
