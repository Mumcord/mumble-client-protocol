# Local test servers

Standard, unmodified Mumble servers in Docker for the live tests in `tests/`. The live tests only
ever connect to these local servers, never to public ones.

Requires Docker with compose, `openssl` and `sqlite3` on the host.

## Start and stop

    testenv/up.sh                 # all four servers
    testenv/up.sh v15 v14         # only some of them
    testenv/down.sh               # stop; add --purge to delete their data too

| Service | Server | Host port (TCP and UDP) |
|---|---|---|
| `v15` | Mumble 1.5.735 | 64738 |
| `v14` | Mumble 1.4.230 | 64739 |
| `v13` | Mumble 1.3 (Debian bullseye `mumble-server` package, built locally on first start) | 64740 |
| `limit` | Mumble 1.5.735 with channel listening disabled | 64741 |

On first use `up.sh` creates:

- `testenv/.env` with a random SuperUser password (the account is `SuperUser`) and an empty
  server password;
- `testenv/certs/` with a self-signed server certificate.

Both are throwaway values for the local servers only and are git-ignored. On the first start of
each server `up.sh` also adds the channels Lobby, Raid and AFK below Root directly in its SQLite
database. Autoban is off, because the tests reconnect often.

## Run the live tests

    testenv/up.sh
    MUMBLE_LIVE=1 cargo test --tests -- --ignored --test-threads=1

The tests read the SuperUser password from `testenv/.env` (or from `SUPERUSER_PASSWORD`).

## Parallel instances

Two optional variables move the servers, for example when the default ports are taken:

    MUMBLE_PORT_BASE=65100 MUMBLE_PROJECT=mine testenv/up.sh
    MUMBLE_PORT_BASE=65100 MUMBLE_LIVE=1 cargo test --tests -- --ignored --test-threads=1
    MUMBLE_PROJECT=mine testenv/down.sh

- `MUMBLE_PORT_BASE` (default 64738): port of `v15`; `v14`, `v13` and `limit` follow at +1 to +3.
- `MUMBLE_PROJECT` (default `mcp-testenv`): compose project name. Each project keeps its data in
  `testenv/data/<project>/`.

## Test variables

| Variable | Meaning |
|---|---|
| `MUMBLE_LIVE=1` | Runs the live tests when they are selected with `--ignored`. |
| `MUMBLE_PORT_BASE` | Port of the `v15` server, as above. |
| `MUMBLE_TESTENV_DIR` | Directory with the `.env` file (default: this directory). |
| `MUMBLE_TEST_SERVER_PASSWORD` | Server password, if the servers have one. |
| `SUPERUSER_PASSWORD` | SuperUser password instead of the one in `.env`. |
