#!/usr/bin/env bash
# rally-coherence-scan.sh — enforce rally SKILL.md coherence per spec
# specs/2026-04-19-rally-subagent-model/spec.yaml (v1.3), ac-15 + ac-17.
#
# Two checks:
#   (1) Keyword scan (ac-15): forbidden phrases absent; required phrases present.
#   (2) Adjacency scan (ac-17): every enforcement verb in §2, §4a, §7, §11 is
#       within the tuned window of either a primitive citation, a trust-marker
#       phrase, or a negation whitelist token.
#
# Exit 0 on PASS, non-zero on FAIL. Failures name the offending line.

set -euo pipefail

SKILL="${1:-plugins/orb/skills/rally/SKILL.md}"

if [[ ! -f "$SKILL" ]]; then
  echo "FAIL: SKILL.md not found at $SKILL" >&2
  exit 2
fi

fail=0

# ---------------------------------------------------------------------------
# (1) Keyword scan — ac-15
# ---------------------------------------------------------------------------

declare -a FORBIDDEN=(
  "tools allow-list"
  "drive's stage logic inline"
  "writes are blocked"
)

declare -a REQUIRED=(
  "trust + post-verify"
  "recursive context separation"
  "drive-full"
  "nested forked Agents"
)

echo "== Keyword scan =="

for phrase in "${FORBIDDEN[@]}"; do
  if grep -nF -- "$phrase" "$SKILL" > /dev/null 2>&1; then
    echo "FAIL (forbidden phrase present): '$phrase'"
    grep -nF -- "$phrase" "$SKILL" | head -5 | sed 's/^/  /'
    fail=1
  else
    echo "ok   forbidden-absent : '$phrase'"
  fi
done

for phrase in "${REQUIRED[@]}"; do
  if grep -nF -- "$phrase" "$SKILL" > /dev/null 2>&1; then
    echo "ok   required-present: '$phrase'"
  else
    echo "FAIL (required phrase absent): '$phrase'"
    fail=1
  fi
done

# ---------------------------------------------------------------------------
# (2) Adjacency scan — ac-17
# ---------------------------------------------------------------------------
#
# Enforcement verbs (case-insensitive, whole-word):
#   enforced|enforces|blocked|blocks|prevented|prevents|guaranteed|guarantees
#
# Accepted primitive citations (literal; scanner strips backticks before match):
#   "Agent tool", "run_in_background", "git worktree add", "git status --porcelain",
#   "git commit", "git checkout", "git branch", "AskUserQuestion", "SessionStart",
#   "Bash", "Read", "Write", "Edit", "Grep", "Glob", "file-on-disk"
#
# Trust-marker phrases:
#   "trust + post-verify", "post-verify", "convention"
#
# Negation whitelist (waives adjacency requirement when present in window):
#   "not", "no ", "never", "cannot", "without", "trust-only"
#
# Window: 80 characters forward from the end of the enforcement verb.
# The window may span ONE line break but not a blank line.
# The 80-char default was tuned against this SKILL.md — see comment below.

# Tuned window value. Default per spec is 80. A run against the current
# rally/SKILL.md with this scan passes with 80 (no legitimate claim fails).
# If a future SKILL.md edit tightens phrasing, narrow this value and rerun.
WINDOW_CHARS=80

# Sections to scan — the ac-17 named set (§2, §4a, §7, §11).
# Represented as a list of "start heading|end heading" pairs. The end heading
# is the next section's heading; the scan excludes it.
declare -a SECTIONS=(
  '^### 2[.] Propose the Rally|^### 3[.] Initialise Rally State'
  '^### 4[.] Stage: Design|^### 5[.] Consolidated Decision Gate'
  '^### 7[.] Stage: Implementation|^### 8[.] Assurance'
  '^### 11[.] Resumption|^### 12[.] rally[.]yaml Validation'
)

# Enforcement verbs (word-boundary, case-insensitive).
VERB_RE='\b(enforced|enforces|blocked|blocks|prevented|prevents|guaranteed|guarantees)\b'

# Accepted primitive citations (literal fixed strings).
declare -a PRIMITIVES=(
  "Agent tool"
  "run_in_background"
  "git worktree add"
  "git status --porcelain"
  "git commit"
  "git checkout"
  "git branch"
  "AskUserQuestion"
  "SessionStart"
  "Bash"
  "Read"
  "Write"
  "Edit"
  "Grep"
  "Glob"
  "file-on-disk"
)

declare -a TRUST_MARKERS=(
  "trust + post-verify"
  "post-verify"
  "convention"
)

declare -a NEGATIONS=(
  "not"
  "no "
  "never"
  "cannot"
  "without"
  "trust-only"
)

