//! A minimal text bot: connects, prints every text message it receives and answers `!ping` with
//! `pong`.
//!
//! ```text
//! cargo run --example text_bot -- <host> [port] [username]
//! ```
//!
//! It accepts any server certificate, which is only acceptable against your own local test
//! server (see `testenv/`). A real client pins the certificate or asks the user.

use std::sync::Arc;
use std::time::Duration;

use mumble_client_protocol::{ConnectConfig, Event, TextTarget, connect};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let host = args
        .next()
        .ok_or("usage: text_bot <host> [port] [username]")?;
    let port = args.next().map(|p| p.parse()).transpose()?.unwrap_or(64738);
    let username = args.next().unwrap_or_else(|| "text-bot".to_owned());

    let mut connection = connect(ConnectConfig {
        host,
        port,
        username,
        password: None,
        tokens: Vec::new(),
        identity: None,
        verify_cert: Arc::new(|_cert| true),
        login_timeout: Duration::from_secs(10),
    })
    .await?;
    println!("connected; waiting for text messages");

    while let Some(event) = connection.events.recv().await {
        match event {
            Event::TextMessage(text) => {
                let sender = text.sender.as_ref().map_or("server", |u| u.name.as_str());
                println!("{sender}: {}", text.message);
                if text.message.trim() == "!ping" {
                    // Answer where the message came from: privately or in the same channels.
                    let reply_to = match text.actor {
                        Some(session) if !text.target.sessions.is_empty() => {
                            TextTarget::user(session)
                        }
                        _ => text.target.clone(),
                    };
                    connection.handle.send_text(&reply_to, "pong")?;
                }
            }
            Event::Disconnected(reason) => {
                println!("disconnected: {reason:?}");
                break;
            }
            _ => {}
        }
    }
    Ok(())
}
