# Contributing

Thanks for helping.

## Before you open a pull request

- Open an issue first for anything larger than a small fix, so the direction can be agreed.
- Run `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo doc --no-deps`. CI runs the same checks.
- Changes to the protocol handling should also pass the live tests against the local servers (see [testenv/README.md](testenv/README.md)). Never test against public or third-party Mumble servers.
- Keep changes small and focused, one logical change per commit, with a clear message ([Conventional Commits](https://www.conventionalcommits.org/) style: `feat:`, `fix:`, `docs:`).
- New logic needs tests. A bug fix starts with a test that reproduces the bug.
- Add a line under `[Unreleased]` in `CHANGELOG.md` for every user-visible change.
- Public items need documentation; the documentation build must stay free of warnings.

## Code rules

- Stay compatible with standard Mumble servers; no private protocol extensions.
- Treat everything the server sends as untrusted input.
- No `unwrap` in library code; report errors through the error types or log them. Never log chat text, names or audio.
- Comments explain why, not what.

## Licence

By submitting a contribution you agree that it is licensed under `MIT OR Apache-2.0`, the same as the crate, without additional terms. The files that come from Mumble keep their BSD-3-Clause licence (see NOTICE).
