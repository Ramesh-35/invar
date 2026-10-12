# Show HN draft (ships nowhere until the founder says fire)

Preconditions before firing: Developer Hearts items 1 through 6 done,
demo recording embedded in the README, and at least the E2 responses
in hand.

## Title

Show HN: INVAR catches cross-repo API breaks before they ship. It
misses 46% and tells you which.

## Body

Every team I know has been broken by a repo they do not control:
a producer renames machines to nodes, drops a field your widget sums,
or changes a default, and nothing in your CI even blinks. INVAR is a
single static binary that checks the fields your code reads from
another repository's API, and its defining rule is that a skip is
never a pass: anything it cannot evaluate is stated with a reason
code and a non-zero exit, because unexamined and clean are different
facts.

The number in the title is real. On a sealed held-out benchmark of 70
real breaks (each pinned to real commits, each sourced from a public
incident), INVAR catches 38. The benchmark was hashed and committed
before the repo went public, and you can reproduce the 38/70 with one
command from the repo's own data, no trust required. The breaks it
does catch were silent in every static tool we scored against them.

Try the demo (three real incidents, replayed offline, milliseconds):

    git clone https://github.com/Ramesh-35/invar && cd invar
    cargo run --release -p invar-falsify -- demo

Rust, MIT OR Apache-2.0; corpus data CDLA-Permissive-2.0; signed
releases for Linux and macOS. I would genuinely value your misses
more than your stars: docs/falsification/R1-CORPUS.md has the
admission rules for real cases.

## Day-after reposts

r/rust and lobste.rs, one short paragraph each, linking the HN thread
rather than restarting it. No invented momentum anywhere.
