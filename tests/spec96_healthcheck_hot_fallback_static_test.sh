#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
STATE_TS="${ROOT_DIR}/apps/pi-extension/src/state.ts"

if rg -n 'focusaFetch\("/status"\)' "$STATE_TS" >/dev/null; then
  echo "✗ FAIL: healthcheck still probes unbounded /v1/status fallback" >&2
  rg -n 'focusaFetch\("/status"\)' "$STATE_TS" >&2 || true
  exit 1
fi

if rg -n 'HEALTHCHECK_STATUS_FALLBACK_PATH = "/status\?summary_only=true"' "$STATE_TS" >/dev/null; then
  echo "✓ PASS: healthcheck fallback route is bounded status summary"
else
  echo "✗ FAIL: bounded status summary fallback constant missing" >&2
  exit 1
fi

if rg -n 'status/deep|single_daemon_ok' "$STATE_TS" >/dev/null; then
  echo "✗ FAIL: healthcheck fallback must not depend on cold/deep diagnostics" >&2
  rg -n 'status/deep|single_daemon_ok' "$STATE_TS" >&2 || true
  exit 1
fi

if rg -n 'healthcheck_hot_fallback_ok' "$STATE_TS" >/dev/null; then
  echo "✓ PASS: healthcheck emits route-aware fallback telemetry"
else
  echo "✗ FAIL: route-aware fallback telemetry missing" >&2
  exit 1
fi

echo "SPEC96 healthcheck hot fallback static test: PASS"
