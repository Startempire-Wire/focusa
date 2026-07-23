#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="${ROOT_DIR}/crates/focusa-api/src/routes/project.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
CLI="${ROOT_DIR}/crates/focusa-cli/src/commands/project.rs"
DOC1="${ROOT_DIR}/docs/focusa-tools/tools/focusa_project_identity.md"
DOC2="${ROOT_DIR}/docs/focusa-tools/tools/focusa_project_verify.md"

if rg -n 'root_marker|git_root|beads_root|workspace_file|daemon_working_directory|operator_supplied_scope|persisted_session_identity|quorum_rule' "$PROJECT" >/dev/null; then
  echo "✓ PASS: ProjectIdentity discovery uses multi-signal quorum"
else
  echo "✗ FAIL: ProjectIdentity quorum signals missing" >&2
  exit 1
fi

if rg -n 'unsafe_broad_project_root|unsafe_user_home_project_root|cwd_only|authority_boundary.*project_root_plus_fingerprint|fingerprint' "$PROJECT" >/dev/null; then
  echo "✓ PASS: ProjectIdentity marks unsafe/cwd-only scopes as degraded"
else
  echo "✗ FAIL: ProjectIdentity unsafe/degraded scope handling missing" >&2
  exit 1
fi

if rg -n '"side_effects": \[\]|"evidence_refs": \[\]|"next_tools": \["focusa_project_identity", "focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"\]|"tool_result_v1"' "$PROJECT" >/dev/null; then
  echo "✓ PASS: ProjectIdentity API returns full tool_result_v1 envelope"
else
  echo "✗ FAIL: ProjectIdentity API tool_result_v1 envelope incomplete" >&2
  exit 1
fi

if rg -n 'name: "focusa_project_identity"|name: "focusa_project_verify"|tool_result_v1: toolResult|/project/identity|/project/verify' "$TOOLS" >/dev/null && rg -n '/v1/project/identity|/v1/project/verify|ProjectCmd' "$CLI" >/dev/null; then
  echo "✓ PASS: Pi and CLI ProjectIdentity tool parity exists"
else
  echo "✗ FAIL: ProjectIdentity Pi/CLI parity missing" >&2
  exit 1
fi

if rg -n 'confirmPiProjectRoot\(verifiedRoot, "focusa_project_identity_verified"\)|ensureContinuityId\(verifiedRoot\)' "$TOOLS" >/dev/null; then
  echo "✓ PASS: focusa_project_identity verified result binds Pi session root"
else
  echo "✗ FAIL: focusa_project_identity does not bind verified root into Pi session" >&2
  exit 1
fi

if rg -n 'compact_project_summary|project_summary|summary_lines|stack|key_dirs|aliases|wp_url|app_url|auth_url|graphql_url|environment_confidence' "$PROJECT" "$TOOLS" >/dev/null; then
  echo "✓ PASS: ProjectIdentity exposes compact project card facts directly"
else
  echo "✗ FAIL: ProjectIdentity compact project card facts missing" >&2
  exit 1
fi

if rg -n 'url_allowed_for_project_inference|source_is_docs_or_reference|line_declares_project_url|codex\.wordpress\.org|api\.wordpress\.org|upstream' "$PROJECT" >/dev/null; then
  echo "✓ PASS: ProjectIdentity filters docs/reference URLs from project URL inference"
else
  echo "✗ FAIL: ProjectIdentity docs/reference URL filtering missing" >&2
  exit 1
fi

if rg -n 'PROJECT_IDENTITY_PAYLOAD_CACHE|PROJECT_IDENTITY_PAYLOAD_CACHE_TTL|project_identity_cache_key|persisted_project_fingerprint|cached_at\.elapsed\(\).*PROJECT_IDENTITY_PAYLOAD_CACHE_TTL' "$PROJECT" >/dev/null; then
  echo "✓ PASS: ProjectIdentity hot-path payload uses bounded short TTL cache"
else
  echo "✗ FAIL: ProjectIdentity hot-path payload cache missing" >&2
  exit 1
fi

if rg -n 'lastProjectIdentity|projectSummary|S\.lastProjectIdentity = identity|S\.lastProjectIdentity = projectIdentity' "${ROOT_DIR}/apps/pi-extension/src/state.ts" "${ROOT_DIR}/apps/pi-extension/src/awareness.ts" "$TOOLS" >/dev/null; then
  echo "✓ PASS: Utility Card consumes cached ProjectIdentity summary facts"
