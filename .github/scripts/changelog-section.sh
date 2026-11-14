#!/usr/bin/env bash
# Prints the body of one CHANGELOG.md section, without its heading:
#   changelog-section.sh 0.1.0        -> the lines below "## [0.1.0] ..."
#   changelog-section.sh Unreleased   -> the lines below "## [Unreleased]"
# Fails if the section does not exist or is empty. Used by the release workflow for the version
# check and for the notes of the GitHub release.
set -euo pipefail

section=${1:?usage: changelog-section.sh <version|Unreleased>}
file=${2:-CHANGELOG.md}

body=$(awk -v want="## [$section]" '
  index($0, "## [") == 1 { inside = (index($0, want) == 1); next }
  inside { print }
' "$file" | sed -e '/./,$!d' | tac | sed -e '/./,$!d' | tac)

if [[ -z "$body" ]]; then
  echo "CHANGELOG.md has no section [$section] (or it is empty)." >&2
  exit 1
fi
printf '%s\n' "$body"
