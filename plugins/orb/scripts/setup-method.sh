#!/usr/bin/env bash
# setup-method.sh — implements /orb:setup §6 (canonical files copy + CLAUDE.md @-imports).
#
# Pipeline (per spec 2026-05-09-orbit-method-md, ac-03; extended for STYLE.md
# per spec 2026-05-20-style-md-plugin-shipping ac-04 + ac-12):
#   1. Legacy-CLAUDE.md detection — scan for ## Workflow (orbit) / ## Orbit vocabulary
#      / ## Current Sprint markers. If present, prompt to migrate atomically with the
#      @-import additions. Decline → REFUSE the entire operation (no canonical files
#      copied, no @-imports). Atomic semantics — never leave dual-source drift.
#   2. Copy each plugin canonical (METHOD.md, STYLE.md) to .orbit/<file>. If
#      destination exists, byte-for-byte compare; mismatch prompts before overwriting.
#   3. Ensure CLAUDE.md contains an `@.orbit/<file>` line for each canonical.
#      Idempotent: append at end-of-file with leading blank line if missing.
#
# Usage:
#   setup-method.sh --project-root <path>
#     [--canonical <path>]              alias for --canonical-method
#     [--canonical-method <path>]
#     [--canonical-style <path>]
#     [--answer-legacy y|n]
#     [--answer-drift y|n]              alias for --answer-method-drift
#     [--answer-method-drift y|n]
#     [--answer-style-drift y|n]
#
# Test affordances:
#   --answer-legacy        scripts the legacy-migration prompt
#   --answer-method-drift  scripts the METHOD.md drift prompt
#   --answer-style-drift   scripts the STYLE.md drift prompt
# All default to interactive (read from stdin).

set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: setup-method.sh --project-root <path>
  [--canonical-method <path>] [--canonical-style <path>]
  [--answer-legacy y|n] [--answer-method-drift y|n] [--answer-style-drift y|n]

Required:
  --project-root <path>          Project root containing CLAUDE.md and .orbit/

Optional:
  --canonical-method <path>      Path to canonical METHOD.md (defaults to the
                                 in-plugin file, resolved relative to this script).
                                 Alias: --canonical
  --canonical-style <path>       Path to canonical STYLE.md (defaults to the
                                 in-plugin file, resolved relative to this script).
  --answer-legacy y|n            Script the legacy-migration prompt (default: interactive).
  --answer-method-drift y|n      Script the METHOD.md drift prompt (default: interactive).
                                 Alias: --answer-drift
  --answer-style-drift y|n       Script the STYLE.md drift prompt (default: interactive).
EOF
  exit 2
}

project_root=""
canonical_method=""
canonical_style=""
answer_legacy=""
answer_method_drift=""
answer_style_drift=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --project-root) project_root="$2"; shift 2 ;;
    --canonical|--canonical-method) canonical_method="$2"; shift 2 ;;
    --canonical-style) canonical_style="$2"; shift 2 ;;
    --answer-legacy) answer_legacy="$2"; shift 2 ;;
    --answer-drift|--answer-method-drift) answer_method_drift="$2"; shift 2 ;;
    --answer-style-drift) answer_style_drift="$2"; shift 2 ;;
    *) echo "setup-method.sh: unknown option: $1" >&2; usage ;;
  esac
done

if [[ -z "$project_root" ]]; then
  usage
fi

if [[ ! -d "$project_root" ]]; then
  echo "setup-method.sh: project root not found: $project_root" >&2
  exit 2
fi

# Default canonicals to the script-relative locations.
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -z "$canonical_method" ]]; then
  canonical_method="$script_dir/../skills/setup/METHOD.md"
fi
if [[ -z "$canonical_style" ]]; then
  canonical_style="$script_dir/../skills/setup/STYLE.md"
fi

if [[ ! -f "$canonical_method" ]]; then
  echo "setup-method.sh: canonical METHOD.md not found: $canonical_method" >&2
  exit 2
fi
if [[ ! -f "$canonical_style" ]]; then
  echo "setup-method.sh: canonical STYLE.md not found: $canonical_style" >&2
  exit 2
fi

claude_md="$project_root/CLAUDE.md"
method_md="$project_root/.orbit/METHOD.md"
style_md="$project_root/.orbit/STYLE.md"

LEGACY_MARKERS=(
  '## Workflow (orbit)'
  '## Orbit vocabulary'
  '## Current Sprint'
)

