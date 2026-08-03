#!/usr/bin/env python3
"""Validate bounded menubar projection without confusing it with Pi-native Canvas."""
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]


def main():
    proof = json.loads(
        (
            R
            / "docs/contracts/spec135/generated-contract-v1/spec135-m1-mission-canvas-shell-proof.json"
        ).read_text()
    )
    authority = (
        R / "docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml"
    ).read_text()
    page = (R / "apps/menubar/src/routes/+page.svelte").read_text()
    canvas = (
        R / "apps/menubar/src/lib/components/MissionCanvasView.svelte"
    ).read_text()
    runtime = (R / "apps/menubar/src/lib/components/RuntimeView.svelte").read_text()
    store = (R / "apps/menubar/src/lib/stores/runtime.svelte.ts").read_text()
    scope = (R / "apps/menubar/src/lib/workLoopScope.js").read_text()
    api = (R / "apps/menubar/src/lib/api.ts").read_text()

    assert proof["status"] == "partial_projection_foundation"
    assert proof["accepted"] is False
    assert proof["implementation"]["host_renderer"] == "menubar_peek"
    assert "pi_native_mission_canvas" in authority
    assert "menubar_peek" in authority
    assert "bounded_status_and_launch_surface_only" in authority

    assert (
        "MissionCanvasView" in page
        and "activeTab === 'mission-canvas'" in page
        and 'title="Mission Canvas"' in page
    )
    assert (
        "RuntimeView" in canvas
        and "Focusa Mission Canvas" in canvas
        and "CockpitView" not in page + canvas
    )
    for marker in [
        "projectIdentity",
        "trajectory",
        "workpointResume",
        "workLoopHealth",
        "memoryTelemetry",
        "releaseProof",
    ]:
        assert marker in runtime or marker in store
    for route in [
        "/v1/project/identity",
        "/v1/trajectory/view",
        "/v1/workpoint/resume",
        "/v1/work-loop/health",
        "/v1/telemetry/memory",
    ]:
        assert route in page + store + scope
    assert "fetchJson" in api and "mission-canvas-grid" in runtime
    for forbidden in [
        'localStorage.setItem("canonical',
        "canonicalReducer",
        "canonicalState =",
    ]:
        assert forbidden not in page + canvas + runtime

    print(
        "Spec 135 M1 menubar Mission Canvas projection: PASS "
        "(bounded; Pi-native Canvas owns current-terminal interaction)"
    )


if __name__ == "__main__":
    main()
