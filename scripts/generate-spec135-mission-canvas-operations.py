#!/usr/bin/env python3
"""Generate contract-defined Spec 135 Mission Canvas operation descriptors."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/contracts/spec135/mission-canvas-v1"

CANONICAL_AUTHORITY_CHAIN = [
    "scope_ref",
    "project_root_key",
    "workstream_id",
    "continuity_id",
    "attachment_key",
    "session_id",
    "instance_id",
    "workspace_binding_id",
    "runtime_object",
    "work_surface_id",
]


def op(
    operation_id: str,
    method: str,
    path: str,
    mode: str,
    request_schema: str,
    response_schema: str,
    permissions: list[str],
    *,
    idempotency: bool = False,
    concurrency: bool = False,
    receipt: bool = False,
    confirmation: str = "none",
) -> dict[str, Any]:
    return {
        "schema": "focusa.mission_canvas.operation_descriptor.v1",
        "operation_id": operation_id,
        "operation_version": "1.0.0",
        "family": "mission_canvas",
        "method": method,
        "path": path,
        "availability": "available",
        "implementation_phase": "P03",
        "mode": mode,
        "request_schema_ref": request_schema,
        "response_schema_ref": response_schema,
        "error_schema_ref": "focusa.tool_result.v1",
        "permissions_required": permissions,
        # `workstream` is the only canonical owner required for a
        # Workstream-level operation.  Subordinate runtime/presentation fields
        # are explicit and are never authority by themselves.
        "scope_required": ["workstream"],
        "scope_optional": [
            "continuity_id",
            "attachment",
            "workspace_binding_id",
            "runtime_object",
            "work_surface_id",
        ],
        "authority_chain": CANONICAL_AUTHORITY_CHAIN,
        "requires_idempotency_key": idempotency,
        "requires_if_match_revision": concurrency,
        "receipt_required": receipt,
        "confirmation": confirmation,
        "generated_ui_eligible": True,
        "docs_ref": "docs/135k-adaptive-mission-canvas-rich-session-host-and-portable-runtime-spec.md",
    }


def operations() -> list[dict[str, Any]]:
    read = ["mission_canvas:read"]
    write = ["mission_canvas:write"]
    host = ["mission_canvas:host"]
    draft = ["mission_canvas:draft"]
    return [
        op("focusa.mission_canvas.projection.get", "GET", "/v1/mission-canvas/projection", "read", "focusa.mission_canvas.projection_get.request.v1", "ResolvedWorkspaceProjection", read),
        op("focusa.mission_canvas.projection.resolve", "POST", "/v1/mission-canvas/projection/resolve", "mutation", "ContributionEligibilityContext", "ResolvedWorkspaceProjection", write, idempotency=True, concurrency=True, receipt=True),
        op("focusa.mission_canvas.profile.list", "GET", "/v1/mission-canvas/profiles", "read", "focusa.mission_canvas.profile_list.request.v1", "WorkspaceProfile[]", read),
        op("focusa.mission_canvas.profile.select", "POST", "/v1/mission-canvas/profiles/select", "mutation", "focusa.mission_canvas.composition_selection.request.v1", "ResolvedWorkspaceProjection", write, idempotency=True, concurrency=True, receipt=True),
        op("focusa.mission_canvas.profile.get", "GET", "/v1/mission-canvas/profiles/{profile_id}", "read", "focusa.mission_canvas.profile_get.request.v1", "WorkspaceProfile", read),
        op("focusa.mission_canvas.activity.list", "GET", "/v1/mission-canvas/activities", "read", "focusa.mission_canvas.activity_list.request.v1", "ActivityMode[]", read),
        op("focusa.mission_canvas.activity.select", "POST", "/v1/mission-canvas/activities/select", "mutation", "focusa.mission_canvas.composition_selection.request.v1", "ResolvedWorkspaceProjection", write, idempotency=True, concurrency=True, receipt=True),
        op("focusa.mission_canvas.domain_pack.install", "POST", "/v1/mission-canvas/domain-packs/install", "mutation", "focusa.mission_canvas.domain_pack_install.request.v1", "DomainPackInstallReceipt", write, idempotency=True, receipt=True, confirmation="explicit"),
        op("focusa.mission_canvas.registry.list", "GET", "/v1/mission-canvas/registries/{registry_kind}", "read", "focusa.mission_canvas.registry_list.request.v1", "RegistryEntry[]", read),
        op("focusa.mission_canvas.layout_memory.get", "GET", "/v1/mission-canvas/layout-memory", "read", "focusa.mission_canvas.layout_memory_get.request.v1", "ProfileLayoutMemory", read),
        op("focusa.mission_canvas.layout_memory.update", "POST", "/v1/mission-canvas/layout-memory", "mutation", "ProfileLayoutMemory", "RecompositionReceipt", write, idempotency=True, concurrency=True, receipt=True),
        op("focusa.mission_canvas.layout.mutate", "POST", "/v1/mission-canvas/layout/mutations", "mutation", "LayoutMutationCommand", "LayoutMutationResult", write, idempotency=True, concurrency=True, receipt=True),
        op("focusa.mission_canvas.rich_host.resolve", "GET", "/v1/mission-canvas/rich-host/resolution", "read", "focusa.mission_canvas.rich_host_resolve.request.v1", "HostRendererResolution", host),
        op("focusa.mission_canvas.rich_host.launch", "POST", "/v1/mission-canvas/rich-host/launch", "mutation", "focusa.mission_canvas.rich_host_command.v1", "HostLifecycleState", host, idempotency=True, receipt=True),
        op("focusa.mission_canvas.rich_host.focus", "POST", "/v1/mission-canvas/rich-host/focus", "mutation", "focusa.mission_canvas.rich_host_command.v1", "HostLifecycleState", host, idempotency=True, receipt=True),
        op("focusa.mission_canvas.rich_host.hide", "POST", "/v1/mission-canvas/rich-host/hide", "mutation", "focusa.mission_canvas.rich_host_command.v1", "HostLifecycleState", host, idempotency=True, receipt=True),
        op("focusa.mission_canvas.rich_host.close", "POST", "/v1/mission-canvas/rich-host/close", "mutation", "focusa.mission_canvas.rich_host_command.v1", "HostLifecycleState", host, idempotency=True, receipt=True, confirmation="explicit"),
        op("focusa.mission_canvas.draft.get", "GET", "/v1/mission-canvas/drafts/{draft_id}", "read", "focusa.mission_canvas.draft_get.request.v1", "CanvasDraftState", draft),
        op("focusa.mission_canvas.draft.sync", "POST", "/v1/mission-canvas/drafts/sync", "mutation", "CanvasDraftState", "CanvasDraftState", draft, idempotency=True, concurrency=True, receipt=True),
        op("focusa.mission_canvas.recipient.resolve", "POST", "/v1/mission-canvas/recipients/resolve", "read", "focusa.mission_canvas.recipient_resolve.request.v1", "RecipientResolution", draft),
        op("focusa.mission_canvas.recomposition.evidence.get", "GET", "/v1/mission-canvas/recompositions/{revision}/evidence", "read", "focusa.mission_canvas.recomposition_get.request.v1", "RecompositionEvidence", read),
        op("focusa.mission_canvas.recomposition.receipt.get", "GET", "/v1/mission-canvas/recompositions/{revision}/receipt", "read", "focusa.mission_canvas.recomposition_get.request.v1", "RecompositionReceipt", read),
        op("focusa.mission_canvas.recomposition.diagnostics.list", "GET", "/v1/mission-canvas/recompositions/{revision}/diagnostics", "read", "focusa.mission_canvas.recomposition_get.request.v1", "OmissionDiagnostic[]", read),
        op("focusa.mission_canvas.pi_session.event.append", "POST", "/v1/mission-canvas/pi-session/events", "mutation", "focusa.mission_canvas.pi_session_event_append.request.v1", "PiSessionEventReceipt", write, idempotency=True, receipt=True),
        op("focusa.mission_canvas.events.stream", "GET", "/v1/mission-canvas/events", "stream", "focusa.mission_canvas.events.request.v1", "ProjectionLifecycleEvent[]", read),
    ]


def registry() -> dict[str, Any]:
    values = operations()
    return {
        "schema": "focusa.mission_canvas.operation_registry.v1",
        "registry_version": "1.0.0",
        "availability": "available",
        "promotion_owner": "P03 runtime implementation",
        "operation_count": len(values),
        "operations": values,
    }


def render(value: Any) -> str:
    return json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    target = OUT / "operation-registry.json"
    expected = render(registry())
    if args.check:
        assert target.exists(), f"missing {target}"
        assert target.read_text() == expected, f"stale {target}"
        print(f"Spec 135 Mission Canvas operation registry: PASS ({len(operations())} operations)")
        return
    OUT.mkdir(parents=True, exist_ok=True)
    target.write_text(expected)
    print(f"Generated {target.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
