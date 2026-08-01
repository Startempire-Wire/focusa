#!/usr/bin/env python3
"""Projection events, proof envelopes, responsive fixtures, and migration contracts."""
from __future__ import annotations

import json
from pathlib import Path

from jsonschema import Draft202012Validator, ValidationError

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = json.loads((ROOT / "schemas/spec135/mission-canvas/composition-bundle.v1.schema.json").read_text())


def validate(name: str, value: object) -> None:
    Draft202012Validator(
        {"$schema": BUNDLE["$schema"], "$ref": f"#/$defs/{name}", "$defs": BUNDLE["$defs"]}
    ).validate(value)


def scope() -> dict:
    return {
        "project_root": "/example/focusa",
        "continuity_id": "mission-canvas",
        "instance_id": "instance:pi",
        "session_id": "session:pi",
        "attachment_id": "attachment:pi",
        "working_subpath_id": "working-subpath:primary",
    }


digest_before = "sha256:" + "1" * 64
digest_after = "sha256:" + "2" * 64
event = {
    "event_id": "projection-event:42",
    "event_kind": "projection_resolved",
    "scope": scope(),
    "contribution_id": None,
    "host_instance_id": "rich-host:pi:1",
    "projection_revision": 13,
    "layout_revision": 6,
    "event_cursor": "event:42",
    "causation_id": "layout-command:split:1",
    "correlation_id": "resolve:13",
    "occurred_at": "2026-07-30T12:00:00Z",
    "payload_ref": "projection:13",
    "evidence_refs": ["recomposition-evidence:13"],
    "receipt_refs": ["recomposition-receipt:13"],
}
validate("ProjectionLifecycleEvent", event)

decision = {
    "contribution_id": "contribution:pi-session",
    "outcome": "eligible",
    "omission": None,
    "merged_into_contribution_id": None,
    "rule_revision": "adaptive-composition:v1",
    "projection_revision": 13,
    "evidence_refs": ["evidence:pi-session"],
}
evidence = {
    "evidence_id": "recomposition-evidence:13",
    "scope": scope(),
    "trigger": "viewport_change",
    "input_projection_digest": digest_before,
    "output_projection_digest": digest_after,
    "rule_revision": "adaptive-composition:v1",
    "candidate_contribution_ids": ["contribution:pi-session"],
    "eligibility_decisions": [decision],
    "layout_decision_refs": ["layout-decision:6"],
    "diagnostic_refs": [],
    "observed_at": "2026-07-30T12:00:00Z",
}
validate("RecompositionEvidence", evidence)
receipt = {
    "receipt_id": "recomposition-receipt:13",
    "scope": scope(),
    "accepted": True,
    "projection_revision": 13,
    "layout_revision": 6,
    "projection_digest": digest_after,
    "event_cursor": "event:42",
    "evidence_id": evidence["evidence_id"],
    "idempotency_key": "resolve:13",
    "error_ref": None,
    "issued_at": "2026-07-30T12:00:00Z",
}
validate("RecompositionReceipt", receipt)
assert receipt["evidence_id"] == evidence["evidence_id"]
assert receipt["projection_digest"] == evidence["output_projection_digest"]

fixtures = json.loads((ROOT / "tests/fixtures/spec135-responsive-evaluations.json").read_text())
assert {item["viewport"]["platform"] for item in fixtures} == {"macOS", "Windows", "Linux"}
assert {item["viewport"]["class"] for item in fixtures} >= {"minimum", "standard", "productive"}
for fixture in fixtures:
    validate("ResponsiveEvaluationFixture", fixture)
    candidates = set(fixture["candidate_contribution_ids"])
    eligible = set(fixture["expected_eligible_contribution_ids"])
    omitted = {item["contribution_id"] for item in fixture["expected_omissions"]}
    assert eligible.isdisjoint(omitted)
    assert candidates == eligible | omitted

migration = {
    "migration_id": "layout-migration:terminal:1",
    "scope": scope(),
    "source_kind": "terminal_local",
    "source_revision": 7,
    "source_digest": digest_before,
    "target_profile_id": "software",
    "target_activity_mode_id": "overview",
    "mappings": [
        {
            "legacy_ref": "terminal-pane:session",
            "target_contribution_id": "contribution:pi-session",
            "mapping_status": "mapped",
            "diagnostic_ref": None,
        }
    ],
    "preserved_draft_ref": "draft:pi:1",
    "target_layout_memory_ref": "layout-memory:software:overview:standard",
    "status": "validated",
    "warning_refs": [],
    "error_ref": None,
    "idempotency_key": "migration:terminal:1",
    "created_at": "2026-07-30T12:00:00Z",
}
validate("LegacyLayoutMigrationEnvelope", migration)
assert migration["preserved_draft_ref"]

invalid_event = dict(event, event_kind="client_invented_panel")
try:
    validate("ProjectionLifecycleEvent", invalid_event)
except ValidationError:
    pass
else:
    raise AssertionError("projection event accepted unknown event taxonomy")

print("Spec 135 projection events and proof contracts: PASS")
