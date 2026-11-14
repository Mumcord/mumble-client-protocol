# shellcheck shell=bash
# Sourced by up.sh and down.sh. Derives the compose variables from two optional overrides:
#   MUMBLE_PORT_BASE   host port of v15 (default 64738); v14, v13 and limit follow at +1..+3
#   MUMBLE_PROJECT     compose project name (default mcp-testenv); each project keeps its own data
#                      dir, so parallel instances never share a database
MUMBLE_PORT_BASE="${MUMBLE_PORT_BASE:-64738}"
MUMBLE_PROJECT="${MUMBLE_PROJECT:-mcp-testenv}"
export COMPOSE_PROJECT_NAME="$MUMBLE_PROJECT"
export PORT_V15=$((MUMBLE_PORT_BASE)) PORT_V14=$((MUMBLE_PORT_BASE + 1))
export PORT_V13=$((MUMBLE_PORT_BASE + 2)) PORT_LIMIT=$((MUMBLE_PORT_BASE + 3))
export DATA_ROOT="data/$MUMBLE_PROJECT"
