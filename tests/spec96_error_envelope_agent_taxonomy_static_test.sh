#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ERR="${ROOT_DIR}/crates/focusa-api/src/middleware/error_envelope.rs"

if rg -n 'StatusCode::METHOD_NOT_ALLOWED => "validation_rejected"' "$ERR" >/dev/null; then
  echo "✓ PASS: method-not-allowed is classified as validation_rejected, not ambiguous completion"
else
  echo "✗ FAIL: method-not-allowed still has ambiguous failure taxonomy" >&2
  exit 1
fi

if rg -n 'StatusCode::NOT_FOUND => "not_found"' "$ERR" >/dev/null; then
  echo "✓ PASS: not-found is classified as not_found, not ambiguous completion"
else
  echo "✗ FAIL: not-found still has ambiguous failure taxonomy" >&2
  exit 1
fi

if rg -n 'request_method|request_path|"request": \{"method": request_method, "path": request_path\}' "$ERR" >/dev/null; then
  echo "✓ PASS: error envelope includes request method/path for agent correction"
else
  echo "✗ FAIL: error envelope lacks request method/path correction context" >&2
  exit 1
fi

if rg -n '_ if status\.is_client_error\(\) => "validation_rejected"' "$ERR" >/dev/null; then
  echo "✓ PASS: generic client errors use validation_rejected instead of ambiguous completion"
else
  echo "✗ FAIL: generic client errors still fall through to ambiguous completion" >&2
  exit 1
fi

echo "SPEC96 API error envelope agent taxonomy static test: PASS"
