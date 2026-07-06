#!/usr/bin/env bash
# Spec 117 .30 — Next Safe Action aligned with Spec 119 §7.6 + §19.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

NSA="$ROOT_DIR/crates/focusa-tui/src/next_safe_action.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
[[ -f "$NSA" ]] || fail "next_safe_action.rs missing"
for needle in \
  'pub struct RecoveryTool' \
  'pub recovery_tools: &' \
  'HEADLESS_PROOF_RECOVERY_TOOL_CAP' \
  'recovery_tools_are_bounded_to_three' \
  'focusa doctor --scope host' \
  'focusa workpoint resume' \
  'focusa walkthrough show --walkthrough first-mission'; do
  grep -qF -- "$needle" "$NSA" || fail "next_safe_action missing: $needle"
done
pass "next_safe_action exposes RecoveryTool struct, recovery_tools field, and bounded cap per Spec 119"
grep -qF 'next_safe_action_recovery_tool_cap' "$MAIN" || fail "headless proof missing recovery_tool_cap"
pass "headless proof exposes next_safe_action_recovery_tool_cap"
echo "focusa-117 next-safe-action-spec119 static test: PASS"
