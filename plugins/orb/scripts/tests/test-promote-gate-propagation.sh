#!/usr/bin/env bash
# test-promote-gate-propagation.sh — verify promote.sh emits [gate] when card scenario sets gate: true
#
# Synthetic card: two scenarios, one with gate: true, one without.
# Asserts the dry-run AC output marks the right line with [gate].

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROMOTE="$REPO_ROOT/plugins/orb/scripts/promote.sh"

if [[ ! -x "$PROMOTE" ]]; then
  echo "FAIL: promote.sh not found or not executable at $PROMOTE" >&2
  exit 1
fi

# Build a self-contained synthetic card YAML in a temp file
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

CARD="$TMP/0999-test-card.yaml"
cat > "$CARD" <<'EOF'
feature: test card for promote.sh gate propagation
as_a: test
i_want: promote.sh to emit [gate] when scenario.gate is true
so_that: gate semantics propagate end-to-end from card to bead

scenarios:
  - name: gate scenario
    given: a card scenario with gate true
    when: promote.sh runs
    then: the resulting AC line carries [gate]
    gate: true

  - name: non-gate scenario
    given: a card scenario without gate
    when: promote.sh runs
    then: the resulting AC line has no [gate] marker

goal: "verify gate propagation through promote.sh"
maturity: planned
specs: []
EOF

# Run promote.sh --dry-run and capture output
OUT=$("$PROMOTE" "$CARD" --dry-run 2>&1)

# Parse the acceptance lines
AC1_LINE=$(echo "$OUT" | grep -E '^- \[ \] ac-01' || true)
AC2_LINE=$(echo "$OUT" | grep -E '^- \[ \] ac-02' || true)

echo "ac-01 line: $AC1_LINE"
echo "ac-02 line: $AC2_LINE"

# Assertions — check for the [gate] MARKER (between ac-NN and the colon),
# not arbitrary [gate] substrings inside scenario then-clauses.
if [[ "$AC1_LINE" =~ ^-\ \[\ \]\ ac-01\ \[gate\]: ]]; then
  echo "PASS: ac-01 (gate scenario) carries [gate] marker"
else
  echo "FAIL: ac-01 (gate scenario) does NOT carry [gate] marker" >&2
  exit 1
fi

if [[ "$AC2_LINE" =~ ^-\ \[\ \]\ ac-02\ \[gate\]: ]]; then
  echo "FAIL: ac-02 (non-gate scenario) wrongly carries [gate] marker" >&2
  exit 1
else
  echo "PASS: ac-02 (non-gate scenario) has no [gate] marker"
fi

echo "OK: gate propagation verified end-to-end (card scenario gate: true → promote.sh AC [gate] marker)"
