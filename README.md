# mumble-client-protocol

The client side of the [Mumble](https://www.mumble.info/) voice chat protocol in Rust, on tokio.

- **Control channel:** TLS connection with an optional client certificate, login, and every
  protobuf message of the protocol, behind a `Handle` for commands and a stream of `Event`s.
- **Server state:** the channel tree, connected users and server settings, kept up to date from
  the server's messages.
- **Commands:** text messages, joining and managing channels, whisper and shout targets, channel
  listening, ACL queries, kick and ban, the ban list, registered users, permission queries and
  plugin data.
- **Voice transport:** encrypted UDP (OCB2-AES128) in the legacy and the protobuf packet format,
  with automatic fallback to the TCP tunnel when UDP is blocked and recovery back to UDP.
- **Server query:** version, user count and user limit through the UDP ping, without connecting.
- **Rate limits:** messages the server rate-limits are paced so that none are dropped.

Licence: `MIT OR Apache-2.0`. Minimum Rust version: 1.90.

> **Status: 0.1.** The interface may still change before 1.0; changes are listed in the
> [changelog](CHANGELOG.md).

## Quick start

```toml
[dependencies]
mumble-client-protocol = "0.1"
tokio = { version = "1", features = ["full"] }
```

```rust,no_run
use std::sync::Arc;
use std::time::Duration;

use mumble_client_protocol::{ConnectConfig, Event, connect};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = connect(ConnectConfig {
        host: "localhost".into(),
        port: 64738,
        username: "bot".into(),
        password: None,
        tokens: Vec::new(),
        identity: None,
        // Local test server only: a real client pins the certificate or asks the user.
        verify_cert: Arc::new(|_cert| true),
        login_timeout: Duration::from_secs(10),
    })
    .await?;

    while let Some(event) = connection.events.recv().await {
        match event {
            Event::TextMessage(text) => println!("{}", text.message),
            Event::Disconnected(reason) => {
                println!("disconnected: {reason:?}");
                break;
            }
            _ => {}
        }
    }
    Ok(())
}
```

A complete bot that answers `!ping` is in [`examples/text_bot.rs`](examples/text_bot.rs):

```sh
cargo run --example text_bot -- localhost 64738 text-bot
```

Text, names and comments that come from the server are untrusted input; sanitise them before you
render them as HTML.

## Compatibility

Works with standard, unmodified Mumble servers (murmur / mumble-server) and needs nothing on the
server side. The live tests run against Mumble 1.5, 1.4 and 1.3 servers. Features that need a
newer server (for example channel listening or plugin data) are refused with an error on older
ones instead of being sent.

## Not included

- Audio capture and playback, the Opus codec, jitter buffering and mixing: the voice transport
  carries encoded Opus frames, which the application encodes and decodes.
- A user interface.
- Server-side code.

## Rate limits and disconnecting

Standard servers drop messages that arrive faster than their rate limit allows, so the client
paces chat, own user state, channel, ACL and plugin data messages: a short burst goes out at once,
the rest waits in a queue. `Handle::disconnect` sends what is still queued first, at the allowed
pace, and closes once the server has confirmed it (a ping answered after the last message), or
after 2 s at the latest; anything left over is logged by message kind and count. If the
connection is lost instead, the queue is dropped.

- `Handle::flush().await` waits until everything queued so far has been sent and processed by the
  server, without closing.
- `Handle::closed().await` waits until the connection has stopped sending, e.g. to delay exiting
  the process after `disconnect`.

## Tests

`cargo test` runs the unit tests. The live tests run against local Mumble servers in Docker,
never against public servers; [`testenv/README.md`](testenv/README.md) explains how to start them:

```sh
testenv/up.sh
MUMBLE_LIVE=1 cargo test --tests -- --ignored --test-threads=1
```

## Support

Best effort. Issues and pull requests are read, but there is no guaranteed response time.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Licence

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT licence](LICENSE-MIT), at
your option.

Contributions are accepted under `MIT OR Apache-2.0`, without additional terms.

Parts that come from Mumble keep their BSD-3-Clause licence (`proto/LICENSE.mumble`): the
vendored protocol definitions `proto/Mumble.proto` and `proto/MumbleUDP.proto`, and the OCB2 voice
crypto in `src/voice/crypt.rs`, which is ported from Mumble. See [NOTICE](NOTICE).

The crate started as part of [Mumcord](https://github.com/Mumcord/mumcord); see
[ORIGIN.md](ORIGIN.md).
