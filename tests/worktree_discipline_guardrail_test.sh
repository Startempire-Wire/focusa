#!/bin/bash
# SPEC-79 worktree discipline guardrail: daemon checks cleanliness before next continuous turn.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_FILE="${ROOT_DIR}/crates/focusa-core/src/runtime/daemon.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
if rg -n 'async fn worktree_is_clean' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "daemon has worktree cleanliness helper"; else log_fail "daemon missing worktree cleanliness helper"; fi
if rg -n 'Action::RequestNextContinuousTurn' "$DAEMON_FILE" >/dev/null 2>&1 && rg -n 'worktree is not clean before requesting the next continuous turn' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "daemon enforces worktree cleanliness before next turn"; else log_fail "daemon missing per-work-unit worktree enforcement"; fi
echo "=== WORKTREE DISCIPLINE GUARDRAIL RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
