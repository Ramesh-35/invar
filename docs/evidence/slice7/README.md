# SLICE7 draft evidence (throwaway prototype, 2026-10-21, on b0a8c79)

prototype.patch adds to codepairs.rs a per-package index of functions with
a declared return type, and ties a receiver `r = [await] f(...)` to shape S
when f is imported by name from S's package and has exactly one such
declaration there, returning S (or Promise, Readonly, array of S). An R6
shape with two distinctive reads through such receivers becomes a
proposal. INVAR_S7_UNIFORM=1 also applies the route to shapes imported by
name (SLICE5's object-shape branch). conversions-*.tsv list each converted
(consumer, producer, shape, function); the *.invar.toml files are the
prototype's init output for the three repositories it changes (the other
seven are byte-identical to b0a8c79). The prototype's E4 line and status
line are placeholders; the draft's S7-D2 and S7-D3 define the real ones.
