#!/usr/bin/env python3
"""Validate F12 real Context mutation through the complete generated-UI stack."""

from __future__ import annotations

import json
from pathlib import Path
from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
registry = json.loads((BUNDLE / "operation-registry.json").read_text())
bindings = json.loads((BUNDLE / "ui-action-bindings.fixture.json").read_text())
openapi = json.loads((BUNDLE / "openapi-3.0.3.json").read_text())
seed = json.loads((BUNDLE / "spec135-alpha0-context-seed-proof.json").read_text())
scenario_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_scenario.v1.schema.json").read_text()
)
result_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_result.v1.schema.json").read_text()
)
scenario = json.loads(
    (BUNDLE / "uiai-eval.alpha0-context-commit.scenario.json").read_text()
)
result = json.loads(
    (BUNDLE / "uiai-eval.alpha0-context-commit.result.json").read_text()
)
core_types = (ROOT / "crates/focusa-core/src/types.rs").read_text()
reducer = (ROOT / "crates/focusa-core/src/reducer.rs").read_text()
route = (ROOT / "crates/focusa-api/src/routes/context_sources.rs").read_text()
proof = (ROOT / "packages/a2ui-renderer/proof/context-commit.ts").read_text()
ts_client = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()
go_client = (ROOT / "packages/generated/spec135/go/client.gen.go").read_text()

operations = {item["operation_id"]: item for item in registry["operations"]}
operation = operations["focusa.context.source.commit"]
assert operation["method"] == "POST"
assert operation["path"] == "/v1/context/sources/commit"
assert operation["canonical"] is True
assert operation["scope"] == {
    "required_keys": ["project_root", "continuity_id", "attachment_id"],
    "project_scoped": True,
    "workstream_scoped": True,
    "attachment_scoped": True,
}
assert operation["control"]["mode"] == "commit"
assert operation["control"]["idempotency_required"] is True
assert operation["control"]["optimistic_concurrency_required"] is True
assert operation["control"]["receipt_required"] is True
assert operation["control"]["permission_scopes"] == ["context:write"]
assert operation["ui"]["allowed_in_generated_ui"] is True
binding = next(
    item
    for item in bindings["bindings"]
    if item["action_id"] == operation["operation_id"]
)
assert binding["scope"]["required_keys"] == [
    "project_root",
    "continuity_id",
    "attachment_id",
]
assert (
    binding["control"]["idempotency_required"]
    and binding["control"]["receipt_required"]
)

post = openapi["paths"]["/v1/context/sources/commit"]["post"]
assert post["operationId"] == operation["operation_id"]
assert post["x-focusa-scope-keys"] == ["project_root", "continuity_id", "attachment_id"]
assert (
    post["x-focusa-idempotency"]
    and post["x-focusa-concurrency"]
    and post["x-focusa-receipt"]
)
for schema_id in (
    "focusa.context_source_commit.request.v1",
    "focusa.context_source_commit_result.v1",
    "focusa.context_source_list.request.v1",
    "focusa.context_source_list.v1",
):
    schema = json.loads((BUNDLE / "json-schema" / f"{schema_id}.json").read_text())
    assert schema["$schema"] == "https://json-schema.org/draft/2020-12/schema"
assert '"/v1/context/sources/commit"' in ts_client
assert "FocusaContextSourceCommit" in go_client

for marker in (
    "pub struct ContextSourceRecord",
    "pub struct ContextSourceEvidence",
    "pub struct ContextSourceReceipt",
    "ContextSourceCommitted",
    "pub context_sources: Vec<ContextSourceRecord>",
):
    assert marker in core_types
for marker in (
    "Context receipt version mismatch",
    "duplicate Context source commit",
    "context_source_commit_is_canonical_scoped_and_idempotency_guarded",
):
    assert marker in reducer
for marker in (
    "Action::EmitEvent",
    "FocusaEvent::ContextSourceCommitted",
    "expected_state_version",
    "idempotency_key",
    "evidence_ref",
    "receipt_ref",
    "ToolResultV1::success",
    "ToolResultV1::failure",
):
    assert marker in route
for marker in (
    'from "@focusa/spec135-client"',
    "ui-action-bindings.fixture.json",
    'operationId = "focusa.context.source.commit"',
    "allowedActionNames: new Set([binding.action_id])",
    'client.POST("/v1/context/sources/commit"',
    "FocusaEvidenceSummary",
):
    assert marker in proof

Draft202012Validator(scenario_schema, format_checker=FormatChecker()).validate(scenario)
Draft202012Validator(result_schema, format_checker=FormatChecker()).validate(result)
assert result["status"] == "passed"
assert all(step["status"] == "passed" for step in result["step_results"])
assert result["focusa_evidence_refs"][0] == seed["evidence_ref"]
assert result["receipt_refs"] == [seed["receipt_ref"]]
assert seed["status"] == "verified"
assert seed["runtime_proof"]["source_count_after_restart"] == 1
assert seed["runtime_proof"]["idempotent_resume_status"] == "no_op"
assert seed["runtime_proof"]["durable_event_type"] == "ContextSourceCommitted"

print(
    "Spec 135 F12 generated Context: PASS (binding/client/API/reducer/event/restart/Evidence/Receipt/UIAI)"
)
