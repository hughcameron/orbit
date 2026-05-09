#!/usr/bin/env bash
# test-promote-gate-propagation.sh — verify promote.sh creates a spec whose
# acceptance_criteria mirror the card scenarios end-to-end against orbit-state.
#
# Synthetic card: two scenarios, one with gate: true, one without.
# Asserts:
#   - stdout is exactly the spec id (no trailing whitespace, no leakage)
#   - `orbit spec show <spec-id>` resolves the new spec
#   - acceptance_criteria has the expected count and gate flags
#   - the spec id has the form `<YYYY-MM-DD>-<slug>`

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PROMOTE="$REPO_ROOT/plugins/orb/scripts/promote.sh"

if [[ ! -x "$PROMOTE" ]]; then
  echo "FAIL: promote.sh not found or not executable at $PROMOTE" >&2
  exit 1
fi

if ! command -v orbit >/dev/null 2>&1; then
  echo "FAIL: orbit binary not on PATH" >&2
  exit 1
fi

# Build a self-contained synthetic card YAML inside a temporary --root so the
# real `.orbit/` is untouched.
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/.orbit/cards" "$TMP/.orbit/specs" "$TMP/.orbit/choices"
CARD="$TMP/.orbit/cards/0999-promote-gate-propagation.yaml"
cat > "$CARD" <<'EOF'
feature: test card for promote.sh gate propagation
as_a: test
i_want: promote.sh to emit gate true on the right AC
so_that: gate semantics propagate from card scenario to spec acceptance_criteria
goal: verify gate propagation through promote.sh end-to-end against orbit-state
maturity: planned
scenarios:
- name: gate scenario
  given: a card scenario with gate true
  when: promote.sh runs
  then: the resulting acceptance_criteria entry has gate true
  gate: true
- name: non-gate scenario
  given: a card scenario without gate
  when: promote.sh runs
  then: the resulting acceptance_criteria entry has gate false
specs: []
EOF

# Canonicalise the synthetic card so orbit accepts it.
orbit --root "$TMP" canonicalise >/dev/null

# Run promote.sh non-dry-run, capture stdout (the spec id) and stderr separately.
STDOUT_FILE="$TMP/promote.stdout"
STDERR_FILE="$TMP/promote.stderr"
"$PROMOTE" "$CARD" --root "$TMP" >"$STDOUT_FILE" 2>"$STDERR_FILE" || {
  echo "FAIL: promote.sh exited non-zero" >&2
  echo "--- stderr ---" >&2
  cat "$STDERR_FILE" >&2
  exit 1
}

SPEC_ID="$(cat "$STDOUT_FILE")"
echo "Spec id: $SPEC_ID"

# Assert stdout shape: exactly one line, matching `<YYYY-MM-DD>-<slug>`
if [[ "$(wc -l <"$STDOUT_FILE")" != "1" ]]; then
  echo "FAIL: stdout is not exactly one line" >&2
  exit 1
fi
if ! [[ "$SPEC_ID" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}-promote-gate-propagation$ ]]; then
  echo "FAIL: spec id $SPEC_ID does not match expected shape" >&2
  exit 1
fi

# Resolve the spec via orbit and inspect acceptance_criteria.
SHOW_JSON=$(orbit --root "$TMP" --json spec show "$SPEC_ID")
echo "$SHOW_JSON" | python3 -c "
import json, sys

env = json.load(sys.stdin)
if not env.get('ok'):
    print('FAIL: orbit spec show returned ok=false')
    sys.exit(1)

spec = env['data']['result']['spec']
acs = spec.get('acceptance_criteria', [])

if len(acs) != 2:
    print(f'FAIL: expected 2 ACs, got {len(acs)}')
    sys.exit(1)

ac01, ac02 = acs[0], acs[1]
if ac01['id'] != 'ac-01' or not ac01.get('gate'):
    print(f'FAIL: ac-01 missing or not gate=true: {ac01}')
    sys.exit(1)
if ac02['id'] != 'ac-02' or ac02.get('gate'):
    print(f'FAIL: ac-02 missing or wrongly gate=true: {ac02}')
    sys.exit(1)
if ac01.get('checked') or ac02.get('checked'):
    print('FAIL: a freshly promoted AC is already checked')
    sys.exit(1)

print('PASS: 2 ACs, ac-01 gate=true, ac-02 gate=false, both unchecked')
"

# Verify orbit's substrate is clean after the port.
orbit --root "$TMP" verify >/dev/null || {
  echo "FAIL: orbit verify returned non-zero after promote.sh" >&2
  exit 1
}

echo "OK: promote.sh ↔ orbit-state end-to-end (gate propagation, spec id contract, verify clean)"
