#!/usr/bin/env bash
# Spec 117 launch blocker/perf — blazing-fast TUI startup static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

SP="$ROOT_DIR/crates/focusa-tui/src/startup_perf.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
[[ -f "$SP" ]] || fail "startup_perf.rs missing"

for needle in \
  'FIRST_PAINT_BUDGET_MS' \
  'SHELL_RENDER_PHASES' \
  'frame_zero_local_defaults' \
  'daemon_state_progressive_fetch' \
  'secondary_panels_lazy_load' \
  'PROGRESSIVE_LOADING_PLAN' \
  'deck_home' \
  'mission_ladder' \
  'proof_meter' \
  'scope_badge' \
  'StartupReport'; do
  grep -qF -- "$needle" "$SP" || fail "startup_perf missing: $needle"
done
pass "startup performance module covers first-paint budget and progressive plan"

grep -qF 'mod startup_perf;' "$MAIN" || fail "main.rs missing startup_perf module"
grep -qF 'first_paint_budget_ms' "$MAIN" || fail "headless proof missing first_paint_budget_ms"
grep -qF 'progressive_loading_plan' "$MAIN" || fail "headless proof missing progressive_loading_plan"
pass "startup perf exposed in headless proof"

echo "focusa-117 tui-startup-perf static test: PASS"
