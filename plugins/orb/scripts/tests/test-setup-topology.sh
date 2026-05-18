#!/usr/bin/env bash
# test-setup-topology.sh — exercise plugins/orb/scripts/setup-topology.sh end-to-end.
#
# Six scenarios per spec 2026-05-18-topology-substrate-wires ac-01:
#   (1) greenfield (no config, no docs/) — accept
#       t1: .orbit/config.yaml carries docs.topology: docs/topology.md
#       t2: docs/topology.md created with stub content
#   (2) brownfield-decline
#       t1: no config.yaml written
#       t2: no topology stub created
#   (3) brownfield-accept (no existing target file)
#       t1: config wired + stub created
#   (4) brownfield-accept (existing target file)
#       t1: pointer wired, existing file content preserved (no overwrite)
#   (5) nested target path (parent dir absent)
#       t1: parent directory tree created
#       t2: stub created at nested path
#   (6) idempotent re-run on wired repo
#       t1: no-op on existing config, no drift, no prompt

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
SETUP="$REPO_ROOT/plugins/orb/scripts/setup-topology.sh"

if [[ ! -x "$SETUP" ]]; then
  echo "FAIL: setup-topology.sh not found or not executable at $SETUP" >&2
  exit 1
fi

# ----------------------------------------------------------------------
# Scenario 1: greenfield (accept)
# ----------------------------------------------------------------------
TMP1=$(mktemp -d)
TMP2=$(mktemp -d)
TMP3=$(mktemp -d)
TMP4=$(mktemp -d)
TMP5=$(mktemp -d)
TMP6=$(mktemp -d)
trap 'rm -rf "$TMP1" "$TMP2" "$TMP3" "$TMP4" "$TMP5" "$TMP6"' EXIT

echo "=== Scenario 1: greenfield (accept) ==="
mkdir -p "$TMP1/.orbit"

"$SETUP" --project-root "$TMP1" --answer-wire y >/dev/null

# t1
if grep -Fxq '  topology: docs/topology.md' "$TMP1/.orbit/config.yaml"; then
  echo "  PASS t1: .orbit/config.yaml carries docs.topology"
else
  echo "  FAIL t1: docs.topology not present in config.yaml" >&2
  cat "$TMP1/.orbit/config.yaml" >&2 || true
  exit 1
fi

# t2
if [[ -f "$TMP1/docs/topology.md" ]] && grep -Fq '# Topology' "$TMP1/docs/topology.md"; then
  echo "  PASS t2: docs/topology.md stub created"
else
  echo "  FAIL t2: stub topology.md missing or wrong content" >&2
  exit 1
fi

# ----------------------------------------------------------------------
# Scenario 2: brownfield-decline
# ----------------------------------------------------------------------
echo "=== Scenario 2: brownfield-decline ==="
mkdir -p "$TMP2/.orbit"

"$SETUP" --project-root "$TMP2" --answer-wire n >/dev/null

# t1
if [[ -f "$TMP2/.orbit/config.yaml" ]]; then
  echo "  FAIL t1: config.yaml created on decline" >&2
  exit 1
fi
echo "  PASS t1: no config.yaml on decline"

# t2
if [[ -f "$TMP2/docs/topology.md" ]]; then
  echo "  FAIL t2: stub created on decline" >&2
  exit 1
fi
echo "  PASS t2: no stub on decline"

# ----------------------------------------------------------------------
# Scenario 3: brownfield-accept, no existing target
# ----------------------------------------------------------------------
echo "=== Scenario 3: brownfield-accept (no existing target) ==="
mkdir -p "$TMP3/.orbit"

"$SETUP" --project-root "$TMP3" --answer-wire y >/dev/null

# t1
if grep -Fxq '  topology: docs/topology.md' "$TMP3/.orbit/config.yaml" \
   && [[ -f "$TMP3/docs/topology.md" ]]; then
  echo "  PASS t1: config wired + stub created"
else
  echo "  FAIL t1: missing config entry or stub" >&2
  exit 1
fi

# ----------------------------------------------------------------------
# Scenario 4: brownfield-accept, existing target preserved
# ----------------------------------------------------------------------
echo "=== Scenario 4: brownfield-accept (existing target preserved) ==="
mkdir -p "$TMP4/.orbit" "$TMP4/docs"
echo "PRESERVED CONTENT MARKER" > "$TMP4/docs/topology.md"

"$SETUP" --project-root "$TMP4" --answer-wire y >/dev/null

# t1
if grep -Fxq '  topology: docs/topology.md' "$TMP4/.orbit/config.yaml" \
   && grep -Fxq 'PRESERVED CONTENT MARKER' "$TMP4/docs/topology.md"; then
  echo "  PASS t1: pointer wired, existing content preserved"
else
  echo "  FAIL t1: pointer not wired OR existing file overwritten" >&2
  cat "$TMP4/docs/topology.md" >&2 || true
  exit 1
fi

# ----------------------------------------------------------------------
# Scenario 5: nested target path (parent dir creation)
# ----------------------------------------------------------------------
echo "=== Scenario 5: nested target path ==="
mkdir -p "$TMP5/.orbit"
cat > "$TMP5/.orbit/config.yaml" <<EOF
docs:
  topology: docs/architecture/topology.md
EOF

"$SETUP" --project-root "$TMP5" --answer-wire y >/dev/null

# t1
if [[ -d "$TMP5/docs/architecture" ]]; then
  echo "  PASS t1: nested parent directory created"
else
  echo "  FAIL t1: nested parent directory not created" >&2
  exit 1
fi

# t2
if [[ -f "$TMP5/docs/architecture/topology.md" ]]; then
  echo "  PASS t2: nested stub created"
else
  echo "  FAIL t2: nested stub not created" >&2
  exit 1
fi

# ----------------------------------------------------------------------
# Scenario 6: idempotent re-run on already-wired repo
# ----------------------------------------------------------------------
echo "=== Scenario 6: idempotent re-run ==="
# Capture pre-state of scenario-1's repo (already wired in §1).
config_before=$(cat "$TMP1/.orbit/config.yaml")
stub_before=$(cat "$TMP1/docs/topology.md")

"$SETUP" --project-root "$TMP1" --answer-wire y >/dev/null

config_after=$(cat "$TMP1/.orbit/config.yaml")
stub_after=$(cat "$TMP1/docs/topology.md")

# t1
if [[ "$config_before" == "$config_after" && "$stub_before" == "$stub_after" ]]; then
  echo "  PASS t1: idempotent re-run leaves substrate untouched"
else
  echo "  FAIL t1: re-run produced drift" >&2
  diff <(echo "$config_before") <(echo "$config_after") || true
  diff <(echo "$stub_before") <(echo "$stub_after") || true
  exit 1
fi

echo ""
echo "test-setup-topology.sh: all scenarios passed."
