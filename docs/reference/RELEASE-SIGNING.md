# Release signing (from v0.1.0-alpha.7)

Each release binary archive is signed with an SSH key (ed25519, OpenSSH
`ssh-keygen -Y sign`, namespace `invar-release`). The signature is published
next to the archive as `<archive>.sig`.

## Trust anchor

The public key lives in `action/release-signers` (OpenSSH allowed_signers
format, principal `invar-release`). The GitHub Action reads it from the action
ref you pinned, so pinning `Ramesh-35/invar-platform@v0.1.0-alpha.7` pins the
key you trust as well as the code.

## What the Action does

1. Download the archive, its `.sha256` and its `.sig`.
2. Check the SHA-256 checksum.
3. Verify the signature against `action/release-signers`.
4. Only then extract and run `invar`.

A signature that fails stops the job; it never falls back to a source build.
A release with no signature (v0.1.0-alpha.6 and earlier) is refused when the
pinned action ref has a `release-signers` file; pin a signed release or leave
`version` empty. If the asset cannot be downloaded at all, the Action builds
from the pinned source, as before.

## Verify by hand

```
ssh-keygen -Y verify -f action/release-signers -I invar-release -n invar-release \
  -s invar-v0.1.0-alpha.7-aarch64-apple-darwin.tar.gz.sig \
  < invar-v0.1.0-alpha.7-aarch64-apple-darwin.tar.gz
```

## Key handling

- The private key exists only as the repository secret
  `INVAR_RELEASE_SIGNING_KEY`. The release job writes it to a file readable
  only by the runner user, signs, and deletes it.
- The release job refuses to publish if the secret is missing, and checks each
  signature against the committed public key before publishing.
- Rotation: generate a new key, set the secret, add the new public key line to
  `action/release-signers` (keep the old line so older releases still verify,
  or remove it to revoke), release.

## Honest scope

This proves a binary was signed by whoever holds the key. It is not a
transparency log or a build-provenance attestation: those need either a
public repository (GitHub artifact attestations) or a public log (Sigstore),
deferred while the repository is private.
