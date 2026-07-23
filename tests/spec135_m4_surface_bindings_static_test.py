#!/usr/bin/env python3
"""SPEC135-M4 exact attachment-scoped Work Surface binding contract lint."""

import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
C = R / "docs/contracts/spec135/generated-contract-v1"


def load(name):
    return json.loads((C / name).read_text())


def main():
    capabilities = (R / "crates/focusa-api/src/routes/agent_capabilities.rs").read_text()
    route = (R / "crates/focusa-api/src/routes/mission_canvas_surfaces.rs").read_text()
    types = (R / "crates/focusa-core/src/types.rs").read_text()
    reducer = (R / "crates/focusa-core/src/reducer.rs").read_text()

    for value in (
        "MissionCanvasSurfaceBindingRecord",
        "MissionCanvasBindingKind",
        "MissionCanvasSurfaceBindingRevised",
    ):
        assert value in types or value in reducer
    for value in (
        "binding_scoped",
        "work_surface_id",
        "Cross-surface binding mutation denied",
        '"/v1/mission-canvas/surface-bindings"',
        '"/v1/mission-canvas/surface-bindings/mutate"',
    ):
        assert value in route

    registry = load("operation-registry.json")
    operations = {row["operation_id"]: row for row in registry["operations"]}
    for operation_id in (
        "focusa.mission_canvas.surface_binding.list",
        "focusa.mission_canvas.surface_binding.mutate",
    ):
        operation = operations[operation_id]
        assert operation_id in capabilities
        assert operation["scope"]["required_keys"] == [
            "project_root",
            "continuity_id",
            "attachment_id",
            "work_surface_id",
        ]

    schema_ids = (
        "focusa.mission_canvas_surface_binding_list.request.v1",
        "focusa.mission_canvas_surface_binding_list.v1",
        "focusa.mission_canvas_surface_binding_mutation.request.v1",
        "focusa.mission_canvas_surface_binding_mutation_result.v1",
    )
    for schema_id in schema_ids:
        schema = load(f"json-schema/{schema_id}.json")
        assert schema["x-focusa-schema-id"] == schema_id

    openapi = load("openapi-3.0.3.json")
    assert openapi["paths"]["/v1/mission-canvas/surface-bindings"]["get"]
    assert openapi["paths"]["/v1/mission-canvas/surface-bindings/mutate"]["post"]

    actions = load("ui-action-bindings.fixture.json")
    action_ids = {row["action_id"] for row in actions["bindings"]}
    assert set(operations).issubset(action_ids)
    assert "surface_binding_list" in (R / "packages/generated/spec135/typescript/schema.d.ts").read_text()
    assert "MissionCanvasSurfaceBinding" in (R / "packages/generated/spec135/go/client.gen.go").read_text()
    print("Spec 135 M4 exact attachment-scoped Work Surface binding lint: PASS")


if __name__ == "__main__":
    main()
