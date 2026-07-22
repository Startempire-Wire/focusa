#!/usr/bin/env python3
"""SPEC135-ST3 canonical parent Beads materialization and stable-ID proof."""

import hashlib
import json
import pathlib
import tempfile
import spec135_role_profile_e2e_test as helper
import spec135_spec_workbench_e2e_test as wb
import spec135_task_plan_e2e_test as tp


def stable(prefix, plan, task):
    h = hashlib.sha256()
    for value in (plan, task):
        h.update(value.encode())
        h.update(b"\0")
    return f"{prefix}-{h.hexdigest()[:24]}"


def main():
    root = pathlib.Path(tempfile.mkdtemp(prefix="focusa-st3-parent-"))
    (root / ".git").mkdir()
    (root / ".beads").mkdir()
    ledger = root / ".beads/issues.jsonl"
    ledger.touch()
    scope = {
        "project_root": str(root),
        "continuity_id": "focusa-cont-st3",
        "attachment_id": "attachment-st3",
    }
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-st3-data-"))
    process = log = None
    oldh, oldw, oldt = helper.SCOPE, wb.SCOPE, tp.SCOPE
    try:
        helper.SCOPE = scope
        wb.SCOPE = scope
        tp.SCOPE = scope
        process, log, base = helper.start(data)
        source = helper.commit_context(base)["source"]
        spec = wb.mutate(
            base,
            "open",
            "st3-spec-open",
            current_ask="Approve canonical parent Beads materialization.",
        )["session"]
        spec = wb.mutate(
            base,
            "upsert_section",
            "st3-spec-section",
            spec,
            section={
                "title": "Shared Beads authority",
                "section_kind": "tasks",
                "order_index": 1,
                "content": "Approved tasks materialize only into canonical parent Beads using stable prefixed IDs and dependency links.",
                "context_refs": [source["source_id"]],
                "evidence_refs": ["evidence:st3:spec"],
                "codebase_refs": [
                    "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md"
                ],
                "research_refs": [],
                "docs_only": False,
            },
        )["session"]
        sid = spec["sections"][0]["section_id"]
        spec = wb.mutate(
            base,
            "approve_section",
            "st3-spec-approve",
            spec,
            decision={
                "section_id": sid,
                "rationale": "Canonical parent and stable ID boundary grounded.",
                "decided_by": "operator:vsmith",
                "evidence_refs": ["evidence:st3:approval"],
            },
        )["session"]
        spec = wb.mutate(base, "final_approve", "st3-spec-final", spec)["session"]
        plan = tp.mutate(
            base,
            "open",
            "st3-plan-open",
            workbench_session_id=spec["workbench_session_id"],
        )["task_plan"]
        plan = tp.mutate(
            base,
            "upsert_task",
            "st3-task-1",
            plan,
            task=tp.task("task-parent", "Create parent task", [], sid),
        )["task_plan"]
        plan = tp.mutate(
            base,
            "upsert_task",
            "st3-task-2",
            plan,
            task=tp.task("task-child", "Create child task", ["task-parent"], sid),
        )["task_plan"]
        plan = tp.mutate(base, "preview", "st3-preview", plan)["task_plan"]
        plan = tp.mutate(
            base,
            "approve",
            "st3-approve",
            plan,
            preview_token=plan["preview_token"],
            approved_by="operator:vsmith",
        )["task_plan"]
        state = tp.listed(base)["state_version"]
        collision = stable("collision", plan["task_plan_id"], "task-parent")
        ledger.write_text(
            json.dumps({"id": collision, "title": "Unrelated", "external_ref": "other"})
            + "\n"
        )
        body = {
            **scope,
            "task_plan_id": plan["task_plan_id"],
            "expected_state_version": state,
            "expected_plan_revision": plan["state_revision"],
            "worktree_prefix": "collision",
            "permission_grant_ref": "permission:operator-approved",
            "idempotency_key": "st3-collision",
        }
        status, p = helper.call(base, "POST", "/v1/task-plans/materialize/beads", body)
        assert status == 409, p
        ledger.write_text("")
        body["worktree_prefix"] = "mcst3"
        body["idempotency_key"] = "st3-materialize"
        status, p = helper.call(base, "POST", "/v1/task-plans/materialize/beads", body)
        assert status == 200, p
        record = p["materialization"]
        assert len(record["tasks"]) == 2 and record["target_ledger_ref"] == str(ledger)
        assert all(x["provider_id"].startswith("mcst3-") for x in record["tasks"])
        refs = {x["provider_neutral_id"]: x for x in record["tasks"]}
        assert refs["task-child"]["provider_dependency_ids"] == [
            refs["task-parent"]["provider_id"]
        ]
        assert not list((root / ".beads").glob("*.db"))
        rows = [json.loads(x) for x in ledger.read_text().splitlines()]
        assert len(rows) == 2 and all(
            x["external_ref"].startswith(f"focusa-task-plan:{plan['task_plan_id']}:")
            for x in rows
        )
        status, replay = helper.call(
            base, "POST", "/v1/task-plans/materialize/beads", body
        )
        assert (
            status == 200 and replay["replayed"] and replay["materialization"] == record
        )
        assert len(ledger.read_text().splitlines()) == 2
        helper.stop(process, log)
        process = log = None
        process, log, base = helper.start(data)
        body["expected_state_version"] = tp.listed(base)["state_version"]
        status, replay = helper.call(
            base, "POST", "/v1/task-plans/materialize/beads", body
        )
        assert (
            status == 200 and replay["replayed"] and replay["materialization"] == record
        )
        print("Spec 135 ST3 canonical parent Beads materialization E2E: PASS")
    finally:
        helper.SCOPE, wb.SCOPE, tp.SCOPE = oldh, oldw, oldt
        if process is not None:
            helper.stop(process, log)


if __name__ == "__main__":
    main()
