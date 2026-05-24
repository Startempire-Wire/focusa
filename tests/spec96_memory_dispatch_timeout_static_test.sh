#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MEMORY="${ROOT_DIR}/crates/focusa-api/src/routes/memory.rs"

if rg -n 'dispatch_memory_action|tokio::time::timeout\(Duration::from_millis\(1500\), state\.command_tx\.send\(action\)\)|memory_dispatch_timeout' "$MEMORY" >/dev/null; then
  echo "✓ PASS: memory routes bound command dispatch waits"
else
  echo "✗ FAIL: memory routes can hang awaiting saturated command channel" >&2
  exit 1
fi

if rg -n 'failure_class": "resource_exhausted"|retry_posture": "safe_retry"|memory action was not enqueued|focusa_resource_mode' "$MEMORY" >/dev/null; then
  echo "✓ PASS: memory dispatch timeout returns typed recovery envelope"
else
  echo "✗ FAIL: memory dispatch timeout lacks typed recovery envelope" >&2
  exit 1
fi

if rg -n 'dispatch_memory_action\(\s*&state,\s*Action::UpsertSemantic|dispatch_memory_action\(&state, Action::ReinforceRule' "$MEMORY" >/dev/null; then
  echo "✓ PASS: semantic upsert and rule reinforcement use bounded dispatch"
else
  echo "✗ FAIL: a memory write route still bypasses bounded dispatch" >&2
  exit 1
fi

echo "SPEC96 memory dispatch timeout static test: PASS"
