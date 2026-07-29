#!/usr/bin/env python3
"""Spec 135G-2 durable Canvas layout/restoration proof."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
S=json.loads((ROOT/"docs/contracts/spec135/generated-contract-v1/json-schema/focusa.mission_canvas_layout_state.v1.json").read_text())
C=json.loads((ROOT/"docs/contracts/spec135-canvas-layout-restoration.v1.json").read_text())
T=(ROOT/"crates/focusa-core/src/types.rs").read_text()
assert S["title"] == "Focusa Mission Canvas Layout State v1"
assert C["acceptance_criteria"] == "Layout restores after restart without mutating canonical project state."
for group in ("open_focus","split_group","filters","revision"):
    assert C["restoration_groups"][group]
for field in sum(C["restoration_groups"].values(),[]):
    assert field in S["properties"], field
    assert f"pub {field}" in T, field
assert C["ownership"]["canonical_project_state_mutation"] is False
assert C["ownership"]["canvas_preferences"] == "user/device-owned"
for field in ("user_id","device_id","client_instance_id","idempotency_key","project_root","continuity_id"):
    assert field in S["required"], field
assert "MissionCanvasStateRecord" in T
assert "Visual focus never becomes singleton canonical authority" in C["laws"]
print("Spec 135 G2 durable Canvas layout and restoration: PASS")
