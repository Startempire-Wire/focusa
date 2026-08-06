#!/usr/bin/env python3
"""Workstream identity contract and hostile Desktop-boundary checks for ID-010."""
from __future__ import annotations

import copy
import json
from pathlib import Path

from jsonschema import Draft202012Validator, ValidationError

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = json.loads((ROOT / "schemas/spec135/mission-canvas/composition-bundle.v1.schema.json").read_text())
FIXTURE = json.loads(
    (ROOT / "apps/desktop/tests/fixtures/mission-canvas/populated-projection.json").read_text()
)
MODEL_SOURCE = (ROOT / "crates/focusa-core/src/mission_canvas/model.rs").read_text()
API_SOURCE = (ROOT / "crates/focusa-api/src/routes/mission_canvas.rs").read_text()


def validate(name: str, value: object) -> None:
    Draft202012Validator(
        {"$schema": BUNDLE["$schema"], "$ref": f"#/$defs/{name}", "$defs": BUNDLE["$defs"]}
    ).validate(value)


def authority(value: dict) -> dict:
    return {
        "workstream": value["workstream"],
        "continuity_id": value.get("continuity_id"),
        "attachment": value.get("attachment"),
        "workspace_binding_id": value.get("workspace_binding_id"),
        "runtime_object": value.get("runtime_object"),
        "work_surface_id": value.get("work_surface_id"),
    }


def exact_authority(value: dict) -> None:
    workstream = value.get("workstream")
    if not isinstance(workstream, dict) or not workstream.get("workstream_id"):
        raise AssertionError("missing WorkstreamKey")
    attachment = value.get("attachment")
    if attachment is None:
        return
    if attachment.get("workstream") != workstream:
        raise AssertionError("foreign AttachmentKey Workstream owner")
    if value.get("continuity_id") not in (None, attachment.get("continuity_id")):
        raise AssertionError("continuity does not belong to AttachmentKey")
    if value.get("workspace_binding_id") not in (None, attachment.get("workspace_binding_id")):
        raise AssertionError("workspace binding does not belong to AttachmentKey")


assert "pub struct MissionCanvasAuthorityContext" in MODEL_SOURCE
assert "pub type WorkstreamAuthorityContext" in MODEL_SOURCE
assert "pub workstream: WorkstreamKey" in MODEL_SOURCE
assert "#[serde(flatten)]" in MODEL_SOURCE
assert "validate_owner" in MODEL_SOURCE
for forbidden in ("pub project_root", "pub session_id: String", "pub attachment_id: String", "pub working_subpath_id"):
    assert forbidden not in MODEL_SOURCE
assert "pub workstream: String" in API_SOURCE
assert "parse_query_json::<WorkstreamKey>" in API_SOURCE
assert "project_root" not in API_SOURCE

workstream = FIXTURE["workstream"]
attachment = FIXTURE["attachment"]
validate("WorkstreamKey", workstream)
validate("AttachmentKey", attachment)
validate("WorkstreamAuthorityContext", authority(FIXTURE))
exact_authority(authority(FIXTURE))

# Canonical projections and receipts carry the exact WorkstreamKey, not a flat
# project/continuity pair or a presentation-selected owner.
validate("ResolvedWorkspaceProjection", FIXTURE)
receipt = {
    "receipt_id": "recomposition-receipt:id-010",
    "workstream": copy.deepcopy(workstream),
    "accepted": True,
    "projection_revision": FIXTURE["projection_revision"],
    "layout_revision": FIXTURE["layout_revision"],
    "projection_digest": FIXTURE["projection_digest"],
    "event_cursor": FIXTURE["durable_event_cursor"],
    "evidence_id": "recomposition-evidence:id-010",
    "idempotency_key": "id-010:receipt",
    "issued_at": "2026-08-06T00:00:00Z",
}
validate("RecompositionReceipt", receipt)
assert receipt["workstream"] == workstream

layout_result = {
    "workstream": copy.deepcopy(workstream),
    "command_id": "layout-command:id-010",
    "accepted": True,
    "projection_revision": FIXTURE["projection_revision"] + 1,
    "layout_revision": FIXTURE["layout_revision"] + 1,
    "projection_digest": FIXTURE["projection_digest"],
    "event_cursor": "event:id-010",
}
validate("LayoutMutationResult", layout_result)

# Missing identity and legacy-only authority never validate as a canonical
# projection.  Compatibility input is a separate, explicitly named shape.
missing_workstream = copy.deepcopy(FIXTURE)
missing_workstream.pop("workstream")
try:
    validate("ResolvedWorkspaceProjection", missing_workstream)
except ValidationError:
    pass
else:
    raise AssertionError("projection accepted missing WorkstreamKey")

legacy_only = {
    "project_root": "/example/focusa",
    "continuity_id": "continuity:legacy",
    "session_id": "session:legacy",
    "attachment_id": "attachment:legacy",
}
validate("LegacyExactScopeCompatibilityInput", legacy_only)
try:
    validate("WorkstreamAuthorityContext", legacy_only)
except ValidationError:
    pass
else:
    raise AssertionError("legacy compatibility input granted canonical authority")

# A foreign Workstream nested in an otherwise well-shaped AttachmentKey is a
# hostile identity, not an alternate presentation or a repair candidate.
foreign = copy.deepcopy(FIXTURE)
foreign["attachment"]["workstream"]["workstream_id"] = "ws:foreign"
try:
    exact_authority(authority(foreign))
except AssertionError as error:
    assert "foreign" in str(error)
else:
    raise AssertionError("foreign AttachmentKey Workstream was accepted")

continuity_foreign = copy.deepcopy(FIXTURE)
continuity_foreign["continuity_id"] = "continuity:foreign"
try:
    exact_authority(authority(continuity_foreign))
except AssertionError as error:
    assert "continuity" in str(error)
else:
    raise AssertionError("foreign ContinuityId was accepted")

binding_foreign = copy.deepcopy(FIXTURE)
binding_foreign["workspace_binding_id"] = "workspace:foreign"
try:
    exact_authority(authority(binding_foreign))
except AssertionError as error:
    assert "binding" in str(error)
else:
    raise AssertionError("foreign WorkspaceBindingId was accepted")

for forbidden in ("cwd", "current_tab", "latest_record", "last_active", "nearest_candidate"):
    assert forbidden not in json.dumps(FIXTURE).lower()

registry = json.loads(
    (ROOT / "docs/contracts/spec135/mission-canvas-v1/operation-registry.json").read_text()
)
for operation in registry["operations"]:
    assert operation["scope_required"] == ["workstream"]
    assert "project_root" not in operation["scope_required"]
    assert "continuity_id" in operation["scope_optional"]
    assert operation["authority_chain"][0:3] == ["scope_ref", "project_root_key", "workstream_id"]

print("Spec 158 Mission Canvas Workstream identity contract: PASS")
