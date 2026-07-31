#!/bin/bash
# Spec 104 phase-gate verification.
# Run between phases to prove no omissions, no deferrals.
# Usage: ./scripts/spec104-phase-gate.sh <phase_id>
#   Phase IDs: P0, P1, P2, P3

set -euo pipefail
cd "$(dirname "$0")/.."
PHASE="${1:-}"
if [ -z "$PHASE" ]; then
  echo "Usage: $0 <P0|P1|P2|P3>"
  exit 1
fi

PHASE_BEAD=""
case "$PHASE" in
  P0) PHASE_BEAD="focusa-fsrc" ;;
  P1) PHASE_BEAD="focusa-n68k" ;;
  P2) PHASE_BEAD="focusa-nodn" ;;
  P3) PHASE_BEAD="focusa-84px" ;;
  *)  echo "Unknown phase: $PHASE"; exit 1 ;;
esac

ERRORS=0

echo "=== Spec 104 Phase Gate: $PHASE ==="
echo ""

# 1. Verify all beads in phase are closed
echo "--- Checking bead closure for $PHASE ---"
OPEN_ITEMS=$(bd children "$PHASE_BEAD" --no-daemon 2>/dev/null | grep -c "OPEN" || true)
if [ "$OPEN_ITEMS" -gt 0 ]; then
  echo "FAIL: $OPEN_ITEMS beads still OPEN in $PHASE_BEAD"
  bd children "$PHASE_BEAD" --no-daemon 2>/dev/null | grep "OPEN" || true
  ERRORS=$((ERRORS+1))
else
  echo "PASS: All $PHASE beads closed"
fi
echo ""

# 2. Run static audit
echo "--- Running static surface audit ---"
python3 tests/spec104_deep_focusa_surface_sweep.py --static-only
echo ""

# 3. Verify no new untracked singleton globals
echo "--- Checking for new singleton globals ---"
NEW_GLOBALS=$(rg -n "OnceLock::new|LazyLock::new" --glob '*.rs' --glob '*.ts' \
  | grep -v "/target/" | grep -v "test" \
  | grep -v "agent_capabilities.rs\|bounded.rs\|context_retrieval.rs\|device_pairing.rs\|mcp.rs\|metacognition.rs\|ontology.rs\|predictions.rs\|project.rs\|proxy.rs\|rate_limit.rs\|snapshots.rs\|turn.rs\|workpoint.rs\|server.rs\|main.rs\|state.ts\|tools.ts" \
  || true)
if [ -n "$NEW_GLOBALS" ]; then
  echo "WARNING: Potential new singleton globals not in Annex A:"
  echo "$NEW_GLOBALS"
  ERRORS=$((ERRORS+1))
fi
echo ""

# 4. Verify Annex B inventory matches repo
echo "--- Checking Annex B inventory coverage ---"
# Quick check: every crate route file is in spec
SPEC_FILE="docs/104-typed-scoped-runtime-and-singleton-elimination-spec.md"
MISSING=0
for f in crates/focusa-api/src/routes/*.rs; do
  name=$(basename "$f")
  if ! grep -q "$name" "$SPEC_FILE" 2>/dev/null; then
    echo "MISSING from spec: $name"
    MISSING=$((MISSING+1))
  fi
done
for f in crates/focusa-cli/src/commands/*.rs; do
  name=$(basename "$f")
  if ! grep -q "$name" "$SPEC_FILE" 2>/dev/null; then
    echo "MISSING from spec: $name"
    MISSING=$((MISSING+1))
  fi
done
if [ "$MISSING" -gt 0 ]; then
  echo "FAIL: $MISSING files missing from Spec 104 inventory"
  ERRORS=$((ERRORS+1))
else
  echo "PASS: All source files inventoried"
fi
echo ""

# 5. Verify no new files in repo without spec inventory entry (checks git-tracked new files)
if git rev-parse --git-dir >/dev/null 2>&1; then
  echo "--- Checking for new files not in Spec 104 ---"
  NEW_FILES=$(git diff --name-only HEAD~1..HEAD 2>/dev/null || git diff --name-only HEAD 2>/dev/null || true)
  if [ -n "$NEW_FILES" ]; then
    for f in $NEW_FILES; do
      if echo "$f" | grep -qvE "\.(rs|ts|tsx|svelte|py|sh)$"; then
        continue
      fi
      if [ -f "$f" ] && ! grep -q "$f" "$SPEC_FILE" 2>/dev/null; then
        echo "NEW FILE not in Spec 104: $f"
        echo "  Add it to Annex B before closing this phase gate."
        ERRORS=$((ERRORS+1))
      fi
    done
  fi
fi
echo ""

# Summary
if [ "$ERRORS" -gt 0 ]; then
  echo "=== PHASE GATE $PHASE: FAIL ($ERRORS errors) ==="
  exit 1
else
  echo "=== PHASE GATE $PHASE: PASS ==="
  echo "Ready to proceed to next phase."
fi
