#!/usr/bin/env python3
"""Registry, vertical composition, matrix, and switching contract gate."""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / "crates/focusa-core/src/mission_canvas/profiles.rs").read_text()
api = (ROOT / "crates/focusa-api/src/routes/mission_canvas.rs").read_text()
frontend = (ROOT / "apps/pi-extension/src/mission-canvas-view.ts").read_text()
matrix = json.loads((ROOT / "tests/fixtures/spec135-profile-activity-matrix.json").read_text())
visual = json.loads((ROOT / "tests/fixtures/spec135-uiai-vertical-recomposition-scenarios.json").read_text())

for registry in ["profiles", "activities", "panels", "home_canvases", "work_surface_renderers", "artifact_renderers", "terminology", "domain_semantics"]:
    assert f"pub {registry}:" in source, registry
for profile in matrix["profiles"]:
    assert re.search(rf'profile\(\s*"{re.escape(profile)}"', source), profile
for activity in matrix["activities"]:
    assert re.search(rf'activity\(\s*"{re.escape(activity)}"', source), activity
for vector in matrix["vectors"]:
    assert vector["expected"], vector
    assert vector["profile"] in matrix["profiles"]
    assert vector["activity"] in matrix["activities"]
assert {vector["activity"] for vector in matrix["vectors"]} == set(matrix["activities"])
assert {vector["profile"] for vector in matrix["vectors"]} == set(matrix["profiles"])

assert "compose_candidate_ids" in source
assert "viable_profiles" in source
assert "install_domain_pack" in source
assert "DomainPackAlreadyInstalled" in source
assert "/v1/mission-canvas/profiles/select" in api
assert "/v1/mission-canvas/activities/select" in api
assert "/v1/mission-canvas/domain-packs/install" in api
assert "layout-memory:" in api
assert "composition_not_viable" in api
assert "selectProfile" in frontend
assert "MissionCanvasActivity" in frontend
assert "resolveContributions" in frontend
assert len(visual["scenarios"]) >= 5
assert {scenario["profile"] for scenario in visual["scenarios"]} >= {"software", "legal", "markets", "research", "custom"}
assert matrix["memory_disappearance_return"]["activity_before"] == matrix["memory_disappearance_return"]["activity_after"]

print("Spec 135 profile/activity registries and vertical recomposition: PASS")
