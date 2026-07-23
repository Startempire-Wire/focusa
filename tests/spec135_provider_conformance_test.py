#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
C = R / "docs/contracts/spec135/generated-contract-v1"


def j(p):
    return json.loads((C / p).read_text())


def main():
    core = (R / "crates/focusa-core/src/provider_execution.rs").read_text()
    api = (R / "crates/focusa-api/src/routes/provider_execution.rs").read_text()
    ui = (R / "packages/a2ui-renderer/proof/provider-conformance.ts").read_text()
    for x in [
        "ProviderGovernanceContract",
        "ProviderExecutionRequest",
        "exact_scope_required",
        "permission_required",
        "idempotency_required",
        "receipt_required",
        "operation_registry_required",
        "direct_canonical_mutation_allowed",
    ]:
        assert x in core
    for x in [
        "query scope must exactly match",
        "registered_operation_ids",
        "execution_performed",
        "canonical_state_mutated",
    ]:
        assert x in api
    ops = {x["operation_id"]: x for x in j("operation-registry.json")["operations"]}
    assert ops["focusa.provider.conformance.evaluate"]["scope"]["required_keys"] == [
        "project_root",
        "continuity_id",
        "attachment_id",
    ]
    assert ops["focusa.provider.conformance.evaluate"]["control"][
        "idempotency_required"
    ]
    assert ops["focusa.provider.conformance.evaluate"]["control"]["receipt_required"]
    for s in [
        "focusa.provider_contract_list.request.v1",
        "focusa.provider_contract_list.v1",
        "focusa.provider_conformance.request.v1",
        "focusa.provider_conformance_response.v1",
    ]:
        assert (C / "json-schema" / f"{s}.json").exists()
    assert (
        "FocusaA2uiRenderer" in ui
        and "Verify All Providers" in ui
        and "playwright" not in ui.lower()
    )
    proof = j("spec135-p1-provider-conformance-proof.json")
    assert proof["status"] == "passed" and len(proof["providers"]) == 7
    assert j("uiai-eval.p1-provider-conformance.result.json")["status"] == "passed"
    print("Spec 135 P1 provider conformance contracts/UI proof: PASS")


if __name__ == "__main__":
    main()
