#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"

pass() { echo "✓ PASS: $1"; }
fail() { echo "✗ FAIL: $1" >&2; exit 1; }

if rg -n "scopeRecoveryContext|operator steering is authority|create focusa_workpoint_checkpoint|Store verbose/build/process rules in focusa_scratch" "$TOOLS" >/dev/null; then
  pass "Trajectory/Workpoint degraded outputs include actionable scope-recovery context"
else
  fail "Missing scope-recovery context for degraded trajectory/workpoint outputs"
fi

if rg -n "allowsWorkpointBootstrapFromClarity|bootstrap_allowed|checkpointing current operator mission is allowed|trajectory unclear; checkpointing explicit operator mission is allowed" "$TOOLS" >/dev/null; then
  pass "Workpoint checkpoint can bootstrap fresh continuity when stale trajectory context conflicts"
else
  fail "Workpoint checkpoint remains blocked by stale/conflicted trajectory context"
fi

if rg -n "namedSlotValidationFallback|Original saved to scratchpad fallback|Suggested current_focus|conciseObjectiveSuggestion" "$TOOLS" >/dev/null; then
  pass "Verbose Focus State slot validation saves scratch fallback and suggests concise current_focus"
else
  fail "Verbose Focus State slot rejection still lacks scratch fallback/suggestion"
fi

if rg -n "Scoped frame/continuity is stale; use latest operator instruction, checkpoint a fresh Workpoint" "$TOOLS" >/dev/null; then
  pass "Stale frame write failures explain continuity recovery instead of generic retry"
else
  fail "Stale frame write feedback lacks continuity recovery guidance"
fi

echo "SPEC96 scope recovery feedback static test: PASS"
