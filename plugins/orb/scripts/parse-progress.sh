#!/usr/bin/env bash
# parse-progress.sh — single-source progress.md parser for orbit.
#
# Authored by card 0003 (implement-session-visibility) and consumed by
# both card 0003's resume reconcile and card 0009's next-unchecked-AC
# surfacing in session-context.sh. After this script merges, there is
# exactly one progress.md parser in the repo.
#
# ## Contract
#
# Three subcommands, each reading a progress.md path argument and
# emitting to stdout. All output is deterministic and machine-readable.
#
#   parse-progress.sh acs <progress.md>
#     One line per acceptance criterion, tab-separated:
#       <ac-id>\t<status>\t<description>\t<is_gate>
#     Where:
#       ac-id       — e.g. ac-01
#       status      — "[ ]" or "[x]"
#       description — the AC text after the colon, trimmed
#       is_gate     — "1" if the AC line contains the literal "(gate)"
#                     annotation, "0" otherwise
#     Only lines INSIDE the `## Acceptance Criteria` section are parsed.
#     Content in `## Detours` is IGNORED (card 0009 ac-09 parser
#     discipline): a `- [x] ac-02` inside `## Detours` never flips
#     ac-02's status.
#
#   parse-progress.sh constraints <progress.md>
#     One line per constraint string from the `## Hard Constraints`
#     section. The leading `- [ ] ` or `- [x] ` prefix is stripped;
#     the raw constraint text is emitted verbatim.
#
#   parse-progress.sh spec-path <progress.md>
#     Single line — the value from the `Spec path:` header field.
#     If the field is absent, exit status is nonzero and nothing is
#     printed.
#
#   parse-progress.sh next-unchecked-ac <progress.md>
#     Emits a single tab-separated line identifying the first unchecked
#     AC (in declaration order): <ac-id>\t<is_gate>. If no unchecked AC
#     exists, emits nothing and exits 0. Detour content is ignored.
#
#   parse-progress.sh post-gate-ac <progress.md> <gate-ac-id>
#     Emits the ac-id of the first unchecked AC declared AFTER the named
#     gate AC. Used by the SessionStart hook to inform the agent which
#     AC becomes startable once the blocking gate closes. If no AC
#     follows, emits nothing and exits 0.
#
#   parse-progress.sh has-unchecked <progress.md>
#     Exit 0 if at least one unchecked AC is present in
#     ## Acceptance Criteria, exit 1 otherwise. Emits nothing on stdout.
#
# ## Error handling
#
# Missing file           → exit 2, message on stderr
# Unknown subcommand     → exit 2, message on stderr
# Missing section        → exit 0 with empty stdout (not an error)
# Missing `Spec path:`   → exit 1 with empty stdout (spec-path only)

set -euo pipefail

usage() {
  cat >&2 <<EOF
Usage: parse-progress.sh <subcommand> <progress.md> [extra]
Subcommands:
  acs               — list ACs: <ac-id>\t<status>\t<description>\t<is_gate>
  constraints       — list constraint strings from ## Hard Constraints
  spec-path         — emit the Spec path: field
  next-unchecked-ac — emit first unchecked AC: <ac-id>\t<is_gate>
  post-gate-ac      — emit first unchecked AC after <gate-ac-id>
  has-unchecked     — exit 0 if any unchecked AC, else exit 1
EOF
  exit 2
}

