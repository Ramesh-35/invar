# SIB-3: judge a pull request against the base branch's index and trust root

Status: r0, FROZEN with its fixtures before any implementation, for
founder ruling on B-D1 to B-D3 (the fixtures encode the recommended
options). Parent: SIB-2 and the 2026-10-27 ruling "the Action must pull
the trust root and index strictly from the base branch". Fixtures:
tests/sibling_base_test.rs; hashes in SIBLING-BASE-SPEC.frozen.

## A. The gap, found while building the demo pair

On `pull_request`, actions/checkout gives the job the pull request's own
tree. SIB-2's step reads `dependents_locks` and `dependents_signers` from
that tree, so the pull request under review controls what it is judged
against: it can replace the trust root and a lock together (a forged clean
report), add its own signed lock, or delete a consumer's lock (that
consumer disappears from the report, unstated). Measured in the demo
rehearsal: the step read whatever the PR tree held.

## B. Rules

- B-1, base on pull requests. When the step receives a base ref
  (`INVAR_BASE_REF`, from the Action's existing `base_ref` input, default
  `github.base_ref`), a workspace-relative `dependents_locks` and
  `dependents_signers` are read from `origin/<base>` (fetched by the step,
  then extracted with `git archive` into the runner's temp directory),
  never from the working tree. The producer files under review are still
  the pull request's: that is the change being judged.
- B-2, fail closed. If the base ref cannot be fetched or the paths are not
  on it, nothing is evaluated: `dependents_exit=3` and the stated line
  `not evaluated: base ref <ref> could not be read; the pull request's own
  index is never used`. Never a fallback to the working tree. The step
  still exits 0 (advisory, SIB-2 R-4).
- B-3, absolute paths. An absolute `dependents_locks` or
  `dependents_signers` (an index the operator checked out outside the
  workspace) is read where it is: not the pull request's to change.
- B-4, no base ref (push to a branch). The workspace is the branch itself
  and is used as today.
- Unchanged: the CLI and the engine (this slice is action/dependents.sh
  and one env line in action.yml); SIB-1 and SIB-2 behavior; exit code
  meanings; the step never fails the job.

## C. Measured before code

The demo rehearsal (2026-10-27): with the base-branch extraction done in
the demo's own workflow, a forged `consumer-signers` planted in the PR
tree was ignored and the lock verified against the base copy. Without it,
the step reads the PR tree (fixtures f01 to f03 fail on the current step).

## D. Fixtures (frozen with r0)

| Fixture | Asserts |
|---|---|
| b3_f01 | PR replaces the trust root: ignored, web's lock verified from base, PR's producer judged (`maxLines absent`) |
| b3_f02 | PR deletes a consumer lock: still evaluated from base |
| b3_f03 | PR adds its own signed lock and key: not trusted, not listed |
| b3_f04 | unreadable base ref: dependents_exit=3, stated line, no fallback, step exits 0 |
| b3_f05 | no base ref: the workspace is used (guard) |
| b3_f06 | absolute paths: used as given (guard) |
| b3_f07 | action.yml passes `INVAR_BASE_REF` to the dependents step |

On the current step: f01 to f04 and f07 FAIL; f05 and f06 (guards) pass.

## E. Pre-registered mutation operators

| Operator | Change | Declared killer |
|---|---|---|
| B3-M1 | base mode skipped (always the workspace) | b3_f01 |
| B3-M2 | only the index from base, trust root from the workspace | b3_f01 |
| B3-M3 | only the trust root from base, index from the workspace | b3_f02 |
| B3-M4 | on an unreadable base, fall back to the workspace | b3_f04 |
| B3-M5 | absolute paths also extracted from base | b3_f06 |
| B3-M6 | INVAR_BASE_REF not passed by action.yml | b3_f07 |

## F. Decision points

- B-D1 (default). Base mode always on pull requests (recommended: an
  opt-in protection is one most workflows will not opt into). Or an input
  to enable it.
- B-D2 (unreadable base). Fail closed, stated (recommended). Or fall back
  to the workspace with a warning.
- B-D3 (absolute paths). Used as given, operator-controlled (recommended).
  Or refused, forcing every index into the repository.

## G. Bounds

No engine, CLI or exit-code change; the step never fails the job; one
network call (git fetch of the base ref, already reachable by checkout's
credentials); zero new dependencies; frozen spec and fixtures before code;
mutation operators with declared killers; a real GitHub run before the
claim is recorded.
