#!/usr/bin/env python3
"""SPEC135-RI1 static contract proof for grounded Role Profile governance."""

import json
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
SCHEMAS = [
    "focusa.project_agent_role_profile_list.request.v1",
    "focusa.project_agent_role_profile_list.v1",
    "focusa.project_agent_role_profile_draft.request.v1",
    "focusa.project_agent_role_profile_review.request.v1",
    "focusa.project_agent_role_profile_mutation_result.v1",
]
schemas = {
    name: json.loads((BUNDLE / "json-schema" / f"{name}.json").read_text())
    for name in SCHEMAS
}
for schema in schemas.values():
    Draft202012Validator.check_schema(schema)
eval_scenario_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_scenario.v1.schema.json").read_text()
)
eval_result_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_result.v1.schema.json").read_text()
)
eval_scenario = json.loads(
    (BUNDLE / "uiai-eval.ri1-role-profile.scenario.json").read_text()
)
eval_result = json.loads(
    (BUNDLE / "uiai-eval.ri1-role-profile.result.json").read_text()
)
Draft202012Validator(eval_scenario_schema).validate(eval_scenario)
Draft202012Validator(eval_result_schema).validate(eval_result)
proof = json.loads((BUNDLE / "spec135-ri1-role-profile-proof.json").read_text())
assert proof["status"] == "passed" and proof["requirement_id"] == "SPEC135-RI1"
assert proof["generated_ui_proof"]["grants_permissions"] is False

profile_schema = schemas["focusa.project_agent_role_profile_mutation_result.v1"][
    "properties"
]["profile"]
assert profile_schema["properties"]["grants_permissions"]["const"] is False
assert profile_schema["properties"]["status"]["enum"] == [
    "draft",
    "pending_operator",
    "approved",
    "superseded",
]
for field in (
    "original_seed",
    "primary_responsibilities",
    "secondary_responsibilities",
    "non_responsibilities",
    "forbidden_assumptions",
    "grounding",
    "assumptions",
    "redlines",
    "permission_profile_refs",
    "review",
):
    assert field in profile_schema["properties"], field

draft_schema = schemas["focusa.project_agent_role_profile_draft.request.v1"]
assert draft_schema["properties"]["permission_assertions"]["maxItems"] == 0
assert draft_schema["x-focusa-at-least-one-canonical-grounding-ref"] == [
    "context_artifact_refs",
    "context_claim_refs",
]
assert schemas["focusa.project_agent_role_profile_review.request.v1"]["properties"][
    "decision"
]["enum"] == ["approve", "reject", "defer"]

registry = json.loads((BUNDLE / "operation-registry.json").read_text())
operations = {item["operation_id"]: item for item in registry["operations"]}
for operation_id in (
    "focusa.role_profile.list",
    "focusa.role_profile.draft",
    "focusa.role_profile.review",
):
    operation = operations[operation_id]
    assert operation["canonical"] is True
    assert operation["scope"]["required_keys"] == [
        "project_root",
        "continuity_id",
        "attachment_id",
    ]
assert (
    operations["focusa.role_profile.review"]["control"]["confirmation"]
    == "consequential"
)

bindings = json.loads((BUNDLE / "ui-action-bindings.fixture.json").read_text())
binding = next(
    item
    for item in bindings["bindings"]
    if item["action_id"] == "focusa.role_profile.review"
)
assert binding["control"]["confirmation"] == "consequential"
assert binding["control"]["receipt_required"] is True
assert binding["scope"]["required_keys"] == [
    "project_root",
    "continuity_id",
    "attachment_id",
]

openapi = json.loads((BUNDLE / "openapi-3.0.3.json").read_text())
for route in (
    "/v1/roles/profiles",
    "/v1/roles/profiles/draft",
    "/v1/roles/profiles/review",
):
    assert route in openapi["paths"]

core = (ROOT / "crates/focusa-core/src/types.rs").read_text()
reducer = (ROOT / "crates/focusa-core/src/reducer.rs").read_text()
route = (ROOT / "crates/focusa-api/src/routes/role_profiles.rs").read_text()
e2e = (ROOT / "tests/spec135_role_profile_e2e_test.py").read_text()
ui = (ROOT / "packages/a2ui-renderer/proof/role-profile.ts").read_text()
ui_html = (ROOT / "packages/a2ui-renderer/proof/role-profile.html").read_text()
ts = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()
for marker in (
    "ProjectAgentRoleProfile",
    "RoleProfileGrounding",
    "RoleAssumptionRecord",
    "RoleRedlineRecord",
    "RoleReviewDecision",
    "ProjectRoleProfileRevised",
    "project_role_profiles",
):
    assert marker in core, marker
for marker in (
    "project role profiles cannot grant permission",
    "requires an operator seed and Context grounding",
    "status does not match its explicit review",
):
    assert marker in reducer, marker
for marker in (
    "permission_assertions",
    "responsibility cannot grant",
    "contains_permission_grant",
    "cannot contain operational permission grants",
    "Context source, Workspace Artifact, or claim grounding",
    "original_seed is immutable",
    "requires an explicit before/after redline",
    "approval requires resolved questions",
    "RoleReviewDecision::Approve",
    "expected_state_version",
    "idempotency_key",
):
    assert marker in route, marker
for marker in (
    "bad_grounding",
    "bad_permission",
    "pending_operator",
    '"approve"',
    '"defer"',
    '"reject"',
    "restart",
):
    assert marker in e2e, marker
for marker in (
    "FocusaRoleSeed",
    "FocusaRoleDraft",
    "FocusaGroundingSources",
    "FocusaRedline",
    "FocusaApprovalCard",
    "Responsibility is not permission",
    "Approve Role Profile",
    "Defer",
    "Reject",
    "roleReviewBinding",
):
    assert marker in ui, marker
assert "Ground and approve the project role" in ui_html
assert "focusa_project_agent_role_profile_draft_request_v1" in ts
assert 'operations["focusa.role_profile.review"]' in ts

print(
    "Spec 135 RI1 Role Profile: PASS (Context grounding, assumptions, redline, permission separation, explicit durable approval)"
)
