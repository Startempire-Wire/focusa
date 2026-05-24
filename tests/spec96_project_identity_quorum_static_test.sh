#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="${ROOT_DIR}/crates/focusa-api/src/routes/project.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
CLI="${ROOT_DIR}/crates/focusa-cli/src/commands/project.rs"
DOC1="${ROOT_DIR}/docs/focusa-tools/tools/focusa_project_identity.md"
DOC2="${ROOT_DIR}/docs/focusa-tools/tools/focusa_project_verify.md"

if rg -n 'root_marker|git_root|beads_root|workspace_file|daemon_working_directory|operator_supplied_scope|quorum_rule' "$PROJECT" >/dev/null; then
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

if rg -n 'compact_project_summary|project_summary|summary_lines|stack|key_dirs|wp_url|app_url|auth_url|graphql_url|environment_confidence' "$PROJECT" "$TOOLS" >/dev/null; then
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

if rg -n 'lastProjectIdentity|projectSummary|S\.lastProjectIdentity = identity|S\.lastProjectIdentity = projectIdentity' "${ROOT_DIR}/apps/pi-extension/src/state.ts" "${ROOT_DIR}/apps/pi-extension/src/awareness.ts" "$TOOLS" >/dev/null; then
  echo "✓ PASS: Utility Card consumes cached ProjectIdentity summary facts"
else
  echo "✗ FAIL: Utility Card does not consume ProjectIdentity summary facts" >&2
  exit 1
fi

if rg -n 'rememberedProjectRootResolution\(cwdInput|same-tree hint|!cwd\.startsWith\(`\$\{remembered\}/`\)' "${ROOT_DIR}/apps/pi-extension/src/state.ts" >/dev/null \
  && ! rg -n 'projectSummary = projectIdentity\.project_summary \|\| S\.lastProjectIdentity' "${ROOT_DIR}/apps/pi-extension/src/awareness.ts" >/dev/null; then
  echo "✓ PASS: remembered ProjectIdentity cache cannot pull other sessions/projects"
else
  echo "✗ FAIL: remembered ProjectIdentity cache may bleed across sessions/projects" >&2
  exit 1
fi

if rg -n 'Agent instruction: infer from cwd/git/beads/repo context first|No modal/select/input UI|Focusa inferred project roots' "${ROOT_DIR}/apps/pi-extension/src/session.ts" "${ROOT_DIR}/apps/pi-extension/src/awareness.ts" >/dev/null \
  && ! rg -n 'ctx\.ui\.input\("Confirm Focusa project_root"|Ask operator in chat now|Focusa inferred possible project roots' "${ROOT_DIR}/apps/pi-extension/src/session.ts" >/dev/null; then
  echo "✓ PASS: project root vital prompt is inference-first and avoids input-only modal"
else
  echo "✗ FAIL: project root vital prompt still relies on input-only operator entry" >&2
  exit 1
fi

if rg -n 'ProjectIdentity|quorum|canonical=false|Backed by `GET /v1/project/identity`|Backed by `POST /v1/project/verify`' "$DOC1" "$DOC2" >/dev/null; then
  echo "✓ PASS: ProjectIdentity docs describe quorum/degraded recovery"
else
  echo "✗ FAIL: ProjectIdentity docs missing quorum/degraded recovery" >&2
  exit 1
fi

echo "SPEC96 ProjectIdentity quorum static test: PASS"
