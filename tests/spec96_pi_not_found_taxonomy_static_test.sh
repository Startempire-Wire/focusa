#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"

if rg -n '\| "not_found"' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi FocusaFailureClass includes not_found"
else
  echo "✗ FAIL: Pi FocusaFailureClass lacks not_found" >&2
  exit 1
fi

if rg -n 'text\.includes\("not_found"\)|case "not_found"|focusa_predict_recent' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi not_found inference and recovery guidance exists"
else
  echo "✗ FAIL: Pi not_found inference/recovery missing" >&2
  exit 1
fi

echo "SPEC96 Pi not-found taxonomy static test: PASS"
