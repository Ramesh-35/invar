# The Simplicity Law

> The smallest correct instrument that can falsify the thesis beats a large one
> that merely looks complete.

INVAR is bound by the Simplicity Law. It is not a style preference; it is an
epistemic control.

## Statements

1. **Prefer a smaller correct foundation over a large fictional implementation.**
   A tree that looks finished but cannot run is worse than a small tree that
   runs and tells the truth.
2. **No speculative implementation presented as complete.** Deferred components
   stay deferred — no `Cargo.toml`, no build, no claim of function.
3. **Introduce a primitive only when a falsifiable requirement needs it.**
   tree-sitter, LSP, CUSUM, HKDF, and a Merkle-DAG are all *deferred* until a
   specific test demands them. Appearing in a specification is not a
   requirement.
4. **No dependency without a job.** The composite config is hand-parsed rather
   than pulling in a TOML crate, because the format is small and explicit.
5. **Every abstraction must earn its keep against a test.** The acceptance tests
   are the ground truth; code exists to satisfy them, not to anticipate a
   hypothetical future.

## Why this is a control, not taste

Complexity hides unverified claims. A large framework can *look* like it proves
something while proving nothing. By keeping the instrument minimal and its
outputs explicit, we make it hard for an over-claim to survive: there is nowhere
for a fabricated verdict to hide.
