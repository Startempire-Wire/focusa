#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
C = R / "docs/contracts/spec135/generated-contract-v1"


def j(name):
    return json.loads((C / name).read_text())


def main():
    t = (R / "crates/focusa-core/src/types.rs").read_text()
    r = (R / "crates/focusa-core/src/reducer.rs").read_text()
    a = (R / "crates/focusa-api/src/routes/mission_canvas_surfaces.rs").read_text()
    u = (R / "packages/a2ui-renderer/proof/mission-surfaces.ts").read_text()
    for x in [
        "MissionCanvasWorkSurfaceRecord",
        "MissionCanvasSurfaceStatus",
        "ViewClosed",
        "canonical_state_refs",
    ]:
        assert x in t
    for x in [
        "bounded handles, never duplicated state payloads",
        "MissionCanvasSurfaceRevised",
        "Work Surface revision",
    ]:
        assert x in r
    for x in [
        "SurfaceAction",
        "Arrange",
        "Suspend",
        "Resume",
        "CloseView",
        "canonical_state_refs",
    ]:
        assert x in a
    ops = {x["operation_id"]: x for x in j("operation-registry.json")["operations"]}
    assert (
        ops["focusa.mission_canvas.surface.mutate"]["materialization_mode"]
        == "canonical_projection_event"
    )
    assert ops["focusa.mission_canvas.surface.list"]["scope"]["required_keys"] == [
        "project_root",
        "continuity_id",
        "attachment_id",
    ]
    for s in [
        "focusa.mission_canvas_surface_list.request.v1",
        "focusa.mission_canvas_surface_list.v1",
        "focusa.mission_canvas_surface_mutation.request.v1",
        "focusa.mission_canvas_surface_mutation_result.v1",
    ]:
        assert (C / "json-schema" / f"{s}.json").exists()
    assert (
        "FocusaA2uiRenderer" in u
        and "Open Split Surfaces" in u
        and "playwright" not in u.lower()
    )
    assert (
        j("spec135-m3-mission-surfaces-proof.json")["contracts"]["operation_count"]
        == 81
    )
    print("Spec 135 M3 multiplexed Mission Canvas Work Surfaces static proof: PASS")


if __name__ == "__main__":
    main()
