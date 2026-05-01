#!/usr/bin/env bash
# promote.sh — card-to-bead promotion for orbit's beads integration.
#
# Reads a card YAML file and creates a beads task with acceptance criteria
# derived from the card's scenarios.
#
# Usage:
#   promote.sh <card-path> [--parent <epic-id>] [--dry-run]
#
# The module:
#   - Extracts feature, goal, and scenarios from the card YAML
#   - Generates numbered acceptance criteria (ac-01 through ac-NN)
#   - Preserves the card path as a reference in the bead description
#   - Sets issue_type=task, priority=1
#   - Writes the acceptance field in orbit convention format
#   - Does NOT create sub-beads, dependency edges, or memories
#
# Output: the created bead ID on stdout (or full JSON with --json)

set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: promote.sh <card-path> [--parent <epic-id>] [--dry-run]

Options:
  --parent <id>   Set parent bead (for rally grouping)
  --dry-run       Print what would be created without creating it
EOF
  exit 2
}

if [[ $# -lt 1 ]]; then
  usage
fi

card_path="$1"
shift

parent_id=""
dry_run=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --parent) parent_id="$2"; shift 2 ;;
    --dry-run) dry_run=1; shift ;;
    *) echo "promote.sh: unknown option: $1" >&2; usage ;;
  esac
done

if [[ ! -f "$card_path" ]]; then
  echo "promote.sh: card not found: $card_path" >&2
  exit 2
fi

# Extract fields from card YAML using python3 (available on all target systems).
# This avoids a yq dependency.
read_card() {
  python3 -c "
import yaml, sys, json

with open('$card_path') as f:
    card = yaml.safe_load(f)

out = {
    'feature': card.get('feature', ''),
    'goal': card.get('goal', ''),
    'scenarios': [],
}

for s in card.get('scenarios', []):
    out['scenarios'].append({
        'name': s.get('name', ''),
        'given': s.get('given', ''),
        'when': s.get('when', ''),
        'then': s.get('then', ''),
        'gate': bool(s.get('gate', False)),
    })

json.dump(out, sys.stdout)
" 2>/dev/null
}

card_json=$(read_card)
if [[ -z "$card_json" ]]; then
  echo "promote.sh: failed to parse card: $card_path" >&2
  exit 2
fi

feature=$(echo "$card_json" | python3 -c "import sys,json; print(json.load(sys.stdin)['feature'])")
goal=$(echo "$card_json" | python3 -c "import sys,json; print(json.load(sys.stdin)['goal'])")
scenario_count=$(echo "$card_json" | python3 -c "import sys,json; print(len(json.load(sys.stdin)['scenarios']))")

if [[ "$scenario_count" -eq 0 ]]; then
  echo "promote.sh: card has no scenarios: $card_path" >&2
  exit 2
fi

# Build the bead title from the card feature
title="$feature"

# Build the description with card reference and goal
description="Promoted from: $card_path

Goal: $goal"

# Generate acceptance criteria from scenarios.
# Each scenario becomes an AC. The naming convention uses the scenario name.
acceptance=$(echo "$card_json" | python3 -c "
import sys, json

card = json.load(sys.stdin)
lines = []
for i, s in enumerate(card['scenarios'], 1):
    ac_id = f'ac-{i:02d}'
    name = s['name']
    then_clause = s['then']
    gate_marker = ' [gate]' if s.get('gate') else ''
    lines.append(f'- [ ] {ac_id}{gate_marker}: {name} — {then_clause}')
print('\n'.join(lines))
")

if [[ "$dry_run" -eq 1 ]]; then
  echo "=== DRY RUN ==="
  echo "Title: $title"
  echo "Description:"
  echo "$description"
  echo ""
  echo "Acceptance criteria:"
  echo "$acceptance"
  if [[ -n "$parent_id" ]]; then
    echo "Parent: $parent_id"
  fi
  exit 0
fi

# Build bd create command
bd_args=(
  create
  "$title"
  -t task
  -p 1
  --acceptance "$acceptance"
  --json
)

if [[ -n "$parent_id" ]]; then
  bd_args+=(--parent "$parent_id")
fi

# Pipe description via stdin
bead_json=$(echo "$description" | bd "${bd_args[@]}" --stdin 2>/dev/null) || {
  echo "promote.sh: bd create failed" >&2
  exit 2
}

# Extract and output the bead ID
bead_id=$(echo "$bead_json" | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")
echo "$bead_id"
