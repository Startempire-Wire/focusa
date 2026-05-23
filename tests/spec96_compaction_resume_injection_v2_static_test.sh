#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
COMPACTION="${ROOT_DIR}/apps/pi-extension/src/compaction.ts"
STATE_TS="${ROOT_DIR}/apps/pi-extension/src/state.ts"
SESSION_TS="${ROOT_DIR}/apps/pi-extension/src/session.ts"

if rg -n 'formatResumePacketV2ForPrompt|WorkpointResumePacketV2|SCHEMA_VERSION|resume_packet_v2' "$COMPACTION" >/dev/null; then
  echo "✓ PASS: compaction injection renders Workpoint Resume Packet v2"
else
  echo "✗ FAIL: Workpoint Resume Packet v2 injection missing" >&2
  exit 1
fi

if rg -n 'normalizeWorkpointResumePacketEnvelope' "$STATE_TS" "$COMPACTION" "$SESSION_TS" >/dev/null && rg -n 'formatResumePacketV2ForPrompt\(scopedPacket\)' "$COMPACTION" >/dev/null; then
  echo "✓ PASS: resume_packet_v2 is preserved on project-bound Workpoint packets before prompt rendering"
else
  echo "✗ FAIL: resume_packet_v2 is not preserved/rendered from scoped packet" >&2
  exit 1
fi

if rg -n 'JSON\.stringify\(scopedPacket|scopedPacket \? S\.activeWorkpointSummary|WORKPOINT active: mission=\$\{scopedPacket\.mission\}' "$COMPACTION" >/dev/null; then
  echo "✗ FAIL: compaction auto-resume can still inject v1/raw unbound Workpoint packet fallback" >&2
  exit 1
else
  echo "✓ PASS: compaction auto-resume omits v1/raw Workpoint packet fallback"
fi

if rg -n 'focusa_workpoint_resume|focusa_trajectory_view|focusa_traverse|focusa_active_object_resolve|focusa_tool_doctor' "$COMPACTION" >/dev/null; then
  echo "✓ PASS: corrected resume tool order is injected"
else
  echo "✗ FAIL: corrected resume tool order missing" >&2
  exit 1
fi

if rg -n 'function semanticCurrentAsk' "$COMPACTION" >/dev/null \
  && rg -n 'isExplicitContinuationAsk\(text\)|isNonTaskStatusLikeText\(text\)' "$COMPACTION" >/dev/null \
  && rg -n 'const ask = semanticCurrentAsk\(\)' "$COMPACTION" >/dev/null; then
  echo "✓ PASS: generic continuation asks cannot overwrite compaction mission/next_slice fallback"
else
  echo "✗ FAIL: generic continuation asks may still overwrite compaction fallback mission/next_slice" >&2
  exit 1
fi

if rg -n 'DO_NOT_USE_BY_DEFAULT|transcript tail as authority|full lineage tree|full ontology graph|deep work-loop status' "$COMPACTION" >/dev/null; then
  echo "✓ PASS: unsafe default reads are excluded from resume prompt"
else
  echo "✗ FAIL: do_not_use defaults missing" >&2
  exit 1
fi

if rg -n 'canonical=true|canonical=false|project_root\+continuity_id|trajectory similarity is advisory grouping only|Never use transcript tail as authority' "$COMPACTION" >/dev/null; then
  echo "✓ PASS: canonical/degraded authority semantics are explicit"
else
  echo "✗ FAIL: canonical/degraded authority semantics missing" >&2
  exit 1
fi

if rg -n 'work-loop/status|ontology/graph|lineage/tree\?include_full_payload|include_full_payload=true' "$COMPACTION" >/dev/null; then
  echo "✗ FAIL: compaction injection references unsafe full/default reads" >&2
  exit 1
else
  echo "✓ PASS: compaction injection avoids full lineage/work-loop/ontology defaults"
fi

echo "SPEC96 compaction resume injection v2 static test: PASS"
