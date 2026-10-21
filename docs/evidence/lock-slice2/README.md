# LOCK-SLICE2 corpus evidence

The LOCK-SLICE1 method (docs/evidence/lock-slice1/tsdump.rs, a throwaway
example printing `surface::code_fields`), run with the SLICE7 reader
(681fc1463275 tree) and the LOCK-SLICE2 reader over every `.ts` and `.tsx`
file of the six cloned STAGE3 repositories at their pinned commits (17,795
files, excluding node_modules, .git, dist and build).

Result: 3 (scope, field) entries added, in 2 files; 0 removed; 0 retyped.
added-fields.tsv lists them: excalidraw RenderPositionOffset x and y (the
finding), and pi SourceValue value (a test file).
