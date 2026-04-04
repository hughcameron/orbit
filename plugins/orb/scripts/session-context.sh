#!/usr/bin/env bash
# session-context.sh — SessionStart hook for orbit workflow context
# Checks for in-flight specs, surfaces hard constraints, and suggests the next workflow step.

set -euo pipefail

# Only run if orbit directories exist (project uses orbit)
if [[ ! -d "specs" ]] && [[ ! -d "cards" ]]; then
  exit 0
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
