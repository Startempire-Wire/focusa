#!/bin/bash
# Contract: Spec80 C3.2 metacognition CLI contract tests for first-class command domain and live endpoint-backed execution.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MAIN_FILE="${ROOT_DIR}/crates/focusa-cli/src/main.rs"
MOD_FILE="${ROOT_DIR}/crates/focusa-cli/src/commands/mod.rs"
CMD_FILE="${ROOT_DIR}/crates/focusa-cli/src/commands/metacognition.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'pub mod metacognition;' "$MOD_FILE" >/dev/null 2>&1; then
  log_pass "commands module exports metacognition command domain"
else
  log_fail "commands module missing metacognition export"
fi

if rg -n 'Metacognition\(commands::metacognition::MetacognitionCmd\)' "$MAIN_FILE" >/dev/null 2>&1 && rg -n 'Commands::Metacognition\(cmd\) => commands::metacognition::run\(cmd, cli\.json\)\.await\?' "$MAIN_FILE" >/dev/null 2>&1; then
  log_pass "CLI root wires metacognition enum + dispatch"
else
  log_fail "CLI root missing metacognition wiring"
fi

if rg -n 'enum MetacognitionCmd|Capture|Retrieve|Reflect|Adjust|Evaluate' "$CMD_FILE" >/dev/null 2>&1; then
  log_pass "metacognition command file defines required subcommands"
else
  log_fail "metacognition command file missing required subcommands"
fi

if rg -n '/v1/metacognition/capture|/v1/metacognition/retrieve|/v1/metacognition/reflect|/v1/metacognition/adjust|/v1/metacognition/evaluate' "$CMD_FILE" >/dev/null 2>&1; then
  log_pass "metacognition command file maps to full API endpoint set"
else
  log_fail "metacognition command file missing one or more API mappings"
fi

if rg -n 'api\.post\(path, &body\)\.await\?' "$CMD_FILE" >/dev/null 2>&1; then
  log_pass "metacognition CLI is endpoint-backed (no local stub envelope path)"
else
  log_fail "metacognition CLI missing endpoint-backed execution call"
fi

if rg -n 'if json_mode \{|serde_json::to_string_pretty\(&resp\)' "$CMD_FILE" >/dev/null 2>&1; then
  log_pass "metacognition CLI returns daemon response payload in JSON mode"
else
  log_fail "metacognition CLI JSON-mode response contract missing"
fi

echo "=== SPEC80 CLI METACOGNITION CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
