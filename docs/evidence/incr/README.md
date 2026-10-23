# CHECK-INCR-1 evidence

Replay of the last 100 first-parent commits of each EVAL-008 repository
(scripts/check-incr-1/replay.py): at every commit, `invar check --no-reuse`
(full evaluation, the reference) against `invar check` (reuse). One TSV per
repository; the last line is the summary. Produced on the sandbox
(Linux x86_64) with the Gate 2 release binary. The lock is taken at the
first commit of the window where every input file exists; at a commit where
an input file does not exist yet, both modes error identically (exit 3),
which the replay also checks.

Gate 3 kill log: mutate.log (scripts/check-incr-1/mutate.sh over the
patches in scripts/check-incr-1/mutants/).
