# Releasing

A release publishes the crate `mumble-client-protocol` to crates.io and creates a GitHub release. The pipeline (`.github/workflows/release.yml`) does the work; the maintainer prepares the version and pushes a tag.

## Steps for a release

1. **Set the version** in `Cargo.toml` (`version = "X.Y.Z"`).
2. **Update `CHANGELOG.md`:** rename `## [Unreleased]` to `## [X.Y.Z] - YYYY-MM-DD` and put a new, empty `## [Unreleased]` above it. The text of the `[X.Y.Z]` section becomes the notes of the GitHub release.
3. **Commit** (`chore: release X.Y.Z`), push to `main` and wait for CI to be green.
4. **Rehearse** (optional, recommended): Actions → Release → Run workflow, leave `dry_run` switched on. This checks and packages everything and publishes nothing.
5. **Tag and push:** `git tag vX.Y.Z && git push origin vX.Y.Z`. The tag must sit on the release commit on `main`. Pushing the tag starts the real release.
6. **Check the result:** the package on crates.io, the page on docs.rs, and the release on GitHub.

Published versions can never be replaced or removed from crates.io (only "yanked", which hides them from new users). If something is wrong after publishing, fix it and release the next patch version.

## What the pipeline does

1. **Check:** the tag equals the version in `Cargo.toml`, and `CHANGELOG.md` has a section for that version. Otherwise it stops before anything is published.
2. **Tests:** the complete CI checks (formatting, Clippy, tests, live tests against the local servers, documentation, examples, minimum Rust version, licence and advisory check, package check).
3. **Publish to crates.io.** A version that is already on crates.io is skipped, so a release that stopped half-way can simply be run again (Actions → the failed run → Re-run failed jobs).
4. **GitHub release** for the tag, with the changelog section as notes.

The log of the publish job says in its first lines whether the run is a **dry run** or a **real publish**. A real publish only happens from a version tag; a run started by hand from a branch refuses to publish even with `dry_run` switched off.

## One-time setup (maintainer)

### The first release: a crates.io token

crates.io can only trust this repository for a package once the package exists. The very first publish therefore uses a crates.io API token. It is stored as the organisation secret `CARGO_REGISTRY_TOKEN`, which this repository can read; nothing has to be set up for the first release. The token needs the scopes `publish-new` and `publish-update`.

### After the first release: Trusted Publishing

With Trusted Publishing, crates.io accepts a publish because it comes from this repository's release workflow; no stored token is needed. Register it once:

1. On crates.io open the package → Settings → Trusted Publishing → Add → GitHub.
2. Fill in: repository owner `Mumcord`, repository name `mumble-client-protocol`, workflow filename `release.yml`, environment `release`.
3. Take this repository off the organisation secret `CARGO_REGISTRY_TOKEN` (organisation Settings → Secrets and variables → Actions → the secret → repository access), or delete the secret and revoke the token on crates.io if nothing else uses it.

From then on the pipeline asks crates.io for a token that is valid for 30 minutes on each release. As long as this repository can read the secret `CARGO_REGISTRY_TOKEN`, the pipeline uses it instead.

It is also worth adding a second owner to the package on crates.io (package → Settings → Owners), so that it does not depend on one account.
