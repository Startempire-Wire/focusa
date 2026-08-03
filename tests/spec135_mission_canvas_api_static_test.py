#!/usr/bin/env python3
"""Static API parity gate for the promoted Mission Canvas runtime surface."""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / "crates/focusa-api/src/routes/mission_canvas.rs").read_text()
server = (ROOT / "crates/focusa-api/src/server.rs").read_text()
capabilities_source = (ROOT / "crates/focusa-api/src/routes/agent_capabilities.rs").read_text()
registry = json.loads((ROOT / "docs/contracts/spec135/mission-canvas-v1/operation-registry.json").read_text())

assert ".merge(routes::mission_canvas::router())" in server
assert "pub mod mission_canvas;" in (ROOT / "crates/focusa-api/src/routes/mod.rs").read_text()
assert "require_permission" in source
assert "scope_incomplete" in source
assert "idempotency_key_missing" in source
assert "projection_revision_conflict" in source
assert "MissionCanvasStore::open(&state.config.data_dir)" in source
assert "events_after" in source

runtime_paths = set(re.findall(r'"(/v1/mission-canvas/[^" ]+)"', source))
registered_ids = set(re.findall(r'"(focusa\.mission_canvas\.[a-z_.]+)"', capabilities_source))
implemented = [entry for entry in registry["operations"] if entry["availability"] == "available"]
def route_matches(template: str, concrete: str) -> bool:
    pattern = "^" + re.sub(r"\\\{[^}]+\\\}", "[^/]+", re.escape(template)) + "$"
    return re.match(pattern, concrete) is not None


for operation in implemented:
    assert any(route_matches(path, operation["path"]) for path in runtime_paths), operation["path"]
    assert operation["operation_id"] in registered_ids, operation["operation_id"]

assert all(entry["availability"] == "available" for entry in registry["operations"])
assert "focusa.mission_canvas.layout.mutate" in registered_ids

print("Spec 135 Mission Canvas API static parity: PASS")
