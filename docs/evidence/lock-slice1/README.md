# LOCK-SLICE1 corpus evidence

tsdump.rs (a throwaway example, not shipped) prints lock's producer
surface (`surface::code_fields`) for every path on stdin. It was run with
the baseline reader and with the LOCK-SLICE1 reader over every `.ts` and
`.tsx` file of the six cloned STAGE3 repositories at their pinned commits
(17,795 files, excluding node_modules, .git, dist and build).

Result: 2344 (scope, field) entries added, in 856 files;
0 removed; 0 retyped. added-fields.tsv lists every addition
(paths relative to the clone root, prefixed by the repository). The
simplified member split of Gate 3 (L-F-01) produces output identical to
the Gate 2 reader on every one of the 17,795 files.
