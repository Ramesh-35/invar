# First GitHub runs of the S3 CI jobs (2026-10-27)

Captured with `gh` from Ramesh-35/invar; raw in this directory.

ci-runs.json: the six most recent workflow runs at capture time, every one
success, including the three pushes of 2026-10-27 (58aef45, e33e7f8,
2667de9), each running "invar-falsify CI".

ci-jobs.json: the jobs of run 40469576329, head 58aef45, the FIRST run on
a tree carrying SECURITY-SPEC S3's two new jobs. All five jobs success:
falsify, pinned-actions, action-selftest, and the two S3 jobs,
dependency-audit (cargo-audit 0.22.2, --deny warnings) and
reproducible-build (scripts/release/repro-check.sh). With the macOS
reproducible-build record (docs/evidence/security/
repro-check-s3-macos.txt, commit 0fef457), this closes S3's stated blind
spots except: advisories not yet published.
