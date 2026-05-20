#!/usr/bin/env bash
# PreToolUse hook for the /orb:act-when-authorised discipline.
#
# Fires on AskUserQuestion tool calls. When the calling agent is in
# mid-session autonomy under /orb:drive or /orb:rally, prints the
# three substrate-typed questions to stderr and exits non-zero to
# suppress the halt — forcing the agent to either act on the answers
# or escalate via the structural NO-GO path.
#
# Scope-gating (per spec 2026-05-19-act-when-authorised ac-01):
#   - Only fires when ORBIT_NONINTERACTIVE=1 (autonomy contract active)
#   - AND at least one .orbit/specs/<id>/drive.yaml exists (a drive is in flight)
# Both conditions must hold. Either absent → exit 0, allow the halt.
#
# Graceful degradation:
#   - skip silently when .orbit/ is absent (non-orbit repos that happen
#     to load the plugin)
#   - skip silently when jq is unavailable (cannot parse the input)
#   - skip silently when tool_name is not AskUserQuestion (defence in
#     depth; matcher should already filter)

set -uo pipefail

# Read tool-call JSON from stdin (Claude Code hook contract).
input="$(cat 2>/dev/null || true)"

# Need jq to parse the input; if absent, silently no-op.
command -v jq >/dev/null 2>&1 || exit 0

tool_name="$(printf '%s' "$input" | jq -r '.tool_name // empty' 2>/dev/null)"

# Defence in depth — matcher should already gate to AskUserQuestion.
[ "$tool_name" = "AskUserQuestion" ] || exit 0

# Scope-gate: only fire under autonomy.
[ "${ORBIT_NONINTERACTIVE:-0}" = "1" ] || exit 0

# Scope-gate: only fire when a drive is in flight (drive.yaml present).
[ -d .orbit/specs ] || exit 0
shopt -s nullglob
drive_yamls=(.orbit/specs/*/drive.yaml)
shopt -u nullglob
[ "${#drive_yamls[@]}" -gt 0 ] || exit 0

# Both gates passed — print the three substrate-typed questions to stderr
# and exit non-zero to suppress the halt.
cat >&2 <<'EOF'
three-question test (mid-autonomy halt-temptation guard):

  1. Recommendation — do I have a single concrete action I am prepared
     to take? (not a menu of options)

  2. Evidence — do I have evidence to act on it? (a memory key, an AC
     text, a prior decision file, or substrate I can cite)

  3. Authorisation — does the contract authorise me? Check:
       - drive.yaml.autonomy (guided | full | supervised)
       - memory mid-session-autonomy-contract-default-to-action-halt
       - spec halt-conditions for the current stage

Three yeses → act, do not ask. One or more no → escalate via the
structural NO-GO path (single-strike park with reason_label), not via
AskUserQuestion.

Severity is reviewer-language, not autonomy-language: REQUEST_CHANGES
under guided / full autonomy is absorbed by the cycle budget. The
Decision Brief frame is for closing recommendations, not in-flight
decisions — mid-autonomy takes the imperative single-action form.

Halt suppressed. Reverse via ORBIT_NONINTERACTIVE=0 in the calling
environment if the halt is genuinely required.
EOF

exit 1
