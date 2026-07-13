#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT="$ROOT/crates/focusa-api/src/routes/project.rs"

fail() { echo "FAIL: $*" >&2; exit 1; }
require() { rg -q "$2" "$1" || fail "missing $2 in $1"; }

require "$PROJECT" 'source_scope: Option<WorkstreamKey>'
require "$PROJECT" 'target_scope: Option<WorkstreamKey>'
require "$PROJECT" 'target_continuity_id: Option<String>'
require "$PROJECT" 'focusa\.project_session_transfer\.v2'
require "$PROJECT" 'target_attachment_pending'
require "$PROJECT" 'requires_target_resume_verification'
require "$PROJECT" 'focusa\.project_session_transition_receipt\.v1'
require "$PROJECT" 'target_resume_verified'
require "$PROJECT" 'target_resume_degraded'
require "$PROJECT" 'target_workpoint_id'
require "$PROJECT" 'target\.root_scope != source_scope\.root_scope'
require "$PROJECT" 'target\.continuity_id == source_scope\.continuity_id'
require "$PROJECT" 'typed source project_root and continuity_id are required; static continuity fallback is forbidden'
require "$PROJECT" 'project_session_transfers_path\(&source_scope\.root_scope\)'
require "$PROJECT" 'project-session-transfers'
require "$PROJECT" 'source_checkpoint_id'
require "$PROJECT" 'compaction_packet_id'

if rg -q 'replace\("project-fnv1a64", "focusa-cont-project"\)' "$PROJECT"; then
  fail "session transfer still invents static continuity from project fingerprint"
fi

echo "PASS: Spec130 rotating-continuity transfer API is typed, scoped, and non-fallback"
