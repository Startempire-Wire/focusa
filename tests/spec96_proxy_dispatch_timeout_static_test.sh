#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PROXY="${ROOT_DIR}/crates/focusa-api/src/routes/proxy.rs"

if rg -n 'dispatch_proxy_telemetry|tokio::time::timeout\(Duration::from_millis\(500\), state\.command_tx\.send\(action\)\)' "$PROXY" >/dev/null; then
  echo "✓ PASS: proxy telemetry dispatch is bounded"
else
  echo "✗ FAIL: proxy telemetry dispatch can block provider response" >&2
  exit 1
fi

if rg -n 'proxy telemetry dispatch timed out; continuing provider response' "$PROXY" >/dev/null; then
  echo "✓ PASS: proxy telemetry timeout is logged as nonblocking degradation"
else
  echo "✗ FAIL: proxy telemetry timeout lacks nonblocking log" >&2
  exit 1
fi

if rg -n 'command_tx\.send\(Action::(IngestSignal|EmitEvent)' "$PROXY" >/dev/null; then
  echo "✗ FAIL: proxy still directly awaits telemetry command_tx.send" >&2
  exit 1
else
  echo "✓ PASS: proxy signal/event telemetry uses bounded helper"
fi

echo "SPEC96 proxy dispatch timeout static test: PASS"
