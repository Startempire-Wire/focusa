#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
COMMANDS="${ROOT_DIR}/crates/focusa-api/src/routes/commands.rs"

if rg -n 'tokio::time::timeout\(Duration::from_millis\(1500\), state\.command_tx\.send\(action\)\)|command_dispatch_timeout' "$COMMANDS" >/dev/null; then
  echo "✓ PASS: command submit bounds daemon channel dispatch wait"
else
  echo "✗ FAIL: command submit can hang awaiting daemon command channel" >&2
  exit 1
fi

if rg -n 'command dispatch pending|failure_class.*resource_exhausted|focusa_resource_mode|command dispatch timed out before enqueue' "$COMMANDS" >/dev/null; then
  echo "✓ PASS: command dispatch timeout returns typed recovery envelope and pending log"
else
  echo "✗ FAIL: command dispatch timeout lacks typed recovery envelope/log" >&2
  exit 1
fi

echo "SPEC96 command dispatch timeout static test: PASS"
