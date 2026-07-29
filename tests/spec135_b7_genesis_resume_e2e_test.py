#!/usr/bin/env python3
"""Spec 135B-7 new/existing/degraded/resumed Genesis → one Workpoint E2E proof."""

import json
import pathlib
import tempfile

import spec135_role_profile_e2e_test as helper


def verify_root(root: pathlib.Path):
    root.mkdir(parents=True, exist_ok=True)
    (root / ".focusa-project.json").write_text(
        json.dumps(
            {
                "schema": "focusa.project_identity.v1",
                "project_root": str(root),
                "verified": True,
            }
        )
        + "\n"
    )


def complete(root: pathlib.Path, continuity: str, key: str):
    return {
        "project_root": str(root),
        "continuity_id": continuity,
        "idempotency_key": key,
        "hlt": "Deliver one evidence-backed first Workpoint without state forks.",
        "hlt_confirmed": True,
        "desired_end_state": "Project Genesis is ready and the first Workpoint is promoted.",
        "current_state": "Project scope is verified and prerequisites are explicit.",
        "specification_ref": "docs/spec.md",
        "acceptance_criteria": ["First Workpoint has durable evidence and exact scope"],
        "mid_level_goal": "Complete Project Genesis",
        "short_term_goal": "Promote the first Workpoint",
        "waypoints": ["Genesis ready", "First Workpoint promoted"],
        "task_provider": "beads",
        "tasks": [
            {
                "id": "task-first",
                "title": "Prove first Workpoint",
                "status": "open",
                "priority": 0,
                "blocked_by": [],
                "evidence_refs": ["evidence:b7:first-workpoint"],
            }
        ],
        "allow_task_decomposition": False,
    }


def post(base: str, action: str, payload: dict, expected=200):
    status, body = helper.call(base, "POST", f"/v1/project/genesis/{action}", payload)
    assert status == expected, body
    return body


def assert_one_workpoint(packet: dict):
    assert packet["status"] == "ready"
    workpoint = packet["first_workpoint"]
    assert workpoint["workpoint_id"]
    assert workpoint["project_root"] == packet["project_root"]
    assert workpoint["continuity_id"] == packet["continuity_id"]
    assert workpoint["evidence_refs"]
    return workpoint["workpoint_id"]


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-b7-data-"))
    roots = pathlib.Path(tempfile.mkdtemp(prefix="focusa-b7-projects-"))
    process = log = None
    try:
        process, log, base = helper.start(data)

        new_root = roots / "new"
        verify_root(new_root)
        request = complete(new_root, "b7-new", "b7-new-start")
        staged = post(base, "start", request)
        assert staged["status"] == "staged"
        replay = post(base, "start", request)
        assert replay == staged
        ready = post(base, "commit", {**request, "confirm": True})
        new_workpoint = assert_one_workpoint(ready)
        resumed_ready = post(base, "resume", {**request, "idempotency_key": "b7-new-resume"})
        assert resumed_ready["genesis_id"] == ready["genesis_id"]
        assert assert_one_workpoint(resumed_ready) == new_workpoint

        degraded_root = roots / "degraded"
        verify_root(degraded_root)
        degraded = post(
            base,
            "start",
            {
                "project_root": str(degraded_root),
                "continuity_id": "b7-degraded",
                "idempotency_key": "b7-degraded-start",
                "acceptance_criteria": [],
                "tasks": [],
                "waypoints": [],
            },
        )
        assert degraded["status"] == "hlt_impasse"
        degraded_id = degraded["genesis_id"]
        degraded_created = degraded["created_at"]
        degraded_receipts = list(degraded["transition_receipts"])
        recovered_request = complete(
            degraded_root, "b7-degraded", "b7-degraded-resume"
        )
        recovered = post(base, "resume", recovered_request)
        assert recovered["genesis_id"] == degraded_id
        assert recovered["created_at"] == degraded_created
        assert recovered["status"] == "staged"
        assert recovered["crist_stage"] == "context_collecting"
        assert len(recovered["transition_receipts"]) == len(degraded_receipts) + 1
        assert len({item["receipt_id"] for item in recovered["transition_receipts"]}) == len(
            recovered["transition_receipts"]
        )
        recovered_ready = post(base, "commit", {**recovered_request, "confirm": True})
        assert_one_workpoint(recovered_ready)

        existing_root = roots / "existing"
        verify_root(existing_root)
        (existing_root / ".beads").mkdir(parents=True)
        (existing_root / ".beads/issues.jsonl").write_text(
            json.dumps(
                {
                    "id": "existing-first",
                    "title": "Adopt existing brownfield task",
                    "status": "open",
                    "priority": 0,
                    "dependencies": [],
                    "evidence_refs": ["evidence:b7:brownfield"],
                }
            )
            + "\n"
        )
        (existing_root / "README.md").write_text("# Existing project\n")
        existing_request = complete(
            existing_root, "b7-existing", "b7-existing-start"
        )
        existing_request["tasks"] = []
        existing = post(base, "start", existing_request)
        assert any(
            task["id"] == "existing-first"
            for task in existing["task_provider_and_task_graph"]["tasks"]
        )
        existing_ready = post(base, "commit", {**existing_request, "confirm": True})
        assert_one_workpoint(existing_ready)

        for root, expected_id in (
            (new_root, ready["genesis_id"]),
            (degraded_root, recovered_ready["genesis_id"]),
            (existing_root, existing_ready["genesis_id"]),
        ):
            status, packet = helper.call(
                base,
                "GET",
                f"/v1/project/genesis/status?project_root={root}",
            )
            assert status == 200 and packet["genesis_id"] == expected_id

        print("Spec 135 B7 Genesis resume E2E: PASS (new, existing, degraded, resumed, one Workpoint)")
    finally:
        if process is not None:
            helper.stop(process, log)


if __name__ == "__main__":
    main()
