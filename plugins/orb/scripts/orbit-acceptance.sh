#!/usr/bin/env bash
# orbit-acceptance.sh — acceptance field parser for orbit-state specs.
#
# Reads a spec's structured acceptance_criteria via `orbit spec show <id> --json`
# and emits the same tuple contract as parse-acceptance.sh, so skills written
# against the bd-era contract translate verbatim apart from the script name.
#
# ## Contract
#
# Subcommands take a spec id and query `orbit spec show --json` for the
# acceptance_criteria array. Use --stdin to feed a raw `orbit` JSON envelope
# directly (for testing without orbit-state).
#
#   orbit-acceptance.sh acs <spec-id|--stdin>
#     One line per AC, tab-separated:
#       <ac-id>\t<status>\t<description>\t<is_gate>
#     Where:
#       ac-id       — e.g. ac-01
#       status      — "[ ]" or "[x]"
#       description — the AC description, trimmed
#       is_gate     — "1" if gate=true, "0" otherwise
#
#   orbit-acceptance.sh next-ac <spec-id|--stdin>
#     Emits the first unchecked AC that is not blocked by an unchecked gate,
#     tab-separated: <ac-id>\t<is_gate>. If all ACs are checked, emits
#     nothing and exits 0.
#
#   orbit-acceptance.sh blocking-gate <spec-id|--stdin>
#     Emits the first unchecked gate AC: <ac-id>\t<description>.
#     If no unchecked gate exists, emits nothing and exits 0.
#
#   orbit-acceptance.sh has-unchecked <spec-id|--stdin>
#     Exit 0 if at least one unchecked AC exists, exit 1 otherwise.
#     Emits nothing on stdout.
#
#   orbit-acceptance.sh check <spec-id> <ac-id>
#     Marks the named AC as checked via `orbit spec update --ac-checked`.
#     Exits nonzero if the AC is not found or already checked.
#
# ## Error handling
#
# Missing spec / orbit error → exit 2, message on stderr
# Unknown subcommand          → exit 2, message on stderr
# Empty acceptance_criteria   → exit 0 with empty stdout (not an error)

set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: orbit-acceptance.sh <subcommand> <spec-id|--stdin> [extra]
Subcommands:
  acs            — list ACs: <ac-id>\t<status>\t<description>\t<is_gate>
  next-ac        — first unchecked AC respecting gate blocks: <ac-id>\t<is_gate>
  blocking-gate  — first unchecked gate: <ac-id>\t<description>
  has-unchecked  — exit 0 if any unchecked AC, else exit 1
  check          — mark an AC as checked: check <spec-id> <ac-id>
EOF
  exit 2
}

if [[ $# -lt 2 ]]; then
  usage
fi

subcmd="$1"
spec_id="$2"

# Fetch acceptance_criteria as JSON, either from orbit or stdin.
# Output: a JSON array of {id, description, gate, checked} objects.
get_acs_json() {
  if [[ "$spec_id" == "--stdin" ]]; then
    cat
  else
    local envelope
    envelope=$(orbit --json spec show "$spec_id" 2>/dev/null) || {
      echo "orbit-acceptance.sh: failed to fetch spec: $spec_id" >&2
      exit 2
    }
    printf '%s' "$envelope" | python3 -c "
import sys, json
env = json.load(sys.stdin)
if not env.get('ok'):
    sys.exit(2)
acs = env.get('data', {}).get('result', {}).get('spec', {}).get('acceptance_criteria', [])
json.dump(acs, sys.stdout)
" 2>/dev/null
  fi
}

emit_acs_tuples() {
  python3 -c "
import sys, json
acs = json.load(sys.stdin)
for ac in acs:
    ac_id = ac.get('id', '')
    desc = (ac.get('description', '') or '').strip()
    is_gate = '1' if ac.get('gate') else '0'
    status = '[x]' if ac.get('checked') else '[ ]'
    print('\t'.join([ac_id, status, desc, is_gate]))
"
}

case "$subcmd" in
  acs)
    get_acs_json | emit_acs_tuples
    ;;

  next-ac)
    # First unchecked AC not blocked by an unchecked gate.
    get_acs_json | python3 -c "
import sys, json
acs = json.load(sys.stdin)
blocked = False
for ac in acs:
    checked = bool(ac.get('checked'))
    is_gate = bool(ac.get('gate'))
    if not checked and not blocked:
        print(f\"{ac.get('id','')}\t{1 if is_gate else 0}\")
        sys.exit(0)
    if is_gate and not checked:
        blocked = True
"
    ;;

  blocking-gate)
    get_acs_json | python3 -c "
import sys, json
acs = json.load(sys.stdin)
for ac in acs:
    if not ac.get('checked') and ac.get('gate'):
        desc = (ac.get('description','') or '').strip()
        print(f\"{ac.get('id','')}\t{desc}\")
        sys.exit(0)
"
    ;;

  has-unchecked)
    found=$(get_acs_json | python3 -c "
import sys, json
acs = json.load(sys.stdin)
for ac in acs:
    if not ac.get('checked'):
        print('1')
        sys.exit(0)
")
    if [[ -n "$found" ]]; then exit 0; else exit 1; fi
    ;;

  check)
    if [[ $# -ne 3 ]]; then
      echo "orbit-acceptance.sh: check requires <spec-id> <ac-id>" >&2
      exit 2
    fi
    ac_id="$3"
    # Verify the AC exists and is unchecked.
    state=$(get_acs_json | python3 -c "
import sys, json
acs = json.load(sys.stdin)
target = '$ac_id'
for ac in acs:
    if ac.get('id') == target:
        print('checked' if ac.get('checked') else 'unchecked')
        sys.exit(0)
print('missing')
")
    case "$state" in
      missing)   echo "orbit-acceptance.sh: $ac_id not found on spec $spec_id" >&2; exit 2 ;;
      checked)   echo "orbit-acceptance.sh: $ac_id is already checked" >&2; exit 1 ;;
      unchecked) ;;
      *)         echo "orbit-acceptance.sh: unexpected state for $ac_id: $state" >&2; exit 2 ;;
    esac
    orbit --json spec update "$spec_id" --ac-check "$ac_id" >/dev/null 2>&1 || {
      echo "orbit-acceptance.sh: failed to update spec $spec_id" >&2
      exit 2
    }
    echo "Checked: $ac_id"
    ;;

  *)
    usage
    ;;
esac
