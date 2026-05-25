#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

if rg -n 'approval_placeholder' "${ROOT_DIR}/apps/pi-extension/src/tool-contracts.ts" "${ROOT_DIR}/scripts/validate-focusa-tool-contracts.mjs" "${ROOT_DIR}/docs/focusa-tools/tools" "${ROOT_DIR}/docs/90-ontology-backed-tool-contracts-parity-spec.md" >/dev/null; then
  echo "✗ FAIL: vague approval_placeholder exemption remains in active contracts/docs/validator" >&2
  exit 1
fi

if rg -n 'focusa_state_hygiene_(doctor|plan)' "${ROOT_DIR}/apps/pi-extension/src/tool-contracts.ts" >/dev/null && rg -n '"pi_only"' "${ROOT_DIR}/apps/pi-extension/src/tool-contracts.ts" >/dev/null; then
  echo "✓ PASS: state hygiene contracts use precise pi_only exemption"
else
  echo "✗ FAIL: state hygiene pi_only exemption not found" >&2
  exit 1
fi

echo "SPEC96 No approval-placeholder exemptions static test: PASS"
