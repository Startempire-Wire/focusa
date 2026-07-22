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
    u = (R / "packages/a2ui-renderer/proof/task-materialization.ts").read_text()
    for x in [
        "TaskMaterializationRecord",
        "MaterializedTaskRef",
        "TaskPlanMaterialized",
    ]:
        assert x in t
    for x in [
        "exact-scoped approved task plan revision",
        "canonical project_root/.beads/issues.jsonl",
        "materialized IDs, external refs, and dependency links must remain stable",
    ]:
        assert x in r
    for x in [
        'root.join(".git").is_dir()',
        "has_local_database",
        "stable provider ID collision",
        "BD_ISSUE_PREFIX",
        "--no-db",
        "TaskPlanMaterialized",
        "idempotency_key",
    ]:
        assert x in a
    ops = {x["operation_id"]: x for x in j("operation-registry.json")["operations"]}
    op = ops["focusa.task_plan.materialize.beads"]
    assert op["scope"]["required_keys"] == [
        "project_root",
        "continuity_id",
        "attachment_id",
    ]
    assert op["control"]["idempotency_required"] and op["control"]["receipt_required"]
    assert op["materialization_mode"] == "external_governed_mutation"
    for s in [
        "focusa.task_plan_beads_materialization.request.v1",
        "focusa.task_plan_beads_materialization_result.v1",
    ]:
        assert (C / "json-schema" / f"{s}.json").exists()
    assert (
        "FocusaA2uiRenderer" in u
        and "Create Tasks in Beads" in u
        and "playwright" not in u.lower()
    )
    proof = j("spec135-st3-task-materialization-proof.json")
    assert proof["status"] == "implemented_pending_remote_validation"
    assert proof["contracts"]["operation_count"] == 77
    print("Spec 135 ST3 canonical Beads materialization static proof: PASS")


if __name__ == "__main__":
    main()
