#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PROXY="${ROOT_DIR}/crates/focusa-api/src/routes/proxy.rs"

if rg -n 'Response::builder\(\)\.status\(status\)' "$PROXY" >/dev/null && rg -n 'proxy_response_build_failed|Could not build Anthropic stream response' "$PROXY" >/dev/null; then
  echo "✓ PASS: Anthropic stream response builder has typed failure envelope"
else
  echo "✗ FAIL: Anthropic stream response builder typed failure envelope missing" >&2
  exit 1
fi

if rg -n 'body\(Body::from_stream\(body_stream\)\)\.unwrap\(\)' "$PROXY" >/dev/null; then
  echo "✗ FAIL: Anthropic stream response builder still unwraps" >&2
  exit 1
fi

echo "SPEC96 Proxy stream builder no-unwrap static test: PASS"
