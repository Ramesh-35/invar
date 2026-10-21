# SLICE8 draft evidence (throwaway probe, 2026-10-21, on db401f4)

propprobe.py classifies every R6 line of the SLICE7 output on the six
cloned repositories (3,065 lines) by whether the consumer reads two
distinctive fields of the shape through a property whose written type is
the shape, at most two hops from a receiver annotated in the file.
prop-probe.tsv is its output: 20 YIELD, 43 THIN, 3,002 NONE.
adjudication/ holds the blind check of the 20 YIELD lines: two agents,
20 of 20 TRUE each, agreement 20 of 20. The probe gained array indexing
and inline object types after a recall check against the R6 probe's
seeded sample, so the 20 are in sample; STAGE3-EVAL-008 judged the
engine's own 34 shapes, 14 of them out of sample.
