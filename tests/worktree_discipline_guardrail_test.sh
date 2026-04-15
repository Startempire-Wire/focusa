#!/bin/bash
# SPEC-79 worktree discipline semantics: worktree state is visible, but dirty state should not hard-block continuation.
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
if rg -n 'worktree is not clean before requesting the next continuous turn' "$DAEMON_FILE" >/dev/null 2>&1; then log_fail "daemon still hard-blocks continuation on dirty worktree"; else log_pass "dirty worktree no longer hard-blocks next-turn continuation"; fi
if rg -n 'worktree_status_snapshot\(' "${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs" >/dev/null 2>&1; then log_pass "worktree state remains visible via work-loop status snapshot"; else log_fail "worktree status snapshot visibility missing"; fi
echo "=== WORKTREE DISCIPLINE SEMANTICS RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
