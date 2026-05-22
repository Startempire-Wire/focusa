#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORKPOINT="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"
DOC="${ROOT_DIR}/docs/focusa-tools/tools/focusa_workpoint_resume.md"
SPEC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if rg -n 'workpoint_resume_packet_v2|focusa\.workpoint_resume_packet\.v2|resume_packet_v2|schema_version' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: Workpoint resume packet v2 renderer is wired"
else
  echo "✗ FAIL: Workpoint v2 renderer missing" >&2
  exit 1
fi

if rg -n 'packet_id|generated_at|resume_source|degraded|confidence|project_identity|session_identity|rendered_summary|resume_summary|trajectory|traversal_slices|tool_affordances|api_provenance|next_tools|failure_class|freshness|details.*tool_result_v1|retry|side_effects|evidence_refs|rehydrate_refs|tool_or_route' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: v2 packet includes full schema fields, freshness, traversal refs, and tool_result_v1"
else
  echo "✗ FAIL: v2 packet full schema fields/tool_result_v1 missing" >&2
  exit 1
fi

if rg -n 'project_root plus continuity_id|similarity.*advisory|must_not_merge_on_similarity|high_level_goal|mid_level_goal|low_level_goal' "$WORKPOINT" "$DOC" "$SPEC" >/dev/null; then
  echo "✓ PASS: v2 packet encodes trajectory grouping without session confusion"
else
  echo "✗ FAIL: trajectory hierarchy/session authority guard missing" >&2
  exit 1
fi

if rg -n 'resume_packet_v2_contains_trajectory_traverse_and_provenance' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: v2 renderer unit coverage exists"
else
  echo "✗ FAIL: v2 renderer unit test missing" >&2
  exit 1
fi

echo "SPEC96 Workpoint Resume Packet v2 static test: PASS"
