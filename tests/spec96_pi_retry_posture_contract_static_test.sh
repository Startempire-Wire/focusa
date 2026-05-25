#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"

if rg -n 'check_scope_or_daemon' "$TOOLS" >/dev/null; then
  echo "✗ FAIL: Pi tool_result fallback emits non-contract retry posture check_scope_or_daemon" >&2
  exit 1
fi

if rg -n 'type FocusaRetryPosture = "safe_retry" \| "retry_with_idempotency_key" \| "check_side_effects_first" \| "do_not_retry_unchanged" \| "operator_required"' "$TOOLS" >/dev/null \
  && rg -n 'posture: result\.ok \? "safe_retry" : "check_side_effects_first"' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi fallback retry postures use FocusaRetryPosture contract values"
else
  echo "✗ FAIL: Pi fallback retry posture contract values missing" >&2
  exit 1
fi

echo "SPEC96 Pi retry posture contract static test: PASS"
