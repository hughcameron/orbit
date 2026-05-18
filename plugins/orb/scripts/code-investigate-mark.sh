#!/usr/bin/env bash
# Record a /orb:code-investigate invocation in the session-state marker.
# Reads by plugins/orb/hooks/code-investigate-nudge.sh (the PreToolUse hook).
#
# Usage: code-investigate-mark.sh <kind> <path>
#   <kind>  file | scope
#   <path>  repo-relative file path (kind=file) or directory/module prefix (kind=scope)
#
# Marker format: .orbit/.code-investigate-recent
#   Line 1: <session-id>             (header — matches .orbit/.session-id)
#   Line 2..N: <unix-ts>\t<kind>\t<path>
#
# Lifecycle: session-scoped. Stale-session markers (header != current
# .session-id) are overwritten on the next mark; the hook also treats
# them as empty. No clock TTL.

set -euo pipefail

if [ $# -ne 2 ]; then
  echo "usage: code-investigate-mark.sh <file|scope> <path>" >&2
  exit 2
fi

kind="$1"
path="$2"

case "$kind" in
  file|scope) ;;
  *) echo "kind must be 'file' or 'scope', got: $kind" >&2; exit 2 ;;
esac

if [ ! -d .orbit ]; then
  echo "no .orbit/ directory; skipping mark" >&2
  exit 0
fi

marker=".orbit/.code-investigate-recent"
session_id_file=".orbit/.session-id"

current_sid=""
if [ -f "$session_id_file" ]; then
  current_sid="$(head -1 "$session_id_file" 2>/dev/null || true)"
fi

# Preserve entries only when the marker is fresh (header matches current sid).
# Empty current_sid matches empty marker_sid — supports the no-active-session
# case (both states absent).
existing_entries=""
if [ -f "$marker" ]; then
  marker_sid="$(head -1 "$marker" 2>/dev/null || true)"
  if [ "$marker_sid" = "$current_sid" ]; then
    existing_entries="$(tail -n +2 "$marker" 2>/dev/null || true)"
  fi
fi

ts="$(date +%s)"
new_entry="$(printf '%s\t%s\t%s' "$ts" "$kind" "$path")"

# Atomic write — temp file in same dir, then rename.
tmp="$(mktemp "${marker}.XXXXXX")"
trap 'rm -f "$tmp"' EXIT

{
  printf '%s\n' "$current_sid"
  if [ -n "$existing_entries" ]; then
    printf '%s\n' "$existing_entries"
  fi
  printf '%s\n' "$new_entry"
} > "$tmp"

mv "$tmp" "$marker"
trap - EXIT

exit 0
