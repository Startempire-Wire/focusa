#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
DOC="${ROOT_DIR}/docs/focusa-tools/tools/focusa_predict_evaluate.md"

if rg -n 'res\.status === 404 \? "not_found"|focusa_predict_recent", "focusa_predict_record"' "$TOOLS" >/dev/null; then
  echo "✓ PASS: focusa_predict_evaluate maps missing prediction ids to not_found recovery"
else
  echo "✗ FAIL: focusa_predict_evaluate still maps missing prediction ids ambiguously" >&2
  exit 1
fi

if rg -n 'failure_class=not_found|focusa_predict_recent.*focusa_predict_record' "$DOC" >/dev/null; then
  echo "✓ PASS: prediction evaluate docs describe not_found recovery"
else
  echo "✗ FAIL: prediction evaluate docs lack not_found recovery guidance" >&2
  exit 1
fi

echo "SPEC96 Predict evaluate not-found static test: PASS"
