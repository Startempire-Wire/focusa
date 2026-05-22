#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_RS="${ROOT_DIR}/crates/focusa-core/src/runtime/daemon.rs"
SERVER_RS="${ROOT_DIR}/crates/focusa-api/src/server.rs"
ROUTES_DIR="${ROOT_DIR}/crates/focusa-api/src/routes"

if rg -n 'serde_json::to_vec\(&\*shared\)|serde_json::to_vec\(&self\.state\)' "$DAEMON_RS" >/dev/null; then
  echo "✗ FAIL: daemon reconciliation still serializes full state for equality" >&2
  rg -n 'serde_json::to_vec\(&\*shared\)|serde_json::to_vec\(&self\.state\)' "$DAEMON_RS" >&2 || true
  exit 1
fi

echo "✓ PASS: daemon reconciliation has no full-state JSON equality serialization"

if rg -n 'external_mutation_epoch\.load\(Ordering::Acquire\)|observed_external_mutation_epoch' "$DAEMON_RS" >/dev/null && rg -n 'mark_external_mutation' "$SERVER_RS" >/dev/null; then
  echo "✓ PASS: explicit external mutation epoch is wired"
else
  echo "✗ FAIL: external mutation epoch wiring missing" >&2
  exit 1
fi

fail=0
while IFS= read -r file; do
  writes=$(rg -o 'state\.focusa\.write\(\)|focusa\.write\(\)' "$file" | wc -l | tr -d ' ')
  marks=$(rg -o 'mark_external_mutation\(\)' "$file" | wc -l | tr -d ' ')
  if [[ "$writes" -gt "$marks" ]]; then
    echo "✗ FAIL: $file has $writes direct focusa writes but only $marks external mutation marks" >&2
    fail=1
  else
    echo "✓ PASS: $(basename "$file") direct writes=$writes mutation marks=$marks"
  fi
done < <(rg -l 'state\.focusa\.write\(\)|focusa\.write\(\)' "$ROUTES_DIR")

if [[ "$fail" != "0" ]]; then
  exit 1
fi

echo "SPEC96 external mutation epoch static test: PASS"
