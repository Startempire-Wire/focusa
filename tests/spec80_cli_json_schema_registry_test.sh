#!/bin/bash
# Contract: Spec80 C4.1 CLI JSON schema registry must define parity-critical schema ids and required fields.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_CLI_JSON_SCHEMA_REGISTRY_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

for id in \
  cli.lineage.tree.v1 \
  cli.lineage.head.v1 \
  cli.metacognition.not_implemented.v1 \
  cli.export.status.not_implemented.v1 \
  cli.export.dataset.not_implemented.v1
  do
  if rg -n "\`$id\`" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "registry includes schema id: $id"
  else
    log_fail "registry missing schema id: $id"
  fi
done

if rg -n '\`session_id\`|\`root\`|\`head\`|\`nodes\`|\`total\`' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "lineage schema required fields are documented"
else
  log_fail "lineage schema required fields missing"
fi

if rg -n '\`status\`.*not_implemented|\`command\`|\`planned_api_path\`|\`reason\`|\`label\`' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "metacognition planned-extension envelope fields are documented"
else
  log_fail "metacognition planned-extension envelope fields missing"
fi

if rg -n '\`dataset_types\`|\`supported_formats\`|\`required_sources\`' "$DOC_FILE" >/dev/null 2>&1 && rg -n '\`dataset_type\`|\`dry_run\`|\`dataset_flags\`' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "export status/dataset envelope fields are documented"
else
  log_fail "export envelope fields missing"
fi

echo "=== SPEC80 CLI JSON SCHEMA REGISTRY RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