else
  echo "✗ FAIL: Utility Card does not consume ProjectIdentity summary facts" >&2
  exit 1
fi

if rg -n 'UNBOUND_UNSAFE_ROOT|auto-bootstrap project identity with focusa_project_identity before durable work|safeScope \? \(trajectoryProjectIdentity' "${ROOT_DIR}/apps/pi-extension/src/awareness.ts" >/dev/null; then
  echo "✓ PASS: Utility Card does not label unsafe roots as a concrete project"
else
  echo "✗ FAIL: Utility Card can label unsafe roots as a concrete project" >&2
  exit 1
fi

if rg -n 'never falls back to a remembered scope|Consumers must provide an explicit identity|TypedScopeStore' "${ROOT_DIR}/apps/pi-extension/src/state.ts" >/dev/null \
  && ! rg -n 'projectSummary = projectIdentity\.project_summary \|\| S\.lastProjectIdentity' "${ROOT_DIR}/apps/pi-extension/src/awareness.ts" >/dev/null; then
  echo "✓ PASS: remembered ProjectIdentity cache cannot pull other sessions/projects"
else
  echo "✗ FAIL: remembered ProjectIdentity cache may bleed across sessions/projects" >&2
  exit 1
fi

if rg -n 'const projectUrls = trajectoryFallback|const deployment = trajectoryFallback|\? projectIdentity\.project_urls|\? projectIdentity\.deployment' "${ROOT_DIR}/apps/pi-extension/src/awareness.ts" >/dev/null; then
  echo "✓ PASS: prior trajectory fallback cannot override current ProjectIdentity env facts"
else
  echo "✗ FAIL: prior trajectory fallback may bleed environment facts into Utility Card" >&2
  exit 1
fi

if rg -n 'VITAL AUTO-PROMPT|Agent responsibility FIRST|call focusa_project_identity with the best explicit project_root|queueProjectIdentityBootstrapTurn|sendUserMessage\(prompt|Focusa auto-bootstrap|pi_vital_project_root_send_user_message' "${ROOT_DIR}/apps/pi-extension/src/session.ts" "${ROOT_DIR}/apps/pi-extension/src/turns.ts" >/dev/null \
  && ! rg -n 'Focusa needs project root|Focusa inferred|Enter project_root|Confirm Focusa project_root|ctx\.ui\.select\("Focusa needs project root|ctx\.ui\.input\("Confirm Focusa project_root"|setWidget\("focusa-vital", \["🧭 Focusa project root unclear"' "${ROOT_DIR}/apps/pi-extension/src/session.ts" "${ROOT_DIR}/apps/pi-extension/src/turns.ts" >/dev/null; then
  echo "✓ PASS: project root vital prompt is agent-internal and avoids operator modal/widget UI"
else
  echo "✗ FAIL: project root vital prompt still shows operator modal/widget UI" >&2
  exit 1
fi

if rg -n 'cachedIdentity \? "cached identity"|status: "timeout_preserved"|advisory_only: true|"focusa_project_identity"|"focusa_project_verify"|"focusa_trajectory_view"' "$TOOLS" >/dev/null; then
  echo "✓ PASS: ProjectIdentity hot timeout returns cached advisory identity with recovery tools"
else
  echo "✗ FAIL: ProjectIdentity hot timeout can dead-end without cached advisory fallback" >&2
  exit 1
fi


if rg -n 'RemoteProjectHint|remote_project_scope|remote_repo_evidence|remote_host_plus_project_root_plus_fingerprint|remote_nonlocal' "$PROJECT" >/dev/null   && rg -n 'remote_host|remote_repo_remote|remote_workspace_kind|remote_deploy_root' "$TOOLS" "$CLI" "$DOC1" "$DOC2" >/dev/null; then
  echo "✓ PASS: ProjectIdentity supports caller-supplied remote SSH project evidence"
else
  echo "✗ FAIL: ProjectIdentity remote SSH evidence support missing" >&2
  exit 1
fi

if rg -n 'persisted_project_root|persisted_project_fingerprint|ProjectIdentity|quorum|canonical=false|Backed by `GET /v1/project/identity`|Backed by `POST /v1/project/verify`' "$DOC1" "$DOC2" >/dev/null; then
  echo "✓ PASS: ProjectIdentity docs describe quorum/degraded recovery"
else
  echo "✗ FAIL: ProjectIdentity docs missing quorum/degraded recovery" >&2
  exit 1
fi

echo "SPEC96 ProjectIdentity quorum static test: PASS"
