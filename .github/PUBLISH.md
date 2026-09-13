# Publishing

`.github/workflows/ci.yml` is the only release pipeline. Main pushes, explicit
dispatches and daily scheduled runs enter the same pipeline; tags do not start
a separate publisher.

## Version and source identity

The shared `prepare-cargo-release` action considers every commit since the last
release: fixes and dependency changes produce a patch, features a minor, and
declared breaking changes a major. Documentation, style and CI-only commits do
not independently release. An explicitly higher manifest version is respected.

When needed, `cargo version-info` updates Rust manifests and lockfiles and runs
the configured companion-version hook for npm, Python and the root package.
GitHub records one verified version commit, guarded by the expected main SHA.
All downstream checks, builds and publishers check out that exact revision.
The release-writer App's push starts main CI, which resumes at that revision.

## Validation and publication

Rust formatting, Clippy and tests, npm tests, Python tests and platform builds
must succeed before creating the immutable version tag and draft GitHub release.
All registry versions remain synchronized. CLI artifacts attach to the release;
Rust, npm and Python packages publish to their registries. The GitHub release
becomes public only after all publishers succeed.

Publication is not an atomic transaction across registries. A partially completed
release is retried at its original tagged revision before preparing a newer one.
Crates.io retries verify the published package's source revision; conflicting
tags or crate revisions fail. Before the first registry upload, an immutable `release-packages.zip` asset
stores the npm tarball, five Python wheels, source revision and SHA-256 hashes.
Retries reuse those original bytes even when a rebuild differs. Existing npm
versions must match the tarball's SHA-512 integrity; existing Python wheels
must match their SHA-256 hashes. Only missing packages are uploaded. Never move a tag to bypass a failure.

## Credentials and recovery

- `CRATES_IO_TOKEN` must authorize publishing dotenvage.
- npm and PyPI trusted publishers must authorize this repository and `ci.yml`.
- The `pypi` environment is used for publication and must not require a human
  approval if unattended release is required.
- `RELEASE_WRITER_APP_ID` and `RELEASE_WRITER_APP_KEY` mint repository-scoped
  App tokens for version writes and dependency merges. Main rules authorize
  that App while preserving code-owner review for outside contributors.
- Publication needs contents write and id-token write permissions.

Daily scheduled runs and manual `gh workflow run ci.yml --ref main` use the same
recovery path. Never invoke a second publisher to repair a failed release.
Monitor Actions failures and credential expiry; failing validation blocks release.

Dependency updates use the same pipeline. See
[Dependency maintenance](DEPENDABOT_AUTOMATION.md).
