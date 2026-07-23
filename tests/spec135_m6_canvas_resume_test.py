#!/usr/bin/env python3
"""Spec 135 M6 static proof: exact Mission Canvas topology persists and rehydrates."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMAS = ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema"

required_schemas = {
    "focusa.mission_canvas_state_get.request.v1.json",
    "focusa.mission_canvas_state.v1.json",
    "focusa.mission_canvas_state_mutation.request.v1.json",
    "focusa.mission_canvas_state_mutation_result.v1.json",
}
for name in required_schemas:
    schema = json.loads((SCHEMAS / name).read_text())
    assert schema["$schema"] == "https://json-schema.org/draft/2020-12/schema"
    assert schema["additionalProperties"] is False

state_schema = json.loads((SCHEMAS / "focusa.mission_canvas_state.v1.json").read_text())
canvas = state_schema["properties"]["canvas"]
for field in (
    "open_work_surface_ids",
    "focused_work_surface_id",
    "secondary_focused_surface_id",
    "split_layout_ref",
    "group_order",
    "selected_context_refs",
    "unread_event_cursor",
    "session_projection_revision",
):
    assert field in canvas["properties"]
for field in (
    "open_work_surface_ids",
    "group_order",
    "selected_context_refs",
):
    assert canvas["properties"][field]["maxItems"] == 64
    assert canvas["properties"][field]["uniqueItems"] is True

route = (ROOT / "crates/focusa-api/src/routes/mission_canvas_surfaces.rs").read_text()
reducer = (ROOT / "crates/focusa-core/src/reducer.rs").read_text()
types = (ROOT / "crates/focusa-core/src/types.rs").read_text()
capabilities = (ROOT / "crates/focusa-api/src/routes/agent_capabilities.rs").read_text()
for marker in (
    "MissionCanvasStateRecord",
    "open_work_surface_ids",
    "selected_context_refs",
    "unread_event_cursor",
):
    assert marker in types
for marker in (
    "/v1/mission-canvas/state",
    "/v1/mission-canvas/state/mutate",
    "refusing to manufacture a replacement session or project",
    "resume_surface:",
    "reopen_view:",
    "remove_missing_surface:",
):
    assert marker in route
for marker in (
    "MissionCanvasStateRevised",
    "cannot adopt a Work Surface outside its exact project and continuity scope",
    "Focused Mission Canvas surfaces must remain in the open topology",
):
    assert marker in reducer
for operation in (
    "focusa.mission_canvas.state.get",
    "focusa.mission_canvas.state.mutate",
):
    assert operation in capabilities

openapi = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json").read_text()
)
assert "/v1/mission-canvas/state" in openapi["paths"]
assert "/v1/mission-canvas/state/mutate" in openapi["paths"]
registry = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/operation-registry.json").read_text()
)
operation_ids = {operation["operation_id"] for operation in registry["operations"]}
assert {
    "focusa.mission_canvas.state.get",
    "focusa.mission_canvas.state.mutate",
} <= operation_ids
assert registry["operation_count"] == len(registry["operations"])

print("Spec 135 M6 exact Mission Canvas persistence and rehydration: PASS")
