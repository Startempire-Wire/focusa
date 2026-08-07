#!/usr/bin/env python3
"""Contract tests for generated Spec 135 Mission Canvas projection schemas."""
from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path

from jsonschema import Draft202012Validator, ValidationError

ROOT = Path(__file__).resolve().parents[1]
BUNDLE_PATH = ROOT / "schemas/spec135/mission-canvas/composition-bundle.v1.schema.json"
BUNDLE = json.loads(BUNDLE_PATH.read_text())


def validator(definition: str) -> Draft202012Validator:
    return Draft202012Validator(
        {
            "$schema": BUNDLE["$schema"],
            "$ref": f"#/$defs/{definition}",
            "$defs": BUNDLE["$defs"],
        }
    )


def valid_geometry() -> dict:
    return {
        "preferred_regions": ["primary", "inspector"],
        "preferred_adjacency": ["contribution:pi-session"],
        "minimum_span": 3,
        "maximum_span": 8,
        "preferred_order": 10,
        "merge_policy": "compatible",
        "tab_policy": "preferred",
        "inspector_side": "profile_default",
    }


def valid_authority() -> dict:
    project_root_key = {
        "scope_kind": "project",
        "scope_id": "project:focusa",
        "root_path": "/example/focusa",
        "canonical_name": "Focusa",
        "fingerprint": "host-a:worktree-main",
    }
    workstream = {
        "scope": {"scope_kind": "project", "scope_key": project_root_key},
        "workstream_id": "ws:mission-canvas",
    }
    attachment = {
        "workstream": workstream,
        "continuity_id": "continuity:mission-canvas",
        "instance_id": "instance:pi",
        "session_id": "session:pi",
        "attachment_id": "attachment:pi",
        "workspace_binding_id": "workspace:mission-canvas",
    }
    return {
        "workstream": workstream,
        "continuity_id": "continuity:mission-canvas",
        "attachment": attachment,
        "workspace_binding_id": "workspace:mission-canvas",
        "runtime_object": {"runtime_kind": "pi_session", "runtime_id": "session:pi"},
        "work_surface_id": "surface:pi",
    }


def valid_ref(kind: str = "work_surface", ref: str = "surface:pi") -> dict:
    return {"kind": kind, "ref": ref, "revision": 7, "freshness": "current"}


def valid_candidate() -> dict:
    return {
        "contribution_id": "contribution:pi-session",
        "kind": "focused_work_surface",
        "semantic_binding_id": "semantic:pi-session",
        "renderer_binding_id": "renderer:pi-session@v1",
        "priority": 100,
        "applicable_profile_ids": ["software", "legal", "markets", "research"],
        "applicable_activity_mode_ids": ["overview", "sessions", "tasks"],
        "active_work_surface_relationships": ["focused"],
        "geometry": valid_geometry(),
        "canonical_content_refs": [valid_ref()],
        "required_capabilities": ["pi_session_stream"],
        "required_permissions": ["session:read"],
        "required_operations": ["focusa.agent_execution.prompt"],
        "preference_ref": "layout-memory:software:overview",
        "configuration_ref": None,
        "candidate_revision": 4,
    }


def valid_context() -> dict:
    return {
        **valid_authority(),
        "workspace_profile_id": "software",
        "workspace_profile_revision": 2,
        "activity_mode_id": "overview",
        "activity_mode_revision": 3,
        "focused_work_surface_id": "surface:pi",
        "open_work_surface_ids": ["surface:pi"],
        "pinned_work_surface_ids": [],
        "canonical_read_model_revision": 41,
        "available_operations": ["focusa.agent_execution.prompt"],
        "capabilities": ["pi_session_stream"],
        "permissions": ["session:read"],
        "viewport": {
            "class": "standard",
            "css_width": 1280,
            "css_height": 800,
            "platform": "Linux",
            "device_pixel_ratio": 1,
            "zoom_percent": 100,
            "high_contrast": False,
            "reduced_motion": False,
            "reduced_transparency": False,
            "text_scale_percent": 100,
        },
        "project_constraint_refs": ["constraint:no-dead-chrome"],
        "user_preference_ref": "preferences:operator",
        "resolver_rule_revision": "adaptive-composition:v1",
        "observed_at": "2026-07-30T12:00:00Z",
    }


def valid_diagnostic() -> dict:
    return {
        "contribution_id": "contribution:empty-work-rail",
        "reason": "no_relevant_content",
        "rule_revision": "adaptive-composition:v1",
        "projection_revision": 12,
        "canonical_input_refs": [valid_ref("work_rail", "work-rail:current")],
        "details_ref": "diagnostics:omission:1",
        "observed_at": "2026-07-30T12:00:00Z",
    }