if [[ $# -lt 2 ]]; then
  usage
fi

subcmd="$1"
path="$2"

if [[ ! -f "$path" ]]; then
  echo "parse-progress.sh: file not found: $path" >&2
  exit 2
fi

case "$subcmd" in
  acs)
    # Emit one tab-separated tuple per unchecked / checked AC line inside
    # ## Acceptance Criteria. ## Detours and other sections are ignored.
    awk '
      /^## Acceptance Criteria[[:space:]]*$/ { in_ac=1; next }
      /^## / && in_ac { in_ac=0 }
      in_ac && /^- \[[ x]\] ac-[0-9]+/ {
        # Extract status bracket
        if (match($0, /\[[ x]\]/)) {
          status = substr($0, RSTART, RLENGTH)
        } else { next }
        # Extract ac-id
        if (match($0, /ac-[0-9]+/)) {
          id = substr($0, RSTART, RLENGTH)
        } else { next }
        # Detect gate annotation anywhere on the line
        is_gate = ($0 ~ /\(gate\)/) ? 1 : 0
        # Description is everything after the first ": " that follows the ac-id
        desc = $0
        # Find position after the ac-id
        p = match(desc, /ac-[0-9]+/)
        rest = substr(desc, p + RLENGTH)
        # Strip an optional " (gate)" that immediately follows ac-id
        sub(/^[[:space:]]*\(gate\)/, "", rest)
        # Strip the leading ": "
        sub(/^[[:space:]]*:[[:space:]]*/, "", rest)
        # Trim trailing whitespace
        sub(/[[:space:]]+$/, "", rest)
        printf "%s\t%s\t%s\t%d\n", id, status, rest, is_gate
      }
    ' "$path"
    ;;

  constraints)
    awk '
      /^## Hard Constraints[[:space:]]*$/ { in_hc=1; next }
      /^## / && in_hc { in_hc=0 }
      in_hc && /^- \[[ x]\] / {
        line = $0
        sub(/^- \[[ x]\] /, "", line)
        sub(/[[:space:]]+$/, "", line)
        print line
      }
    ' "$path"
    ;;

  next-unchecked-ac)
    # Emit first unchecked AC: <ac-id>\t<is_gate>. No output if none.
    awk '
      /^## Acceptance Criteria[[:space:]]*$/ { in_ac=1; next }
      /^## / && in_ac { in_ac=0 }
      in_ac && /^- \[ \] ac-[0-9]+/ {
        match($0, /ac-[0-9]+/)
        id = substr($0, RSTART, RLENGTH)
        is_gate = ($0 ~ /\(gate\)/) ? 1 : 0
        printf "%s\t%d\n", id, is_gate
        exit
      }
    ' "$path"
    ;;

  post-gate-ac)
    # Emit first unchecked AC id declared AFTER $3 (the gate ac-id).
    if [[ $# -ne 3 ]]; then
      echo "parse-progress.sh: post-gate-ac requires <progress.md> <gate-ac-id>" >&2
      exit 2
    fi
    gate_id="$3"
    awk -v gate="$gate_id" '
      /^## Acceptance Criteria[[:space:]]*$/ { in_ac=1; next }
      /^## / && in_ac { in_ac=0 }
      in_ac && /^- \[ \] ac-[0-9]+/ {
        match($0, /ac-[0-9]+/)
        id = substr($0, RSTART, RLENGTH)
        if (seen_gate) { print id; exit }
        if (id == gate) seen_gate=1
      }
    ' "$path"
    ;;

  has-unchecked)
    # Exit 0 if any unchecked AC inside ## Acceptance Criteria, else 1.
    found=$(awk '
      /^## Acceptance Criteria[[:space:]]*$/ { in_ac=1; next }
      /^## / && in_ac { in_ac=0 }
      in_ac && /^- \[ \] ac-[0-9]+/ { print "1"; exit }
    ' "$path")
    if [[ -n "$found" ]]; then exit 0; else exit 1; fi
    ;;

  spec-path)
    # Field is plain text (not bold), matching card 0009's schema contract.
    line=$(grep -m1 '^Spec path:' "$path" 2>/dev/null || true)
    if [[ -z "$line" ]]; then
      exit 1
    fi
    value="${line#Spec path:}"
    # Trim leading whitespace
    value="${value#"${value%%[![:space:]]*}"}"
    # Trim trailing whitespace
    value="${value%"${value##*[![:space:]]}"}"
    printf "%s\n" "$value"
    ;;

  *)
    usage
    ;;
esac
