//! Registered users against the local test server.
//! Start the server first, then:
//! `MUMBLE_LIVE=1 cargo test --test live_registered_users -- --ignored --test-threads=1`
//!
//! What the run measures on mumble-server 1.5.735 with the default settings:
//! - an entry-less `UserList` asks for the list, and the server answers with every account it has;
//!   it sends the list **only** when asked, so a change is followed by a fresh query and never by a
//!   push. A client should therefore show the server's list after saving, not its own wish;
//! - `UserState` with `user_id = 0` registers an online person under their current name. The person
//!   and the admin both learn the new id through `UserState`, so a client can stop offering
//!   registration afterwards;
//! - one `UserList` entry with a name renames that account, one without a name removes it, and the
//!   person can connect again as a guest (no `user_id`);
//! - a rename onto a name another account holds is **not** refused: the server logs it, keeps the
//!   old name in its list and gives no answer of its own, so the caller has to read the list back;
//! - the server reports `last_seen` and `last_channel` for an account it has seen;
//! - a client without the Register right is refused with `PermissionDenied` naming `Perm::Register`
//!   in the root channel, so a client can hide user management.
//!
//! The test server may be shared with other test runs, so the run removes every account it created.

use std::sync::Arc;
use std::time::Duration;

use mumble_client_protocol::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use mumble_client_protocol::{
    ClientIdentity, ConnectConfig, Connection, DenyKind, Event, Handle, RegisteredUser, connect,
    permissions, proto,
};
use tokio::sync::mpsc::UnboundedReceiver;

/// Port of the v15 server; `MUMBLE_PORT_BASE` moves it like `testenv/up.sh` does.
fn v15() -> u16 {
    std::env::var("MUMBLE_PORT_BASE")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(64738)
}
const WAIT: Duration = Duration::from_secs(10);

/// Names this test owns, so another test run cannot collide with them.
const GUEST: &str = "live-reg-guest";
const OTHER: &str = "live-reg-other";
const RENAMED: &str = "live-reg-renamed";

/// A client certificate that stays the same across reconnects, so the same person can be checked
/// before and after their registration is removed.
struct Person {
    name: String,
    cert: Vec<u8>,
    key: Vec<u8>,
}

fn person(name: &str) -> Person {
    let generated = rcgen::generate_simple_self_signed(vec![name.to_owned()]).expect("cert");
    Person {
        name: name.to_owned(),
        cert: generated.cert.der().to_vec(),
        key: generated.signing_key.serialize_der(),
    }
}

impl Person {
    fn client(&self) -> ClientIdentity {
        ClientIdentity {
            chain: vec![CertificateDer::from(self.cert.clone())],
            key: PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(self.key.clone())),
        }
    }

    async fn connect(&self) -> Connection {
        connect(config(&self.name, self.client(), false))
            .await
            .unwrap_or_else(|error| panic!("{} could not connect: {error}", self.name))
    }
}

fn superuser_password() -> Option<String> {
    let from_env = std::env::var("SUPERUSER_PASSWORD")
        .ok()
        .filter(|p| !p.is_empty());
    from_env.or_else(|| {
        let dir = std::env::var("MUMBLE_TESTENV_DIR")
            .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/testenv").to_owned());
        std::fs::read_to_string(format!("{dir}/.env"))
            .ok()?
            .lines()
            .filter_map(|line| line.split_once('='))
            .find(|(key, _)| *key == "SUPERUSER_PASSWORD")
            .map(|(_, value)| value.trim().to_owned())
            .filter(|value| !value.is_empty())
    })
}

fn config(login: &str, ident: ClientIdentity, admin: bool) -> ConnectConfig {
    ConnectConfig {
        host: "127.0.0.1".to_owned(),
        port: v15(),
        username: login.to_owned(),
        password: if admin { superuser_password() } else { None },
        tokens: Vec::new(),
        identity: Some(ident),
        verify_cert: Arc::new(|_| true),
        login_timeout: WAIT,
    }
}

/// SuperUser: the login name plus the password. The certificate only names the client.
async fn connect_superuser(name: &str) -> Connection {
    let pki = person(name);
    connect(config("SuperUser", pki.client(), true))
        .await
        .unwrap_or_else(|error| panic!("SuperUser could not connect: {error}"))
}

