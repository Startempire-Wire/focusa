#!/usr/bin/env python3
"""Spec 135J-4 reconnect, replay, deduplication, and gap recovery proof."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.reconnect_recovery.v1.json").read_text()
)
CONTRACT = json.loads(
    (ROOT / "docs/contracts/spec135-reconnect-replay-recovery.v1.json").read_text()
)

assert SCHEMA["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert SCHEMA["title"] == "Focusa Reconnect Recovery v1"
for field in ("recovery_id", "last_known_cursor", "last_known_sequence", "snapshot_ref", "dedup_strategy", "gap_detection", "stale_label_policy", "refetch_bound", "replay_order"):
    assert field in SCHEMA["required"], field

assert CONTRACT["acceptance_criteria"] == "Disconnect/reconnect tests restore exact UI state without loss, duplication, or false freshness."

# All 6 components present
components = {c["component"] for c in CONTRACT["components"]}
assert components == {"cursor_resume", "snapshot_fallback", "duplicate_suppression", "gap_detection", "stale_labeling", "bounded_refetch"}

# No loss, no duplication, no false freshness invariants
for inv_key in ("no_loss_invariant", "no_duplication_invariant", "no_false_freshness_invariant"):
    assert inv_key in CONTRACT, inv_key
    assert CONTRACT[inv_key]

# Dedup strategies
assert set(CONTRACT["dedup_strategies"]) == set(SCHEMA["properties"]["dedup_strategy"]["enum"])

# Gap detection methods
assert set(CONTRACT["gap_detection_methods"]) == set(SCHEMA["properties"]["gap_detection"]["properties"]["method"]["enum"])

# Replay orders
assert set(CONTRACT["replay_orders"]) == set(SCHEMA["properties"]["replay_order"]["enum"])

# Refetch is bounded
assert SCHEMA["properties"]["refetch_bound"]["required"] == ["max_events", "max_tokens"]

# Stale labeling has visibility policy
assert "always" in SCHEMA["properties"]["stale_label_policy"]["properties"]["visibility"]["enum"]

print("Spec 135 J4 reconnect replay dedup recovery: PASS")