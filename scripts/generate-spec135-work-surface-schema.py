#!/usr/bin/env python3
"""Spec 135G-1: generate the durable focusa.work_surface.v1 JSON Schema and
rehydration invariant contract from the existing MissionCanvasWorkSurfaceRecord
Rust model. Every Work Surface rehydrates with exact scope, attachment,
lifecycle, queue, and isolation identity."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema"
SCHEMA_PATH = SCHEMA_DIR / "focusa.work_surface.v1.json"
CONTRACT_PATH = ROOT / "docs/contracts/spec135-work-surface-rehydration.v1.json"

SCHEMA = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://docs.startempire.ai/focusa/spec135/focusa.work_surface.v1.json",
    "title": "Focusa Work Surface v1",
    "description": "Durable multiplexed Work Surface record that rehydrates with exact scope, attachment, lifecycle, queue, and isolation identity. No singleton canonical authority.",
    "type": "object",
    "required": [
        "schema", "work_surface_id", "state_revision",
        "project_root", "continuity_id", "attachment_id", "instance_id",
        "mission_ref", "title", "surface_kind", "status",
        "pane_id", "tab_index", "pinned", "unread",
        "canonical_state_refs", "idempotency_key", "created_at", "updated_at",
    ],
    "properties": {
        "schema": {"const": "focusa.work_surface.v1"},
        "work_surface_id": {"type": "string", "minLength": 1},
        "state_revision": {"type": "integer", "minimum": 1},
        "project_root": {"type": "string", "minLength": 1},
        "continuity_id": {"type": "string", "minLength": 1},
        "attachment_id": {"type": "string", "minLength": 1},
        "instance_id": {"type": "string", "minLength": 1},
        "session_id": {"type": "string"},
        "workpoint_id": {"type": "string"},
        "mission_ref": {"type": "string", "minLength": 1},
        "title": {"type": "string", "minLength": 1},
        "surface_kind": {"type": "string", "minLength": 1},
        "status": {"type": "string", "enum": ["active", "suspended", "paused", "terminated", "closed", "rehydrating"]},
        "pane_id": {"type": "string", "minLength": 1},
        "tab_index": {"type": "integer", "minimum": 0},
        "pinned": {"type": "boolean"},
        "unread": {"type": "boolean"},
        "canonical_state_refs": {"type": "array", "items": {"type": "string", "minLength": 1}},
        "idempotency_key": {"type": "string", "minLength": 1},
        "created_at": {"type": "string", "minLength": 1},
        "updated_at": {"type": "string", "minLength": 1},
    },
    "additionalProperties": False,
}

contract = {
    "schema": "focusa.spec135.work_surface_rehydration.v1",
    "spec_ref": "docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md",
    "acceptance_criteria": "Every Work Surface rehydrates with exact scope, attachment, lifecycle, queue, and isolation identity.",
    "rehydration_identity_invariants": [
        "scope: project_root + continuity_id must be identical across rehydrate",
        "attachment: attachment_id + instance_id must be identical across rehydrate",
        "lifecycle: surface_kind + status + state_revision preserved with versioning",
        "queue: canonical_state_refs preserved exactly",
        "isolation: pane_id + tab_index preserved for split/container identity",
        "idempotency_key ensures replayable rehydration",
        "focused Work Surface state never becomes singleton canonical authority",
    ],
    "rust_model_ref": "crates/focusa-core/src/types.rs::MissionCanvasWorkSurfaceRecord",
    "schema_path": "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.work_surface.v1.json",
}


def main() -> None:
    SCHEMA_DIR.mkdir(parents=True, exist_ok=True)
    SCHEMA_PATH.write_text(json.dumps(SCHEMA, indent=2) + "\n")
    CONTRACT_PATH.write_text(json.dumps(contract, indent=2) + "\n")
    print("Spec 135G-1 work surface schema + rehydration contract generated")


if __name__ == "__main__":
    main()