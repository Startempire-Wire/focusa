#!/usr/bin/env python3
"""Workstream identity contract and hostile Desktop-boundary checks for ID-010."""
from __future__ import annotations

import copy
import hashlib
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



# CORE-009 migration fixtures -------------------------------------------------
# These fixtures are deliberately exercised at the generated-contract boundary.
# They describe the core-owned migration mapping and quarantine result; they do
# not add a client-side resolver, operation, route, renderer, or Work Surface.
IDENTITY_FIXTURE_DIR = ROOT / "tests/fixtures/spec158-mission-canvas-identity"
IDENTITY_FIXTURE_SCHEMA = "focusa.spec158.mission_canvas_identity_migration_fixture.v1"
MIGRATION_MAPPING_SCHEMA = "focusa.workstream_migration_mapping.v1"


def canonical_json(value: object) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def workstream_key(mapping: dict) -> dict:
    return {
        "scope": copy.deepcopy(mapping["scope_ref"]),
        "workstream_id": mapping["workstream_id"],
    }


def mapping_is_proven_for(record: dict, mapping: dict) -> bool:
    required = (
        "schema",
        "source_refs",
        "scope_ref",
        "workstream_id",
        "confidence",
        "evidence_refs",
        "rationale",
        "approved_by",
        "approval_ref",
        "created_at",
    )
    if any(field not in mapping for field in required):
        return False
    if mapping["schema"] != MIGRATION_MAPPING_SCHEMA or mapping["confidence"] != "proven":
        return False
    if not mapping["source_refs"] or record["source_ref"] not in mapping["source_refs"]:
        return False
    if not mapping["evidence_refs"] or any(not ref.strip() for ref in mapping["evidence_refs"]):
        return False
    if not mapping["rationale"].strip() or not mapping["approval_ref"].strip():
        return False
    if mapping["approved_by"] not in ("migration-rule", "operator"):
        return False
    return True


class LegacyFixture:
    """Bounded test harness for the core migration expectations.

    The production migration owner is the typed core mapping/quarantine path
    (CORE-005 and Spec 158). This harness only turns checked-in fixture inputs
    into expected DTO-shaped values so the contract test can reject guessing.
    """

    @staticmethod
    def _quarantine(record: dict, reason: str, candidate_keys: list[dict]) -> dict:
        source_hash = "sha256:" + hashlib.sha256(
            canonical_json(record["legacy"]).encode("utf-8")
        ).hexdigest()
        return {
            "source_ref": record["source_ref"],
            "source_hash": source_hash,
            "payload_ref": record["payload_ref"],
            "reason": reason,
            "candidate_workstreams": copy.deepcopy(candidate_keys),
            "evidence_refs": copy.deepcopy(record["evidence_refs"]),
            "quarantined_at": record["quarantined_at"],
        }

    def migrate(self, fixture: dict) -> dict:
        migrated: list[dict] = []
        quarantined: list[dict] = []
        for record in fixture["records"]:
            mappings = record.get("migration_candidates", [])
            declared_candidates = record.get("candidate_workstreams", [])
            if len(mappings) != 1:
                candidate_keys = [workstream_key(mapping) for mapping in mappings]
                if not candidate_keys:
                    candidate_keys = copy.deepcopy(declared_candidates)
                reason = (
                    "multiple_candidate_workstreams"
                    if len(candidate_keys) > 1
                    else "missing_workstream_identity"
                )
                quarantined.append(self._quarantine(record, reason, candidate_keys))
                continue

            mapping = mappings[0]
            key = workstream_key(mapping)
            if not mapping_is_proven_for(record, mapping):
                quarantined.append(self._quarantine(record, "invalid_migration_mapping", [key]))
                continue
            mapped_scope = mapping["scope_ref"].get("scope_key", {})
            if mapped_scope.get("root_path") != record["legacy"]["project_root"]:
                quarantined.append(self._quarantine(record, "conflicting_project_roots", [key]))
                continue
            if declared_candidates and key not in declared_candidates:
                quarantined.append(self._quarantine(record, "conflicting_thread_lineage", [key]))
                continue

            binding = record.get("authority_binding")
            if not isinstance(binding, dict) or not binding.get("workspace_binding_id"):
                quarantined.append(self._quarantine(record, "missing_workstream_identity", [key]))
                continue

            legacy = record["legacy"]
            attachment = {
                "workstream": copy.deepcopy(key),
                "continuity_id": legacy["continuity_id"],
                "instance_id": legacy.get("instance_id") or "",
                "session_id": legacy["session_id"],
                "attachment_id": legacy["attachment_id"],
                "workspace_binding_id": binding["workspace_binding_id"],
            }
            authority = {
                "workstream": copy.deepcopy(key),
                "continuity_id": legacy["continuity_id"],
                "attachment": attachment,
                "workspace_binding_id": binding["workspace_binding_id"],
                "runtime_object": copy.deepcopy(binding.get("runtime_object")),
                "work_surface_id": copy.deepcopy(binding.get("work_surface_id")),
            }
            migrated.append({"source_ref": record["source_ref"], "authority": authority})
        return {"migrated": migrated, "quarantined": quarantined}


class AmbiguousFixture:
    def quarantine(self, fixture: dict) -> list[dict]:
        result = legacy_fixture.migrate(fixture)
        assert not result["migrated"], "ambiguous fixture must never migrate a row"
        return result["quarantined"]


legacy_fixture = LegacyFixture()
ambiguous_fixture = AmbiguousFixture()


