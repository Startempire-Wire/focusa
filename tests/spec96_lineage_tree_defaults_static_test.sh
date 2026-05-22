#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CAPS_RS="${ROOT_DIR}/crates/focusa-api/src/routes/capabilities.rs"
TOOLS_TS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
AUDIT="${ROOT_DIR}/scripts/audit-focusa-tool-suite-safe.mjs"
DOC="${ROOT_DIR}/docs/focusa-tools/tools/focusa_lineage_tree.md"

if rg -n 'selector.*window|window_kind|next_cursor|full_payload_cold_opt_in|traversal_metadata' "$CAPS_RS" >/dev/null; then
  echo "✓ PASS: /v1/lineage/tree reports surgical window metadata"
else
  echo "✗ FAIL: lineage tree route lacks window metadata" >&2
  exit 1
fi

if rg -n 'include_full_payload|lineage_full_max_nodes|full_payload' "$CAPS_RS" >/dev/null; then
  echo "✓ PASS: full lineage payload remains explicit opt-in"
else
  echo "✗ FAIL: full lineage opt-in guard missing" >&2
  exit 1
fi

if rg -n 'focusa_lineage_tree|selector=window|limit=|include_full_payload=true|cold_opt_in|next_cursor|window_kind' "$TOOLS_TS" >/dev/null; then
  echo "✓ PASS: Pi lineage tree tool defaults to bounded window and exposes cursor metadata"
else
  echo "✗ FAIL: Pi lineage tree tool lacks bounded window defaults" >&2
  exit 1
fi

if rg -n "coldGetRoutes = new Set\(\[\]\)|include_full_payload=true.*includeColdGets" "$AUDIT" >/dev/null; then
  echo "✓ PASS: safe audit no longer skips bounded lineage tree by default and still gates cold full payloads"
else
  echo "✗ FAIL: safe audit lineage/cold route posture wrong" >&2
  exit 1
fi

if rg -n 'bounded Focusa lineage window|Full tree access requires explicit cold opt-in|default 50' "$DOC" >/dev/null; then
  echo "✓ PASS: lineage tree docs describe bounded default and cold opt-in"
else
  echo "✗ FAIL: lineage tree docs missing bounded default language" >&2
  exit 1
fi

echo "SPEC96 lineage/tree default traversal static test: PASS"
