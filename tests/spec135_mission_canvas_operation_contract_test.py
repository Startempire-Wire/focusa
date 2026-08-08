#!/usr/bin/env python3
"""Contract gate for one generated Mission Canvas operation.

The operation packet is selected at runtime.  Keep the common generated
contract assertions here and put operation-specific authority assertions next
to the owning core/API seam instead of making each Desktop task inherit a
previous task's fixtures.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs/contracts/spec135/mission-canvas-v1/operation-registry.json"
OPENAPI = ROOT / "docs/contracts/spec135/mission-canvas-v1/openapi-3.0.3.json"
CLIENT = ROOT / "docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated.ts"
TRANSPORT = ROOT / "apps/desktop/src/lib/mission-canvas/http-transport.ts"
ROUTE = ROOT / "crates/focusa-api/src/routes/mission_canvas.rs"
CONSUMER = ROOT / "apps/desktop/tests/operation-consumer-runtime.mjs"
BUNDLE = ROOT / "schemas/spec135/mission-canvas/composition-bundle.v1.schema.json"

parser = argparse.ArgumentParser()
parser.add_argument("--operation", required=True)
args = parser.parse_args()
operation_id = args.operation

registry = json.loads(REGISTRY.read_text())
operation = next(item for item in registry["operations"] if item["operation_id"] == operation_id)
openapi = json.loads(OPENAPI.read_text())
route = openapi["paths"][operation["path"]][operation["method"].lower()]
client = CLIENT.read_text()
transport = TRANSPORT.read_text()
route_text = ROUTE.read_text()
consumer = CONSUMER.read_text()

assert operation["schema"] == "focusa.mission_canvas.operation_descriptor.v1"
assert operation["operation_version"] == "1.0.0"
assert operation["availability"] == "available"
assert operation["scope_required"] == ["workstream"]
assert operation["authority_chain"] == [
    "scope_ref", "project_root_key", "workstream_id", "continuity_id",
    "attachment_key", "session_id", "instance_id", "workspace_binding_id",
    "runtime_object", "work_surface_id",
]
assert route["operationId"] == operation_id
assert route["x-focusa-scope-required"] == ["workstream"]
assert operation_id in client
assert "registry.operations" in transport
assert "validateMissionCanvasContract('WorkstreamAuthorityContext'" in transport
assert "generated operation ID" not in transport  # no handwritten route catalog language
assert operation_id in consumer

if operation_id == "focusa.mission_canvas.projection.get":
    assert operation["method"] == "GET"
    assert operation["path"] == "/v1/mission-canvas/projection"
    assert operation["mode"] == "read"
    assert operation["permissions_required"] == ["mission_canvas:read"]
    assert operation["response_schema_ref"] == "ResolvedWorkspaceProjection"
    assert operation["requires_idempotency_key"] is False
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is False

    for marker in (
        "get(get_projection)",
        "require_permission",
        "query.scope()",
        "store.get_projection",
        "projection.validate_scope",
        "projection_scope_invalid",
    ):
        assert marker in route_text, marker
    for marker in (
        "authorityFromProjection",
        "validateProjectionResponse",
        "foreign_projection_scope",
        "foreign_contribution_scope",
        "stale_projection_revision",
        "stale_projection_layout_revision",
        "stale_projection_cursor",
    ):
        assert marker in transport, marker
    for marker in (
        "projectionGet",
        "ResolvedWorkspaceProjection",
        "empty exact Workstream",
        "stale projection",
    ):
        assert marker in consumer, marker
    model = (ROOT / "crates/focusa-core/src/mission_canvas/model.rs").read_text()
    assert "pub fn validate_scope" in model
    assert "validate_resolved_contribution_scope" in model

elif operation_id == "focusa.mission_canvas.projection.resolve":
    assert operation["method"] == "POST"
    assert operation["path"] == "/v1/mission-canvas/projection/resolve"
    assert operation["mode"] == "mutation"
    assert operation["permissions_required"] == ["mission_canvas:write"]
    assert operation["request_schema_ref"] == "ContributionEligibilityContext"
    assert operation["response_schema_ref"] == "ResolvedWorkspaceProjection"
    assert operation["requires_idempotency_key"] is True
    assert operation["requires_if_match_revision"] is True
    assert operation["receipt_required"] is True

    for marker in (
        "post(resolve)",
        "Json(request): Json<ProjectionResolveRequest>",
        "ContributionEligibilityContextRequest",
        "require_permission",
        "validate_authority",
        "host_renderer_workstream_context",
        "WorkstreamContext::extract",
        "previous_projection_revision",
        "previous_layout_revision",
        "projection_cursor_conflict",
        "ensure_resolver_catalog",
        "resolve_projection",
        "put_projection",
        "serde_json::to_value(result.projection)",
    ):
        assert marker in route_text, marker
    resolver = (ROOT / "crates/focusa-core/src/mission_canvas/reducer.rs").read_text()
    for marker in (
        "ResolveProjectionInput",
        "validate_scope",
        "collect_candidates",
        "resolve_eligibility",
        "resolve_layout",
        "validate_no_dead_chrome",
        "RecompositionEvidence",
        "RecompositionReceipt",
        "projection_resolved",
    ):
        assert marker in resolver, marker
    for marker in (
        "requires_if_match_revision",
        "readIfMatchRevision",
        "if_match_revision_required",
        "If-Match",
        "Idempotency-Key",
        "validateProjectionResponse",
        "foreign_projection_scope",
        "foreign_contribution_scope",
        "stale_projection_revision",
        "stale_projection_layout_revision",
        "stale_projection_cursor",
    ):
        assert marker in transport, marker
    for marker in (
        "projectionResolve",
        "ContributionEligibilityContext",
        "Core-owned direct projection",
        "If-Match/idempotency",
        "foreign authority",
        "stale revision/layout/cursor",
    ):
        assert marker in consumer, marker

elif operation_id == "focusa.mission_canvas.profile.get":
    assert operation["method"] == "GET"
    assert operation["path"] == "/v1/mission-canvas/profiles/{profile_id}"
    assert operation["mode"] == "read"
    assert operation["permissions_required"] == ["mission_canvas:read"]
    assert operation["request_schema_ref"] == "focusa.mission_canvas.profile_get.request.v1"
    assert operation["response_schema_ref"] == "WorkspaceProfile"
    assert operation["requires_idempotency_key"] is False
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is False

    for marker in (
        "get(get_profile)",
        "require_permission(&headers, \"mission_canvas:read\")",
        "let scope = query.scope()?",
        "exact_workstream_context(&scope, &headers)",
        "registered_profile",
        "WorkspaceProfileDefinition",
        "profile_not_found",
        "serde_json::to_value(profile)",
    ):
        assert marker in route_text, marker
    for marker in (
        "focusa.mission_canvas.profile.get",
        "validateOperationRequest",
        "profile_id",
        "resolvePath",
        "path_parameter_required",
        "validateResponse",
        "profile_id_mismatch",
        "invalid_response",
    ):
        assert marker in transport, marker
    for marker in (
        "profileGet",
        "exact Workstream GET",
        "direct WorkspaceProfile",
        "profile_not_found",
        "profile_id_mismatch",
        "missing:profile_id",
        "invalid_response",
        "permission",
    ):
        assert marker in consumer, marker

elif operation_id == "focusa.mission_canvas.profile.select":
    assert operation["method"] == "POST"
    assert operation["path"] == "/v1/mission-canvas/profiles/select"
    assert operation["mode"] == "mutation"
    assert operation["permissions_required"] == ["mission_canvas:write"]
    assert operation["request_schema_ref"] == "focusa.mission_canvas.composition_selection.request.v1"
    assert operation["response_schema_ref"] == "ResolvedWorkspaceProjection"
    assert operation["requires_idempotency_key"] is True
    assert operation["requires_if_match_revision"] is True
    assert operation["receipt_required"] is True

    for marker in (
        "post(select_profile)",
        "require_permission_with_state",
        "validate_authority",
        "exact_workstream_context",
        "required_header",
        "required_if_match_revision",
        "selection_candidates",
        "ProfileSelectionService",
        "put_projection",
        "serde_json::to_value(result.projection)",
        "idempotency_key_mismatch",
        "projection_cursor_conflict",
    ):
        assert marker in route_text, marker
    reducer = (ROOT / "crates/focusa-core/src/mission_canvas/reducer.rs").read_text()
    for marker in (
        "PROFILE_SELECT_OPERATION",
        "ProfileSelectionCommand",
        "ProfileSelectionService",
        "validate_profile_selection_context",
        "collect_candidates",
        "resolve_eligibility",
        "resolve_layout",
        "validate_no_dead_chrome",
        "profile_change",
        "RecompositionReceipt",
        "profile_changed",
    ):
        assert marker in reducer, marker
    for marker in (
        "validateOperationRequest",
        "selection_id",
        "If-Match",
        "Idempotency-Key",
        "validateProjectionResponse",
        "foreign_projection_scope",
        "foreign_contribution_scope",
        "stale_projection_revision",
        "stale_projection_layout_revision",
        "stale_projection_cursor",
    ):
        assert marker in transport, marker
    for marker in (
        "profileSelect",
        "exact Workstream POST",
        "Core-owned direct projection",
        "receipt",
        "foreign authority",
        "missing If-Match",
        "stale revision/layout/cursor",
        "empty contribution",
    ):
        assert marker in consumer, marker

elif operation_id == "focusa.mission_canvas.profile.list":
    assert operation["method"] == "GET"
    assert operation["path"] == "/v1/mission-canvas/profiles"
    assert operation["mode"] == "read"
    assert operation["permissions_required"] == ["mission_canvas:read"]
    assert operation["response_schema_ref"] == "WorkspaceProfile[]"
    assert operation["requires_idempotency_key"] is False
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is False

    for marker in (
        "get(list_profiles)",
        "require_permission",
        "query.scope()",
        "exact_workstream_context",
        "store.get_projection",
        "projection.validate_scope",
        "eligible_contributions",
        "meaningful_profiles_for_projection",
        "WorkspaceProfileDefinition",
        "serde_json::to_value(viable)",
    ):
        assert marker in route_text, marker
    profiles = (ROOT / "crates/focusa-core/src/mission_canvas/profiles.rs").read_text()
    for marker in (
        "pub fn meaningful_profiles_for_projection",
        "profile.installed",
        "activity_contribution_ids",
        "eligible_contribution_ids",
    ):
        assert marker in profiles, marker
    for marker in (
        "validateResponse",
        "schemaRef.endsWith('[]')",
        "expected array",
    ):
        assert marker in transport, marker
    for marker in (
        "profileList",
        "meaningful eligible profile",
        "empty profile list",
        "foreign scope",
        "missing authority",
        "permission",
    ):
        assert marker in consumer, marker

elif operation_id == "focusa.mission_canvas.activity.list":
    assert operation["method"] == "GET"
    assert operation["path"] == "/v1/mission-canvas/activities"
    assert operation["mode"] == "read"
    assert operation["permissions_required"] == ["mission_canvas:read"]
    assert operation["request_schema_ref"] == "focusa.mission_canvas.activity_list.request.v1"
    assert operation["response_schema_ref"] == "ActivityMode[]"
    assert operation["requires_idempotency_key"] is False
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is False

    for marker in (
        "get(list_activities)",
        "require_permission",
        "exact_workstream_context",
        "store.get_projection",
        "projection.validate_scope",
        "registered_activity_modes",
        "meaningful_activities_for_projection",
        "ActivityModeDefinition",
        "serde_json::to_value(viable)",
    ):
        assert marker in route_text, marker
    profiles = (ROOT / "crates/focusa-core/src/mission_canvas/profiles.rs").read_text()
    for marker in (
        "pub fn meaningful_activities_for_projection",
        "activity.activity_mode_id",
        "profile_contribution_ids",
        "eligible_contribution_ids",
        "deny_unknown_fields",
    ):
        assert marker in profiles, marker
    for marker in (
        "validateResponse",
        "schemaRef.endsWith('[]')",
        "expected array",
    ):
        assert marker in transport, marker
    for marker in (
        "activityList",
        "exact Workstream GET",
        "registered ActivityMode",
        "empty activity list",
        "foreign scope",
        "missing authority",
        "permission",
    ):
        assert marker in consumer, marker

elif operation_id == "focusa.mission_canvas.activity.select":
    assert operation["method"] == "POST"
    assert operation["path"] == "/v1/mission-canvas/activities/select"
    assert operation["mode"] == "mutation"
    assert operation["permissions_required"] == ["mission_canvas:write"]
    assert operation["request_schema_ref"] == "focusa.mission_canvas.composition_selection.request.v1"
    assert operation["response_schema_ref"] == "ResolvedWorkspaceProjection"
    assert operation["requires_idempotency_key"] is True
    assert operation["requires_if_match_revision"] is True
    assert operation["receipt_required"] is True

    for marker in (
        "post(select_activity)",
        "require_permission_with_state",
        "validate_authority",
        "exact_workstream_context",
        "required_header",
        "required_if_match_revision",
        "activity_for_selection",
        "ActivitySelectionService",
        "put_projection",
        "serde_json::to_value(result.projection)",
        "idempotency_key_mismatch",
        "projection_cursor_conflict",
        "activity_selection_error",
    ):
        assert marker in route_text, marker
    reducer = (ROOT / "crates/focusa-core/src/mission_canvas/reducer.rs").read_text()
    for marker in (
        "ACTIVITY_SELECT_OPERATION",
        "ActivitySelectionCommand",
        "ActivitySelectionError",
        "ActivitySelectionService",
        "validate_activity_selection_context",
        "collect_candidates",
        "resolve_eligibility",
        "resolve_layout",
        "validate_no_dead_chrome",
        "activity_mode_change",
        "activity_mode_changed",
        "RecompositionReceipt",
    ):
        assert marker in reducer, marker
    for marker in (
        "focusa.mission_canvas.activity.select",
        "validateOperationRequest",
        "selection_id",
        "If-Match",
        "Idempotency-Key",
        "unknown:${field}",
        "validateProjectionResponse",
        "foreign_projection_scope",
        "foreign_contribution_scope",
        "stale_projection_revision",
        "stale_projection_layout_revision",
        "stale_projection_cursor",
    ):
        assert marker in transport, marker
    for marker in (
        "activitySelect",
        "exact Workstream POST",
        "Core-owned direct recomposition",
        "trusted recursive projection data",
        "receipt",
        "foreign authority",
        "missing If-Match",
        "stale revision/layout/cursor",
        "empty omission",
    ):
        assert marker in consumer, marker

elif operation_id == "focusa.mission_canvas.registry.list":
    assert operation["method"] == "GET"
    assert operation["path"] == "/v1/mission-canvas/registries/{registry_kind}"
    assert operation["mode"] == "read"
    assert operation["permissions_required"] == ["mission_canvas:read"]
    assert operation["request_schema_ref"] == "focusa.mission_canvas.registry_list.request.v1"
    assert operation["response_schema_ref"] == "RegistryEntry[]"
    assert operation["requires_idempotency_key"] is False
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is False

    for marker in (
        "get(list_registry)",
        "require_permission(&headers, \"mission_canvas:read\")",
        "let scope = query.scope()?",
        "exact_workstream_context(&scope, &headers)",
        "WorkspaceProfileRegistry",
        "registered_registry_entries",
        "registry_catalog_invalid",
        "registry_catalog_identity_mismatch",
        "registry_kind_unknown",
    ):
        assert marker in route_text, marker
    for marker in (
        "path_parameter_required",
        "schemaRef.endsWith('[]')",
        "validateResponse",
        "expected array",
    ):
        assert marker in transport, marker
    for marker in (
        "registryList",
        "exact Workstream GET",
        "path param",
        "direct RegistryEntry[]",
        "empty registry list",
        "missing path",
        "missing authority",
        "permission",
    ):
        assert marker in consumer, marker

elif operation_id == "focusa.mission_canvas.rich_host.resolve":
    assert operation["method"] == "GET"
    assert operation["path"] == "/v1/mission-canvas/rich-host/resolution"
    assert operation["mode"] == "read"
    assert operation["permissions_required"] == ["mission_canvas:host"]
    assert operation["response_schema_ref"] == "HostRendererResolution"
    assert operation["requires_idempotency_key"] is False
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is False

    host_schema = json.loads(BUNDLE.read_text())["$defs"]["HostRendererResolution"]
    renderer_enum = host_schema["properties"]["selected_renderer"]["enum"]
    assert "focusa_desktop_tauri" in renderer_enum
    assert "pi_terminal_projection" in renderer_enum
    assert "workstream" in host_schema["properties"]

    for marker in (
        "get(resolve_host_renderer)",
        "require_permission_with_state",
        "host_renderer_workstream_context",
        "WorkstreamContext::extract",
        "MissingActor",
        "MissingAuthority",
        "header_values(&headers, \"x-focusa-capabilities\")",
        "HostRendererResolutionService",
        "HostPlatform::current()",
        "HostRendererResolutionError",
    ):
        assert marker in route_text, marker
    for marker in (
        "RICH_HOST_RESOLVE_OPERATION",
        "RICH_HOST_RESOLVE_CAPABILITY",
        "DESKTOP_TAURI_RENDERER",
        "PI_OVERLAY_RENDERER",
        "focusa_pi_rich_window",
        "Focusa Desktop Tauri is the primary Mission Canvas host",
        "HostRendererResolutionService",
        "CapabilityUnavailable",
        "WorkstreamMismatch",
    ):
        assert marker in (ROOT / "crates/focusa-core/src/mission_canvas/host.rs").read_text(), marker
    host_text = (ROOT / "crates/focusa-core/src/mission_canvas/host.rs").read_text()
    for marker in (
        "sameWorkstreamAuthorityContext",
        "authorityFromResolution",
        "foreign_resolution_scope",
        "focusa_desktop_tauri",
    ):
        assert marker in consumer or marker in transport, marker
    assert "Pi overlay is a compatibility-only fallback" in host_text

elif operation_id == "focusa.mission_canvas.rich_host.launch":
    assert operation["method"] == "POST"
    assert operation["path"] == "/v1/mission-canvas/rich-host/launch"
    assert operation["mode"] == "mutation"
    assert operation["permissions_required"] == ["mission_canvas:host"]
    assert operation["response_schema_ref"] == "HostLifecycleState"
    assert operation["request_schema_ref"] == "focusa.mission_canvas.rich_host_command.v1"
    assert operation["requires_idempotency_key"] is True
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is True

    lifecycle_schema = json.loads(BUNDLE.read_text())["$defs"]["HostLifecycleState"]
    assert "workstream" in lifecycle_schema["properties"]
    assert "renderer_resolution" in lifecycle_schema["properties"]
    assert "host_instance_id" in lifecycle_schema["required"]
    for marker in (
        "post(launch_host)",
        "RichHostCommandRequest",
        "require_permission_with_state",
        "host_renderer_workstream_context",
        "HostLifecycleLaunchCommand",
        "HostLifecycleService",
        "HostPlatform::current()",
        "x-focusa-capabilities",
        "idempotency_key",
        "HostLifecycleState",
    ):
        assert marker in route_text, marker
    host_text = (ROOT / "crates/focusa-core/src/mission_canvas/host.rs").read_text()
    for marker in (
        "RICH_HOST_LAUNCH_OPERATION",
        "RICH_HOST_PERMISSION",
        "HostLifecycleLaunchCommand",
        "HostLifecycleService",
        "put_idempotent_lifecycle_document",
        "HostLifecycleState",
        "without forking Pi",
        "host_launched",
        "host_lifecycle_scope_mismatch",
    ):
        assert marker in host_text, marker
    for marker in (
        "authorityFromLifecycleState",
        "sameWorkstreamAuthorityContext",
        "foreign_lifecycle_scope",
        "stale_lifecycle_revision",
        "stale_lifecycle_cursor",
        "HostLifecycleState",
    ):
        assert marker in transport, marker
    assert "mission_canvas_host_lifecycle" in (ROOT / "crates/focusa-core/src/mission_canvas/persistence.rs").read_text()
    for marker in (
        "rich_hostLaunch",
        "POST",
        "idempotency_key",
        "foreign lifecycle",
        "stale lifecycle",
    ):
        assert marker in consumer, marker
    assert "fork(" not in transport
    assert "child_process" not in transport

elif operation_id == "focusa.mission_canvas.rich_host.focus":
    assert operation["method"] == "POST"
    assert operation["path"] == "/v1/mission-canvas/rich-host/focus"
    assert operation["mode"] == "mutation"
    assert operation["permissions_required"] == ["mission_canvas:host"]
    assert operation["response_schema_ref"] == "HostLifecycleState"
    assert operation["request_schema_ref"] == "focusa.mission_canvas.rich_host_command.v1"
    assert operation["requires_idempotency_key"] is True
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is True

    lifecycle_schema = json.loads(BUNDLE.read_text())["$defs"]["HostLifecycleState"]
    assert "workstream" in lifecycle_schema["properties"]
    assert "renderer_resolution" in lifecycle_schema["properties"]
    for marker in (
        "post(focus_host)",
        "RichHostCommandRequest",
        "require_permission_with_state",
        "host_renderer_workstream_context",
        "HostLifecycleFocusCommand",
        "HostLifecycleService",
        "HostPlatform::current()",
        "x-focusa-capabilities",
        "idempotency_key",
        "HostLifecycleState",
    ):
        assert marker in route_text, marker
    host_text = (ROOT / "crates/focusa-core/src/mission_canvas/host.rs").read_text()
    for marker in (
        "RICH_HOST_FOCUS_OPERATION",
        "HostLifecycleFocusCommand",
        "pub fn focus(",
        "put_idempotent_lifecycle_document",
        "host_focused",
        "PresentationNotFound",
        "canonical_activity_changed",
        "projection_revision: 0",
        "without changing canonical activity",
    ):
        assert marker in host_text, marker
    for marker in (
        "authorityFromLifecycleState",
        "sameWorkstreamAuthorityContext",
        "foreign_lifecycle_scope",
        "stale_lifecycle_revision",
        "stale_lifecycle_cursor",
        "invalid_response:focus_state",
        "invalid_response:focus_renderer",
        "HostLifecycleState",
    ):
        assert marker in transport, marker
    assert "mission_canvas_host_lifecycle" in (ROOT / "crates/focusa-core/src/mission_canvas/persistence.rs").read_text()
    for marker in (
        "rich_hostFocus",
        "POST",
        "idempotency_key",
        "foreign focus",
        "stale focus",
        "canonical activity",
    ):
        assert marker in consumer, marker
    assert "focusa.mission_canvas.rich_host.focus" in client
    assert "fork(" not in transport
    assert "child_process" not in transport

elif operation_id == "focusa.mission_canvas.rich_host.hide":
    assert operation["method"] == "POST"
    assert operation["path"] == "/v1/mission-canvas/rich-host/hide"
    assert operation["mode"] == "mutation"
    assert operation["permissions_required"] == ["mission_canvas:host"]
    assert operation["response_schema_ref"] == "HostLifecycleState"
    assert operation["request_schema_ref"] == "focusa.mission_canvas.rich_host_command.v1"
    assert operation["requires_idempotency_key"] is True
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is True

    lifecycle_schema = json.loads(BUNDLE.read_text())["$defs"]["HostLifecycleState"]
    assert "workstream" in lifecycle_schema["properties"]
    assert "renderer_resolution" in lifecycle_schema["properties"]
    for marker in (
        "post(hide_host)",
        "RichHostCommandRequest",
        "require_permission_with_state",
        "host_renderer_workstream_context",
        "HostLifecycleHideCommand",
        "HostLifecycleService",
        "HostPlatform::current()",
        "x-focusa-capabilities",
        "idempotency_key",
        "HostLifecycleState",
    ):
        assert marker in route_text, marker
    host_text = (ROOT / "crates/focusa-core/src/mission_canvas/host.rs").read_text()
    for marker in (
        "RICH_HOST_HIDE_OPERATION",
        "HostLifecycleHideCommand",
        "pub fn hide(",
        "host_hidden",
        "projection_revision: 0",
        "PresentationNotFound",
    ):
        assert marker in host_text, marker
    for marker in (
        "authorityFromLifecycleState",
        "sameWorkstreamAuthorityContext",
        "foreign_lifecycle_scope",
        "stale_lifecycle_revision",
        "stale_lifecycle_cursor",
        "invalid_response:hide_state",
        "invalid_response:hide_renderer",
        "HostLifecycleState",
    ):
        assert marker in transport, marker
    assert "focusa.mission_canvas.rich_host.hide" in client
    assert "mission_canvas_host_lifecycle" in (ROOT / "crates/focusa-core/src/mission_canvas/persistence.rs").read_text()
    for marker in (
        "rich_hostHide",
        "POST",
        "idempotency_key",
        "foreign lifecycle",
        "stale lifecycle",
    ):
        assert marker in consumer, marker
    assert "fork(" not in transport
    assert "child_process" not in transport

elif operation_id == "focusa.mission_canvas.layout_memory.get":
    assert operation["method"] == "GET"
    assert operation["path"] == "/v1/mission-canvas/layout-memory"
    assert operation["mode"] == "read"
    assert operation["permissions_required"] == ["mission_canvas:read"]
    assert operation["request_schema_ref"] == "focusa.mission_canvas.layout_memory_get.request.v1"
    assert operation["response_schema_ref"] == "ProfileLayoutMemory"
    assert operation["requires_idempotency_key"] is False
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is False

    for marker in (
        "get(get_layout_memory)",
        "require_permission(&headers, \"mission_canvas:read\")",
        "let scope = query.scope()?",
        "validate_authority(&scope)",
        "exact_workstream_context(&scope, &headers)",
        "profile_id",
        "activity_mode_id",
        "viewport_class",
        "get_document(\"mission_canvas_layout_memory\"",
        "validate_profile_layout_memory",
        "ProfileLayoutMemory",
        "layout_memory_not_found",
        "serde_json::to_value(memory)",
    ):
        assert marker in route_text, marker
    memory = (ROOT / "crates/focusa-core/src/mission_canvas/memory.rs").read_text()
    for marker in (
        "pub fn validate_profile_layout_memory",
        "memory.scope",
        "profile_mismatch",
        "activity_mode_mismatch",
        "viewport_class_mismatch",
        "memory_id_mismatch",
        "placement_duplicate",
    ):
        assert marker in memory, marker
    for marker in (
        "focusa.mission_canvas.layout_memory.get",
        "validateOperationRequest",
        "ProfileLayoutMemory",
        "validateProfileLayoutMemoryResponse",
        "foreign_profile_memory_scope",
        "foreign_profile_memory_profile_id",
        "stale_profile_memory_revision",
        "invalid_response:memory_id_mismatch",
    ):
        assert marker in transport, marker
    for marker in (
        "layout_memoryGet",
        "exact Workstream/profile GET",
        "direct ProfileLayoutMemory",
        "foreign authority/profile",
        "stale memory revision",
        "missing selectors",
        "no local composition",
    ):
        assert marker in consumer, marker

elif operation_id == "focusa.mission_canvas.layout_memory.update":
    assert operation["method"] == "POST"
    assert operation["path"] == "/v1/mission-canvas/layout-memory"
    assert operation["mode"] == "mutation"
    assert operation["permissions_required"] == ["mission_canvas:write"]
    assert operation["request_schema_ref"] == "ProfileLayoutMemory"
    assert operation["response_schema_ref"] == "RecompositionReceipt"
    assert operation["requires_idempotency_key"] is True
    assert operation["requires_if_match_revision"] is True
    assert operation["receipt_required"] is True

    for marker in (
        "post(put_layout_memory)",
        "Json(memory): Json<ProfileLayoutMemory>",
        "require_permission_with_state",
        "validate_authority",
        "exact_workstream_context",
        "required_header",
        "required_if_match_revision",
        "LayoutMemoryUpdateCommand",
        "LayoutMemoryUpdateService",
        "idempotency_key_mismatch",
        "serde_json::to_value(receipt)",
    ):
        assert marker in route_text, marker
    memory = (ROOT / "crates/focusa-core/src/mission_canvas/memory.rs").read_text()
    for marker in (
        "LAYOUT_MEMORY_UPDATE_OPERATION",
        "LAYOUT_MEMORY_UPDATE_PERMISSION",
        "LayoutMemoryUpdateCommand",
        "LayoutMemoryUpdateService",
        "validate_layout_memory_update_command",
        "validate_profile_layout_memory",
        "expected_memory_revision",
        "update_layout_memory",
        "RecompositionReceipt",
    ):
        assert marker in memory, marker
    persistence = (ROOT / "crates/focusa-core/src/mission_canvas/persistence.rs").read_text()
    for marker in (
        "pub fn update_layout_memory",
        "LayoutMemoryIdempotencyConflict",
        "request_digest",
        "causation_id",
        "preference_change",
        "transaction.commit()",
        "receipt",
        "mission_canvas_layout_memory",
    ):
        assert marker in persistence, marker
    assert "layout_memoryUpdate" in client
    for marker in (
        "focusa.mission_canvas.layout_memory.update",
        "readIfMatchRevision",
        "validateLayoutMemoryUpdateReceipt",
        "memory_revision",
        "stale_layout_memory_revision",
        "stale_layout_memory_cursor",
        "foreign_layout_memory_receipt_scope",
        "invalid_response:idempotency_key_mismatch",
        "invalid_response:layout_memory_revision_mismatch",
    ):
        assert marker in transport, marker
    for marker in (
        "layout_memoryUpdate",
        "exact Workstream POST",
        "direct RecompositionReceipt",
        "Idempotency-Key",
        "If-Match",
        "foreign receipt authority",
        "stale memory revision/cursor",
        "empty/ineligible memory",
        "no local composition",
    ):
        assert marker in consumer, marker

elif operation_id == "focusa.mission_canvas.domain_pack.install":
    assert operation["method"] == "POST"
    assert operation["path"] == "/v1/mission-canvas/domain-packs/install"
    assert operation["confirmation"] == "explicit"
    assert operation["permissions_required"] == ["mission_canvas:write"]
    assert operation["requires_idempotency_key"] is True
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is True
    assert operation["response_schema_ref"] == "DomainPackInstallReceipt"
    for marker in (
        "post(install_domain_pack)",
        "require_permission_with_state",
        "validate_authority",
        "DomainPackInstallCommand",
        "DomainPackInstallService",
        "WorkstreamContext::extract",
        "x-focusa-capabilities",
        "MissingAuthority",
        "confirmation",
        "idempotency_key",
    ):
        assert marker in route_text, marker
    service = (ROOT / "crates/focusa-core/src/mission_canvas/domain_pack.rs").read_text()
    for marker in (
        "DOMAIN_PACK_INSTALL_OPERATION",
        "DOMAIN_PACK_INSTALL_CAPABILITY",
        "DOMAIN_PACK_INSTALL_PERMISSION",
        "WorkstreamContext",
        "ConfirmationRequired",
        "CapabilityUnavailable",
        "PermissionDenied",
        "request_digest",
        "install_domain_pack(",
    ):
        assert marker in service, marker
    persistence = (ROOT / "crates/focusa-core/src/mission_canvas/persistence.rs").read_text()
    for marker in (
        "mission_canvas_domain_pack_installations",
        "DomainPackIdempotencyConflict",
        "transaction.commit()",
        "receipt_json",
        "receipt_refs",
    ):
        assert marker in persistence, marker
elif operation_id == "focusa.mission_canvas.events.stream":
    assert operation["method"] == "GET"
    assert operation["path"] == "/v1/mission-canvas/events"
    assert operation["mode"] == "stream"
    assert operation["permissions_required"] == ["mission_canvas:read"]
    assert operation["requires_idempotency_key"] is False
    assert operation["requires_if_match_revision"] is False
    assert operation["receipt_required"] is False
    assert operation["response_schema_ref"] == "ProjectionLifecycleEvent[]"
    for marker in (
        "after_cursor",
        "Last-Event-ID",
        "events_after",
        "latest_event_sequence",
        "projection_lifecycle_event",
        "event_cursor",
        "require_permission",
        "Value::Array",
    ):
        assert marker in route_text, marker
    for marker in (
        "after_cursor",
        "ProjectionLifecycleEvent[]",
        "foreign_event_scope",
        "event_cursor_regressed",
        "projection_revision_regressed",
    ):
        assert marker in transport or marker in consumer, marker

else:
    # The generated operation is still covered by common registry/transport
    # parity; operation-specific behavior belongs to its own packet branch.
    assert operation["response_schema_ref"]

print(f"Spec 135 Mission Canvas operation contract: PASS ({operation_id})")