echo
echo "== Adjacency scan (window=${WINDOW_CHARS}) =="

# Extract each target section into a temp buffer, record starting line number
# in the source so we can report true line numbers on failure.
scan_section() {
  local start_re="$1"
  local end_re="$2"

  # Find start line; if absent, skip.
  local start_ln
  start_ln=$(grep -n -E "$start_re" "$SKILL" | head -1 | cut -d: -f1 || true)
  [[ -z "$start_ln" ]] || true
  if [[ -z "$start_ln" ]]; then
    echo "warn section start not found: $start_re"
    return 0
  fi

  local end_ln
  end_ln=$(awk -v start="$start_ln" -v pat="$end_re" \
    'NR > start && $0 ~ pat { print NR; exit }' "$SKILL")
  if [[ -z "$end_ln" ]]; then
    end_ln=$(wc -l < "$SKILL")
    end_ln=$((end_ln + 1))
  fi

  # Read the section lines (start..end-1) as a single block with newlines.
  local section
  section=$(awk -v s="$start_ln" -v e="$end_ln" 'NR >= s && NR < e' "$SKILL")

  # Walk line by line; for each verb hit, build an 80-char window forward
  # including (at most) the next non-blank line; require primitive, trust
  # marker, or negation in the window.
  local lineno line window ok verb_found
  local section_lineno=0
  while IFS= read -r line; do
    section_lineno=$((section_lineno + 1))
    lineno=$((start_ln + section_lineno - 1))

    # Use grep -io to detect a verb hit on this line (case-insensitive).
    if ! echo "$line" | grep -iE -- "$VERB_RE" > /dev/null 2>&1; then
      continue
    fi

    # Build the window: current line (from the first verb match forward) plus
    # up to one following non-blank line, truncated to WINDOW_CHARS.
    # Strip backticks from the window so `Agent tool` and Agent tool both match.
    local verb_offset
    verb_offset=$(echo "$line" | grep -bioE -- "$VERB_RE" | head -1 | cut -d: -f1)
    if [[ -z "$verb_offset" ]]; then continue; fi

    # The window starts at the END of the matched verb — use perl for reliable
    # byte-offset extraction.
    window=$(echo "$line" | perl -e '
      my $line = <STDIN>; chomp $line;
      my $verb = shift;
      my $win = shift;
      if ($line =~ /($verb)/i) {
        my $end = $+[0];
        my $tail = substr($line, $end);
        print substr($tail, 0, $win);
      }
    ' -- "$VERB_RE" "$WINDOW_CHARS")

    # If the remaining tail is shorter than window, append the next non-blank
    # line from the section (bounded by remaining chars).
    local remaining=$((WINDOW_CHARS - ${#window}))
    if (( remaining > 0 )); then
      # Read ahead: find next non-blank line within the section.
      local next_line_offset=$((section_lineno))
      local next_line
      next_line=$(echo "$section" | awk -v s="$next_line_offset" '
        NR > s && NF > 0 { print; exit }
      ')
      if [[ -n "$next_line" ]]; then
        window="$window $(echo "$next_line" | cut -c1-"$remaining")"
      fi
    fi

    # Strip backticks for matching.
    window_stripped=$(echo "$window" | tr -d '`')

    ok=0

    # Check primitives.
    for p in "${PRIMITIVES[@]}"; do
      if [[ "$window_stripped" == *"$p"* ]]; then ok=1; break; fi
    done

    # Check trust markers.
    if (( ok == 0 )); then
      for t in "${TRUST_MARKERS[@]}"; do
        if [[ "$window_stripped" == *"$t"* ]]; then ok=1; break; fi
      done
    fi

    # Check negation whitelist (the negation may precede OR follow the verb
    # within the full line; if present on the line, accept).
    if (( ok == 0 )); then
      for n in "${NEGATIONS[@]}"; do
        if [[ "$line" == *"$n"* ]]; then ok=1; break; fi
      done
    fi

    if (( ok == 0 )); then
      echo "FAIL $SKILL:$lineno — enforcement verb with no primitive/trust/negation within ${WINDOW_CHARS}ch:"
      echo "  $line"
      fail=1
    else
      echo "ok   $SKILL:$lineno — enforcement verb backed within window"
    fi
  done <<< "$section"
}

for pair in "${SECTIONS[@]}"; do
  start_re="${pair%%|*}"
  end_re="${pair##*|}"
  scan_section "$start_re" "$end_re"
done

echo
if (( fail == 0 )); then
  echo "rally-coherence-scan: PASS"
  exit 0
else
  echo "rally-coherence-scan: FAIL"
  exit 1
fi
