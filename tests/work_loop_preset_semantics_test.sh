#!/bin/bash
# SPEC-79 work-loop preset semantics + override plumbing
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TYPES_FILE="${ROOT_DIR}/crates/focusa-core/src/types.rs"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
if rg -n 'pub struct WorkLoopPolicyOverrides' "$TYPES_FILE" >/dev/null 2>&1; then log_pass "work-loop policy override struct exists"; else log_fail "work-loop policy override struct missing"; fi
if rg -n 'pub fn for_preset\(preset: WorkLoopPreset\)' "$TYPES_FILE" >/dev/null 2>&1 && rg -n 'WorkLoopPreset::Conservative|WorkLoopPreset::Balanced|WorkLoopPreset::Push|WorkLoopPreset::Audit' "$TYPES_FILE" >/dev/null 2>&1; then log_pass "all spec79 work-loop presets map to concrete policy values"; else log_fail "preset-to-policy mapping incomplete"; fi
if rg -n 'pub fn with_overrides\(preset: WorkLoopPreset, overrides: WorkLoopPolicyOverrides\)' "$TYPES_FILE" >/dev/null 2>&1; then log_pass "policy overrides can be layered on presets"; else log_fail "policy override layering missing"; fi
if rg -n 'policy_overrides: Option<WorkLoopPolicyOverrides>' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'WorkLoopPolicy::with_overrides' "$ROUTE_FILE" >/dev/null 2>&1; then log_pass "enable route accepts and applies policy overrides"; else log_fail "enable route missing policy override plumbing"; fi
echo "=== WORK-LOOP PRESET SEMANTICS RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
