#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FOCUSA_SKILL="${ROOT_DIR}/apps/pi-extension/skills/focusa/SKILL.md"
WORKPOINT_SKILL="${ROOT_DIR}/apps/pi-extension/skills/focusa-workpoint/SKILL.md"
TROUBLE_SKILL="${ROOT_DIR}/apps/pi-extension/skills/focusa-troubleshooting/SKILL.md"
CONSTRAINT_DOC="${ROOT_DIR}/docs/focusa-tools/tools/focusa_constraint.md"
RESUME_DOC="${ROOT_DIR}/docs/focusa-tools/tools/focusa_workpoint_resume.md"
QUICKSTART="${ROOT_DIR}/docs/current/AGENT_AWARENESS_QUICKSTART.md"
SCOPE_GUARD="${ROOT_DIR}/docs/current/WORKPOINT_SESSION_SCOPE_GUARD.md"

if rg -n "declarative architecture boundaries|retry once with compliant noun-phrase|focusa_scratch" "$FOCUSA_SKILL" "$CONSTRAINT_DOC" "$TROUBLE_SKILL" >/dev/null; then
  echo "✓ PASS: model-facing Focus State instructions clarify validation rejection recovery"
else
  echo "✗ FAIL: Focus State validation recovery instructions missing" >&2
  exit 1
fi

if rg -n "project_root.*continuity_id|continuity_id.*stable logical session|session_id.*temporal metadata|Trajectory.*corroborating|goals.*corroborating" "$FOCUSA_SKILL" "$WORKPOINT_SKILL" "$RESUME_DOC" "$QUICKSTART" "$SCOPE_GUARD" >/dev/null; then
  echo "✓ PASS: model-facing instructions clarify strong identity axes and corroborating signals"
else
  echo "✗ FAIL: identity-axis instructions missing" >&2
  exit 1
fi

if rg -n "project/session mismatch|project_root.*only.*continuity boundary" "$FOCUSA_SKILL" "$WORKPOINT_SKILL" "$TROUBLE_SKILL" "$CONSTRAINT_DOC" "$RESUME_DOC" "$QUICKSTART" "$SCOPE_GUARD" >/dev/null; then
  echo "✗ FAIL: ambiguous or project-root-only identity wording remains in model-facing instructions" >&2
  exit 1
else
  echo "✓ PASS: ambiguous/project-root-only identity wording removed from model-facing instructions"
fi

echo "SPEC96 model tool instruction static test: PASS"
