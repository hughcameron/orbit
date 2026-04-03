#!/usr/bin/env bash
# session-context.sh — SessionStart hook for orbit workflow context
# Checks for in-flight specs and suggests the next workflow step.

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
has_spec_review=$(ls "$latest_spec"/review-spec-*.md 2>/dev/null | head -1)
has_pr_review=$(ls "$latest_spec"/review-pr-*.md 2>/dev/null | head -1)

if [[ "$has_spec" == "0" && "$has_interview" == "1" ]]; then
  echo "orbit: $topic — interview done, needs spec. Next: /orb:spec $latest_spec/interview.md"
elif [[ "$has_spec" == "1" && -z "$has_spec_review" && -z "$has_pr_review" ]]; then
  echo "orbit: $topic — spec ready. Next: implement or /orb:review-spec $latest_spec/spec.yaml"
elif [[ "$has_spec" == "1" && -n "$has_spec_review" && -z "$has_pr_review" ]]; then
  echo "orbit: $topic — spec reviewed, implement and then /orb:review-pr"
elif [[ -n "$has_pr_review" ]]; then
  echo "orbit: $topic — review complete. Ready to merge."
fi
