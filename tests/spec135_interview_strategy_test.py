#!/usr/bin/env python3
"""Static and generated-contract gates for SPEC135-RI2."""

import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/spec135/generated-contract-v1"


def text(path):
    return (ROOT / path).read_text()


def load(path):
    return json.loads((CONTRACT / path).read_text())


def main():
    core = text("crates/focusa-core/src/runtime/interview_strategy.rs")
    api = text("crates/focusa-api/src/routes/interview_strategy.rs")
    ui = text("packages/a2ui-renderer/proof/interview-strategy.ts")
    html = text("packages/a2ui-renderer/proof/interview-strategy.html")
    assert "GRILL_WITH_DOCS_STRATEGY_ID" in core
    assert "GrillTranche::ALL" in core and "active_branch_has_gap" in core
    assert "recommendation_basis_refs.is_empty()" in core
    assert "environment_facts_checked.is_empty()" in core
    assert "operator_answer_is_authoritative: true" in core
    assert "approved_role" in api and "canonical_context_refs" in api
    assert "Focusa Interview Engine" in api and "advisory_strategy: true" in api

    registry = load("operation-registry.json")
    operations = {item["operation_id"]: item for item in registry["operations"]}
    operation = operations["focusa.interview.strategy.grill_with_docs.next_question"]
    assert operation["materialization_mode"] == "advisory_projection"
    assert operation["permissions_required"] == ["interview:read"]
    assert operation["scope"]["required_keys"] == [
        "project_root",
        "continuity_id",
        "attachment_id",
    ]
    assert operation["requires_idempotency_key"] is False
    assert operation["requires_if_match_version"] is False

    request = load("json-schema/focusa.grill_interview_context.v1.json")
    response = load("json-schema/focusa.grill_interview_strategy_response.v1.json")
    tranche = request["properties"]["gaps"]["items"]["properties"]["tranche"]["enum"]
    assert tranche == [
        "discovery",
        "boundary",
        "failure",
        "evidence",
        "architecture",
        "spec_readiness",
    ]
    proposal = response["properties"]["result"]["properties"]["proposal"]
    assert (
        proposal["properties"]["strategy_id"]["const"]
        == "focusa.interview.strategy.grill-with-docs.v1"
    )
    assert proposal["properties"]["operator_answer_is_authoritative"]["const"] is True

    openapi = load("openapi-3.0.3.json")
    route = openapi["paths"]["/v1/interview/strategy/grill-with-docs/next-question"][
        "post"
    ]
    assert (
        route["operationId"]
        == "focusa.interview.strategy.grill_with_docs.next_question"
    )
    bindings = load("ui-action-bindings.fixture.json")
    binding = next(
        item
        for item in bindings["bindings"]
        if item["action_id"] == route["operationId"]
    )
    assert (
        binding["contracts"]["input_schema_ref"] == "focusa.grill_interview_context.v1"
    )
    assert binding["presentation"]["allowed_in_generated_ui"] is True

    assert "FocusaA2uiRenderer" in ui and "@focusa/spec135-client" in ui
    assert "one_question_only" in ui and "operator_answer_is_authoritative" in ui
    assert "Run Grill Strategy" in ui and "Ask less. Decide better." in html
    assert "playwright" not in ui.lower() and "playwright" not in html.lower()
    proof = load("spec135-ri2-interview-strategy-proof.json")
    result = load("uiai-eval.ri2-interview-strategy.result.json")
    assert proof["status"] == "passed" and result["status"] == "passed"
    assert result["diagnostics_ref"] == "uiai-diagnostics:session=zFmKYl6M:seq=7"
    print("Spec 135 RI2 Interview strategy contracts/UI proof: PASS")


if __name__ == "__main__":
    main()
