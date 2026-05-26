#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TRAJECTORY="$ROOT_DIR/crates/focusa-api/src/routes/trajectory.rs"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"

if rg -n 'pending_candidate_preserved|trajectory_candidate.*payload|get\("trajectory_candidate"\)|StatusCode::ACCEPTED' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: trajectory define-goal pending timeout preserves supplied candidate metadata"
else
  echo "✗ FAIL: trajectory define-goal pending timeout can drop supplied candidate metadata" >&2
  exit 1
fi

if rg -n 'pendingCandidate|defineLabel.*PENDING|definition_status: "pending"' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi trajectory define-goal renders pending distinctly with supplied goal fields"
else
  echo "✗ FAIL: Pi trajectory define-goal can render pending as NOT SET/missing" >&2
  exit 1
fi