# Function: copy_canonical <canonical-path> <operator-path> <answer>
# Implements §6b's byte-compare-and-prompt-on-drift semantics for one file.
copy_canonical() {
  local canonical="$1"
  local operator="$2"
  local answer="$3"
  local file_name
  file_name="$(basename "$operator")"

  if [[ -f "$operator" ]]; then
    if cmp -s "$canonical" "$operator"; then
      return 0  # byte-identical, no-op
    fi
    echo "orbit: $operator differs from the canonical (the plugin has updated, or the file has been edited locally)."

    local ans="$answer"
    if [[ -z "$ans" ]]; then
      read -r -p "Overwrite with canonical? (y/N) " ans || ans=""
    fi

    case "${ans,,}" in
      y|yes)
        cp "$canonical" "$operator"
        echo "orbit: $operator overwritten with canonical."
        ;;
      *)
        echo "orbit: keeping local $operator (canonical not applied)."
        ;;
    esac
  else
    cp "$canonical" "$operator"
  fi
}

# Function: ensure_at_import <import-line>
# Implements §6c's idempotent @-import-append semantics for one import line.
ensure_at_import() {
  local import_line="$1"

  if [[ ! -f "$claude_md" ]]; then
    printf '\n%s\n' "$import_line" > "$claude_md"
  elif ! grep -Fxq "$import_line" "$claude_md"; then
    if [[ -s "$claude_md" ]]; then
      [[ "$(tail -c1 "$claude_md")" == $'\n' ]] || printf '\n' >> "$claude_md"
      printf '\n%s\n' "$import_line" >> "$claude_md"
    else
      printf '%s\n' "$import_line" >> "$claude_md"
    fi
  fi
}

# 6a — legacy CLAUDE.md detection.
legacy_present=0
if [[ -f "$claude_md" ]]; then
  for marker in "${LEGACY_MARKERS[@]}"; do
    if grep -Fxq "$marker" "$claude_md"; then
      legacy_present=1
      break
    fi
  done
fi

if [[ "$legacy_present" -eq 1 ]]; then
  echo "orbit: CLAUDE.md contains legacy workflow blocks (## Workflow (orbit) / ## Orbit vocabulary / ## Current Sprint)."
  echo "orbit: migration removes them and adds @.orbit/METHOD.md + @.orbit/STYLE.md as the single sources of truth."

  if [[ -n "$answer_legacy" ]]; then
    answer="$answer_legacy"
  else
    read -r -p "Migrate now? (y/N) " answer || answer=""
  fi

  case "${answer,,}" in
    y|yes)
      # Atomic migrate: remove legacy blocks AND copy canonicals AND add @-imports in one go.
      mkdir -p "$project_root/.orbit"
      cp "$canonical_method" "$method_md"
      cp "$canonical_style" "$style_md"

      python3 - "$claude_md" <<'PY'
import re, sys
path = sys.argv[1]
with open(path) as f:
    text = f.read()

markers = ['## Workflow (orbit)', '## Orbit vocabulary', '## Current Sprint']

def strip_section(body: str, marker: str) -> str:
    # Remove from `marker` (at line start) up to the next top-level heading or EOF.
    pattern = re.compile(
        r'(^|\n)' + re.escape(marker) + r'\s*\n.*?(?=\n##\s|\n#\s|\Z)',
        flags=re.DOTALL,
    )
    return pattern.sub('', body)

for m in markers:
    text = strip_section(text, m)

# Collapse 3+ consecutive blank lines back to 2.
text = re.sub(r'\n{3,}', '\n\n', text)

# Ensure exactly one @.orbit/METHOD.md and one @.orbit/STYLE.md import line.
for import_line in ('@.orbit/METHOD.md', '@.orbit/STYLE.md'):
    if import_line not in text:
        if not text.endswith('\n'):
            text += '\n'
        if not text.endswith('\n\n'):
            text += '\n'
        text += import_line + '\n'

with open(path, 'w') as f:
    f.write(text)
PY
      echo "orbit: legacy blocks removed; .orbit/METHOD.md + .orbit/STYLE.md created; @-imports added to CLAUDE.md."
      exit 0
      ;;
    *)
      echo "orbit: setup aborted. Re-run /orb:setup once you have removed the legacy blocks, or accept the migration prompt." >&2
      exit 1
      ;;
  esac
fi

# 6b — copy canonical files (no legacy blocks present).
mkdir -p "$project_root/.orbit"
copy_canonical "$canonical_method" "$method_md" "$answer_method_drift"
copy_canonical "$canonical_style"  "$style_md"  "$answer_style_drift"

# 6c — ensure CLAUDE.md @-imports (idempotent for each).
ensure_at_import '@.orbit/METHOD.md'
ensure_at_import '@.orbit/STYLE.md'

exit 0
