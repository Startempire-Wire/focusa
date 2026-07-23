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
    a = (R / "crates/focusa-api/src/routes/task_plans.rs").read_text()
    u = (R / "packages/a2ui-renderer/proof/task-plan.ts").read_text()
    for x in [
        "ProviderNeutralTaskRecord",
        "ProviderNeutralTaskPlanRecord",
        "TaskPlanStatus",
        "materialized",
    ]:
        assert x in t
    for x in [
        "task dependency graph must be acyclic",
        "final-approved Spec Workbench",
        "ProviderNeutralTaskPlanRevised",
        "task plan approval requires prior revision-bound preview",
    ]:
        assert x in r
    for x in [
        "UpsertTask",
        "RemoveTask",
        "Preview",
        "Approve",
        "matching prior preview token required",
        "approved task plan is immutable",
    ]:
        assert x in a
    ops = {x["operation_id"]: x for x in j("operation-registry.json")["operations"]}
    op = ops["focusa.task_plan.mutate"]
    assert op["scope"]["required_keys"] == [
        "project_root",
        "continuity_id",
        "attachment_id",
    ]
    assert (
        op["control"]["idempotency_required"]
        and op["control"]["receipt_required"]
        and op["control"]["mode"] == "preview"
    )
    for s in [
        "focusa.provider_neutral_task_plan_list.request.v1",
        "focusa.provider_neutral_task_plan_list.v1",
        "focusa.provider_neutral_task_plan_mutation.request.v1",
        "focusa.provider_neutral_task_plan_mutation_result.v1",
    ]:
        assert (C / "json-schema" / f"{s}.json").exists()
    assert (
        "FocusaA2uiRenderer" in u
        and "Preview Task Plan" in u
        and "playwright" not in u.lower()
    )
    assert j("spec135-st2-task-plan-proof.json")["status"] == "passed"
    assert j("uiai-eval.st2-task-plan.result.json")["status"] == "passed"
    print("Spec 135 ST2 provider-neutral task plan contracts/UI proof: PASS")


if __name__ == "__main__":
    main()