fixture_paths = sorted(IDENTITY_FIXTURE_DIR.glob("*.json"))
assert {path.stem for path in fixture_paths} == {
    "legacy_fixture",
    "ambiguous_fixture",
    "cross_workstream_fixture",
}
fixtures = {path.stem: json.loads(path.read_text()) for path in fixture_paths}
for fixture in fixtures.values():
    assert fixture["schema"] == IDENTITY_FIXTURE_SCHEMA
    assert fixture["cardinal_translation_ref"] == "CARDINAL-135-SVELTE-001"
    assert fixture["generated_contracts"]["generated_operation"] is None
    assert fixture["fixture_call"] in {"legacy_fixture.migrate", "ambiguous_fixture.quarantine"}
    for contract_name in (
        "legacy_input",
        "workstream",
        "attachment",
        "authority",
    ):
        assert fixture["generated_contracts"][contract_name]
    for record in fixture["records"]:
        validate("LegacyExactScopeCompatibilityInput", record["legacy"])
        assert record["source_ref"]
        assert record["payload_ref"]
        assert record["evidence_refs"]
        for candidate in record.get("candidate_workstreams", []):
            validate("WorkstreamKey", candidate)
        for mapping in record.get("migration_candidates", []):
            validate("WorkstreamKey", workstream_key(mapping))
            assert mapping["scope_ref"] == workstream_key(mapping)["scope"]

for fixture_name, fixture in fixtures.items():
    if fixture_name == "ambiguous_fixture":
        actual_quarantine = ambiguous_fixture.quarantine(fixture)
        actual = {"migrated": [], "quarantined": actual_quarantine}
    else:
        actual = legacy_fixture.migrate(fixture)
    assert actual == fixture["expected"], f"non-deterministic expectation: {fixture_name}"

    for migrated in actual["migrated"]:
        validate("WorkstreamAuthorityContext", migrated["authority"])
        authority_value = migrated["authority"]
        exact_authority(authority_value)
        # A legacy row never manufactures runtime or Work Surface authority.
        # Those values may return only from an explicit canonical binding.
        assert authority_value["runtime_object"] is None
        assert authority_value["work_surface_id"] is None

    for quarantined in actual["quarantined"]:
        assert quarantined["reason"] in {
            "multiple_candidate_workstreams",
            "missing_workstream_identity",
            "invalid_migration_mapping",
            "conflicting_project_roots",
            "conflicting_thread_lineage",
        }
        assert quarantined["source_hash"].startswith("sha256:")
        for candidate in quarantined["candidate_workstreams"]:
            validate("WorkstreamKey", candidate)

# Hostile migration cases: flat identity is compatibility input only, foreign
# scope is quarantined, and two valid candidates are never resolved by order.
for source_fixture in (fixtures["legacy_fixture"], fixtures["cross_workstream_fixture"]):
    flat_only = copy.deepcopy(source_fixture)
    for record in flat_only["records"]:
        record["migration_candidates"] = []
        record["candidate_workstreams"] = []
    flat_result = legacy_fixture.migrate(flat_only)
    assert not flat_result["migrated"]
    assert {row["reason"] for row in flat_result["quarantined"]} == {"missing_workstream_identity"}

foreign_scope = copy.deepcopy(fixtures["legacy_fixture"])
foreign_scope["records"][0]["migration_candidates"][0]["scope_ref"]["scope_key"]["root_path"] = "/example/other"
foreign_result = legacy_fixture.migrate(foreign_scope)
assert not foreign_result["migrated"]
assert foreign_result["quarantined"][0]["reason"] == "conflicting_project_roots"

duplicate_candidates = copy.deepcopy(fixtures["legacy_fixture"])
duplicate = copy.deepcopy(duplicate_candidates["records"][0]["migration_candidates"][0])
duplicate["workstream_id"] = "ws:other"
duplicate_candidates["records"][0]["migration_candidates"].append(duplicate)
duplicate_result = legacy_fixture.migrate(duplicate_candidates)
assert not duplicate_result["migrated"]
assert duplicate_result["quarantined"][0]["reason"] == "multiple_candidate_workstreams"

missing_attachment_authority = copy.deepcopy(fixtures["legacy_fixture"])
missing_attachment_authority["records"][0]["authority_binding"] = None
missing_result = legacy_fixture.migrate(missing_attachment_authority)
assert not missing_result["migrated"]
assert missing_result["quarantined"][0]["reason"] == "missing_workstream_identity"

cross_migrated = legacy_fixture.migrate(fixtures["cross_workstream_fixture"])["migrated"]
cross_ids = [row["authority"]["workstream"]["workstream_id"] for row in cross_migrated]
assert cross_ids == ["ws:alpha", "ws:beta"]
assert len(set(cross_ids)) == 2
for row in cross_migrated:
    authority_value = row["authority"]
    assert authority_value["attachment"]["workstream"] == authority_value["workstream"]
    assert authority_value["continuity_id"] == authority_value["attachment"]["continuity_id"]

# Migration fixtures are identity-only. They cannot introduce a renderer,
# layout, contribution, route, or operation into the generated transport.
for fixture in fixtures.values():
    encoded = json.dumps(fixture).lower()
    for forbidden in ("current_tab", "latest_record", "last_active", "nearest_candidate", "default_workstream"):
        assert forbidden not in encoded
    for record in fixture["records"]:
        assert "work_surface_id" not in record["legacy"]
        assert "operation_id" not in record["legacy"]
        assert "renderer_binding_id" not in record["legacy"]

print("Spec 158 Mission Canvas Workstream identity contract and CORE-009 migration fixtures: PASS")
