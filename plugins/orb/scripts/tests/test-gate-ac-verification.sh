#!/usr/bin/env bash
# test-gate-ac-verification.sh — exercise review-spec Pass 1's gate-AC description check
#
# Pipes a synthetic acceptance_criteria JSON array (one passing gate AC + one
# failing gate AC) into orbit-acceptance.sh acs --stdin, then runs the three
# deterministic rules (non-empty / not-placeholder / minimum-length ≥20 chars)
# against each is_gate=1 row.
#
# This is the regression guard for the parser+rule semantics that the review-spec
# skill executes conceptually. The skill itself has no inline test code — this
# fixture proves the semantics survive parser refactors.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
PARSE="$REPO_ROOT/plugins/orb/scripts/orbit-acceptance.sh"

if [[ ! -x "$PARSE" ]]; then
  echo "FAIL: orbit-acceptance.sh not found or not executable at $PARSE" >&2
  exit 1
fi

# Synthetic acceptance_criteria array (the shape orbit spec show --json emits
# under data.result.spec.acceptance_criteria):
#   ac-01 — gate, 51 chars, not placeholder → PASS all three rules
#   ac-02 — gate, 3 chars, "TBD"            → FAIL placeholder + length rules
SYNTH=$(cat <<'EOF'
[
  {"id": "ac-01", "description": "Decide hash algorithm before drift detection", "gate": true, "checked": false},
  {"id": "ac-02", "description": "TBD", "gate": true, "checked": false}
]
EOF
)

# Deterministic-rule predicates — bash implementation of review-spec Pass 1 step 5
PLACEHOLDER_TOKENS_REGEX='^(TBD|TODO|FIXME|PLACEHOLDER|XXX|\?\?\?)$'

check_rules() {
  local desc="$1"
  local trimmed
  trimmed="$(echo "$desc" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
  local results=()

  # Non-empty
  if [[ -n "$trimmed" ]]; then
    results+=("non-empty:PASS")
  else
    results+=("non-empty:FAIL")
  fi

  # Not a placeholder token (case-insensitive)
  local upper="${trimmed^^}"
  if [[ "$upper" =~ $PLACEHOLDER_TOKENS_REGEX ]]; then
    results+=("not-placeholder:FAIL")
  else
    results+=("not-placeholder:PASS")
  fi

  # Minimum length ≥20
  if (( ${#trimmed} >= 20 )); then
    results+=("min-length:PASS")
  else
    results+=("min-length:FAIL")
  fi

  printf '%s\n' "${results[@]}"
}

# Parse the synthetic field via orbit-acceptance.sh (stdin mode bypasses the
# orbit binary and reads the raw JSON array directly).
PARSED=$(printf '%s' "$SYNTH" | "$PARSE" acs --stdin)

if [[ -z "$PARSED" ]]; then
  echo "FAIL: orbit-acceptance.sh produced no output" >&2
  exit 1
fi

OVERALL_PASS=1

while IFS=$'\t' read -r ac_id status desc is_gate; do
  if [[ "$is_gate" != "1" ]]; then
    continue
  fi

  echo "Checking gate AC: $ac_id"
  echo "  description: '$desc'"

  # Run the three deterministic rules
  rule_results=$(check_rules "$desc")
  echo "$rule_results" | sed 's/^/  /'

  # Per-AC pass/fail aggregation
  if echo "$rule_results" | grep -q "FAIL"; then
    failed=$(echo "$rule_results" | grep "FAIL" | cut -d: -f1 | paste -sd, -)
    case "$ac_id" in
      ac-01)
        echo "  -> UNEXPECTED FAIL on ac-01 (rules: $failed)"
        OVERALL_PASS=0
        ;;
      ac-02)
        # Expect placeholder + length to fail
        if [[ "$failed" == "not-placeholder,min-length" ]]; then
          echo "  -> FAIL (expected — placeholder+length): $failed"
        else
          echo "  -> UNEXPECTED FAIL pattern on ac-02 (got: $failed; expected: not-placeholder,min-length)"
          OVERALL_PASS=0
        fi
        ;;
      *)
        echo "  -> UNEXPECTED gate AC id $ac_id"
        OVERALL_PASS=0
        ;;
    esac
  else
    case "$ac_id" in
      ac-01) echo "  -> PASS (all three rules)" ;;
      ac-02)
        echo "  -> UNEXPECTED PASS on ac-02 (should have failed placeholder+length)"
        OVERALL_PASS=0
        ;;
      *)
        echo "  -> UNEXPECTED gate AC id $ac_id"
        OVERALL_PASS=0
        ;;
    esac
  fi
done <<< "$PARSED"

if [[ "$OVERALL_PASS" == "1" ]]; then
  echo "OK: gate-AC verification rules behave as expected (ac-01 PASS, ac-02 FAIL on placeholder+length)"
  exit 0
else
  echo "FAIL: gate-AC verification rules did not behave as expected" >&2
  exit 1
fi
