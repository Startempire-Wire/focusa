#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SERVER_RS="${ROOT_DIR}/crates/focusa-api/src/server.rs"
SESSION_RS="${ROOT_DIR}/crates/focusa-api/src/routes/session.rs"

if rg -n 'fn lowmem_background_throttle|background_concurrency == 0|matches!\(status\.mode, "lowmem" \| "emergency"\)' "$SERVER_RS" >/dev/null; then
  echo "✓ PASS: LowMem background throttle predicate exists"
else
  echo "✗ FAIL: LowMem background throttle predicate missing" >&2
  exit 1
fi

if rg -n 'reflection scheduler tick throttled by LowMem background policy|continuous work supervisor tick throttled by LowMem background policy' "$SERVER_RS" >/dev/null; then
  echo "✓ PASS: reflection scheduler and continuous supervisor are throttled in LowMem"
else
  echo "✗ FAIL: background loops missing LowMem throttle logs" >&2
  exit 1
fi

if rg -n 'background_throttled_ticks' "$SERVER_RS" >/dev/null && rg -n 'background_throttled_ticks' "$SESSION_RS" >/dev/null; then
  echo "✓ PASS: background throttle counter is exposed through status runtime_perf"
else
  echo "✗ FAIL: background throttle counter missing from status runtime_perf" >&2
  exit 1
fi

echo "SPEC96 LowMem background throttle static test: PASS"
