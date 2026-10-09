# F1: default switch (spec frozen before batch 8 is sourced)

Written 2026-10-09 from batches 1 to 7 only. Batch 8 cases are not sourced
until this file is hashed and recorded. Any rule added after that is disclosed
as post-hoc and does not count toward the batch 8 score.

## Problem

The one miss mechanism that has recurred across held-out batches is a default
version or format switch: the producer keeps the old emission code (a legacy
path), so every field literal the consumer reads is still present at head, and
INVAR reports HOLDS. What actually changed is the value of a selector that
decides which output the consumer gets by default. The four known cases, with
the exact changed text:

| Case | Base | Head |
|---|---|---|
| r1-059 | `empty_with_version("3"...)` | `empty_with_version("4"...)` |
| r1-075 | `defaultLockfileVersion = 2` | `defaultLockfileVersion = 3` |
| r1-099 | `("version", 2)` only | `("version", 2)` and `("version", 3)` |
| r1-100 | `v8format ... defaultValue: false` | `v8format = true` |

## Rule (change-scoped and lock modes only; strict mode unchanged)

### Selector extraction (per line of producer text, both base and head)

A **selector occurrence** is a pair (name, value). Name matching is
case-insensitive. A value literal is an integer, `true`, `false`, or a quoted
string whose content is an integer or `v` plus an integer; values are
normalized (quotes and a leading `v` stripped). Four textual patterns:

1. **Assignment.** An identifier containing `version` or `format`, followed
   (after optional type annotation text on the same line) by `=` or `:` and a
   value literal. Covers `defaultLockfileVersion = 2` and `v8format = true`.
2. **Default-value key.** An identifier containing `version` or `format`,
   with `defaultValue` followed by `:` or `=` and a value literal later on
   the same line. Covers `v8format: schema.boolean({ defaultValue: false })`.
3. **Call argument.** An identifier containing `version` (not `format`,
   which would match `.format(` string formatting), immediately followed by
   `(` and a value literal as the first argument. Covers
   `empty_with_version("3".to_string())`.
4. **Quoted key.** A quoted string containing `version` or `format`, followed
   by `,` or `:` and a value literal. Covers `jb.set_int("version", 3)`.

Lines whose first non-whitespace characters mark a comment (`//`, `#`, `*`,
`/*`) are ignored.

### Switch detection (per producer source bound by a contract)

Group occurrences by name. For every name that has at least one occurrence
with a value in BOTH the base and the head text: let VB and VH be the value
sets. A **default switch** is detected when VH contains a value not in VB.
A name present on only one side never fires. A value disappearing with
nothing new appearing never fires.

### Outcome

F1 never claims VIOLATED. Textual analysis cannot prove which output path the
consumer receives; a default switch is evidence that the path may have
changed while every literal survived. So:

- If the contract already has residue, F1 adds nothing.
- Otherwise, every bound field of a source with a detected switch becomes an
  UNEVALUATED site:
  `default selector 'name' changed OLD -> NEW in source; emission of 'field' not evaluated`.
  The contract is NOT FULLY EVALUATED (exit 4), never HOLDS.

## Measure

- Primary: silent miss rate on held-out batch 8 (real breaks reported HOLDS),
  frozen alpha.10 engine versus the F1 engine. F1 converts misses to NOT
  EVALUATED; it does not catch. The spec says so plainly and the score keeps
  the two columns separate.
- Guard: controls can never turn VIOLATED under F1 (by construction). A
  control flipped to NOT EVALUATED is cost and is reported.
- Cost: share of all batch 1 to 7 contracts flipped from HOLDS to NOT FULLY
  EVALUATED by F1, measured on the tuning rerun and reported with the build.

## Non-goals

No new parser, no new dependency, no per-language rules beyond the four
textual patterns, no execution of producer code, no attempt to decide which
branch is live.
