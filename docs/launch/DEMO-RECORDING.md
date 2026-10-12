# Demo recording brief (45 seconds, one take, no speedups)

The recording is a latency claim, so it obeys measurement rules: real
terminal, real time, no cuts inside a command, no post-speedup. If a run
is slow that day, that is the recording.

## Setup (founder's Mac)

- Terminal at 100x28, a plain dark theme, 16pt monospace, no
  transparency; prompt shortened to `$ ` (`PS1='$ '` in the session).
- Fresh clone in a clean directory so the clone is part of the truth:
  work from `~/Desktop/rec/`.
- Recorder: asciinema (`brew install asciinema`, `asciinema rec
  invar-demo.cast`) for the README embed, or QuickTime screen recording
  cropped to the terminal for a GIF via `agg` (`agg invar-demo.cast
  invar-demo.gif`).

## The take

1. `git clone https://github.com/Ramesh-35/invar && cd invar` (the wait
   is real; leave it in)
2. `cargo run --release -p invar-falsify -- demo` on a WARM build, and
   say so in the README caption ("engine prebuilt; the demo itself is
   what you are watching"), or use the release binary: `./invar demo`
   after downloading and verifying it on camera if the 45s budget holds.
3. Let the full output scroll: three VIOLATED verdicts, the pinned
   references, the measured milliseconds, and the honest coda must all
   be visible; end on the coda, hold two seconds, stop.

## What the caption under the embed says

"Three real cross-repository breaks, replayed offline from pinned
bytes. Wall times as measured on this machine. On held-out real breaks
overall INVAR currently catches 38 of 70; what it cannot evaluate it
states instead of passing."

## Rejects

Re-record rather than edit if: a typo needed backspacing more than
twice, a notification banner appeared, or the terminal wrapped lines.
Never trim dead time inside a command; trimming between commands is
allowed and stated here.
