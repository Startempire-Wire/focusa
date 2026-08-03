#!/usr/bin/env python3
"""Spec 135J-4: generate the durable reconnect, replay, deduplication, and gap
recovery contract. Disconnect/reconnect tests restore exact UI state without
loss, duplication, or false freshness."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIR = ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema"
SCHEMA_PATH = SCHEMA_DIR / "focusa.reconnect_recovery.v1.json"
CONTRACT_PATH = ROOT / "docs/contracts/spec135-reconnect-replay-recovery.v1.json"

SCHEMA = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "$id": "https://docs.startempire.ai/focusa/spec135/focusa.reconnect_recovery.v1.json",
    "title": "Focusa Reconnect Recovery v1",
    "description": "A durable reconnect recovery plan that restores exact UI state without loss, duplication, or false freshness after daemon disconnect/reconnect.",
    "type": "object",
    "required": [
        "schema", "recovery_id", "project_root", "continuity_id",
        "last_known_cursor", "last_known_sequence", "snapshot_ref",
        "dedup_strategy", "gap_detection", "stale_label_policy",
        "refetch_bound", "replay_order",
    ],
    "properties": {
        "schema": {"const": "focusa.reconnect_recovery.v1"},
        "recovery_id": {"type": "string", "minLength": 1},
        "project_root": {"type": "string", "minLength": 1},
        "continuity_id": {"type": "string", "minLength": 1},
        "last_known_cursor": {"type": "string", "minLength": 1},
        "last_known_sequence": {"type": "integer", "minimum": 0},
        "snapshot_ref": {"type": "string", "minLength": 1},
        "dedup_strategy": {"type": "string", "enum": ["event_id_exact", "invalidation_key", "sequence_gte", "composite"]},
        "gap_detection": {"type": "object", "required": ["method", "action"], "properties": {"method": {"type": "string", "enum": ["sequence_gap", "heartbeat_timeout", "cursor_hash_mismatch"]}, "action": {"type": "string", "enum": ["refetch_from_cursor", "full_snapshot_resync", "bounded_refetch"]}}},
        "stale_label_policy": {"type": "object", "required": ["label", "visibility"], "properties": {"label": {"type": "string", "minLength": 1}, "visibility": {"type": "string", "enum": ["always", "on_gap", "never"]}}},
        "refetch_bound": {"type": "object", "required": ["max_events", "max_tokens"], "properties": {"max_events": {"type": "integer", "minimum": 1}, "max_tokens": {"type": "integer", "minimum": 1}}},
        "replay_order": {"type": "string", "enum": ["sequence_ascending", "snapshot_then_delta", "delta_then_snapshot_verify"]},
    },
    "additionalProperties": False,
}

contract = {
    "schema": "focusa.spec135.reconnect_replay_recovery.v1",
    "spec_ref": "docs/135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md",
    "acceptance_criteria": "Disconnect/reconnect tests restore exact UI state without loss, duplication, or false freshness.",
    "components": [
        {
            "component": "cursor_resume",
            "description": "Resume from last_known_cursor; events before cursor are assumed received",
        },
        {
            "component": "snapshot_fallback",
            "description": "If cursor is stale or gap exceeds refetch bound, fall back to full snapshot resync",
        },
        {
            "component": "duplicate_suppression",
            "description": "Deduplicate by event_id_exact or invalidation_key; never replay the same event twice",
        },
        {
            "component": "gap_detection",
            "description": "Detect sequence gaps, heartbeat timeouts, or cursor hash mismatches; refetch from cursor or snapshot resync",
        },
        {
            "component": "stale_labeling",
            "description": "Mark recovered state as stale until verified live; no false freshness",
        },
        {
            "component": "bounded_refetch",
            "description": "Refetch is bounded by max_events and max_tokens; no unbounded replays",
        },
    ],
    "no_loss_invariant": "Replay restores exact UI state: no events lost between disconnect and reconnect.",
    "no_duplication_invariant": "Duplicate suppression prevents the same event from rendering twice.",
    "no_false_freshness_invariant": "Recovered state is labeled stale until the cursor reaches live; false freshness is prohibited.",
    "dedup_strategies": ["event_id_exact", "invalidation_key", "sequence_gte", "composite"],
    "gap_detection_methods": ["sequence_gap", "heartbeat_timeout", "cursor_hash_mismatch"],
    "replay_orders": ["sequence_ascending", "snapshot_then_delta", "delta_then_snapshot_verify"],
    "schema_path": "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.reconnect_recovery.v1.json",
}


def main() -> None:
    SCHEMA_DIR.mkdir(parents=True, exist_ok=True)
    SCHEMA_PATH.write_text(json.dumps(SCHEMA, indent=2) + "\n")
    CONTRACT_PATH.write_text(json.dumps(contract, indent=2) + "\n")
    print("Spec 135J-4 reconnect replay dedup recovery contract generated")


if __name__ == "__main__":
    main()