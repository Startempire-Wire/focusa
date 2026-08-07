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

if operation_id == "focusa.mission_canvas.rich_host.resolve":
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
