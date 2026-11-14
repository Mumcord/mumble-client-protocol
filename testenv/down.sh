#!/bin/bash
# Usage: ./down.sh [--purge]   Stops this project's servers; --purge also deletes their data.
set -euo pipefail
cd "$(dirname "$0")"
. ./env.sh
PUID=$(id -u)
PGID=$(id -g)
# Compose needs the variable to parse the file even when it only stops containers.
export PUID PGID SUPERUSER_PASSWORD="${SUPERUSER_PASSWORD:-unused}"
docker compose down --remove-orphans
if [ "${1:-}" = --purge ]; then rm -rf "$DATA_ROOT"; fi
