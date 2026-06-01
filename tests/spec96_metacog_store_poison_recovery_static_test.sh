#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
METACOG="${ROOT_DIR}/crates/focusa-api/src/routes/metacognition.rs"

if rg -n 'metacog store poisoned' "$METACOG" >/dev/null; then
  echo "✗ FAIL: metacognition routes still panic on poisoned store lock" >&2
  exit 1
fi

count=$(rg -n 'unwrap_or_else\(\|poisoned\| poisoned\.into_inner\(\)\)' "$METACOG" | wc -l | tr -d ' ')
if [ "$count" -ge 8 ]; then
  echo "✓ PASS: metacognition store locks recover from poison instead of panicking"
else
  echo "✗ FAIL: metacognition store poison recovery not applied broadly" >&2
  exit 1
fi

echo "SPEC96 Metacog store poison recovery static test: PASS"
