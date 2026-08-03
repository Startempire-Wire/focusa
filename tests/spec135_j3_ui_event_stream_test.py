#!/usr/bin/env python3
"""Spec 135J-3 durable UI event stream proof."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md").read_text()
MASTER = (ROOT / "docs/135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md").read_text()
SCHEMA = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.ui_event.v1.json").read_text()
)
CONTRACT = json.loads(
    (ROOT / "docs/contracts/spec135-ui-event-stream-durable.v1.json").read_text()
)

assert SCHEMA["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert SCHEMA["title"] == "Focusa UI Event v1"
for field in ("event_id", "event_type", "sequence", "cursor", "invalidation_key", "project_root", "continuity_id", "session_origin", "attachment_id", "payload_max_bytes"):
    assert field in SCHEMA["required"], field

# All 24 required projection events present in schema enum and contract list
schema_events = set(SCHEMA["properties"]["event_type"]["enum"])
contract_events = set(CONTRACT["required_projection_events"])
assert schema_events == contract_events
assert len(contract_events) == 24

# Invariants cover restart-survival and identity-preservation
for inv in ("scope", "cursor", "invalidation key", "bounded by payload_max_bytes", "Session origin identity preserved across restart", "monotonic per", "Receipt ref"):
    assert any(inv in line for line in CONTRACT["invariants"]), inv

# Bounded payload rule: no transcripts/page bodies/images/browser storage
bounded = CONTRACT["bounded_payload_rule"]
for forbidden in ("full transcripts", "page bodies", "images", "browser storage"):
    assert forbidden in bounded, f"bounded rule missing: {forbidden}"

# Read models covered
read_models = CONTRACT["required_read_models"]
for rm in ("mission_canvas.summary", "mission_canvas.open_surfaces", "mission_canvas.surface_detail", "mission_canvas.session_inventory", "mission_canvas.project_activity", "mission_canvas.contention"):
    assert rm in read_models, rm

for spec_text in (
    "Required projection events include",
    "Events contain scoped IDs, versions, cursors, and invalidation keys",
    "durable replayable UI event stream",
    "session-origin identity",
):
    assert spec_text in SPEC or spec_text in MASTER, spec_text

print("Spec 135 J3 durable UI event stream: PASS")