#!/bin/bash
# Contract: /tree integration is surfaced as first-class LI tools and a lineage tree API route.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CAP_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/capabilities.rs"
TOOLS_FILE="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'async fn lineage_tree\(' "$CAP_FILE" >/dev/null 2>&1 && rg -n 'route\("/v1/lineage/tree", get\(lineage_tree\)\)' "$CAP_FILE" >/dev/null 2>&1; then
  log_pass "capabilities API exposes /v1/lineage/tree handler and route wiring"
else
  log_fail "capabilities API missing /v1/lineage/tree route/handler"
fi

if rg -n 'name:\s*"focusa_lineage_tree"' "$TOOLS_FILE" >/dev/null 2>&1; then
  log_pass "pi extension registers focusa_lineage_tree first-class tool"
else
  log_fail "pi extension missing focusa_lineage_tree tool"
fi

if rg -n 'name:\s*"focusa_li_tree_extract"' "$TOOLS_FILE" >/dev/null 2>&1; then
  log_pass "pi extension registers focusa_li_tree_extract first-class tool"
else
  log_fail "pi extension missing focusa_li_tree_extract tool"
fi

if rg -n '/lineage/tree\$\{query\}' "$TOOLS_FILE" >/dev/null 2>&1; then
  log_pass "LI tools ingest lineage tree payload from API"
else
  log_fail "LI tools do not fetch lineage tree payload"
fi

echo "=== QWFI LINEAGE TREE TOOLCHAIN RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
