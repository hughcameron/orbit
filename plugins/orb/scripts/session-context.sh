#!/usr/bin/env bash
# session-context.sh — SessionStart hook for orbit workflow context
# Checks for in-flight specs, surfaces hard constraints, and suggests the next workflow step.

set -euo pipefail

# Only run if orbit directories exist (project uses orbit)
if [[ ! -d "specs" ]] && [[ ! -d "cards" ]]; then
  exit 0
fi

# Surface outstanding memos (not yet referenced by any card)
if [[ -d "cards/memos" ]]; then
  memo_files=$(find cards/memos -maxdepth 1 -name '*.md' 2>/dev/null | sort)
  if [[ -n "$memo_files" ]]; then
    # Collect all references from card YAML files
    all_refs=$(grep -h '^\s*- ' cards/*.yaml 2>/dev/null | grep 'cards/memos/' | sed 's/^[[:space:]]*- //' | sed 's/^"//' | sed 's/"$//' || true)

    outstanding=()
    while IFS= read -r memo; do
      # Check if any card references this memo path
      if ! echo "$all_refs" | grep -qF "$memo"; then
        outstanding+=("$(basename "$memo")")
      fi
    done <<< "$memo_files"

    if [[ ${#outstanding[@]} -gt 0 ]]; then
      echo "orbit: ${#outstanding[@]} outstanding memo(s) in cards/memos/:"
      for name in "${outstanding[@]}"; do
        echo "  - $name"
      done
    fi
  fi
fi

# Detect active rally (specs/rally.yaml)
# When a rally is active, it becomes the primary orchestration context
# and individual drive states are subordinated (shown as sub-items, not independent status lines).
active_rally=""
rally_file="specs/rally.yaml"
if [[ -f "$rally_file" ]]; then
  # Parse top-level fields tolerantly: missing fields yield empty strings, never abort the hook.
  # `|| true` keeps set -e / pipefail from killing the script when grep finds no match.
  rally_goal=$( { grep '^rally:' "$rally_file" 2>/dev/null || true; } | head -1 | sed 's/^rally:[[:space:]]*//; s/^"//; s/"$//')
  rally_phase=$( { grep '^phase:' "$rally_file" 2>/dev/null || true; } | head -1 | sed 's/^phase:[[:space:]]*//; s/^"//; s/"$//')
  rally_autonomy=$( { grep '^autonomy:' "$rally_file" 2>/dev/null || true; } | head -1 | sed 's/^autonomy:[[:space:]]*//; s/^"//; s/"$//')

  # Validate required fields. If rally.yaml exists but is malformed, surface a clear error
  # and skip the rally display — do not silently abort the hook.
  if [[ -z "$rally_phase" || -z "$rally_goal" ]]; then
    missing=()
    [[ -z "$rally_goal"  ]] && missing+=("rally")
    [[ -z "$rally_phase" ]] && missing+=("phase")
    echo "orbit: rally.yaml at $rally_file is malformed — missing required field(s): ${missing[*]}"
    echo "  Fix the file or remove it to start fresh. Rally state will not be surfaced."
  elif [[ "$rally_phase" != "complete" ]]; then
    active_rally="$rally_file"

    # Count cards by status using awk — emits a single integer even when no matches.
    # (grep -c || echo 0 is unsafe: grep prints "0" AND exit is nonzero, so || echo 0
    #  concatenates "0\n0" into the variable.)
    total_cards=$(awk '/^  - path:/ {n++} END {print n+0}' "$rally_file")
    complete_cards=$(awk '/^    status: complete[[:space:]]*$/ {n++} END {print n+0}' "$rally_file")
    parked_cards=$(awk '/^    status: parked[[:space:]]*$/ {n++} END {print n+0}' "$rally_file")

    echo "orbit: Active rally — \"$rally_goal\" ($rally_autonomy mode, phase: $rally_phase)"
    echo "  Cards: $total_cards total, $complete_cards complete, $parked_cards parked"

    # Surface per-card status lines as sub-items
    awk '
      /^  - path:/ {
        path = $0
        sub(/^  - path:[[:space:]]*/, "", path)
        gsub(/"/, "", path)
        card_path = path
        next
      }
      /^    status:/ {
        status = $0
        sub(/^    status:[[:space:]]*/, "", status)
        gsub(/"/, "", status)
        if (card_path != "") {
          printf "    - %s [%s]\n", card_path, status
          card_path = ""
        }
      }
    ' "$rally_file"

    # Surface parked card constraints
    if [[ "$parked_cards" -gt 0 ]]; then
      echo "  Parked constraints:"
      awk '
        /^  - path:/ { path = $0; sub(/^  - path:[[:space:]]*/, "", path); gsub(/"/, "", path); card_path = path; parked = 0; next }
        /^    status: parked/ { parked = 1 }
        /^    parked_constraint:/ {
          if (parked) {
            c = $0
            sub(/^    parked_constraint:[[:space:]]*/, "", c)
            gsub(/"/, "", c)
            printf "    - %s: %s\n", card_path, c
          }
        }
      ' "$rally_file"
    fi

    echo "  Next: run /orb:rally to resume at phase \"$rally_phase\""
  elif [[ "$rally_phase" == "complete" ]]; then
    echo "orbit: Completed rally — \"$rally_goal\" (awaiting archival on next rally)"
  fi
fi

# Detect active drives (drive.yaml in any spec directory)
# v1 constraint: single drive at a time outside a rally. Inside a rally, drive states are
# subordinated to the rally display above and not re-surfaced here.
active_drive=""
if [[ -n "$active_rally" ]]; then
  drive_file=""
else
  drive_file=$(find specs -maxdepth 2 -name 'drive.yaml' 2>/dev/null | head -1)
fi
if [[ -n "$drive_file" ]]; then
  drive_dir=$(dirname "$drive_file")
  # Parse drive.yaml fields (lightweight — no yq dependency)
  drive_card=$(grep '^card:' "$drive_file" 2>/dev/null | sed 's/^card:[[:space:]]*//')
  drive_autonomy=$(grep '^autonomy:' "$drive_file" 2>/dev/null | sed 's/^autonomy:[[:space:]]*//')
  drive_iteration=$(grep '^iteration:' "$drive_file" 2>/dev/null | sed 's/^iteration:[[:space:]]*//')
  drive_budget=$(grep '^budget:' "$drive_file" 2>/dev/null | sed 's/^budget:[[:space:]]*//')
  drive_status=$(grep '^status:' "$drive_file" 2>/dev/null | sed 's/^status:[[:space:]]*//')
  drive_current_spec=$(grep '^current_spec:' "$drive_file" 2>/dev/null | sed 's/^current_spec:[[:space:]]*//')

  if [[ "$drive_status" != "complete" && "$drive_status" != "escalated" ]]; then
    active_drive="$drive_file"

    # Determine suggested next action from status
    case "$drive_status" in
      design)      drive_next="run /orb:drive $drive_card $drive_autonomy to resume at design" ;;
      spec)        drive_next="run /orb:drive $drive_card $drive_autonomy to resume at spec generation" ;;
      review-spec) drive_next="run /orb:drive $drive_card $drive_autonomy to resume at spec review" ;;
      implement)   drive_next="run /orb:drive $drive_card $drive_autonomy to resume at implementation" ;;
      review)      drive_next="run /orb:drive $drive_card $drive_autonomy to resume at PR review" ;;
      *)        drive_next="run /orb:drive $drive_card $drive_autonomy to resume" ;;
    esac

    echo "orbit: Active drive — $drive_card ($drive_autonomy mode, iteration $drive_iteration/$drive_budget, status: $drive_status)"
    echo "  Next: $drive_next"
  elif [[ "$drive_status" == "escalated" ]]; then
    echo "orbit: ESCALATED drive — $drive_card (exhausted $drive_budget iterations). Card needs human rethinking."
  fi
