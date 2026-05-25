#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

if rg -n 'safe_retry_after_recovery' "${ROOT_DIR}/crates/focusa-api/src" "${ROOT_DIR}/apps/pi-extension/src" "${ROOT_DIR}/crates/focusa-core/src" -g '*.rs' -g '*.ts' >/dev/null; then
  echo "✗ FAIL: nonstandard retry posture safe_retry_after_recovery still present" >&2
  exit 1
fi

if rg -n '"posture": if failure_class == "validation_rejected" \{ "do_not_retry_unchanged" \} else \{ "safe_retry" \}|"posture": "safe_retry"' "${ROOT_DIR}/crates/focusa-api/src/routes" >/dev/null; then
  echo "✓ PASS: API failure envelopes use standard retry posture vocabulary"
else
  echo "✗ FAIL: standard retry posture vocabulary not found in API route envelopes" >&2
  exit 1
fi

echo "SPEC96 Retry posture vocabulary static test: PASS"
