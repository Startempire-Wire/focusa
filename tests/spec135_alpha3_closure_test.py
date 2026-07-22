#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
C = R / "docs/contracts/spec135/generated-contract-v1"


def j(name):
    return json.loads((C / name).read_text())


def main():
    proof = j("spec135-alpha3-spec-task-proof.json")
    assert proof["status"] == "passed"
    assert [x["requirement"] for x in proof["composition"]] == [
        "SPEC135-ST1",
        "SPEC135-P1",
        "SPEC135-ST2",
        "SPEC135-ST3",
    ]
    for item in proof["composition"]:
        assert j(item["proof"])["status"] == "passed"
    assert proof["shared_beads_authority"]["worktree_local_database_created"] is False
    assert proof["remote_validation"]["rust_job"] == "passed"
    assert proof["remote_validation"]["st3_e2e_result"].endswith("PASS")
    assert j("uiai-eval.st1-spec-workbench.result.json")["status"] == "passed"
    assert j("uiai-eval.st2-task-plan.result.json")["status"] == "passed"
    route = (R / "crates/focusa-api/src/routes/task_plans.rs").read_text()
    reducer = (R / "crates/focusa-core/src/reducer.rs").read_text()
    assert (
        'root.join(".git").is_dir()' in route
        and "has_local_database" in route
        and "acquire_materialization_lock" in route
    )
    assert (
        "materialized IDs, external refs, and dependency links must remain stable"
        in reducer
    )
    assert proof["closure_receipt"].startswith("receipt:spec135-alpha3:")
    print("Spec 135 Alpha 3 Spec-to-Beads closure: PASS")


if __name__ == "__main__":
    main()
