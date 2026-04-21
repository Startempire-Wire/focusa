#!/bin/bash
# Contract: SPEC80 tree/lineage contract pack must include required tools, errors, layers, and binding references.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC80_TREE_LINEAGE_TOOL_CONTRACTS_2026-04-21.md"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

for tool in \
  focusa_tree_head \
  focusa_tree_path \
  focusa_tree_snapshot_state \
  focusa_tree_restore_state \
  focusa_tree_diff_context
  do
  if rg -n "\`$tool\`" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "contract pack includes tool: $tool"
  else
    log_fail "contract pack missing tool: $tool"
  fi
done

for code in \
  TREE_HEAD_UNAVAILABLE \
  SESSION_NOT_FOUND \
  CLT_NODE_NOT_FOUND \
  SNAPSHOT_WRITE_DENIED \
  SNAPSHOT_CONFLICT \
  SNAPSHOT_NOT_FOUND \
  RESTORE_CONFLICT \
  AUTHORITY_DENIED \
  DIFF_SCOPE_INVALID
  do
  if rg -n "$code" "$DOC_FILE" >/dev/null 2>&1; then
    log_pass "contract pack includes error code: $code"
  else
    log_fail "contract pack missing error code: $code"
  fi
done

if rg -n '/v1/lineage/head|/v1/lineage/tree|/v1/lineage/node/|/v1/lineage/path/|/v1/lineage/children/|/v1/lineage/summaries' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "contract pack cites implemented lineage API binding substrate"
else
  log_fail "contract pack missing lineage API binding substrate references"
fi

if rg -n 'focusa lineage head\|tree\|node\|path\|children\|summaries' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "contract pack cites CLI lineage parity binding"
else
  log_fail "contract pack missing CLI lineage parity binding"
fi

if rg -n 'planned-extension gap remains for snapshot/restore/diff|/v1/focus/snapshots\*' "$DOC_FILE" >/dev/null 2>&1; then
  log_pass "contract pack captures planned-extension snapshot/restore/diff gap"
else
  log_fail "contract pack missing planned-extension gap declaration"
fi

echo "=== SPEC80 TREE/LINEAGE CONTRACT PACK RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
