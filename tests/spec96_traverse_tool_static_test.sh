#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE="${ROOT_DIR}/crates/focusa-api/src/routes/traverse.rs"
MOD="${ROOT_DIR}/crates/focusa-api/src/routes/mod.rs"
SERVER="${ROOT_DIR}/crates/focusa-api/src/server.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
CONTRACTS="${ROOT_DIR}/apps/pi-extension/src/tool-contracts.ts"
DOC="${ROOT_DIR}/docs/focusa-tools/tools/focusa_traverse.md"
SKILL="${ROOT_DIR}/apps/pi-extension/skills/focusa/SKILL.md"

if rg -n 'route\("/v1/traverse"|route\("/v1/traverse/verify-tags"|traverse_response|tag_scheme|tool_result_v1' "$ROUTE" "$MOD" "$SERVER" >/dev/null; then
  echo "✓ PASS: /v1/traverse API and verify-tags route are registered"
else
  echo "✗ FAIL: traverse API route registration missing" >&2
  exit 1
fi

if rg -n 'surface|selector|anchor|query|cursor|limit|depth|radius|fields|tags|tag_mode|include_payload|include_full_payload|session_identity' "$ROUTE" "$TOOLS" >/dev/null; then
  echo "✓ PASS: traverse API/tool accepts surgical traversal inputs"
else
  echo "✗ FAIL: traverse surgical inputs missing" >&2
  exit 1
fi

if rg -n 'bounded_window|field_projection|full_payload_blocked_by_pressure|canonical|degraded|failure_class|next_tools|details.*tool_result_v1' "$ROUTE" >/dev/null; then
  echo "✓ PASS: traverse route uses bounded helpers and tool_result_v1 taxonomy"
else
  echo "✗ FAIL: traverse bounded metadata/envelope missing" >&2
  exit 1
fi

if rg -n 'name: "focusa_traverse"|/traverse/verify-tags|Read-only surgical traversal|include_full_payload' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi focusa_traverse tool is registered"
else
  echo "✗ FAIL: Pi focusa_traverse tool missing" >&2
  exit 1
fi

if rg -n 'include_payload: raw\.include_payload|include_payload: .*include_full_payload' "$TOOLS" >/dev/null; then
  echo "✗ FAIL: Pi focusa_traverse sends duplicate include_payload/include_full_payload aliases" >&2
  exit 1
else
  echo "✓ PASS: Pi focusa_traverse normalizes include_payload alias before API request"
fi

if rg -n 'TraversedItem|traversed_items|verified_tags|stale_tags|tag_mode|range_tag_format|window_tag_format|surface_tag_format|algorithm|includes_anchor|includes_surface_version|tag_verify_preserves_item_tag_after_unrelated_change' "$ROUTE" "$TOOLS" "$DOC" >/dev/null; then
  echo "✓ PASS: traversal tags_verify semantics and tag policies are covered"
else
  echo "✗ FAIL: traversal tag semantics or policies missing" >&2
  exit 1
fi

if rg -n '"name": "focusa_traverse"|"family": "traversal"|POST /v1/traverse|POST /v1/traverse/verify-tags' "$CONTRACTS" "${ROOT_DIR}/docs/current/focusa-tool-contracts.json" >/dev/null; then
  echo "✓ PASS: focusa_traverse contract registry is present"
else
  echo "✗ FAIL: focusa_traverse contract missing" >&2
  exit 1
fi

if rg -n 'focusa_traverse|bounded traversal|full payload defaults|tool_result_v1' "$DOC" "$SKILL" "${ROOT_DIR}/README.md" >/dev/null; then
  echo "✓ PASS: focusa_traverse docs/skill/readme coverage present"
else
  echo "✗ FAIL: focusa_traverse docs coverage missing" >&2
  exit 1
fi

echo "SPEC96 focusa_traverse static test: PASS"
