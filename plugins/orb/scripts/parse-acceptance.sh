#!/usr/bin/env bash
# parse-acceptance.sh — acceptance field parser for orbit's beads integration.
#
# Reads a bead's acceptance_criteria field (via bd show --json) and parses
# the orbit AC convention: `- [ ] ac-NN [gate]: description`
#
# See .orbit/conventions/acceptance-field.md for format specification.
#
# ## Contract
#
# Subcommands take a bead ID and query bd show --json for the acceptance field.
# Use --stdin to read acceptance text directly (for testing without bd).
#
#   parse-acceptance.sh acs <bead-id|--stdin>
#     One line per AC, tab-separated:
#       <ac-id>\t<status>\t<description>\t<is_gate>
#     Where:
#       ac-id       — e.g. ac-01
#       status      — "[ ]" or "[x]"
#       description — the AC text after the colon, trimmed
#       is_gate     — "1" if the line contains "[gate]", "0" otherwise
#     Lines not matching the convention regex are skipped with a warning
#     on stderr.
#
#   parse-acceptance.sh next-ac <bead-id|--stdin>
#     Emits the first unchecked AC that is not blocked by an unchecked gate,
#     tab-separated: <ac-id>\t<is_gate>. If all ACs are checked, emits
#     nothing and exits 0.
#
#   parse-acceptance.sh blocking-gate <bead-id|--stdin>
#     Emits the first unchecked gate AC: <ac-id>\t<description>.
#     If no unchecked gate exists, emits nothing and exits 0.
#
#   parse-acceptance.sh has-unchecked <bead-id|--stdin>
#     Exit 0 if at least one unchecked AC exists, exit 1 otherwise.
#     Emits nothing on stdout.
#
#   parse-acceptance.sh check <bead-id> <ac-id>
#     Marks the named AC as checked in the bead's acceptance field via
#     bd update --acceptance. Exits nonzero if the AC is not found or
#     is already checked.
#
# ## Error handling
#
# Missing bead / bd error   → exit 2, message on stderr
# Unknown subcommand        → exit 2, message on stderr
# Empty acceptance field    → exit 0 with empty stdout (not an error)
# Malformed line            → warning on stderr, line skipped

set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: parse-acceptance.sh <subcommand> <bead-id|--stdin> [extra]
Subcommands:
  acs            — list ACs: <ac-id>\t<status>\t<description>\t<is_gate>
  next-ac        — first unchecked AC respecting gate blocks: <ac-id>\t<is_gate>
  blocking-gate  — first unchecked gate: <ac-id>\t<description>
  has-unchecked  — exit 0 if any unchecked AC, else exit 1
  check          — mark an AC as checked: check <bead-id> <ac-id>
EOF
  exit 2
}

