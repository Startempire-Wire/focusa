#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
STATE="${ROOT_DIR}/apps/pi-extension/src/state.ts"
COMPACTION="${ROOT_DIR}/apps/pi-extension/src/compaction.ts"
TURNS="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
SESSION="${ROOT_DIR}/apps/pi-extension/src/session.ts"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
FOCUS="${ROOT_DIR}/crates/focusa-api/src/routes/focus.rs"
WORKPOINT="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"
PROJECT="${ROOT_DIR}/crates/focusa-api/src/routes/project.rs"

if rg -n 'UNSAFE_PROJECT_AUTHORITY_ROOTS|"/root"|unsafe_broad_project_root|isProjectRootAuthoritySafe|projectRootAuthorityFailure' "$STATE" >/dev/null; then
  echo "✓ PASS: Pi state defines unsafe broad project roots"
else
  echo "✗ FAIL: Pi state lacks broad-root authority guard" >&2
  exit 1
fi

if rg -n 'isProjectRootAuthoritySafe\(currentProjectRoot\).*isProjectRootAuthoritySafe\(packetProjectRoot\)' "$STATE" >/dev/null; then
  echo "✓ PASS: scoped Workpoint guard rejects unsafe current/packet roots"
else
  echo "✗ FAIL: scoped Workpoint guard does not reject unsafe roots" >&2
  exit 1
fi

if rg -n 'adoptSafeScopeFromActiveWorkpoint|ensure_pi_frame_unsafe_cwd|pi_scope_recovered_from_active_workpoint|scopedQs\.set\("project_root"' "$STATE" >/dev/null; then
  echo "✓ PASS: Focus State write recovery adopts safe Workpoint scope before frame writes"
else
  echo "✗ FAIL: Focus State write recovery cannot adopt safe scope from active Workpoint" >&2
  exit 1
fi

if rg -n 'getScopedWorkpointPacket\(\)|projectRootAuthorityFailure\(S\.sessionCwd|No scoped Workpoint packet recorded' "$COMPACTION" >/dev/null; then
  echo "✓ PASS: compaction prompt uses scoped packet and unsafe-root fallback"
else
  echo "✗ FAIL: compaction prompt can still inject unscoped packet" >&2
  exit 1
fi

if rg -n 'if \(!isProjectRootAuthoritySafe\(root\)\) return \[\]' "$TURNS" >/dev/null; then
  echo "✓ PASS: Focus Slice suppresses trajectory projection for unsafe roots"
else
  echo "✗ FAIL: Focus Slice may project unsafe-root trajectory context" >&2
  exit 1
fi

if rg -n 'isProjectRootAuthoritySafe\(S\.sessionCwd|isWorkpointPacketScopedToCurrentSession\(candidate\)' "$SESSION" >/dev/null; then
  echo "✓ PASS: session resume refuses unsafe/unscoped Workpoint packets"
else
  echo "✗ FAIL: session resume can adopt unsafe/unscoped Workpoint packet" >&2
  exit 1
fi

if rg -n 'workpoint (checkpoint|resume) blocked → unsafe project_root|failure_class: "scope_mismatch"' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi Workpoint tools block unsafe broad project_root defaults"
else
  echo "✗ FAIL: Pi Workpoint tools do not block unsafe broad project roots" >&2
  exit 1
fi


if rg -n 'rejected_unsafe_project_root|active_frame_has_unsafe_project_root|resolve_scoped_frame_rejects_unsafe_global_active_when_project_requested|requested_project_root' "$FOCUS" >/dev/null; then
  echo "✓ PASS: Focus frame current/push/update rejects unsafe broad-root scope"
else
  echo "✗ FAIL: Focus frame broad-root isolation guard missing" >&2
  exit 1
fi

if rg -n 'rejected_unsafe_project_root|unsafe_checkpoint_rejection|unsafe_broad_project_root|broad_project_root_rejects_before_resume_injection' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: API Workpoint resume/checkpoint rejects unsafe broad roots"
else
  echo "✗ FAIL: API Workpoint broad-root rejection missing" >&2
  exit 1
fi

if rg -n 'unsafe_scope|broad_root_never_verifies_as_project_identity|project_root_authority' "$PROJECT" >/dev/null; then
  echo "✓ PASS: ProjectIdentity never verifies unsafe broad roots"
else
  echo "✗ FAIL: ProjectIdentity can still verify broad roots" >&2
  exit 1
fi

echo "SPEC96 broad-root scope isolation static test: PASS"
