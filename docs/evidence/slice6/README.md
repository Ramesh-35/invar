# SLICE6 draft evidence (throwaway prototype, 2026-10-20, on d4488ac)

paths-prototype.patch: tsconfig `paths` in egress discovery (K1), root
config only, binary d40fd244faa0. omniroute-paths-prototype.invar.toml is
its OmniRoute output (the other nine repositories are byte-identical to
d4488ac). classify_k1.py classifies the 1,393 imports that newly reach
K1's silent branch (input: the prototype's INVAR_CENSUS stderr lines);
k1-silences.tsv is the result: 1,225 declared in the target as a non-Zod
value or type, 166 through the target's re-exports, 2 Zod non-object
schemas (unearned). The classifier's first version scoped declarations to
one top-level directory and reported 53 "declared nowhere"; widening the
scope showed every one is a re-exported value, and the final split found
none that needs more than the target file and its re-exports.

Correction (landing commit): k1-silences.tsv as first committed (b09f2d1)
was the narrow-scope run's output, which overwrote the corrected one; it
is replaced here. The 1,393 split into 1,284 alias imports (newly
reachable) and 109 relative imports already silent on main (S6-F-01).