def valid_resolved() -> dict:
    return {
        "contribution_id": "contribution:pi-session",
        "kind": "focused_work_surface",
        "semantic_binding_id": "semantic:pi-session",
        "renderer_binding_id": "renderer:pi-session@v1",
        "data_ref": valid_ref(),
        "operation_ids": ["focusa.agent_execution.prompt"],
        "authority": {
            "canonical_owner": "Focusa Core",
            "mutation_owner": "Pi AgentExecutionAdapter",
            **valid_authority(),
            "read_only": False,
            "approval_required": False,
            "contention_ref": None,
        },
        "freshness": {
            "status": "current",
            "observed_at": "2026-07-30T12:00:00Z",
            "stale_reason": None,
            "refresh_operation_id": None,
        },
        "resolved_geometry": valid_geometry(),
        "accessibility": {
            "label": "Active Pi session",
            "description": "Live transcript and governed prompt controls",
            "landmark_role": "region",
            "focus_semantic_id": "focus:pi-session",
            "live_region": "off",
            "keyboard_operation_ids": ["focusa.agent_execution.prompt"],
        },
        "contribution_revision": 9,
        "evidence_refs": ["evidence:pi-session-binding"],
    }


subprocess.run(
    ["python3", "scripts/generate-spec135-mission-canvas-schemas.py", "--check"],
    cwd=ROOT,
    check=True,
)

assert BUNDLE["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert set(
    (
        "ContributionId",
        "ContributionKind",
        "CandidateContribution",
        "ContributionEligibilityContext",
        "EligibilityDecision",
        "OmissionDiagnostic",
        "ResolvedContribution",
    )
).issubset(BUNDLE["$defs"])

validator("ContributionId").validate("contribution:pi-session")
for invalid_id in ("pi-session", "contribution:", "contribution:UPPER", "contribution:has space"):
    try:
        validator("ContributionId").validate(invalid_id)
    except ValidationError:
        pass
    else:
        raise AssertionError(f"invalid contribution id accepted: {invalid_id}")

validator("CandidateContribution").validate(valid_candidate())
validator("ContributionEligibilityContext").validate(valid_context())
validator("OmissionDiagnostic").validate(valid_diagnostic())
validator("ResolvedContribution").validate(valid_resolved())

eligible = {
    "contribution_id": "contribution:pi-session",
    "outcome": "eligible",
    "omission": None,
    "merged_into_contribution_id": None,
    "rule_revision": "adaptive-composition:v1",
    "projection_revision": 12,
    "evidence_refs": [],
}
validator("EligibilityDecision").validate(eligible)

omitted = copy.deepcopy(eligible)
omitted.update(
    contribution_id="contribution:empty-work-rail",
    outcome="omitted",
    omission=valid_diagnostic(),
)
validator("EligibilityDecision").validate(omitted)

invalid_omitted = copy.deepcopy(omitted)
invalid_omitted["omission"] = None
try:
    validator("EligibilityDecision").validate(invalid_omitted)
except ValidationError:
    pass
else:
    raise AssertionError("omitted contribution accepted without non-null omission diagnostic")

invalid_candidate = valid_candidate()
invalid_candidate["unknown_visual_slot"] = "right-card-3"
try:
    validator("CandidateContribution").validate(invalid_candidate)
except ValidationError:
    pass
else:
    raise AssertionError("candidate accepted unknown fixed-slot field")

invalid_context = valid_context()
invalid_context["viewport"]["platform"] = "macOS-only"
try:
    validator("ContributionEligibilityContext").validate(invalid_context)
except ValidationError:
    pass
else:
    raise AssertionError("eligibility context accepted unsupported platform")

assert valid_geometry()["minimum_span"] <= valid_geometry()["maximum_span"]

layout = {
    "node_id": "layout:root",
    "kind": "split",
    "orientation": "horizontal",
    "ratio": 0.7,
    "children": [
        {"node_id": "layout:primary", "kind": "single", "contribution_id": "contribution:pi-session"},
        {
            "node_id": "layout:inspector",
            "kind": "single",
            "contribution_id": "contribution:focusa-inspector",
        },
    ],
}
validator("LayoutNode").validate(layout)
invalid_layout = copy.deepcopy(layout)
invalid_layout["children"] = invalid_layout["children"][:1]
try:
    validator("LayoutNode").validate(invalid_layout)
except ValidationError:
    pass
else:
    raise AssertionError("split layout accepted fewer than two children")

profile = {
    "profile_id": "software",
    "revision": 2,
    "display_name": "Software Engineering",
    "candidate_contribution_ids": ["contribution:pi-session", "contribution:focusa-inspector"],
    "density": "standard",
    "terminology_registry_ref": "registry:terminology:software",
    "renderer_registry_ref": "registry:renderer:software",
    "domain_semantic_binding_registry_ref": "registry:semantics:software",
    "viability_rule_revision": "profile-viability:v1",
    "installed": True,
}
activity = {
    "activity_mode_id": "overview",
    "revision": 1,
    "display_name": "Overview",
    "candidate_contribution_ids": ["contribution:pi-session", "contribution:focusa-inspector"],
    "terminology_overrides_ref": None,
    "viability_rule_revision": "activity-viability:v1",
}
registry = {
    "registry_kind": "WorkspaceProfileRegistry",
    "entry_id": "profile:software",
    "revision": 2,
    "schema_ref": "workspace-profile.schema.json",
    "payload_ref": "profile:software@2",
    "required_capabilities": [],
    "required_permissions": [],
    "enabled": True,
    "supersedes_entry_id": None,
}
validator("WorkspaceProfile").validate(profile)
validator("ActivityMode").validate(activity)
validator("RegistryEntry").validate(registry)