if [[ $# -lt 2 ]]; then
  usage
fi

subcmd="$1"
bead_id="$2"

# Fetch acceptance field text, either from bd or stdin.
get_acceptance() {
  if [[ "$bead_id" == "--stdin" ]]; then
    cat
  else
    local json
    json=$(bd show "$bead_id" --json 2>/dev/null) || {
      echo "parse-acceptance.sh: failed to fetch bead: $bead_id" >&2
      exit 2
    }
    # bd show --json returns a JSON array; extract acceptance_criteria from first element.
    printf '%s' "$json" | python3 -c "
import sys, json
data = json.load(sys.stdin)
item = data[0] if isinstance(data, list) else data
print(item.get('acceptance_criteria', ''))
" 2>/dev/null
  fi
}

case "$subcmd" in
  acs)
    get_acceptance | awk '
      /^- \[[ x]\] ac-[0-9][0-9]/ {
        # Extract check status
        if (match($0, /\[[ x]\]/)) {
          status = substr($0, RSTART, RLENGTH)
        } else { next }
        # Extract ac-id
        if (match($0, /ac-[0-9][0-9][0-9]?/)) {
          id = substr($0, RSTART, RLENGTH)
        } else { next }
        # Detect gate marker
        is_gate = ($0 ~ /\[gate\]/) ? 1 : 0
        # Extract description: everything after "ac-NN [gate]: " or "ac-NN: "
        desc = $0
        p = match(desc, /ac-[0-9][0-9][0-9]?/)
        rest = substr(desc, p + RLENGTH)
        sub(/^[[:space:]]*\[gate\]/, "", rest)
        sub(/^[[:space:]]*:[[:space:]]*/, "", rest)
        sub(/[[:space:]]+$/, "", rest)
        printf "%s\t%s\t%s\t%d\n", id, status, rest, is_gate
        next
      }
      /^- / {
        # Line starts with "- " but does not match convention — warn
        printf "parse-acceptance.sh: skipping malformed line: %s\n", $0 > "/dev/stderr"
      }
    '
    ;;

  next-ac)
    # First unchecked AC that is not blocked by an unchecked gate.
    # Gate enforcement: an unchecked gate blocks all subsequent ACs.
    get_acceptance | awk '
      /^- \[[ x]\] ac-[0-9][0-9]/ {
        if (match($0, /\[[ x]\]/)) {
          status = substr($0, RSTART, RLENGTH)
        } else { next }
        match($0, /ac-[0-9][0-9][0-9]?/)
        id = substr($0, RSTART, RLENGTH)
        is_gate = ($0 ~ /\[gate\]/) ? 1 : 0
        checked = (status == "[x]") ? 1 : 0

        if (!checked) {
          # This is unchecked. If we have not been blocked by a prior
          # unchecked gate, this is the next AC.
          if (!blocked) {
            printf "%s\t%d\n", id, is_gate
            exit
          }
        }
        # If this AC is an unchecked gate, everything after it is blocked.
        if (is_gate && !checked) {
          blocked = 1
        }
      }
    '
    ;;

  blocking-gate)
    # First unchecked gate AC.
    get_acceptance | awk '
      /^- \[ \] ac-[0-9][0-9]/ && /\[gate\]/ {
        match($0, /ac-[0-9][0-9][0-9]?/)
        id = substr($0, RSTART, RLENGTH)
        # Extract description
        desc = $0
        p = match(desc, /ac-[0-9][0-9][0-9]?/)
        rest = substr(desc, p + RLENGTH)
        sub(/^[[:space:]]*\[gate\]/, "", rest)
        sub(/^[[:space:]]*:[[:space:]]*/, "", rest)
        sub(/[[:space:]]+$/, "", rest)
        printf "%s\t%s\n", id, rest
        exit
      }
    '
    ;;

  has-unchecked)
    found=$(get_acceptance | awk '
      /^- \[ \] ac-[0-9][0-9]/ { print "1"; exit }
    ')
    if [[ -n "$found" ]]; then exit 0; else exit 1; fi
    ;;

  check)
    if [[ $# -ne 3 ]]; then
      echo "parse-acceptance.sh: check requires <bead-id> <ac-id>" >&2
      exit 2
    fi
    ac_id="$3"
    # Fetch current acceptance field
    current=$(get_acceptance)
    if [[ -z "$current" ]]; then
      echo "parse-acceptance.sh: empty acceptance field for bead $bead_id" >&2
      exit 2
    fi
    # Check if the AC exists and is unchecked
    if ! echo "$current" | grep -q "^- \[ \] ${ac_id}"; then
      if echo "$current" | grep -q "^- \[x\] ${ac_id}"; then
        echo "parse-acceptance.sh: $ac_id is already checked" >&2
        exit 1
      fi
      echo "parse-acceptance.sh: $ac_id not found in acceptance field" >&2
      exit 2
    fi
    # Replace the unchecked marker with checked for the specific AC
    updated=$(echo "$current" | sed "s/^- \[ \] ${ac_id}/- [x] ${ac_id}/")
    bd update "$bead_id" --acceptance "$updated" --json >/dev/null 2>&1 || {
      echo "parse-acceptance.sh: failed to update bead $bead_id" >&2
      exit 2
    }
    echo "Checked: $ac_id"
    ;;

  *)
    usage
    ;;
esac
