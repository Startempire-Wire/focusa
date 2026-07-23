#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
C = R / "docs/contracts/spec135/generated-contract-v1"


def j(p):
    return json.loads((C / p).read_text())


def main():
    t = (R / "crates/focusa-core/src/types.rs").read_text()
    r = (R / "crates/focusa-core/src/reducer.rs").read_text()
    a = (R / "crates/focusa-api/src/routes/spec_workbench.rs").read_text()
    u = (R / "packages/a2ui-renderer/proof/spec-workbench.ts").read_text()
    for x in [
        "SpecWorkbenchSessionRecord",
        "SpecSectionRecord",
        "SpecRoundRecord",
        "SpecObjectionRecord",
        "SpecOperatorGateRecord",
        "SpecAmendmentRecord",
    ]:
        assert x in t
    for x in [
        "approved Spec section requires grounding",
        "final Spec approval requires all sections approved",
        "SpecWorkbenchSessionRevised",
    ]:
        assert x in r
    for x in [
        "AddObjection",
        "ResolveObjection",
        "ApproveSection",
        "AmendSection",
        "FinalApprove",
        "Reopen",
    ]:
        assert x in a
    ops = {x["operation_id"]: x for x in j("operation-registry.json")["operations"]}
    assert (
        ops["focusa.spec_workbench.session.mutate"]["materialization_mode"]
        == "canonical_event"
    )
    assert ops["focusa.spec_workbench.session.list"]["scope"]["required_keys"] == [
        "project_root",
        "continuity_id",
        "attachment_id",
    ]
    for s in [
        "focusa.spec_workbench_session_list.request.v1",
        "focusa.spec_workbench_session_list.v1",
        "focusa.spec_workbench_mutation.request.v1",
        "focusa.spec_workbench_mutation_result.v1",
    ]:
        assert (C / "json-schema" / f"{s}.json").exists()
    assert (
        "FocusaA2uiRenderer" in u
        and "Run Workbench Proof" in u
        and "playwright" not in u.lower()
    )
    assert j("spec135-st1-spec-workbench-proof.json")["status"] == "passed"
    assert j("uiai-eval.st1-spec-workbench.result.json")["status"] == "passed"
    print("Spec 135 ST1 Spec Workbench contracts/UI proof: PASS")


if __name__ == "__main__":
    main()
