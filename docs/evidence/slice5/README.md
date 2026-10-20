# SLICE5 draft evidence (throwaway prototypes)

model/ is the Python model of the receiver rule, run on the 87 code pairs
adjudicated in STAGE3-EVAL-003 and -004 (labels.tsv, labeled-out.tsv) and
on all 206 code-pair proposals of EVAL-004 (allcode.tsv, all-out.tsv).
receiver.py reads the pinned clones and the EVAL tomls from the session
workspace; set RV_EXT=1 for forms E1 and E2 (explicit type argument, local
return type), which the draft keeps.

engine-prototype/ is the rule written into codepairs.rs on top of SLICE4
(receiver-prototype.patch, binary 483501da0799), its init output on the
five cloned repositories, and the 12 tests it changes
(fixture-collisions.txt; 295 of 307 pass unchanged). It is evidence for
the draft, not the implementation: the slice is written from scratch at
Gate 2 after the fixtures are frozen.