fi

# Find the most recent spec directory
latest_spec=$(find specs -maxdepth 1 -type d -name '20*' 2>/dev/null | sort -r | head -1)

if [[ -z "$latest_spec" ]]; then
  # No specs yet — check for cards
  card_count=$(find cards -maxdepth 1 -name '*.yaml' 2>/dev/null | wc -l | tr -d ' ')
  if [[ "$card_count" -gt 0 ]]; then
    echo "orbit: $card_count card(s) in cards/. Next step: /orb:design to refine one into a spec."
  fi
  exit 0
fi

topic=$(basename "$latest_spec")
has_interview=$(test -f "$latest_spec/interview.md" && echo 1 || echo 0)
has_spec=$(test -f "$latest_spec/spec.yaml" && echo 1 || echo 0)
has_spec_review=$(find "$latest_spec" -maxdepth 1 -name 'review-spec-*.md' 2>/dev/null | head -1)
has_pr_review=$(find "$latest_spec" -maxdepth 1 -name 'review-pr-*.md' 2>/dev/null | head -1)

if [[ "$has_spec" == "0" && "$has_interview" == "1" ]]; then
  echo "orbit: $topic — interview done, needs spec. Next: /orb:spec $latest_spec/interview.md"
elif [[ "$has_spec" == "1" && -z "$has_spec_review" && -z "$has_pr_review" ]]; then
  echo "orbit: $topic — spec ready. Next: implement or /orb:review-spec $latest_spec/spec.yaml"
elif [[ "$has_spec" == "1" && -n "$has_spec_review" && -z "$has_pr_review" ]]; then
  echo "orbit: $topic — spec reviewed, implement and then /orb:review-pr"
elif [[ -n "$has_pr_review" ]]; then
  echo "orbit: $topic — review complete. Ready to merge."
fi

# Surface hard constraints from the in-flight spec
# Constraints are non-negotiable and must be visible even if /orb:implement is never invoked
if [[ "$has_spec" == "1" && -z "$has_pr_review" ]]; then
  spec_file="$latest_spec/spec.yaml"

  # Parse YAML constraints block, joining multi-line entries into single lines.
  # Strategy: extract the constraints block, then collapse continuation lines
  # (lines that don't start with "  - ") onto the previous entry.
  constraints_block=$(sed -n '/^constraints:/,/^[a-z_]*:/{
    /^constraints:/d
    /^[a-z_]*:/d
    p
  }' "$spec_file" 2>/dev/null)

  if [[ -n "$constraints_block" ]]; then
    # Collapse multi-line YAML entries: join continuation lines with the previous "- " line
    constraints=$(echo "$constraints_block" | awk -v sq="'" '
      /^  - / {
        if (line != "") print line
        line = $0
        sub(/^  - /, "", line)
        sub(/^"/, "", line)
        sub(/"$/, "", line)
        if (substr(line,1,1) == sq) line = substr(line, 2)
        if (substr(line,length(line),1) == sq) line = substr(line, 1, length(line)-1)
        next
      }
      {
        cont = $0
        gsub(/^[[:space:]]+/, "", cont)
        sub(/"$/, "", cont)
        if (substr(cont,length(cont),1) == sq) cont = substr(cont, 1, length(cont)-1)
        line = line " " cont
      }
      END { if (line != "") print line }
    ')

    if [[ -n "$constraints" ]]; then
      echo ""
      echo "orbit: Hard constraints ($topic):"
      while IFS= read -r constraint; do
        constraint=$(echo "$constraint" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')
        if [[ -n "$constraint" ]]; then
          echo "  - $constraint"
        fi
      done <<< "$constraints"
    fi
  fi
fi
