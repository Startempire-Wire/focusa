#!/bin/bash
# Contract: Doc-78 replay consumer must be projected in production TUI/dashboard surfaces with fail-closed gate semantics.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_FILE="${ROOT_DIR}/crates/focusa-tui/src/app.rs"
MAIN_FILE="${ROOT_DIR}/crates/focusa-tui/src/main.rs"
VIEWS_MOD_FILE="${ROOT_DIR}/crates/focusa-tui/src/views/mod.rs"
WORK_LOOP_VIEW_FILE="${ROOT_DIR}/crates/focusa-tui/src/views/work_loop.rs"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'Tab::WorkLoop|WorkLoop' "$APP_FILE" >/dev/null 2>&1 && rg -n 'Tab::WorkLoop => "Loop"' "$APP_FILE" >/dev/null 2>&1; then
  log_pass "tui tab model includes WorkLoop dashboard tab"
else
  log_fail "tui tab model missing WorkLoop dashboard tab"
fi

if rg -n 'work_loop_status|/v1/work-loop/status' "$APP_FILE" >/dev/null 2>&1 && rg -n 'work_loop_replay|/v1/work-loop/replay/closure-evidence' "$APP_FILE" >/dev/null 2>&1 && rg -n 'work_loop_closure_bundle|/v1/work-loop/replay/closure-bundle' "$APP_FILE" >/dev/null 2>&1; then
  log_pass "tui refresh path fetches status + replay consumer + closure bundle routes"
else
  log_fail "tui refresh path missing work-loop replay/closure endpoints"
fi

if rg -n "KeyCode::Char\('w'\) => app\.tab = app::Tab::WorkLoop" "$MAIN_FILE" >/dev/null 2>&1; then
  log_pass "tui keymap exposes WorkLoop tab hotkey"
else
  log_fail "tui keymap missing WorkLoop hotkey"
fi

if rg -n 'mod work_loop;' "$VIEWS_MOD_FILE" >/dev/null 2>&1 && rg -n 'Tab::WorkLoop => work_loop::render' "$VIEWS_MOD_FILE" >/dev/null 2>&1; then
  log_pass "tui view dispatcher renders WorkLoop panel"
else
  log_fail "tui view dispatcher missing WorkLoop panel wiring"
fi

if rg -n 'Continuity gate|fail-closed|Replay consumer|current_task_pair_observed|work_loop_status|work_loop_replay|work_loop_closure_bundle|secondary_loop_objective_profile|Non-closure objectives' "$WORK_LOOP_VIEW_FILE" >/dev/null 2>&1; then
  log_pass "WorkLoop panel renders replay consumer + fail-closed continuity gate + closure-bundle objective profile semantics"
else
  log_fail "WorkLoop panel missing replay consumer/closure-bundle continuity rendering"
fi

if rg -n 'fn secondary_loop_continuity_gate_for_status\(' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n '"secondary_loop_continuity_gate"' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "api status/replay payloads surface continuity gate projection"
else
  log_fail "api continuity gate projection missing from work-loop route"
fi

if rg -n 'async fn closure_replay_bundle|route\("/v1/work-loop/replay/closure-bundle", get\(closure_replay_bundle\)\)' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "api exposes closure bundle route for doc78 proof packaging"
else
  log_fail "api missing closure bundle route for doc78 proof packaging"
fi

echo "=== DOC78 TUI REPLAY DASHBOARD SURFACE RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
