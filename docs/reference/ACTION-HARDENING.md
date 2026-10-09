# Action hardening (from v0.1.0-alpha.8)

The threat: whoever controls a repository can move its tags. In March 2025 the
tags of tj-actions/changed-files were repointed to a malicious commit that
printed CI secrets into build logs (CVE-2025-30066). Any workflow that used
the action by tag ran the attacker's code.

## Use INVAR by full commit SHA

```yaml
- uses: Ramesh-35/invar-platform@<40-character commit SHA> # v0.1.0-alpha.8
  with:
    release_token: ${{ secrets.INVAR_TOKEN }}   # only if this repo is private to you
```

- Pinned by the SHA of a release commit, the Action downloads that release's
  binary and verifies its SHA-256 and SSH signature (key in
  `action/release-signers` at that same commit) before running it.
- Pinned by any other SHA, it builds from that exact source.
- Pinned by a tag, it still works, and warns on every run that tags can move.
  A moved tag would bring both the attacker's code and the attacker's signing
  key: signatures protect the binary, not a tag you trust blindly.
- Each release's commit SHA is in its release notes and in
  docs/epistemic/STATUS.md.

## What the Action does not do

- It uses no third-party actions: every step is a bash script in this
  repository (`action/*.sh`).
- Tokens reach `invar` only through environment variables, never argv, and
  are never printed.
- `release_token` is used only to download the release binary; `github_token`
  is used for remote spec fetches and the PR comment.
- Do not run it under `pull_request_target` with a checkout of the PR head:
  that gives untrusted code your secrets. Use `pull_request`.

## What this repository does

- Every third-party action in its own workflows is pinned to a full commit
  SHA; CI fails if one is not (`scripts/check-pins.sh`, job `pinned-actions`).
  This matters most for the release job, which holds the signing key.
- CI runs with `contents: read` only. The release workflow has write access
  only in the publish job.
- Release tags `v*` are protected by a repository ruleset against update and
  deletion, where the plan allows it (see STATUS.md for whether it is active).

## Honest scope

SHA pinning and signatures stop a moved tag or a swapped binary. They do not
stop a malicious commit merged into this repository and then released; review
and branch protection are the controls for that.
