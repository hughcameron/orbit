#!/usr/bin/env bash
# test-setup-method.sh — exercise plugins/orb/scripts/setup-method.sh end-to-end.
#
# Scenarios per spec 2026-05-09-orbit-method-md ac-07 (extended for STYLE.md
# per spec 2026-05-20-style-md-plugin-shipping ac-04 + ac-12):
#   (1) fresh project (non-interactive)
#       t1: .orbit/METHOD.md byte-equal canonical on first run
#       t2: CLAUDE.md contains exactly one @.orbit/METHOD.md line
#       t3: re-run is idempotent — no duplicates, no METHOD.md drift, no prompt
#       t4: .orbit/STYLE.md byte-equal canonical on first run
#       t5: CLAUDE.md contains exactly one @.orbit/STYLE.md line
#       t6: re-run is idempotent for STYLE.md (no duplicates, no drift)
#   (2) drift-prompt firing (assertion-only)
#       t1: modifying .orbit/METHOD.md then re-running fires the drift prompt
#       t2: modifying .orbit/STYLE.md then re-running fires the STYLE.md drift prompt
#   (3) legacy migration (scripted answers)
#       t1: --answer-legacy y removes legacy blocks, creates METHOD.md + STYLE.md,
#           adds both @-imports
#       t2: --answer-legacy n leaves blocks intact, no METHOD.md, no STYLE.md,
#           no @-imports, recovery message on stderr

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
SETUP="$REPO_ROOT/plugins/orb/scripts/setup-method.sh"
CANONICAL="$REPO_ROOT/plugins/orb/skills/setup/METHOD.md"
CANONICAL_STYLE="$REPO_ROOT/plugins/orb/skills/setup/STYLE.md"

if [[ ! -x "$SETUP" ]]; then
  echo "FAIL: setup-method.sh not found or not executable at $SETUP" >&2
  exit 1
fi
if [[ ! -f "$CANONICAL" ]]; then
  echo "FAIL: canonical METHOD.md not found at $CANONICAL" >&2
  exit 1
fi
if [[ ! -f "$CANONICAL_STYLE" ]]; then
  echo "FAIL: canonical STYLE.md not found at $CANONICAL_STYLE" >&2
  exit 1
fi

# ----------------------------------------------------------------------
# Scenario 1: fresh project (non-interactive)
# ----------------------------------------------------------------------
TMP1=$(mktemp -d)
trap 'rm -rf "$TMP1" "$TMP2" "$TMP3a" "$TMP3b"' EXIT

echo "=== Scenario 1: fresh project ==="
mkdir -p "$TMP1"
touch "$TMP1/CLAUDE.md"

"$SETUP" --project-root "$TMP1" --canonical "$CANONICAL" >/dev/null

# t1
if cmp -s "$CANONICAL" "$TMP1/.orbit/METHOD.md"; then
  echo "  PASS t1: .orbit/METHOD.md byte-equal canonical"
else
  echo "  FAIL t1: .orbit/METHOD.md does not match canonical" >&2
  exit 1
fi

# t2
import_count=$(grep -Fxc '@.orbit/METHOD.md' "$TMP1/CLAUDE.md" || true)
if [[ "$import_count" == "1" ]]; then
  echo "  PASS t2: exactly one @.orbit/METHOD.md import line"
else
  echo "  FAIL t2: expected 1 @-import line, got $import_count" >&2
  exit 1
fi

# t3 — re-run idempotent
"$SETUP" --project-root "$TMP1" --canonical "$CANONICAL" >/dev/null
import_count_after=$(grep -Fxc '@.orbit/METHOD.md' "$TMP1/CLAUDE.md" || true)
if [[ "$import_count_after" == "1" ]] && cmp -s "$CANONICAL" "$TMP1/.orbit/METHOD.md"; then
  echo "  PASS t3: re-run idempotent (no duplicate import, no drift)"
else
  echo "  FAIL t3: re-run produced drift (imports=$import_count_after, METHOD.md may differ)" >&2
  exit 1
fi

# t4
if cmp -s "$CANONICAL_STYLE" "$TMP1/.orbit/STYLE.md"; then
  echo "  PASS t4: .orbit/STYLE.md byte-equal canonical"
else
  echo "  FAIL t4: .orbit/STYLE.md does not match canonical" >&2
  exit 1
fi

# t5
style_import_count=$(grep -Fxc '@.orbit/STYLE.md' "$TMP1/CLAUDE.md" || true)
if [[ "$style_import_count" == "1" ]]; then
  echo "  PASS t5: exactly one @.orbit/STYLE.md import line"
