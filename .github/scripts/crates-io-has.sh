#!/usr/bin/env bash
# Succeeds if crates.io already has the given version of a package:
#   crates-io-has.sh mumble-client-protocol 0.1.0
# Asks the crates.io index, which is what cargo itself reads. Used by the release workflow to
# skip versions that are already published and to wait until a new one can be downloaded.
set -euo pipefail

name=${1:?usage: crates-io-has.sh <package> <version>}
version=${2:?usage: crates-io-has.sh <package> <version>}

# Index layout for names of four or more characters: first two letters / next two / name.
entry=$(mktemp)
trap 'rm -f "$entry"' EXIT
url="https://index.crates.io/${name:0:2}/${name:2:2}/$name"
status=$(curl --silent --show-error --location --output "$entry" \
  --write-out '%{http_code}' "$url")
case "$status" in
  200) jq --exit-status --slurp --arg version "$version" \
         'any(.[]; .vers == $version)' "$entry" > /dev/null ;;
  404) exit 1 ;; # the package does not exist yet
  *)
    echo "crates.io index answered $status for $name" >&2
    exit 2
    ;;
esac
