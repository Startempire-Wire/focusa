#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "✗ FAIL: $1"; exit 1; }
pass(){ echo "✓ PASS: $1"; }

rg -n 'summary_only.*default_true|include_full_payload|cursor|next_cursor|bounded_metadata' \
  "$ROOT_DIR/crates/focusa-api/src/routes/ecs.rs" >/dev/null || fail "ECS handles missing bounded summary/cursor metadata"
pass "ECS handles bounded metadata present"

rg -n 'summary_only.*default_true|include_full_payload|cursor|next_cursor|bounded_metadata' \
  "$ROOT_DIR/crates/focusa-api/src/routes/memory.rs" >/dev/null || fail "semantic memory missing bounded summary/cursor metadata"
pass "semantic memory bounded metadata present"

rg -n 'cursor_objects|cursor_links|category_rehydrate|object_type_counts|link_type_counts|include_action_catalog|include_working_sets' \
  "$ROOT_DIR/crates/focusa-api/src/routes/ontology.rs" >/dev/null || fail "ontology world missing category counts/rehydrate/cursors"
pass "ontology world category counts/rehydrate/cursors present"

rg -n 'summary_only|omitted_categories|rehydrate.*work-loop/status' \
  "$ROOT_DIR/crates/focusa-api/src/routes/work_loop.rs" >/dev/null || fail "work-loop status summary metadata missing"
pass "work-loop status summary metadata present"

rg -n 'next_cursor|bounds|cursor|truncated' \
  "$ROOT_DIR/crates/focusa-api/src/routes/telemetry.rs" >/dev/null || fail "telemetry events bounded cursor metadata missing"
pass "telemetry bounded cursor metadata present"

rg -n 'response_size_histograms|last_pressure_transition|peak_rss_kb|record_json_response_size' \
  "$ROOT_DIR/crates/focusa-api/src/routes/telemetry.rs" "$ROOT_DIR/crates/focusa-api/src/routes/bounded.rs" >/dev/null || fail "memory telemetry missing pressure transition/response histogram/peak RSS surfaces"
rg -n 'record_json_response_size\("/v1/(work-loop/status|events/recent|references/salient|telemetry/productivity|telemetry/autonomy|telemetry/tokens|telemetry/tools)' \
  "$ROOT_DIR/crates/focusa-api/src/routes/work_loop.rs" "$ROOT_DIR/crates/focusa-api/src/routes/events_sqlite.rs" "$ROOT_DIR/crates/focusa-api/src/routes/capabilities_extra.rs" "$ROOT_DIR/crates/focusa-api/src/routes/telemetry.rs" >/dev/null || fail "target route response histogram instrumentation missing"
pass "memory telemetry pressure transition, histogram, peak RSS, and target-route instrumentation present"

rg -n 'metacog_max_captures|metacog_max_reflections|metacog_max_adjustments|metacog_ttl_minutes|metacog_retrieve_max_k' \
  "$ROOT_DIR/crates/focusa-core/src/types.rs" >/dev/null || fail "FocusaConfig metacog fields missing"
rg -n 'FOCUSA_METACOG_MAX_CAPTURES|FOCUSA_METACOG_TTL_MINUTES|FOCUSA_METACOG_RETRIEVE_MAX_K' \
  "$ROOT_DIR/docs/current/RUNTIME_CONFIG_KEYS.md" >/dev/null || fail "documented metacog config keys missing"
pass "metacog config fields and documented env overrides present"

rg -n 'trace/batch|total_queued|flush_reason|truncated|turn_end' \
  "$ROOT_DIR/apps/pi-extension/src/turns.ts" >/dev/null || fail "Pi trace batching missing turn-end truncation metadata"
pass "Pi trace batching metadata present"

echo "SPEC94 response-size/metadata contract: PASS"


EVENTS_SQLITE_RS="${ROOT_DIR}/crates/focusa-api/src/routes/events_sqlite.rs"
if rg -n 'events_failure|events_db_failure|event_not_found|recovery_hint|misuse_hint|tool_result_v1' "$EVENTS_SQLITE_RS" >/dev/null; then
  echo "✓ PASS: SQLite event failures expose no-guess recovery contract"
else
  echo "✗ FAIL: SQLite event failures lack no-guess recovery contract" >&2
  exit 1
fi


ECS_RS="${ROOT_DIR}/crates/focusa-api/src/routes/ecs.rs"
if rg -n 'ecs_failure|ecs_validation_rejected|ecs_dispatch_failed|ecs_handle_not_found|ecs_blob_not_found|recovery_hint|misuse_hint|tool_result_v1' "$ECS_RS" >/dev/null; then
  echo "✓ PASS: ECS failures expose no-guess recovery contract"
else
  echo "✗ FAIL: ECS failures lack no-guess recovery contract" >&2
  exit 1
fi


EXTRA_RS="${ROOT_DIR}/crates/focusa-api/src/routes/capabilities_extra.rs"
if rg -n 'extra_failure|extra_dispatch_failed|extra_session_not_active|recovery_hint|misuse_hint|tool_result_v1' "$EXTRA_RS" >/dev/null; then
  echo "✓ PASS: Capabilities-extra failures expose no-guess recovery contract"
else
  echo "✗ FAIL: Capabilities-extra failures lack no-guess recovery contract" >&2
  exit 1
fi


EVENTS_LEGACY_RS="${ROOT_DIR}/crates/focusa-api/src/routes/events.rs"
if rg -n 'legacy_event_failure|legacy_event_log_read_failed|legacy_event_log_not_found|legacy_event_not_found|recovery_hint|misuse_hint|tool_result_v1' "$EVENTS_LEGACY_RS" >/dev/null; then
  echo "✓ PASS: Legacy event failures expose no-guess recovery contract"
else
  echo "✗ FAIL: Legacy event failures lack no-guess recovery contract" >&2
  exit 1
fi


if rg -n 'info_failure|telemetry_debug_disabled|training_not_found|recovery_hint|misuse_hint|tool_result_v1' "${ROOT_DIR}/crates/focusa-api/src/routes/info.rs" "${ROOT_DIR}/crates/focusa-api/src/routes/telemetry.rs" "${ROOT_DIR}/crates/focusa-api/src/routes/training.rs" >/dev/null; then
  echo "✓ PASS: Small route failures expose no-guess recovery contract"
else
  echo "✗ FAIL: Small route failures lack no-guess recovery contract" >&2
  exit 1
fi


if rg -n 'attachment_failure|attachment_invalid_uuid|attachment_dispatch_failed|gate_failure|gate_forbidden|gate_dispatch_failed|sync_failure|sync_persistence_failed|sync_peer_not_found|sync_upstream_failed|sync_validation_failed|sync_delegate_failed|recovery_hint|misuse_hint|tool_result_v1' "${ROOT_DIR}/crates/focusa-api/src/routes/attachments.rs" "${ROOT_DIR}/crates/focusa-api/src/routes/gate.rs" "${ROOT_DIR}/crates/focusa-api/src/routes/sync.rs" >/dev/null; then
  echo "✓ PASS: Raw StatusCode route failures expose no-guess recovery contract"
else
  echo "✗ FAIL: Raw StatusCode route failures lack no-guess recovery contract" >&2
  exit 1
fi
