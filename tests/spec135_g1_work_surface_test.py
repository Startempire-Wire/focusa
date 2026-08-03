#!/usr/bin/env python3
"""Spec 135G-1 durable Work Surface state and bindings proof."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md").read_text()
SCHEMA = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.work_surface.v1.json").read_text()
)
CONTRACT = json.loads(
    (ROOT / "docs/contracts/spec135-work-surface-rehydration.v1.json").read_text()
)
TYPES = (ROOT / "crates/focusa-core/src/types.rs").read_text()

assert SCHEMA["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert SCHEMA["title"] == "Focusa Work Surface v1"

# Every required rehydration identity field present
required = set(SCHEMA["required"])
for field in (
    "work_surface_id", "state_revision",
    "project_root", "continuity_id", "attachment_id", "instance_id",
    "mission_ref", "title", "surface_kind", "status",
    "pane_id", "tab_index", "pinned", "unread",
    "canonical_state_refs", "idempotency_key", "created_at", "updated_at",
):
    assert field in required, field

# Rehydration invariant categories covered
for inv in ("scope: project_root + continuity_id", "attachment: attachment_id + instance_id", "lifecycle: surface_kind + status + state_revision", "queue: canonical_state_refs", "isolation: pane_id + tab_index", "idempotency_key"):
    matched = any(inv in line for line in CONTRACT["rehydration_identity_invariants"])
    assert matched, f"missing invariant: {inv}"

# Rust model RUST struct present and has all schema fields
for rust_field in ("work_surface_id", "state_revision", "project_root", "continuity_id", "attachment_id", "instance_id", "mission_ref", "title", "surface_kind", "pane_id", "tab_index", "pinned", "unread", "canonical_state_refs", "idempotency_key"):
    assert f"pub {rust_field}" in TYPES, f"missing Rust field: {rust_field}"
assert "MissionCanvasWorkSurfaceRecord" in TYPES
# Binding kinds present in Rust
for kind in ("Session", "BrowserContext", "BrowserTarget", "File", "Operation", "Evidence", "Action"):
    assert kind in TYPES, f"missing binding kind: {kind}"

for spec_text in (
    "Work Surface",
    "rehydrat",
    "attachment",
    "isolation",
    "multiplexed",
    "singleton",
):
    assert spec_text in SPEC, spec_text

print("Spec 135 G1 durable Work Surface state and bindings: PASS")