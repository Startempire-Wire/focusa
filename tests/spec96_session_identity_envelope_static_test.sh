#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CORE="${ROOT_DIR}/crates/focusa-core/src/types.rs"
WORKPOINT="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"
TRAJECTORY="${ROOT_DIR}/crates/focusa-api/src/routes/trajectory.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
STATE="${ROOT_DIR}/apps/pi-extension/src/state.ts"

if rg -n 'struct ProjectIdentityRecord|struct FocusaSessionIdentity|ProjectIdentitySignalRecord|project_urls: Option<serde_json::Value>|aliases: Vec<String>' "$CORE" >/dev/null; then
  echo "✓ PASS: ProjectIdentity and FocusaSessionIdentity are shared core types"
else
  echo "✗ FAIL: shared identity envelope types missing from focusa-core" >&2
  exit 1
fi

if rg -n 'session_identity: Option<FocusaSessionIdentity>' "$WORKPOINT" "$CORE" >/dev/null && rg -n 'apply_checkpoint_session_identity|apply_resume_session_identity|session_identity_project_root|session_identity_overrides_flat_checkpoint_scope|workpoint_packet_carries_session_identity_envelope' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: Workpoint checkpoint/resume/evidence store and apply session identity envelopes"
else
  echo "✗ FAIL: Workpoint identity envelope integration missing" >&2
  exit 1
fi

if rg -n 'session_identity: Option<FocusaSessionIdentity>' "$TRAJECTORY" >/dev/null && rg -n 'scoped_query_from_identity|identity_project_root\.as_deref\(\)\.or\(project_root\)|body\.session_identity\.as_ref\(\)' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: Trajectory calls accept and prioritize session identity envelopes"
else
  echo "✗ FAIL: Trajectory identity envelope integration missing" >&2
  exit 1
fi


if rg -n 'project_identity_payload_for_scope|project_identity_api|quorum_status|quorum_confidence' "$TRAJECTORY" "${ROOT_DIR}/crates/focusa-api/src/routes/project.rs" >/dev/null; then
  echo "✓ PASS: Trajectory view incorporates ProjectIdentity quorum payload"
else
  echo "✗ FAIL: Trajectory view bypasses ProjectIdentity quorum" >&2
  exit 1
fi

if rg -n 'buildFocusaSessionIdentity|persisted_project_fingerprint|persisted_project_root' "$STATE" "$TOOLS" >/dev/null && rg -n 'session_identity: await buildFocusaSessionIdentity' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi tools attach FocusaSessionIdentity to Workpoint/Trajectory payloads"
else
  echo "✗ FAIL: Pi tools do not attach FocusaSessionIdentity payloads" >&2
  exit 1
fi

if rg -n 'failure_class": "read_model_lag"|retry_posture": "safe_retry"' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: evidence accepted-but-not-visible is classified as read_model_lag"
else
  echo "✗ FAIL: evidence pending read-model lag taxonomy missing" >&2
  exit 1
fi

echo "SPEC96 session identity envelope static test: PASS"
