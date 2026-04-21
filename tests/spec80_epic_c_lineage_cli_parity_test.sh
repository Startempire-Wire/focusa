#!/bin/bash
# Contract: Spec80 Epic C lineage CLI domain must expose machine-stable parity surface for /v1/lineage endpoints.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MAIN_FILE="${ROOT_DIR}/crates/focusa-cli/src/main.rs"
MOD_FILE="${ROOT_DIR}/crates/focusa-cli/src/commands/mod.rs"
LINEAGE_FILE="${ROOT_DIR}/crates/focusa-cli/src/commands/lineage.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'pub mod lineage;' "$MOD_FILE" >/dev/null 2>&1; then
  log_pass "commands module exports lineage command domain"
else
  log_fail "commands module missing lineage command domain export"
fi

if rg -n 'Lineage\(commands::lineage::LineageCmd\)' "$MAIN_FILE" >/dev/null 2>&1 && rg -n 'Commands::Lineage\(cmd\) => commands::lineage::run\(cmd, cli\.json\)\.await\?' "$MAIN_FILE" >/dev/null 2>&1; then
  log_pass "CLI root wires lineage subcommand enum + dispatch"
else
  log_fail "CLI root missing lineage subcommand wiring"
fi

if rg -n 'enum LineageCmd|Head|Tree|Node|Path|Children|Summaries' "$LINEAGE_FILE" >/dev/null 2>&1; then
  log_pass "lineage command file declares required parity subcommands"
else
  log_fail "lineage command file missing one or more required subcommands"
fi

if rg -n '/v1/lineage/head|/v1/lineage/tree|/v1/lineage/node/|/v1/lineage/path/|/v1/lineage/children/|/v1/lineage/summaries' "$LINEAGE_FILE" >/dev/null 2>&1; then
  log_pass "lineage subcommands target full /v1/lineage parity endpoint set"
else
  log_fail "lineage subcommands missing one or more parity endpoints"
fi

echo "=== SPEC80 EPIC C LINEAGE CLI PARITY RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
