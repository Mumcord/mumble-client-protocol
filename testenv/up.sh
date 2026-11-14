#!/bin/bash
# Usage: ./up.sh [v15|v14|v13|limit ...]   (no argument starts all four)
# Creates a throwaway SuperUser password (.env) and a self-signed certificate on first use; both
# stay on this machine and are git-ignored.
set -euo pipefail
cd "$(dirname "$0")"
. ./env.sh
services=("$@")
[ ${#services[@]} -gt 0 ] || services=(v15 v14 v13 limit)
for svc in "${services[@]}"; do
  case "$svc" in v15|v14|v13|limit) ;; *) echo "unknown service: $svc" >&2; exit 2;; esac
done
command -v sqlite3 >/dev/null || { echo "sqlite3 is required on the host to seed channels" >&2; exit 1; }
if [ ! -f .env ]; then
  umask 077
  printf 'SUPERUSER_PASSWORD=%s\nSERVER_PASSWORD=\n' "$(openssl rand -hex 16)" > .env
  echo "created .env with a random SuperUser password"
fi
set -a; . ./.env; set +a
if [ ! -f certs/cert.pem ]; then
  mkdir -p certs
  openssl req -x509 -newkey rsa:2048 -nodes -days 3650 -subj "/CN=mumble-client-protocol test" \
    -keyout certs/key.pem -out certs/cert.pem 2>/dev/null
  echo "generated the server certificate"
fi
PUID=$(id -u)
PGID=$(id -g)
export PUID PGID
for svc in "${services[@]}"; do mkdir -p "$DATA_ROOT/$svc"; done
docker compose up -d --build --force-recreate "${services[@]}"
for svc in "${services[@]}"; do
  [ -f "$DATA_ROOT/$svc/.seeded" ] && continue
  db="$DATA_ROOT/$svc/mumble-server.sqlite"; [ "$svc" = v13 ] && db="$DATA_ROOT/$svc/murmur.sqlite"
  # The schema appears early but is rolled back if the container stops mid-initialisation; the
  # root channel row is written once the server has booted its virtual server.
  ready=0
  for _ in $(seq 120); do
    if [ -f "$db" ] && [ "$(sqlite3 "$db" "select count(*) from channels where channel_id = 0" 2>/dev/null)" = 1 ]; then
      ready=1; break
    fi
    sleep 1
  done
  [ "$ready" = 1 ] || { echo "$svc did not initialise its database within 120s" >&2; docker compose logs --tail 20 "$svc" >&2; exit 1; }
  # The server only reads channels at startup, so seed while it is stopped.
  docker compose stop "$svc" >/dev/null
  sqlite3 "$db" "insert or ignore into channels (server_id, channel_id, parent_id, name, inheritacl) values
    (1,1,0,'Lobby',1),(1,2,0,'Raid',1),(1,3,0,'AFK',1);"
  touch "$DATA_ROOT/$svc/.seeded"
  docker compose up -d "$svc"
done
echo "up: ${services[*]} (ports from $MUMBLE_PORT_BASE, project $MUMBLE_PROJECT)"
