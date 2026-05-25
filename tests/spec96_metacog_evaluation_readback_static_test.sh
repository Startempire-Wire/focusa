#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
META="${ROOT_DIR}/crates/focusa-api/src/routes/metacognition.rs"
CLI="${ROOT_DIR}/crates/focusa-cli/src/commands/metacognition.rs"

if rg -n 'recent_evaluations\(|/v1/metacognition/evaluations/recent|"evaluations": window' "$META" >/dev/null; then
  echo "✓ PASS: metacognition API exposes recent evaluation readback"
else
  echo "✗ FAIL: metacognition evaluations persist without API readback" >&2
  exit 1
fi

if rg -n 'RecentEvaluations|/v1/metacognition/evaluations/recent\?limit=|recent-evaluations' "$CLI" >/dev/null; then
  echo "✓ PASS: metacognition CLI exposes recent evaluation readback"
else
  echo "✗ FAIL: metacognition CLI lacks recent evaluations command" >&2
  exit 1
fi

echo "SPEC96 Metacog evaluation readback static test: PASS"
