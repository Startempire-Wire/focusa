#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PROOF="${ROOT_DIR}/scripts/prove-focusa-tool-contracts-live.mjs"

if rg -n 'FOCUSA_LIVE_PROOF_ATTEMPTS|requestAttempts|for \(let attempt = 1; attempt <= requestAttempts; attempt\+\+\)' "$PROOF" >/dev/null; then
  echo "✓ PASS: live tool contract proof retries transient GET failures"
else
  echo "✗ FAIL: live tool contract proof still uses single-shot GET probes" >&2
  exit 1
fi

if rg -n 'FOCUSA_LIVE_PROOF_TIMEOUT_MS|request_timeout_ms|request_attempts' "$PROOF" >/dev/null; then
  echo "✓ PASS: live proof exposes timeout/attempt metadata"
else
  echo "✗ FAIL: live proof lacks timeout/attempt metadata" >&2
  exit 1
fi

echo "SPEC96 live tool contract retry static test: PASS"
