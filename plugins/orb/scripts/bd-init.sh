#!/usr/bin/env bash
# bd-init.sh — initialise beads in a project with orbit conventions.
#
# Usage:
#   bd-init.sh [--objective "measurable objective statement"]
#
# This script:
#   1. Runs bd init --quiet (idempotent — safe on already-initialised projects)
#   2. Installs orbit's custom PRIME.md into .beads/PRIME.md
#   3. Optionally sets a pinned memory for the project's objective function
#
# The script is idempotent — running it twice produces the same result.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRIME_TEMPLATE="$SCRIPT_DIR/../PRIME.md"
objective=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --objective) objective="$2"; shift 2 ;;
    -h|--help)
      echo "Usage: bd-init.sh [--objective \"measurable objective statement\"]"
      exit 0
      ;;
    *) echo "bd-init.sh: unknown option: $1" >&2; exit 2 ;;
  esac
done

# Check bd is available
if ! command -v bd &>/dev/null; then
  echo "bd-init.sh: bd not found. Install from ~/github/gastownhall/beads" >&2
  exit 2
fi

# Step 1: Initialise beads (idempotent)
# bd init --quiet skips if already initialised
if [[ -d ".beads" ]]; then
  echo "beads already initialised — skipping bd init"
else
  bd init --quiet 2>&1 | grep -v "^$" || true
  echo "beads initialised"
fi

# Step 2: Install orbit's custom PRIME.md
if [[ ! -f "$PRIME_TEMPLATE" ]]; then
  echo "bd-init.sh: PRIME.md template not found at $PRIME_TEMPLATE" >&2
  exit 2
fi

mkdir -p .beads
cp "$PRIME_TEMPLATE" .beads/PRIME.md
echo "PRIME.md installed → .beads/PRIME.md"

# Step 3: Set objective function as pinned memory (if provided)
if [[ -n "$objective" ]]; then
  bd remember --key "objective-function" "$objective" 2>/dev/null || {
    echo "bd-init.sh: failed to set objective function memory" >&2
    exit 2
  }
  echo "Objective function set: $objective"
fi

echo "orbit beads integration ready"