else
  echo "  FAIL t5: expected 1 STYLE.md @-import line, got $style_import_count" >&2
  exit 1
fi

# t6 — re-run idempotent for STYLE.md
style_import_count_after=$(grep -Fxc '@.orbit/STYLE.md' "$TMP1/CLAUDE.md" || true)
if [[ "$style_import_count_after" == "1" ]] && cmp -s "$CANONICAL_STYLE" "$TMP1/.orbit/STYLE.md"; then
  echo "  PASS t6: re-run idempotent for STYLE.md (no duplicate import, no drift)"
else
  echo "  FAIL t6: re-run produced STYLE.md drift (imports=$style_import_count_after)" >&2
  exit 1
fi

# ----------------------------------------------------------------------
# Scenario 2: drift-prompt firing (assertion-only, no interaction)
# ----------------------------------------------------------------------
echo "=== Scenario 2: drift-prompt firing ==="
TMP2=$(mktemp -d)
mkdir -p "$TMP2"
touch "$TMP2/CLAUDE.md"

"$SETUP" --project-root "$TMP2" --canonical "$CANONICAL" >/dev/null

# Modify the project's METHOD.md so re-run detects drift.
echo "<!-- locally edited -->" >> "$TMP2/.orbit/METHOD.md"

# Drive the script with --answer-method-drift n to pick up the prompt path
# without blocking on stdin. Also decline the STYLE.md prompt (just in case
# the seed wrote different bytes), so neither file gets overwritten.
out=$("$SETUP" --project-root "$TMP2" --canonical "$CANONICAL" \
  --answer-method-drift n --answer-style-drift n 2>&1)
if echo "$out" | grep -q "METHOD.md differs from the canonical"; then
  echo "  PASS t1: drift prompt fires when METHOD.md modified"
else
  echo "  FAIL t1: METHOD.md drift prompt did NOT fire" >&2
  echo "$out" >&2
  exit 1
fi

# Confirm decline kept the local edit on METHOD.md.
if grep -q "locally edited" "$TMP2/.orbit/METHOD.md"; then
  echo "  PASS t1.aux: decline kept local METHOD.md edits"
else
  echo "  FAIL t1.aux: decline silently overwrote local METHOD.md" >&2
  exit 1
fi

# Now modify STYLE.md and re-run to verify the STYLE.md drift prompt fires.
echo "<!-- locally edited -->" >> "$TMP2/.orbit/STYLE.md"
out=$("$SETUP" --project-root "$TMP2" --canonical "$CANONICAL" \
  --answer-method-drift n --answer-style-drift n 2>&1)
if echo "$out" | grep -q "STYLE.md differs from the canonical"; then
  echo "  PASS t2: drift prompt fires when STYLE.md modified"
else
  echo "  FAIL t2: STYLE.md drift prompt did NOT fire" >&2
  echo "$out" >&2
  exit 1
fi

# Confirm decline kept the local edit on STYLE.md.
if grep -q "locally edited" "$TMP2/.orbit/STYLE.md"; then
  echo "  PASS t2.aux: decline kept local STYLE.md edits"
else
  echo "  FAIL t2.aux: decline silently overwrote local STYLE.md" >&2
  exit 1
fi

# ----------------------------------------------------------------------
# Scenario 3a: legacy migration accepted
# ----------------------------------------------------------------------
echo "=== Scenario 3a: legacy migration accepted ==="
TMP3a=$(mktemp -d)
mkdir -p "$TMP3a"
cat > "$TMP3a/CLAUDE.md" <<'EOF'
# project

Some intro text.

## Workflow (orbit)

This project uses the orbit workflow.

- /orb:card
- /orb:design

## Orbit vocabulary

- **Card** — a capability.
- **Spec** — a unit of work.

## Current Sprint

goal: "do things"

## Other section

This stays.
EOF

"$SETUP" --project-root "$TMP3a" --canonical "$CANONICAL" --answer-legacy y >/dev/null

# t1: legacy blocks removed
if grep -Fxq "## Workflow (orbit)" "$TMP3a/CLAUDE.md"; then
  echo "  FAIL t1: ## Workflow (orbit) still present after migration" >&2
  exit 1
fi
if grep -Fxq "## Orbit vocabulary" "$TMP3a/CLAUDE.md"; then
  echo "  FAIL t1: ## Orbit vocabulary still present after migration" >&2
  exit 1
fi
if grep -Fxq "## Current Sprint" "$TMP3a/CLAUDE.md"; then
  echo "  FAIL t1: ## Current Sprint still present after migration" >&2
  exit 1
fi

