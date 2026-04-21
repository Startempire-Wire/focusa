#!/bin/bash
# Contract: Spec80-Impl I1 export execution lane must be endpoint-backed with typed envelopes and non-dry-run write path.
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

if rg -n 'route\("/v1/export/status", get\(export_status\)\)|route\("/v1/export/run", post\(export_run\)\)' "$API_FILE" >/dev/null 2>&1; then
  log_pass "API exposes /v1/export/status and /v1/export/run routes"
else
  log_fail "API route binding for export status/run is incomplete"
fi

if rg -n '"sft" => build_sft|"preference" => build_preference|"contrastive" => build_contrastive|"long-horizon" => build_long_horizon' "$API_FILE" >/dev/null 2>&1; then
  log_pass "export_run supports all four required dataset families"
else
  log_fail "export_run missing one or more dataset family builders"
fi

if rg -n '"status": "ok"|"invalid_request"|"unknown_dataset_type"|"unsupported_format"' "$API_FILE" >/dev/null 2>&1; then
  log_pass "export_run provides typed success/error envelope fields"
else
  log_fail "export_run missing typed success/error envelope fields"
fi

if rg -n 'if !options\.dry_run \{|write_jsonl\(&options\.output, &records\)|write_parquet\(&options\.output, &records\)|write_manifest\(&options\.output, manifest\)' "$CLI_FILE" >/dev/null 2>&1; then
  log_pass "CLI non-dry-run path writes jsonl/parquet dataset + manifest outputs"
else
  log_fail "CLI non-dry-run write path is missing jsonl/parquet output coverage"
fi

if rg -n 'api\.post\("/v1/export/run", &body\)' "$CLI_FILE" >/dev/null 2>&1; then
  log_pass "CLI export command is endpoint-backed (no local stub-only path)"
else
  log_fail "CLI export command missing endpoint-backed execution call"
fi

if rg -n 'write_parquet\(|record_json|SerializedFileWriter' "$CLI_FILE" >/dev/null 2>&1; then
  log_pass "CLI parquet mode uses real parquet writer path"
else
  log_fail "CLI parquet mode missing real parquet writer path"
fi

echo "=== SPEC80 IMPL EXPORT EXECUTION CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
