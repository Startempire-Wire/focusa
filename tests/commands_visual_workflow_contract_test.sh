#!/bin/bash
# SPEC-79 / Doc-62 slice B: command surface must expose visual workflow actions and evidence persistence aliases.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CMD_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/commands.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '"visual.start_run"|"start_visual_run"|"visual.close_run"|"close_visual_run"' "$CMD_FILE" >/dev/null 2>&1; then
  log_pass "command aliases include visual run lifecycle actions"
else
  log_fail "visual run lifecycle command aliases missing"
fi

if rg -n '"visual.register_reference_artifacts"|"visual.create_blueprint"|"visual.record_build_output"|"visual.record_comparison"|"visual.record_critique"|"visual.synthesize_fixes"|"visual.apply_fix_set"' "$CMD_FILE" >/dev/null 2>&1; then
  log_pass "command aliases include visual evidence workflow actions"
else
  log_fail "visual evidence workflow command aliases missing"
fi

if rg -n 'struct VisualEvidencePayload|fn to_artifact_label\(&self\) -> String' "$CMD_FILE" >/dev/null 2>&1; then
  log_pass "visual evidence payload schema and canonical label mapping exist"
else
  log_fail "visual evidence payload schema/label mapping missing"
fi

if rg -n 'Ok\(Action::StoreArtifact \{' "$CMD_FILE" >/dev/null 2>&1; then
  log_pass "visual evidence commands persist through StoreArtifact action"
else
  log_fail "visual evidence commands are not wired to StoreArtifact action"
fi

echo "=== COMMANDS VISUAL WORKFLOW CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
