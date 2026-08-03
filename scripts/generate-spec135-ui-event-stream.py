#!/usr/bin/env python3
"""Spec 135J-3: generate the durable UI event stream contract. Events survive
daemon restart and preserve project/workstream/session/origin identity.
Contains scoped IDs, versions, cursors, invalidation keys, and bounded
payloads — never full transcripts/page bodies/images/browser storage."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema"
SCHEMA_PATH = SCHEMA_DIR / "focusa.ui_event.v1.json"
CONTRACT_PATH = ROOT / "docs/contracts/spec135-ui-event-stream-durable.v1.json"

REQUIRED_EVENTS = [
    "mission_canvas_surface_created",
    "mission_canvas_surface_updated",
    "mission_canvas_surface_focused",
    "mission_canvas_surface_suspended",
    "mission_canvas_surface_rehydrated",
    "mission_canvas_surface_closed",
    "mission_canvas_layout_changed",
    "attachment_added",
    "attachment_role_changed",
    "attachment_detached",
    "session_started",
    "session_state_changed",
    "session_ended",
    "browser_context_created",
    "browser_context_isolation_changed",
    "browser_context_closed",
    "browser_target_opened",
    "browser_target_navigated",
    "browser_target_moved",
    "browser_target_closed",
    "surface_unread_changed",
    "surface_approval_required",
    "surface_conflict_changed",
    "surface_writer_lease_changed",
]

SCHEMA = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://docs.startempire.ai/focusa/spec135/focusa.ui_event.v1.json",
    "title": "Focusa UI Event v1",
    "description": "Durable scoped UI event that survives daemon restart and preserves project/workstream/session/origin identity.",
    "type": "object",
    "required": [
        "schema", "event_id", "event_type", "sequence",
        "cursor", "invalidation_key", "project_root", "continuity_id",
        "session_origin", "attachment_id", "created_at", "payload_max_bytes",
    ],
    "properties": {
        "schema": {"const": "focusa.ui_event.v1"},
        "event_id": {"type": "string", "minLength": 1},
        "event_type": {"type": "string", "enum": REQUIRED_EVENTS},
        "sequence": {"type": "integer", "minimum": 1},
        "cursor": {"type": "string", "minLength": 1},
        "invalidation_key": {"type": "string", "minLength": 1},
        "project_root": {"type": "string", "minLength": 1},
        "continuity_id": {"type": "string", "minLength": 1},
        "session_origin": {"type": "string", "minLength": 1},
        "attachment_id": {"type": "string", "minLength": 1},
        "work_surface_id": {"type": "string"},
        "browser_context_ref": {"type": "string"},
        "payload": {"type": "object"},
        "payload_max_bytes": {"type": "integer", "minimum": 1},
        "receipt_ref": {"type": "string"},
        "created_at": {"type": "string", "minLength": 1},
    },
    "additionalProperties": False,
}

contract = {
    "schema": "focusa.spec135.ui_event_stream_durable.v1",
    "spec_ref": "docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md",
    "acceptance_criteria": "Events survive daemon restart and preserve project/workstream/session/origin identity.",
    "required_projection_events": REQUIRED_EVENTS,
    "invariants": [
        "Events contain scoped IDs, versions, cursors, and invalidation keys",
        "Events never contain full transcripts, page bodies, images, or browser storage",
        "Payloads are bounded by payload_max_bytes",
        "Sequence is monotonic per (project_root, continuity_id) scope",
        "Cursor enables replay from any point after restart",
        "Invalidation key allows clients to skip redundant re-render",
        "Receipt ref makes suspicious or impactful events auditable",
        "Session origin identity preserved across restart",
    ],
    "bounded_payload_rule": "Events contain scoped IDs, versions, cursors, and invalidation keys rather than full transcripts, page bodies, images, or browser storage.",
    "schema_path": "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.ui_event.v1.json",
    "required_read_models": [
        "mission_canvas.summary",
        "mission_canvas.open_surfaces",
        "mission_canvas.surface_detail",
        "mission_canvas.session_inventory",
        "mission_canvas.project_activity",
        "mission_canvas.contention",
    ],
}


def main() -> None:
    SCHEMA_DIR.mkdir(parents=True, exist_ok=True)
    SCHEMA_PATH.write_text(json.dumps(SCHEMA, indent=2) + "\n")
    CONTRACT_PATH.write_text(json.dumps(contract, indent=2) + "\n")
    print(f"Spec 135J-3 durable UI event stream generated: {len(REQUIRED_EVENTS)} event types")


if __name__ == "__main__":
    main()