fn session(handle: &Handle) -> u32 {
    handle.with_state(|state| state.session).expect("session")
}

/// The next list the server sends; it sends one only in answer to a query.
async fn read_list(events: &mut UnboundedReceiver<Event>) -> Vec<RegisteredUser> {
    tokio::time::timeout(WAIT, async {
        loop {
            match events.recv().await {
                Some(Event::UserList(list)) => return list,
                Some(Event::PermissionDenied(denied)) => panic!("refused: {denied:?}"),
                Some(_) => {}
                None => panic!("stream ended"),
            }
        }
    })
    .await
    .expect("no registered users answer")
}

/// Asks for the list the server holds, which is the only way to learn what a change did.
async fn query(handle: &Handle, events: &mut UnboundedReceiver<Event>) -> Vec<RegisteredUser> {
    handle.query_user_list().expect("query");
    handle.flush().await.expect("flush");
    read_list(events).await
}

async fn read_denial(events: &mut UnboundedReceiver<Event>) -> Event {
    tokio::time::timeout(WAIT, async {
        loop {
            match events.recv().await {
                Some(event @ Event::PermissionDenied(_)) => return event,
                Some(_) => {}
                None => panic!("stream ended"),
            }
        }
    })
    .await
    .expect("no refusal")
}

/// Waits until the person's own session carries a registration id, which the server sends to the
/// person as well once it created the account.
async fn wait_registered(events: &mut UnboundedReceiver<Event>, own: u32) -> u32 {
    tokio::time::timeout(WAIT, async {
        loop {
            match events.recv().await {
                Some(Event::UserUpdated(change))
                    if change.user.session == own && change.user.user_id.is_some() =>
                {
                    return change.user.user_id.expect("checked above");
                }
                Some(_) => {}
                None => panic!("stream ended"),
            }
        }
    })
    .await
    .expect("the person was never given a registration id")
}

fn named<'a>(list: &'a [RegisteredUser], name: &str) -> Vec<&'a RegisteredUser> {
    list.iter()
        .filter(|user| user.name.as_deref().is_some_and(|found| found == name))
        .collect()
}

fn entry_of(list: &[RegisteredUser], user_id: u32) -> Option<&RegisteredUser> {
    list.iter().find(|user| user.user_id == user_id)
}

/// Removes the accounts of these names that the server still holds, so a run starts and ends clean
/// on a test server other runs may share.
async fn remove(admin: &mut Connection, names: &[&str]) {
    let list = query(&admin.handle, &mut admin.events).await;
    for entry in list.iter().filter(|user| {
        user.name
            .as_deref()
            .is_some_and(|name| names.contains(&name))
    }) {
        admin
            .handle
            .change_registered_user(entry.user_id, None)
            .expect("remove");
    }
    admin.handle.flush().await.expect("flush");
}

