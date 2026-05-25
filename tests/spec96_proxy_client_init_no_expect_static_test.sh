#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PROXY="${ROOT_DIR}/crates/focusa-api/src/routes/proxy.rs"

if rg -n 'failed to build HTTP client|\.build\(\)\s*\.expect\(' "$PROXY" >/dev/null; then
  echo "✗ FAIL: proxy HTTP client initialization still panics on builder failure" >&2
  exit 1
fi

if rg -n 'Proxy HTTP client builder failed; using default reqwest client fallback|Client::new\(\)' "$PROXY" >/dev/null; then
  echo "✓ PASS: proxy HTTP client initialization has logged fallback"
else
  echo "✗ FAIL: proxy HTTP client initialization fallback missing" >&2
  exit 1
fi

echo "SPEC96 Proxy client init no-expect static test: PASS"
