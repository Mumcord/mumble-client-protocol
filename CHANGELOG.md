# Changelog

All notable changes to this project are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project follows [Semantic Versioning](https://semver.org/). Below 1.0, interface changes may happen in any release and are listed here.

## [Unreleased]

## [0.1.0] - 2026-11-14

First release.

### Added

- Control channel: TLS with an optional client certificate, a server-certificate check that hands the decision to the caller, login, keep-alive pings and an event stream with typed errors.
- Server state: channel tree, users and server settings, built from the server's messages.
- Commands: text messages, channels, whisper and shout targets, channel listening, ACL queries, kick and ban, the ban list, registered users, permission queries and plugin data.
- Client-side pacing for messages the server rate-limits, with `Handle::flush`, `Handle::disconnect` and `Handle::closed`.
- Voice transport: OCB2-AES128 encrypted UDP in the legacy and protobuf packet formats, with fallback to the TCP tunnel and back.
- Server query through the UDP ping, in the protobuf and the legacy format.
- A packet tap for debugging tools.
- `examples/text_bot.rs`, a minimal bot that answers `!ping`.
- Local Mumble 1.5, 1.4 and 1.3 test servers in `testenv/` for the live tests.

### Changed

- Extracted from the Mumcord `crates/protocol` crate (`mumcord-protocol`) and relicensed under `MIT OR Apache-2.0`.
- The default client release name servers show is now `mumble-client-protocol <version>`; `set_client_release` still overrides it.