#[tokio::test]
#[ignore = "needs the local test server (testenv/up.sh v15)"]
async fn registers_renames_and_unregisters_an_account() {
    if std::env::var("MUMBLE_LIVE").is_ok_and(|v| v != "1") {
        return;
    }
    let mut admin = connect_superuser("live-reg-admin").await;
    remove(&mut admin, &[GUEST, OTHER, RENAMED]).await;

    // 1. The query answers with the server's accounts (issue: load with an empty `UserList`).
    let before = query(&admin.handle, &mut admin.events).await;
    assert!(
        named(&before, GUEST).is_empty(),
        "the account must not exist before the test registers it"
    );

    // 2. Register an online person: `UserState` with `user_id = 0` takes the
    //    name they connect under and the certificate they presented.
    let tove = person(GUEST);
    let mut guest = tove.connect().await;
    let guest_session = session(&guest.handle);
    admin
        .handle
        .send_admin_user_state(proto::UserState {
            session: Some(guest_session),
            user_id: Some(0),
            ..Default::default()
        })
        .expect("register");
    admin.handle.flush().await.expect("flush");
    let id = wait_registered(&mut guest.events, guest_session).await;

    // 3. The list holds the account under that name, with last seen and last channel.
    let list = query(&admin.handle, &mut admin.events).await;
    let account = named(&list, GUEST)
        .into_iter()
        .next()
        .expect("the registered person is listed");
    assert_eq!(
        account.user_id, id,
        "the id the person learned is the one listed"
    );
    assert!(
        account.last_channel.is_some(),
        "this server reports the channel an account was last seen in"
    );
    assert!(
        account.last_seen.is_some(),
        "this server reports when an account was last used"
    );

    // 4. Rename: one entry with a new name, which the server applies to that id alone.
    admin
        .handle
        .change_registered_user(id, Some(RENAMED))
        .expect("rename");
    let after = query(&admin.handle, &mut admin.events).await;
    assert_eq!(
        entry_of(&after, id).and_then(|user| user.name.as_deref()),
        Some(RENAMED),
        "the account keeps its id under the new name"
    );
    assert!(named(&after, GUEST).is_empty(), "the old name is gone");

    // 5. A second account, then a rename onto its name. The server takes the message without an
    //    answer of its own but keeps the old name, so a caller that does not read the list back
    //    would show a state the server does not have.
    let freya = person(OTHER);
    let other = freya.connect().await;
    admin
        .handle
        .send_admin_user_state(proto::UserState {
            session: Some(session(&other.handle)),
            user_id: Some(0),
            ..Default::default()
        })
        .expect("register");
    admin.handle.flush().await.expect("flush");
    let two = query(&admin.handle, &mut admin.events).await;
    let id2 = named(&two, OTHER)
        .into_iter()
        .next()
        .expect("the second account is listed")
        .user_id;
    assert_ne!(id, id2, "the two accounts have different ids");

    admin
        .handle
        .change_registered_user(id, Some(OTHER))
        .expect("rename onto the other name");
    let kept = query(&admin.handle, &mut admin.events).await;
    assert_eq!(
        named(&kept, OTHER).len(),
        1,
        "the server does not let two accounts hold one name, and says nothing about it — so a
         client must show the list it reads back, not the name that was typed"
    );
    assert_eq!(
        entry_of(&kept, id).and_then(|user| user.name.as_deref()),
        Some(RENAMED),
        "the account that was to be renamed keeps the name the server holds"
    );

    // 6. Unregister: one entry without a name. The account is out of the list,
    //    the other one stands, and the person may connect again — this time as a guest.
    admin
        .handle
        .change_registered_user(id, None)
        .expect("unregister");
    let one = query(&admin.handle, &mut admin.events).await;
    assert!(
        entry_of(&one, id).is_none(),
        "the account is out of the list"
    );
    assert_eq!(named(&one, OTHER).len(), 1, "the other account stays");

    admin
        .handle
        .change_registered_user(id2, None)
        .expect("unregister the second");
    let none = query(&admin.handle, &mut admin.events).await;
    assert!(
        named(&none, OTHER).is_empty() && named(&none, RENAMED).is_empty(),
        "both accounts are out of the list"
    );
    guest.handle.disconnect();
    other.handle.disconnect();
    drop(guest);
    drop(other);
    tokio::time::sleep(Duration::from_millis(500)).await;
    let back = freya.connect().await;
    let own = session(&back.handle);
    let seen = back.handle.with_state(|state| {
        state
            .users
            .values()
            .find(|user| user.session == own)
            .and_then(|user| user.user_id)
    });
    assert_eq!(seen, None, "an unregistered person connects as a guest");
    back.handle.disconnect();

    // 7. Without the Register right the same query is refused, and the server names the bit it
    //    checked, so a client can hide user management.
    let mut plain = person("live-reg-plain").connect().await;
    plain.handle.query_user_list().expect("send");
    plain.handle.flush().await.expect("flush");
    let Event::PermissionDenied(denied) = read_denial(&mut plain.events).await else {
        panic!("expected a refusal")
    };
    assert_eq!(denied.kind, DenyKind::Permission);
    assert_eq!(
        denied.permission,
        Some(permissions::REGISTER),
        "the server checked the Register right in the root channel"
    );
    assert_eq!(denied.channel_id, Some(0));
    plain.handle.disconnect();

    remove(&mut admin, &[GUEST, OTHER, RENAMED]).await;
}
