# `invar demo` (spec, frozen 2026-10-11)

Ratified direction; implementation follows observed-reality locking or in
parallel. One honesty correction to the ratifying dispatch, recorded here
so the record is straight: the demo replays cases from OUR corpus of real
cross-repository breaks. It does not replay the CrowdStrike channel-file or
Azure Front Door incidents: those producers (kernel C++, binary channel
files, closed config planes) are outside INVAR's supported scope, and
demoing them would claim coverage we do not have. The horizon report's
counterfactual analyses of those incidents stay what they are: analyses.

## What it does

`invar demo` (no arguments, no flags needed):

1. Ships 2 or 3 real corpus cases' pinned artifacts inside the repository
   (and inside release tarballs) under `demo/`: producer base, producer
   head, consumer, exactly the bytes the corpus pins, with their BLAKE3
   checks. Candidates, chosen for small artifacts and clear stories, all
   CAUGHT on held-out scoring: r1-109 (authentik removed the coordinate
   fields a dashboard summed), r1-112 (headscale renamed machines to
   nodes under a UI), r1-117 (ghostfolio removed the performance fields a
   widget read). RATIFY the final pick.
2. Runs the normal engine on them, fully offline, and prints the normal
   `[VIOLATED]` output with the real repository names, the field, the
   consumer file and line, and one line of what happened in the real world
   (from case.toml), plus measured wall time.
3. Ends with the honest coda, verbatim: "This is a real break INVAR
   catches. On held-out real breaks overall it currently catches 38 of 70;
   what it cannot evaluate it states instead of passing. invar init to try
   it on your own repositories."

## Constraints

- Zero network, zero setup, works in a fresh clone and from a release
  tarball. Wall time is printed as measured, never promised in advance.
- The demo uses the stock engine path (`evaluate_with`), not a special
  mode: what it shows is what `invar check` does.
- Artifact licenses: the embedded files are small excerpts of open-source
  code pinned by the corpus, shipped with their repository names and
  licenses noted in demo/NOTICE. RATIFY.

## Measure

TTFV (MOAT R6) is measured WITH the demo as the entry point: time from
binary download to the demo verdict, and separately to a first real
verdict on the user's own pair. Published in MOAT.md as measured.

## Implementation record (2026-10-11, v0.1.0-alpha.20)

Shipped as specified; deviations and clarifications, on the record:

- Case pick RATIFIED as the spec's own candidates: r1-109 (authentik),
  r1-112 (headscale), r1-117 (ghostfolio). All three replay to VIOLATED on
  the exact corpus-pinned bytes (demo/demo.toml pins every file's source
  reference and BLAKE3; the command verifies the bytes before every run and
  errors on any mismatch).
- Offline by construction: the demo evaluates through the stock engine path
  (`evaluate_with`) with the local resolver and no transport wired in, so a
  network call is impossible, not merely avoided.
- Wall time is printed as measured per case on the running machine, per the
  frozen constraint; no latency figure is promised anywhere in output or
  docs. For the record, first measurements in the build container were 0.3
  to 6.6 ms per case; that is a report, not a commitment.
- The coda is the frozen verbatim text of this spec. The 2026-10-11
  ratifying dispatch restated it in different words and asked for a <20ms
  execution bound; both yield to the frozen spec (measured, never
  promised), stated here so the record is straight.
- "Release tarball" means the source archives GitHub attaches to every
  release, which include demo/. The prebuilt binary assets do not carry
  demo/, so `invar demo` from a bare binary download reports plainly that
  it needs a clone or an unpacked source tarball (exit 3). Embedding the
  bytes in the binary was considered and not done: it would grow every
  binary for one command and blur what "pinned in the repository" means.
- A demo case that does not come out VIOLATED is a packaging error, exit 3
  with the case named, never a quiet pass of a broken demonstration.
- demo/NOTICE records each embedded file's repository, license (MIT,
  GPL-3.0, BSD-3-Clause, AGPL-3.0), and pinned commit; the files are
  unmodified data the engine reads, not code INVAR links.

## `invar coverage` (shipped alongside, same ratification)

`invar coverage [--config <path>] [--contract <id>] [--format json]`
disclosure, never a verdict (exit 0 success, 3 error, never 1). Per
contract it states: the static leg actually in force (locked slice,
change-scoped diff, full surface, or SKIPPED with the unsupported language
named), what the engine accuses versus what it only states (with the
stable reason codes), the observed-reality leg (observation count, window,
path witness counts with wildcard counts, staleness, and the witness
thresholds), and the fixed blind spots: field values and semantics,
runtime behavior, non-JSON payloads. The ratifying dispatch's "macros,
inline assembly" phrasing does not map to this engine's surfaces and was
not implemented; the blind spots disclosed are the engine's real ones.
Every check-style summary ends with the measured held-out record, so
coverage output cannot read as an effectiveness claim.
