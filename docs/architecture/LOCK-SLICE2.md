# LOCK-SLICE2: one-line `Readonly<{ ... }>` bodies in lock

Status: RATIFIED (founder, 2026-10-21, on STAGE3-EVAL-008 finding
S8-F-05), implemented and landed with INIT-DISCOVERY-SLICE8.

## Finding

excalidraw packages/element/src/renderElement.ts:118 declares
`export type RenderPositionOffset = Readonly<{ x: number; y: number }>;`.
`invar lock` bound none of its fields: LOCK-SLICE1 reads members written on
a declaration's own line only after `= {`, and the brace after `Readonly<`
is inside angle brackets, which it skips on purpose (a type parameter's
constraint is not a body). init proposes `Readonly<{ ... }>` bodies as
object shapes (codepairs `type_kind`), so lock and init disagreed on what
a shape's fields are. The contract held through its other shape, so the
gap was silent.

## Rule

On a declaration line, `= Readonly<{` (spaces allowed between the tokens)
opens the body at that brace, exactly as `= {` does; the members are read
as LOCK-SLICE1 reads them. Nothing else inside `<...>` is opened: not
`Partial<{`, `Required<{` or any other generic, not a `Readonly` in a type
parameter list, not a member whose type is `Readonly<{...}>`.

## Gates

Gate 1: tests/lock_slice2_test.rs frozen before the fix (6031deae0262): F1
binds and holds, F2 a removed member is VIOLATED, F3 spacing and nesting,
F4 other wrappers stay at 0 bindings (NOT EVALUATED), F5 a Readonly in a
constraint is not opened, F6 a Readonly member type is not a body, N1 a
multi-line Readonly body unchanged. 3 failed on the baseline as declared.
Gate 2: surface.rs `same_line_members` only.
Gate 3: 5 of 5 mutants caught (scripts/lock-slice2).
Gate 4: with SLICE8, 397 of 397 tests, 0 warnings.
Corpus re-measurement (the LOCK-SLICE1 method, docs/evidence/lock-slice2):
lock's producer surface over all 17,795 `.ts` and `.tsx` files of the six
cloned repositories: 3 fields added in 2 files, 0 removed, 0 retyped.
Effect on STAGE3-EVAL-008: M-S-5 24 of 25 before, 25 of 25 after.