# t1: METHOD.md created
if cmp -s "$CANONICAL" "$TMP3a/.orbit/METHOD.md"; then
  echo "  PASS t1.method: .orbit/METHOD.md created byte-equal canonical"
else
  echo "  FAIL t1.method: .orbit/METHOD.md missing or does not match canonical" >&2
  exit 1
fi

# t1: METHOD.md @-import added
import_count=$(grep -Fxc '@.orbit/METHOD.md' "$TMP3a/CLAUDE.md" || true)
if [[ "$import_count" == "1" ]]; then
  echo "  PASS t1.method-import: @.orbit/METHOD.md present"
else
  echo "  FAIL t1.method-import: expected 1 @-import line, got $import_count" >&2
  exit 1
fi

# t1: STYLE.md created
if cmp -s "$CANONICAL_STYLE" "$TMP3a/.orbit/STYLE.md"; then
  echo "  PASS t1.style: .orbit/STYLE.md created byte-equal canonical"
else
  echo "  FAIL t1.style: .orbit/STYLE.md missing or does not match canonical" >&2
  exit 1
fi

# t1: STYLE.md @-import added
style_import_count=$(grep -Fxc '@.orbit/STYLE.md' "$TMP3a/CLAUDE.md" || true)
if [[ "$style_import_count" == "1" ]]; then
  echo "  PASS t1.style-import: @.orbit/STYLE.md present"
else
  echo "  FAIL t1.style-import: expected 1 STYLE.md @-import line, got $style_import_count" >&2
  exit 1
fi

# Sanity — non-orbit content preserved
if grep -Fxq "## Other section" "$TMP3a/CLAUDE.md" && grep -Fxq "# project" "$TMP3a/CLAUDE.md"; then
  echo "  PASS t1.preserve: non-orbit content survived migration"
else
  echo "  FAIL t1.preserve: migration nuked non-orbit content" >&2
  cat "$TMP3a/CLAUDE.md" >&2
  exit 1
fi

# ----------------------------------------------------------------------
# Scenario 3b: legacy migration declined (atomic refuse)
# ----------------------------------------------------------------------
echo "=== Scenario 3b: legacy migration declined (atomic refuse) ==="
TMP3b=$(mktemp -d)
mkdir -p "$TMP3b"
cat > "$TMP3b/CLAUDE.md" <<'EOF'
# project

## Workflow (orbit)

Legacy content.

## Orbit vocabulary

- Card
EOF

# Capture stderr too — the script should print the recovery line on refuse.
set +e
out=$("$SETUP" --project-root "$TMP3b" --canonical "$CANONICAL" --answer-legacy n 2>&1)
rc=$?
set -e

# t2: exit non-zero
if [[ "$rc" == "0" ]]; then
  echo "  FAIL t2: refuse path returned 0 (expected non-zero)" >&2
  exit 1
fi

# t2: legacy blocks intact
if ! grep -Fxq "## Workflow (orbit)" "$TMP3b/CLAUDE.md"; then
  echo "  FAIL t2: ## Workflow (orbit) was removed despite refuse" >&2
  exit 1
fi
if ! grep -Fxq "## Orbit vocabulary" "$TMP3b/CLAUDE.md"; then
  echo "  FAIL t2: ## Orbit vocabulary was removed despite refuse" >&2
  exit 1
fi

# t2: no METHOD.md created
if [[ -f "$TMP3b/.orbit/METHOD.md" ]]; then
  echo "  FAIL t2: .orbit/METHOD.md was created despite refuse" >&2
  exit 1
fi

# t2: no STYLE.md created
if [[ -f "$TMP3b/.orbit/STYLE.md" ]]; then
  echo "  FAIL t2: .orbit/STYLE.md was created despite refuse" >&2
  exit 1
fi

# t2: no @-imports added
if grep -Fxq '@.orbit/METHOD.md' "$TMP3b/CLAUDE.md"; then
  echo "  FAIL t2: @.orbit/METHOD.md was added despite refuse" >&2
  exit 1
fi
if grep -Fxq '@.orbit/STYLE.md' "$TMP3b/CLAUDE.md"; then
  echo "  FAIL t2: @.orbit/STYLE.md was added despite refuse" >&2
  exit 1
fi

# t2: recovery message printed
if echo "$out" | grep -q "Re-run /orb:setup once you have removed"; then
  echo "  PASS t2: legacy intact, no METHOD.md, no @-import, recovery message printed, rc=$rc"
else
  echo "  FAIL t2: recovery message missing from output" >&2
  echo "$out" >&2
  exit 1
fi

echo ""
echo "OK: setup-method.sh end-to-end (fresh / drift-prompt / legacy-accept / legacy-refuse)"
