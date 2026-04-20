#!/usr/bin/env bash
# session-context.sh — SessionStart hook for orbit workflow context
# Checks for in-flight specs, surfaces hard constraints, and suggests the next workflow step.

set -euo pipefail

# Gate: only run on repos using the orbit layout.
# Four states (matching plugins/orb/skills/setup/SKILL.md §1):
#   1. orbit/ present                       → run the full hook
#   2. legacy bare dirs present, no orbit/  → emit one-line nudge, exit 0
#   3. orbit/ AND bare dirs both present    → run the full hook, warn about mixed state
#   4. neither                              → exit silently
legacy_bare_present=0
for d in cards specs decisions discovery; do
  if [[ -d "$d" ]]; then
    legacy_bare_present=1
    break
  fi
done

if [[ ! -d "orbit" ]]; then
  if [[ "$legacy_bare_present" -eq 1 ]]; then
    echo "orbit: legacy layout detected. Run /orb:setup to migrate."
  fi
  exit 0
fi

if [[ "$legacy_bare_present" -eq 1 ]]; then
  echo "orbit: mixed layout detected (orbit/ AND bare artefact dirs both present)."
  echo "  Run /orb:setup to review — it will refuse and report the collisions."
fi

# Surface outstanding memos (not yet referenced by any card)
if [[ -d "orbit/cards/memos" ]]; then
  memo_files=$(find orbit/cards/memos -maxdepth 1 -name '*.md' 2>/dev/null | sort)
  if [[ -n "$memo_files" ]]; then
    # Collect all references from card YAML files
    all_refs=$(grep -h '^\s*- ' orbit/cards/*.yaml 2>/dev/null | grep 'orbit/cards/memos/' | sed 's/^[[:space:]]*- //' | sed 's/^"//' | sed 's/"$//' || true)

    outstanding=()
    while IFS= read -r memo; do
      # Check if any card references this memo path
      if ! echo "$all_refs" | grep -qF "$memo"; then
        outstanding+=("$(basename "$memo")")
      fi
    done <<< "$memo_files"

    if [[ ${#outstanding[@]} -gt 0 ]]; then
      echo "orbit: ${#outstanding[@]} outstanding memo(s) in orbit/cards/memos/:"
      for name in "${outstanding[@]}"; do
        echo "  - $name"
      done
    fi
  fi
fi

# Detect active rally (orbit/specs/rally.yaml)
# When a rally is active, it becomes the primary orchestration context
# and individual drive states are subordinated (shown as sub-items, not independent status lines).
active_rally=""
rally_file="orbit/specs/rally.yaml"
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
elif [[ -d "orbit/specs" ]]; then
  # Guard on dir existence: under `set -euo pipefail`, a missing orbit/specs
  # would cause `find` to exit 1 and abort the hook. The hook must survive
  # partial orbit/ layouts (e.g. a manually-created orbit/ without subdirs).
  drive_file=$(find orbit/specs -maxdepth 2 -name 'drive.yaml' 2>/dev/null | head -1 || true)
else
  drive_file=""
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

# Find the most recent spec directory.
# Guard on dir existence so pipefail doesn't kill the hook when orbit/ was
# created manually without its standard subdirs.
latest_spec=""
if [[ -d "orbit/specs" ]]; then
  latest_spec=$(find orbit/specs -maxdepth 1 -type d -name '20*' 2>/dev/null | sort -r | head -1 || true)
fi

if [[ -z "$latest_spec" ]]; then
  # No specs yet — check for cards
  card_count=0
  if [[ -d "orbit/cards" ]]; then
    card_count=$(find orbit/cards -maxdepth 1 -name '*.yaml' 2>/dev/null | wc -l | tr -d ' ')
  fi
  if [[ "$card_count" -gt 0 ]]; then
    echo "orbit: $card_count card(s) in orbit/cards/. Next step: /orb:design to refine one into a spec."
  fi
  exit 0
fi

topic=$(basename "$latest_spec")
has_interview=$(test -f "$latest_spec/interview.md" && echo 1 || echo 0)
has_spec=$(test -f "$latest_spec/spec.yaml" && echo 1 || echo 0)
has_progress=$(test -f "$latest_spec/progress.md" && echo 1 || echo 0)
has_spec_review=$(find "$latest_spec" -maxdepth 1 -name 'review-spec-*.md' 2>/dev/null | head -1)
has_pr_review=$(find "$latest_spec" -maxdepth 1 -name 'review-pr-*.md' 2>/dev/null | head -1)

# ac-03 / ac-08 — mission-resilience drift check and next-AC surface.
# Canonical drift-notice string (declared in plugins/orb/skills/implement/SKILL.md §4a).
# Any change to the wording is a schema change that routes through card 0009.
DRIFT_NOTICE="spec modified since implementation started, re-review recommended"

if [[ "$has_spec" == "1" && "$has_progress" == "1" ]]; then
  spec_path="$latest_spec/spec.yaml"
  progress_path="$latest_spec/progress.md"

  # Extract recorded Spec hash from progress.md. Raw bytes, no normalisation.
  # ac-03: silently skip the drift check when the field is absent.
  recorded_hash=$(grep -m1 '^Spec hash: sha256:' "$progress_path" 2>/dev/null | sed 's/^Spec hash: sha256://' || true)

  if [[ -n "$recorded_hash" ]]; then
    # Compute sha256 over raw bytes — no line-ending conversion, no trimming.
    # shasum -a 256 on macOS, sha256sum on Linux. Both read in binary mode by default.
    if command -v sha256sum >/dev/null 2>&1; then
      current_hash=$(sha256sum "$spec_path" | awk '{print $1}')
    else
      current_hash=$(shasum -a 256 "$spec_path" | awk '{print $1}')
    fi

    if [[ "$current_hash" != "$recorded_hash" ]]; then
      echo ""
      echo "orbit: $DRIFT_NOTICE"
    fi
  fi

  # ac-08 — surface next unchecked AC from ## Acceptance Criteria.
  # Parser discipline (card 0009 ac-09): ## Detours content is ignored — only
  # lines inside ## Acceptance Criteria decide AC status.
  #
  # Card 0003 ac-08 refactored the previously-inlined parsing into a single
  # shared helper at plugins/orb/scripts/parse-progress.sh. Both the next-AC
  # surfacing below and the resume reconcile block further down delegate to
  # that helper. After merge, there is exactly one progress.md parser in
  # the repo (ac-08(d)/(e)).
  PARSE_PROGRESS="$(dirname "$0")/parse-progress.sh"

  # Fetch the first unchecked AC (if any) via the shared helper.
  next_ac_line=$("$PARSE_PROGRESS" next-unchecked-ac "$progress_path" 2>/dev/null || true)

  if [[ -n "$next_ac_line" ]]; then
    # Split tab-separated <ac-id>\t<is_gate> without external tools.
    next_id="${next_ac_line%%	*}"
    next_is_gate="${next_ac_line##*	}"

    if [[ "$next_is_gate" == "1" ]]; then
      # Ask the helper which AC becomes startable after the gate.
      post_gate_id=$("$PARSE_PROGRESS" post-gate-ac "$progress_path" "$next_id" 2>/dev/null || true)
      if [[ -n "$post_gate_id" ]]; then
        echo "orbit: Next AC — $next_id is a blocking gate. $post_gate_id becomes startable once $next_id closes."
      else
        echo "orbit: Next AC — $next_id is a blocking gate. No AC follows it."
      fi
    else
      echo "orbit: Next AC — $next_id"
    fi
  else
    # No unchecked AC found. Only comment if the section is present.
    if grep -q '^## Acceptance Criteria' "$progress_path" 2>/dev/null; then
      echo "orbit: No unchecked ACs remain in $topic/progress.md."
    fi
  fi

  # ac-03 — resume reconcile surface for orbit-implement tasks (card 0003).
  #
  # This block runs AFTER card 0009's pre-AC check sequence (backfill → drift
  # check → gate surfacing) so the drift notice, if any, is already emitted.
  # If the drift check halted the session non-interactively (exit 1), the
  # hook never reaches here — which is the required skip behaviour (spec
  # constraint #13 and ac-03 description).
  #
  # Claude Code's Task tools (TaskCreate, TaskList, TaskUpdate) are agent-turn
  # primitives, not callable from a SessionStart bash hook. The hook's role
  # is therefore limited to (1) surfacing that a reconcile is pending so the
  # agent's first action on resume runs the §5 reconcile algorithm, and
  # (2) invoking the shared parser so ac-08(d) and ac-08(e) are satisfied.
  # The reconcile algorithm itself — the filter by metadata.spec_path, the
  # in-sync / drift comparison, and the cancel-then-recreate on drift — is
  # authored in implement/SKILL.md §5 and executed by the agent.
  reconcile_spec_path=$("$PARSE_PROGRESS" spec-path "$progress_path" 2>/dev/null || true)
  if [[ -n "$reconcile_spec_path" ]] && "$PARSE_PROGRESS" has-unchecked "$progress_path" 2>/dev/null; then
    echo "orbit: Task list reconcile pending — /orb:implement will reconcile TaskList against progress.md on next turn (scoped by metadata.spec_path=$reconcile_spec_path)."
  fi
fi

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
