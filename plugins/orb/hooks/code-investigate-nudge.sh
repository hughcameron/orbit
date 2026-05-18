#!/usr/bin/env bash
# PreToolUse hook for the /orb:code-investigate discipline.
#
# Fires on Edit/Write tool calls. Reads the session-state marker at
# .orbit/.code-investigate-recent (written by code-investigate-mark.sh)
# and emits a non-blocking soft warning when the file being edited has
# no recent investigation in this session.
#
# Never blocks — always exits 0. Edits proceed regardless.
#
# Graceful degradation:
#   - skip silently when .orbit/ is absent (non-orbit repos that
#     happen to load the plugin)
#   - skip silently for paths under .orbit/, .claude/, or *.lock
#   - warn when .orbit/ exists but the marker is missing or session-stale

set -uo pipefail

# Read tool-call JSON from stdin (Claude Code hook contract).
input="$(cat 2>/dev/null || true)"

# Need jq to parse the input; if absent, silently no-op.
command -v jq >/dev/null 2>&1 || exit 0

tool_name="$(printf '%s' "$input" | jq -r '.tool_name // empty' 2>/dev/null)"
file_path="$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty' 2>/dev/null)"

case "$tool_name" in
  Edit|Write) ;;
  *) exit 0 ;;
esac

[ -n "$file_path" ] || exit 0

# Resolve to a repo-relative path. Hooks run with cwd at repo root by
# Claude Code convention; the file_path may be absolute or relative.
rel_path="${file_path#$PWD/}"
rel_path="${rel_path#./}"

# Path filter — skip substrate, plugin config, lockfiles.
case "$rel_path" in
  .orbit/*|.claude/*|*.lock) exit 0 ;;
esac

# Graceful degradation — skip when .orbit/ absent.
[ -d .orbit ] || exit 0

marker=".orbit/.code-investigate-recent"
session_id_file=".orbit/.session-id"

warn() {
  echo "consider /orb:code-investigate before editing $rel_path" >&2
}

if [ ! -f "$marker" ]; then
  warn
  exit 0
fi

current_sid=""
if [ -f "$session_id_file" ]; then
  current_sid="$(head -1 "$session_id_file" 2>/dev/null || true)"
fi

marker_sid="$(head -1 "$marker" 2>/dev/null || true)"

# Session-stale marker → treat as empty, fire warning.
# Empty current_sid matches empty marker_sid — supports the no-active-session
# case where both the mark script and the hook see the same absent state.
if [ "$marker_sid" != "$current_sid" ]; then
  warn
  exit 0
fi

# Scan entries for a file-exact or scope-prefix match.
matched=0
while IFS=$'\t' read -r _ts kind entry_path; do
  [ -z "${kind:-}" ] && continue
  case "$kind" in
    file)
      if [ "$entry_path" = "$rel_path" ]; then
        matched=1
        break
      fi
      ;;
    scope)
      case "$rel_path" in
        "$entry_path"|"$entry_path"/*) matched=1; break ;;
      esac
      ;;
  esac
done < <(tail -n +2 "$marker" 2>/dev/null || true)

if [ "$matched" -eq 0 ]; then
  warn
fi

exit 0
