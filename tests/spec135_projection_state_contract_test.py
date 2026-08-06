#!/usr/bin/env python3
"""State, host, capability, and mutation contracts for Spec 135 projections."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

from jsonschema import Draft202012Validator, ValidationError

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = json.loads((ROOT / "schemas/spec135/mission-canvas/composition-bundle.v1.schema.json").read_text())


def validate(name: str, value: object) -> None:
    Draft202012Validator(
        {"$schema": BUNDLE["$schema"], "$ref": f"#/$defs/{name}", "$defs": BUNDLE["$defs"]}
    ).validate(value)


def authority() -> dict:
    project_root_key = {
        "scope_kind": "project",
        "scope_id": "project:focusa",
        "root_path": "/example/focusa",
        "canonical_name": "Focusa",
        "fingerprint": "host-a:worktree-main",
    }
    workstream = {"scope": {"scope_kind": "project", "scope_key": project_root_key}, "workstream_id": "ws:mission-canvas"}
    attachment = {
        "workstream": workstream,
        "continuity_id": "continuity:mission-canvas",
        "instance_id": "instance:pi",
        "session_id": "session:pi",
        "attachment_id": "attachment:pi",
        "workspace_binding_id": "workspace:mission-canvas",
    }
    return {
        "workstream": workstream,
        "continuity_id": "continuity:mission-canvas",
        "attachment": attachment,
        "workspace_binding_id": "workspace:mission-canvas",
        "runtime_object": {"runtime_kind": "pi_session", "runtime_id": "session:pi"},
        "work_surface_id": "surface:pi",
    }


memory = {
    "memory_id": "layout-memory:software:overview:standard",
    **authority(),
    "profile_id": "software",
    "activity_mode_id": "overview",
    "viewport_class": "standard",
    "placements": [
        {
            "contribution_id": "contribution:work-rail",
            "preferred_regions": ["rail"],
            "preferred_order": 30,
            "minimum_span": 2,
            "maximum_span": 12,
            "preferred_adjacency": ["contribution:pi-session"],
            "last_compatible_layout_node_id": "layout:rail",
        }
    ],
    "absent_contribution_ids": ["contribution:work-rail"],
    "focused_semantic_target": "focus:pi-session",
    "memory_revision": 4,
    "idempotency_key": "memory:4",
    "updated_at": "2026-07-30T12:00:00Z",
}
validate("ProfileLayoutMemory", memory)
assert memory["absent_contribution_ids"] == ["contribution:work-rail"]

content = "Preserve this unsent instruction"
draft = {
    "draft_id": "draft:canvas:pi",
    **authority(),
    "owner": "canvas_prompt_editor",
    "content": content,
    "content_sha256": hashlib.sha256(content.encode()).hexdigest(),
    "recipient_ref": "session:pi",
    "attachment": authority()["attachment"],
    "selection_start": 0,
    "selection_end": len(content),
    "draft_revision": 3,
    "sync_state": "synchronized",
    "conflict_ref": None,
    "idempotency_key": "draft:3",
    "updated_at": "2026-07-30T12:00:00Z",
}
validate("CanvasDraftState", draft)
assert draft["attachment"]["attachment_id"] == authority()["attachment"]["attachment_id"]

renderer = {
    "interaction_mode": "canvas-guided",
    "selected_renderer": "focusa_pi_rich_window",
    "platform": "Windows",
    "availability": "available",
    "resolution_reason": "portable rich-host assets verified",
    "asset_version": "0.9.141",
    "asset_digest": "sha256:" + "1" * 64,
    "resolver_revision": "host-resolver:v1",
    "diagnostic_ref": None,
}
validate("HostRendererResolution", renderer)
host = {
    "host_instance_id": "rich-host:pi:1",
    **authority(),
    "renderer_resolution": renderer,
    "state": "focused",
    "process_id": 1234,
    "window_id": "window:mission-canvas",
    "focused": True,
    "durable_event_cursor": "event:41",
    "pi_draft_ref": "draft:pi:1",
    "canvas_draft_ref": "draft:canvas:pi",
    "last_error_ref": None,
    "lifecycle_revision": 8,
    "updated_at": "2026-07-30T12:00:00Z",
}
validate("HostLifecycleState", host)

capabilities = {
    **authority(),
    "capabilities": ["pi_session_stream", "rich_host"],
    "permissions": ["session:read", "session:prompt"],
    "available_operation_ids": ["focusa.agent_execution.prompt"],
    "unavailable_operations": [
        {
            "operation_id": "focusa.browser.click",
            "reason": "capability_not_present",
            "diagnostic_ref": "diagnostic:browser-absent",
        }
    ],
    "capability_revision": 5,
    "observed_at": "2026-07-30T12:00:00Z",
}
validate("CapabilityProjection", capabilities)
assert "focusa.browser.click" not in capabilities["available_operation_ids"]

command = {
    "command_id": "layout-command:split:1",
    **authority(),
    "action": "split_horizontal",
    "attachment": authority()["attachment"],
    "target_work_surface_id": "surface:pi",
    "secondary_work_surface_id": "surface:evidence",
    "target_contribution_id": "contribution:pi-session",
    "target_layout_node_id": "layout:primary",
    "target_index": None,
    "split_ratio": 0.65,
    "expected_projection_revision": 12,
    "expected_layout_revision": 5,
    "idempotency_key": "layout:split:1",
    "requested_at": "2026-07-30T12:00:00Z",
}
validate("LayoutMutationCommand", command)
result = {
    "workstream": authority()["workstream"],
    "command_id": command["command_id"],
    "accepted": True,
    "projection_revision": 13,
    "layout_revision": 6,
    "projection_digest": "sha256:" + "2" * 64,
    "event_cursor": "event:42",
    "error_ref": None,
    "evidence_ref": "evidence:layout:split:1",
    "receipt_ref": "receipt:layout:split:1",
}
validate("LayoutMutationResult", result)
assert command["expected_layout_revision"] < result["layout_revision"]
assert command["attachment"]["attachment_id"] == authority()["attachment"]["attachment_id"]

invalid_command = dict(command)
invalid_command["split_ratio"] = 1.2
try:
    validate("LayoutMutationCommand", invalid_command)
except ValidationError:
    pass
else:
    raise AssertionError("layout mutation accepted invalid split ratio")

invalid_host = dict(host)
invalid_host["renderer_resolution"] = dict(renderer, platform="macOS-only")
try:
    validate("HostLifecycleState", invalid_host)
except ValidationError:
    pass
else:
    raise AssertionError("host lifecycle accepted unsupported platform")

print("Spec 135 projection state contracts: PASS")
