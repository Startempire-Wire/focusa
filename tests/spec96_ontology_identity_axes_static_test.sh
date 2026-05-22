#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ONT="${ROOT_DIR}/crates/focusa-api/src/routes/ontology.rs"
WORKPOINT="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"
SPEC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if rg -n 'ontology_identity_axes_payload|ontology_identity_axes_v1|identity_axes|project_root_plus_continuity_id' "$ONT" >/dev/null; then
  echo "✓ PASS: ontology active context exposes identity axes projection"
else
  echo "✗ FAIL: ontology identity axes projection missing" >&2
  exit 1
fi

if rg -n 'workpoint_continuation_card|daemon_session_id|adapter_session|temporal_metadata_only|session_id_as_authority_gate' "$ONT" "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: identity axes map Workpoint card, daemon/session metadata, and authority warnings"
else
  echo "✗ FAIL: identity axes mappings/warnings missing" >&2
  exit 1
fi

if rg -n 'workpoint_identity_axes_payload|workpoint_identity_axes_v1|identity_axes.*authority_gate' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: Workpoint Resume Packet v2 includes identity axes"
else
  echo "✗ FAIL: Workpoint Resume Packet v2 identity axes missing" >&2
  exit 1
fi

if rg -n 'focusa_workpoint_resume|focusa_trajectory_view|focusa_traverse|rehydrate_refs' "$ONT" "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: identity axes include targeted rehydrate refs"
else
  echo "✗ FAIL: identity axes rehydrate refs missing" >&2
  exit 1
fi

if rg -n 'Ontology identity axes' "$SPEC" >/dev/null; then
  echo "✓ PASS: Spec documents ontology identity axes boundary"
else
  echo "✗ FAIL: Spec identity axes documentation missing" >&2
  exit 1
fi

echo "SPEC96 ontology identity axes static test: PASS"
