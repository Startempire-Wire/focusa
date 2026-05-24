#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
AUDIT="${ROOT_DIR}/scripts/audit-focusa-tool-suite-safe.mjs"

if rg -n 'getJson\(endpoint, \{ recordLatency: false \}\)|recordLatencyGuardrail\(endpoint, bestElapsedMs, true\)' "$AUDIT" >/dev/null; then
  echo "✓ PASS: audit retries safe probes without recording failed-attempt latency as final failure"
else
  echo "✗ FAIL: audit retry can record transient failed-attempt latency as final failure" >&2
  exit 1
fi

if rg -n 'FOCUSA_AUDIT_HOT_ROUTE_LATENCY_SAMPLES|hotRouteLatencySamples|Math\.min\(bestElapsedMs, sampled\.elapsed_ms\)|hot_latency_samples' "$AUDIT" >/dev/null; then
  echo "✓ PASS: audit hot-route latency guardrail uses bounded best-of sampling"
else
  echo "✗ FAIL: audit hot-route latency guardrail still relies on a single jitter-prone sample" >&2
  exit 1
fi

if rg -n 'const body = await getJsonWithRetry\(endpoint\)' "$AUDIT" >/dev/null && ! rg -n 'const body = await getJson\(endpoint\)' "$AUDIT" >/dev/null; then
  echo "✓ PASS: audit safe GET loop uses retry helper"
else
  echo "✗ FAIL: audit safe GET loop bypasses retry helper" >&2
  exit 1
fi

echo "SPEC96 tool suite audit retry static test: PASS"
