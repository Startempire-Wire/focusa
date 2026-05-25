#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TRAINING="${ROOT_DIR}/crates/focusa-api/src/routes/training.rs"
CONSTITUTION="${ROOT_DIR}/crates/focusa-api/src/routes/constitution.rs"

for file in "$TRAINING" "$CONSTITUTION"; do
  if rg -n '"failure_class": "not_found"' "$file" >/dev/null && rg -n '"posture": "do_not_retry_unchanged", "reason": "not_found"' "$file" >/dev/null; then
    echo "✓ PASS: $(basename "$file") stable not_found uses do_not_retry_unchanged"
  else
    echo "✗ FAIL: $(basename "$file") stable not_found retry posture is unsafe or missing" >&2
    exit 1
  fi
done

if rg -n 'contribution queue item not found.*"posture": "safe_retry"|No active constitution[\s\S]*"posture": "safe_retry"' "$TRAINING" "$CONSTITUTION" >/dev/null; then
  echo "✗ FAIL: stable not_found path still advertises safe_retry" >&2
  exit 1
fi

echo "SPEC96 Stable not-found retry posture static test: PASS"
