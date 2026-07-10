#!/usr/bin/env bash
# Trajectory audit phase 1 inventory guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/130-focusa-trajectory-audit-inventory.md"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$DOC" ] || fail "trajectory inventory doc missing"

for section in \
  'Public Trajectory surfaces' \
  'Request shapes' \
  'Internal data structures' \
  'intelligence_view fields' \
  'Active gap / gap reasons' \
  'clarity_gate.blocking_reasons' \
  'Current inventory observations'; do
  grep -q "$section" "$DOC" || fail "missing section: $section"
done

for surface in 'View' 'Define goal' 'Assess' 'Propose Workpoint' 'Checkpoint' 'Resume'; do
  grep -q "$surface" "$DOC" || fail "missing surface: $surface"
done

for structure in 'TrajectoryProjectionRecord' 'WorkpointRecord' 'FrameRecord'; do
  grep -q "$structure" "$DOC" || fail "missing structure: $structure"
done

for field in \
  'context_sufficiency.score' \
  'trajectory_workpoint_reconciliation' \
  'focus_trajectory_sync' \
  'next_workpoint_candidate' \
  'tool_affordances'; do
  grep -q "$field" "$DOC" || fail "missing intelligence_view field: $field"
done

for reason in \
  'Trajectory definition required before ladder projection' \
  'Trajectory gap unclear until desired end state and current verified state are both present' \
  'conflicting_project_or_continuity_scope' \
  'stale_or_missing_evidence_refs' \
  'agent_runtime_directory'; do
  grep -q "$reason" "$DOC" || fail "missing gap/clarity reason: $reason"
done

pass "Trajectory audit inventory static guard passed"
