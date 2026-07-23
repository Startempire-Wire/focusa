#!/usr/bin/env python3
"""SPEC135-C3 static/contract proof for canonical claims, contradiction decisions, and reactive projection."""

import json
from pathlib import Path
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
types = (ROOT / "crates/focusa-core/src/types.rs").read_text()
reducer = (ROOT / "crates/focusa-core/src/reducer.rs").read_text()
api = (ROOT / "crates/focusa-api/src/routes/context_claims.rs").read_text()
registry = json.loads((BUNDLE / "operation-registry.json").read_text())
openapi = json.loads((BUNDLE / "openapi-3.0.3.json").read_text())
bindings = json.loads((BUNDLE / "ui-action-bindings.fixture.json").read_text())
request = json.loads(
    (BUNDLE / "json-schema/focusa.context_graph_mutation.request.v1.json").read_text()
)
result = json.loads(
    (BUNDLE / "json-schema/focusa.context_graph_mutation_result.v1.json").read_text()
)
read_result = json.loads(
    (BUNDLE / "json-schema/focusa.context_graph.v1.json").read_text()
)
scenario_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_scenario.v1.schema.json").read_text()
)
result_schema = json.loads(
    (BUNDLE / "uiai.focusa_ui_eval_result.v1.schema.json").read_text()
)
scenario = json.loads(
    (BUNDLE / "uiai-eval.c3-context-claims.scenario.json").read_text()
)
ui_result = json.loads((BUNDLE / "uiai-eval.c3-context-claims.result.json").read_text())
proof = json.loads((BUNDLE / "spec135-c3-context-claim-graph-proof.json").read_text())
ts = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()
go = (ROOT / "packages/generated/spec135/go/client.gen.go").read_text()

for marker in [
    "ContextClaimRecord",
    "ContextContradictionRecord",
    "ContextDecisionRecord",
    "ReactiveContextProjection",
    "ContextClaimProposed",
    "ContextContradictionResolved",
]:
    assert marker in types, marker
for marker in [
    "refresh_reactive_context",
    "accepted_claim_refs",
    "blocked_claim_refs",
    "unresolved_contradiction_refs",
    "ContextClaimReviewed",
    "ContextContradictionOpened",
]:
    assert marker in reducer, marker
for marker in [
    "propose_claim",
    "review_claim",
    "open_contradiction",
    "resolve_contradiction",
    "expected_state_version",
    "write_serial_lock",
    "drop(_writer)",
    "replay_match",
]:
    assert marker in api, marker

ops = {op["operation_id"]: op for op in registry["operations"]}
for operation_id, path, method in [
    ("focusa.context.graph.read", "/v1/context/graph", "GET"),
    ("focusa.context.graph.mutate", "/v1/context/graph/mutate", "POST"),
]:
    op = ops[operation_id]
    assert op["path"] == path and op["method"] == method
    assert (
        op["scope"]["project_scoped"]
        and op["scope"]["workstream_scoped"]
        and op["scope"]["attachment_scoped"]
    )
    assert any(binding["action_id"] == operation_id for binding in bindings["bindings"])
    assert path in openapi["paths"]

for schema in [request, result, read_result]:
    Draft202012Validator.check_schema(schema)
assert request["properties"]["action"]["enum"] == [
    "propose_claim",
    "review_claim",
    "open_contradiction",
    "resolve_contradiction",
]
props = result["properties"]
assert props["canonical"]["const"] is True
claim = props["claims"]["items"]["properties"]
assert claim["status"]["enum"] == [
    "candidate",
    "accepted",
    "contradicted",
    "rejected",
    "superseded",
]
projection = props["projection"]["properties"]
for key in [
    "accepted_claim_refs",
    "candidate_claim_refs",
    "blocked_claim_refs",
    "unresolved_contradiction_refs",
]:
    assert key in projection
assert "focusa_context_graph_mutation_request_v1" in ts
assert "FocusaContextGraphMutate" in go and "FocusaContextGraphRead" in go
Draft202012Validator(scenario_schema).validate(scenario)
Draft202012Validator(result_schema).validate(ui_result)
assert ui_result["status"] == "passed" and all(
    step["status"] == "passed" for step in ui_result["step_results"]
)
assert (
    proof["status"] == "verified"
    and proof["runtime_proof"]["blocked_after_resolution"] == 0
)
assert proof["runtime_proof"]["idempotent_replay"] == "no_op"

print(
    f"Spec 135 C3 Context claim graph: PASS ({len(ops)} operations; canonical claims/decisions/reactive projection)"
)