inspector = valid_resolved()
inspector.update(
    contribution_id="contribution:focusa-inspector",
    kind="inspector",
    semantic_binding_id="semantic:focusa-inspector",
    renderer_binding_id="renderer:focusa-inspector@v1",
    contribution_revision=3,
)
projection = {
    "schema": "focusa.resolved_workspace_projection.v1",
    **valid_authority(),
    "workspace_profile_id": "software",
    "workspace_profile_revision": 2,
    "activity_mode_id": "overview",
    "activity_mode_revision": 1,
    "focused_work_surface_id": "surface:pi",
    "canonical_read_model_revision": 41,
    "candidate_contribution_ids": [
        "contribution:pi-session",
        "contribution:focusa-inspector",
        "contribution:empty-work-rail",
    ],
    "eligible_contributions": [valid_resolved(), inspector],
    "omission_diagnostics": [valid_diagnostic()],
    "layout_tree": layout,
    "operation_bindings": [
        {
            "operation_id": "focusa.agent_execution.prompt",
            "target_contribution_id": "contribution:pi-session",
            "enabled": True,
            "authority_ref": "authority:pi-session",
            "confirmation": "none",
            "disabled_reason_ref": None,
        }
    ],
    "focused_semantic_target": "focus:pi-session",
    "projection_revision": 12,
    "layout_revision": 5,
    "durable_event_cursor": "event:41",
    "projection_digest": "sha256:" + "0" * 64,
    "resolved_at": "2026-07-30T12:00:00Z",
    "evidence_refs": [],
    "receipt_refs": [],
}
validator("ResolvedWorkspaceProjection").validate(projection)

candidate_ids = set(projection["candidate_contribution_ids"])
eligible_ids = {entry["contribution_id"] for entry in projection["eligible_contributions"]}
omitted_ids = {entry["contribution_id"] for entry in projection["omission_diagnostics"]}
assert eligible_ids.isdisjoint(omitted_ids)
assert candidate_ids == eligible_ids | omitted_ids


def canonical_digest(value: dict) -> str:
    normalized = copy.deepcopy(value)
    normalized.pop("projection_digest", None)
    normalized.pop("resolved_at", None)
    payload = json.dumps(normalized, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    return "sha256:" + hashlib.sha256(payload).hexdigest()


first_digest = canonical_digest(projection)
second_digest = canonical_digest(json.loads(json.dumps(projection, sort_keys=False)))
assert first_digest == second_digest
validator("ProjectionDigest").validate(first_digest)

invalid_projection = copy.deepcopy(projection)
invalid_projection["unknown_client_panel"] = {"title": "Unavailable"}
try:
    validator("ResolvedWorkspaceProjection").validate(invalid_projection)
except ValidationError:
    pass
else:
    raise AssertionError("projection accepted client-invented panel")

# CORE-009 identity migration fixtures are consumed by the same generated
# projection contract gate. Migration is core-owned; this check only validates
# that its deterministic Workstream/Attachment expectations remain transport-safe.
identity_fixture_dir = ROOT / "tests/fixtures/spec158-mission-canvas-identity"
identity_fixture_paths = sorted(identity_fixture_dir.glob("*.json"))
assert {path.stem for path in identity_fixture_paths} == {
    "legacy_fixture",
    "ambiguous_fixture",
    "cross_workstream_fixture",
}
for identity_fixture_path in identity_fixture_paths:
    identity_fixture = json.loads(identity_fixture_path.read_text())
    assert identity_fixture["schema"] == "focusa.spec158.mission_canvas_identity_migration_fixture.v1"
    assert identity_fixture["generated_contracts"]["generated_operation"] is None
    for record in identity_fixture["records"]:
        validator("LegacyExactScopeCompatibilityInput").validate(record["legacy"])
        for candidate in record.get("candidate_workstreams", []):
            validator("WorkstreamKey").validate(candidate)
        for mapping in record.get("migration_candidates", []):
            mapping_workstream = {
                "scope": mapping["scope_ref"],
                "workstream_id": mapping["workstream_id"],
            }
            validator("WorkstreamKey").validate(mapping_workstream)
    for migrated in identity_fixture["expected"]["migrated"]:
        validator("WorkstreamAuthorityContext").validate(migrated["authority"])
        authority = migrated["authority"]
        assert authority["attachment"]["workstream"] == authority["workstream"]
        assert authority["continuity_id"] == authority["attachment"]["continuity_id"]
        # Identity migration cannot invent a focusable Work Surface or runtime.
        assert authority["work_surface_id"] is None
        assert authority["runtime_object"] is None
    if identity_fixture_path.stem == "ambiguous_fixture":
        assert identity_fixture["expected"]["migrated"] == []
        assert identity_fixture["expected"]["quarantined"][0]["reason"] == "multiple_candidate_workstreams"
        assert len(identity_fixture["expected"]["quarantined"][0]["candidate_workstreams"]) == 2

print("Spec 135 resolved projection contract foundation and CORE-009 identity fixtures: PASS")
