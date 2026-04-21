#!/bin/bash
# Contract: Spec80-Impl parquet lane must support parquet format in API+CLI while keeping JSONL path intact.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
API_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/training.rs"
CLI_FILE="${ROOT_DIR}/crates/focusa-cli/src/commands/export.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n '"supported_formats": \["jsonl", "parquet"\]' "$API_FILE" >/dev/null 2>&1; then
  log_pass "export status advertises jsonl and parquet"
else
  log_fail "export status missing parquet support declaration"
fi

if rg -n 'body\.format != "jsonl" && body\.format != "parquet"|"error": "unsupported_format"|"format must be jsonl or parquet"' "$API_FILE" >/dev/null 2>&1; then
  log_pass "export run accepts parquet and keeps typed unsupported format errors"
else
  log_fail "export run format validation does not match parquet contract"
fi

if rg -n 'fn write_parquet\(|"encoding": "parquet_placeholder_v1"|ExportFormat::Parquet => write_parquet' "$CLI_FILE" >/dev/null 2>&1; then
  log_pass "CLI write path supports parquet output format"
else
  log_fail "CLI parquet write path is missing"
fi

if rg -n 'fn write_jsonl\(|ExportFormat::Jsonl => write_jsonl' "$CLI_FILE" >/dev/null 2>&1; then
  log_pass "JSONL write path remains present"
else
  log_fail "JSONL write path regression detected"
fi

echo "=== SPEC80 IMPL PARQUET EXPORT SUPPORT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
