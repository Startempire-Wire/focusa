#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
STATE="$ROOT_DIR/apps/pi-extension/src/state.ts"

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

if rg -n "Focus State frame unavailable|Use scratchpad for this note; checkpoint/resume a project-bound Workpoint" "$TOOLS" >/dev/null \
  && ! rg -n "decision NOT recorded in Focus State|constraint NOT recorded in Focus State|failure NOT recorded in Focus State|Attentive and awaiting operator direction.*decision" "$TOOLS" >/dev/null; then
  pass "Stale frame write failures use concise scratchpad/workpoint recovery copy"
else
  fail "Stale frame write feedback is noisy or lacks continuity recovery guidance"
fi

if rg -n 'Explicit safe project folder/root; use after compaction if Pi cwd is broad like /root\.' "$TOOLS" >/dev/null \
  && rg -n 'enforceTrajectoryClarityPrecondition\(projectRoot, "workpoint evidence link", \{' "$TOOLS" >/dev/null \
  && rg -n 'continuityId: p\.continuity_id' "$TOOLS" >/dev/null \
  && rg -n 'sessionId: p\.session_id' "$TOOLS" >/dev/null \
  && rg -n 'buildFocusaSessionIdentity\(projectRoot, "manual", \{' "$TOOLS" >/dev/null \
  && rg -n 'cwdForIdentity = safe && !ambientInsideProject \? projectRoot : ambientCwd' "$STATE" >/dev/null; then
  pass "Evidence tools carry explicit project/continuity context through clarity gate and session identity"
else
  fail "Evidence tools still depend on ambient Pi cwd/continuity after compaction"
fi

echo "SPEC96 scope recovery feedback static test: PASS"
