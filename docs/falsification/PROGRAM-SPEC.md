# PROGRAM r0: the program that reads the file, as a recorded line (r0)

Drafted 2026-11-09, frozen with the eight `rd_` fixtures in
crates/invar-falsify/tests/reader_test.rs BEFORE implementation; freeze
record: PROGRAM-SPEC.frozen. The PROG-0 experiment that motivates it is
checked in under docs/falsification/prog0/ (pre-registration frozen
before any program ran, deviations each written before the run they
affect, results).

## Why

PROG-0 (2026-11-09): of ten SchemaStore changes INVAR judged VIOLATED
whose file has one named program reading it, six programs loaded, without
an error, a document their own published schema rejects; in three the
value was not even what the file said (a float silently truncated, a key
stored but never used, a value not validated at all). The schema verdict
alone cannot see this, and no tool on the market runs a schema change's
counterexamples through the real program. The sound form (per the
2026-11-09 research review): a separate line per named program that never
overrides the schema verdict, produced by running the real binary, never
an emulation.

## Ruling PROGRAM (r0)

`invar reader BASE HEAD --cmd '<template>'` judges the pair with the same
engine as `invar judge`, then:

- builds the control: the minimal document the HEAD contract accepts
  (the counterexample builder's own `minimal`); when it cannot be built,
  that is stated and the runs still happen;
- writes each document to a file in a fresh temp directory and runs the
  command (`{}` replaced by the file's path, the whole template run by
  `sh -c`), first on the control, then on every witness counterexample;
- records per run exactly what happened: `exit 0`, `exit N`, or
  `timed out after Ts` (default 30, `--timeout` to change it). Exit 0 is
  the only "loaded" signal; a nonzero exit is recorded with its code and
  nothing more is claimed about it (a command that cannot start, exit
  127 say, is indistinguishable at this grain, which is what the control
  run is for: a control that does not exit 0 is flagged "the command may
  not read this document at all");
- clears the environment first: PATH is kept, HOME and TMPDIR point into
  the fresh temp directory, everything else is dropped; `--env K=V`
  passes a variable deliberately. INVAR does not sandbox the command and
  does not block the network; the command runs with the caller's
  permissions, and the caller isolates it when that matters (stated in
  the output);
- never moves the verdict or the exit code: `invar reader` exits with
  the judge's own code, byte-for-byte the same verdict with or without
  a command, and "silently dropped" detection (a program that loads a
  document and then ignores the value) is out of r0: it needs
  per-program probes, as PROG-0 needed;
- `--out FILE` writes a canonical report (sorted keys, BLAKE3 footer,
  `receipt_kind: "reader"`): both schema paths and hashes, the command,
  the timeout, the control and every run with its document and the
  document's hash. `verify-receipt` on it checks the footer and that the
  schema files still hash as recorded (drift is exit 1), then says the
  program result is recorded, not re-derived: re-deriving it would mean
  re-running a program whose version the engine cannot pin, the same
  line that keeps harness results out of judge receipts.

## Fixtures (frozen before implementation; 0 of 8 pass today)

- rd_f01 exit 0 recorded, control recorded, verdict stands.
- rd_f02 nonzero exit recorded with its code; the control exits 0.
- rd_f03 guard: judge and reader agree on exit, verdict and witnesses;
  the reader JSON adds a `reader` object and nothing else changes.
- rd_f04 the environment is cleared: a variable on invar's own process
  is absent and HOME is fresh.
- rd_f05 a command that outlives --timeout is killed and stated.
- rd_f06 nothing to run (NOT EVALUATED, no counterexample): stated.
- rd_f07 the --out report verifies and names its limit.
- rd_f08 schema drift after writing fails verification.

## Mutation operators

- RD-M1 the environment not cleared        -> rd_f04
- RD-M2 a nonzero exit recorded as exit 0  -> rd_f02
- RD-M3 the control run skipped            -> rd_f02
- RD-M4 the timeout not enforced           -> rd_f05

## Pre-registered prediction

No corpus moves anywhere: the reader records, it does not judge. Suite,
clippy, WG gate and every earlier mutant unchanged, plus the four above.
