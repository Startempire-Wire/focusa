#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
B = R / "docs/contracts/spec135/generated-contract-v1"
route = (R / "crates/focusa-api/src/routes/interview_sessions.rs").read_text()
capabilities = (R / "crates/focusa-api/src/routes/agent_capabilities.rs").read_text()
ts = (R / "packages/generated/spec135/typescript/schema.d.ts").read_text()
registry = json.loads((B / "operation-registry.json").read_text())
openapi = json.loads((B / "openapi-3.0.3.json").read_text())
schema = json.loads(
    (B / "json-schema/focusa.interview_closure_package.v1.json").read_text()
)

for marker in (
    "/v1/interviews/closure-package",
    "focusa.interview_closure_package.v1",
    "glossary_candidates",
    "adr_candidates",
    "compendium",
    "approved_role_profile_ref",
    "receipt:interview-closure:",
    "Closed | ProjectInterviewSessionStatus::ReadyForSpec",
):
    assert marker in route
assert "take(64)" in route and "take(128)" in route
assert "focusa.interview.closure_package.get" in capabilities
assert any(
    row["operation_id"] == "focusa.interview.closure_package.get"
    for row in registry["operations"]
)
assert openapi["paths"]["/v1/interviews/closure-package"]["get"]
assert schema["x-focusa-schema-id"] == "focusa.interview_closure_package.v1"
assert 'operations["focusa.interview.closure_package.get"]' in ts
assert not (R / "packages/generated/spec135/go").exists()
print("Spec 135 RI4 governed Role/Interview closure package: PASS")
