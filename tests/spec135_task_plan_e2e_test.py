#!/usr/bin/env python3
"""SPEC135-ST2 provider-neutral task DAG preview/edit/approval and restart proof."""

import pathlib
import tempfile
import time
import urllib.parse

import spec135_role_profile_e2e_test as helper
import spec135_spec_workbench_e2e_test as wb

SCOPE = {
    "project_root": "/tmp/focusa-spec135-st2",
    "continuity_id": "focusa-cont-st2",
    "attachment_id": "attachment-st2",
}
LIST = "/v1/task-plans"
MUTATE = "/v1/task-plans/mutate"


def listed(base, pid=None):
    q = urllib.parse.urlencode({**SCOPE, **({"task_plan_id": pid} if pid else {})})
    status, p = helper.call(base, "GET", f"{LIST}?{q}")
    assert status == 200, p
    return p


def mutate(base, action, key, plan=None, **extra):
    for _ in range(30):
        current = listed(base, plan["task_plan_id"] if plan else None)
        body = {
            **SCOPE,
            "idempotency_key": key,
            "expected_state_version": current["state_version"],
            "expected_plan_revision": plan["state_revision"] if plan else 0,
            "action": action,
            **extra,
        }
        if plan:
            body["task_plan_id"] = plan["task_plan_id"]
        status, p = helper.call(base, "POST", MUTATE, body)
        if status == 200:
            return p
        assert status == 409, p
        time.sleep(0.05)
    raise RuntimeError("task plan writer busy")


def task(i, title, deps, section):
    return {
        "provider_neutral_id": i,
        "title": title,
        "description": f"Deliver {title} through canonical Focusa authority.",
        "order_index": 1 if not deps else 2,
        "linked_spec_sections": [section],
        "requirement_refs": ["SPEC135-ST2"],
        "acceptance_criteria": [f"{title} is evidence-backed"],
        "evidence_requirements": [f"evidence:{i}"],
        "semantic_object_refs": ["task-plan:alpha3"],
        "allowed_action_type_ids": ["implement", "verify"],
        "verification_policy_ref": "policy:spec135-proof",
        "allowed_scope": [SCOPE["project_root"]],
        "dependencies": deps,
        "blockers": [],
        "task_class": "implementation",
        "closure_kind": "evidence_and_receipt",
        "closure_policy_ref": "policy:closure",
        "preferred_provider": "work_item.bd",
        "provider_ref": None,
    }


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-st2-"))
    process = log = None
    oldh = helper.SCOPE
    oldw = wb.SCOPE
    try:
        helper.SCOPE = SCOPE
        wb.SCOPE = SCOPE
        process, log, base = helper.start(data)
        capabilities = listed(base)["provider_capabilities"]
        assert {capability["provider"] for capability in capabilities} == {
            "beads",
            "github_issues",
            "linear",
            "asana",
            "markdown_checklist",
        }
        allowed_states = {
            "configured and operational",
            "configured but unhealthy",
            "read-only",
            "credentials missing",
            "adapter unavailable",
            "schema-only support",
            "mutation approval required",
        }
        assert all(capability["status"] in allowed_states for capability in capabilities)
        assert all(capability["mutation_approval_required"] for capability in capabilities)
        source = helper.commit_context(base)["source"]
        spec = wb.mutate(
            base,
            "open",
            "st2-spec-open",
            current_ask="Approve a provider-neutral Alpha 3 task DAG.",
        )["session"]
        spec = wb.mutate(
            base,
            "upsert_section",
            "st2-spec-section",
            spec,
            section={
                "title": "Task graph authority",
                "section_kind": "tasks",
                "order_index": 1,
                "content": "Tasks preserve requirement, dependency, acceptance, and proof links; no provider mutation occurs before approval.",
                "context_refs": [source["source_id"]],
                "evidence_refs": ["evidence:st2:spec"],
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
            "st2-spec-approve",
            spec,
            decision={
                "section_id": sid,
                "rationale": "Grounded task authority.",
                "decided_by": "operator:vsmith",
                "evidence_refs": ["evidence:st2:spec-approval"],
            },
        )["session"]
        spec = wb.mutate(base, "final_approve", "st2-spec-final", spec)["session"]
        plan = mutate(
            base, "open", "st2-open", workbench_session_id=spec["workbench_session_id"]
        )["task_plan"]
        assert plan["status"] == "draft" and not plan["materialized"]
        plan = mutate(
            base,
            "upsert_task",
            "st2-task-1",
            plan,
            task=task("task-context", "Prepare context", [], sid),
        )["task_plan"]
        plan = mutate(
            base,
            "upsert_task",
            "st2-task-2",
            plan,
            task=task(
                "task-implement", "Implement approved spec", ["task-context"], sid
            ),
        )["task_plan"]
        preview = mutate(base, "preview", "st2-preview-1", plan)
        plan = preview["task_plan"]
        token = plan["preview_token"]
        assert (
            plan["status"] == "pending_operator"
            and not preview["materialization_allowed"]
        )
        edited = task("task-context", "Prepare verified context", [], sid)
        plan = mutate(base, "upsert_task", "st2-edit", plan, task=edited)["task_plan"]
        assert plan["status"] == "draft" and plan.get("preview_token") is None
        plan = mutate(base, "preview", "st2-preview-2", plan)["task_plan"]
        assert plan["preview_token"] != token
        bad = {
            **SCOPE,
            "task_plan_id": plan["task_plan_id"],
            "idempotency_key": "st2-bad-approve",
            "expected_state_version": listed(base)["state_version"],
            "expected_plan_revision": plan["state_revision"],
            "action": "approve",
            "preview_token": token,
            "approved_by": "operator:vsmith",
        }
        status, p = helper.call(base, "POST", MUTATE, bad)
        assert status == 422, p
        approved = mutate(
            base,
            "approve",
            "st2-approve",
            plan,
            preview_token=plan["preview_token"],
            approved_by="operator:vsmith",
        )
        plan = approved["task_plan"]
        assert (
            plan["status"] == "approved"
            and approved["materialization_allowed"]
            and not plan["materialized"]
        )
        status, p = helper.call(
            base,
            "POST",
            MUTATE,
            {
                **SCOPE,
                "task_plan_id": plan["task_plan_id"],
                "idempotency_key": "st2-post-approval-edit",
                "expected_state_version": listed(base)["state_version"],
                "expected_plan_revision": plan["state_revision"],
                "action": "remove_task",
                "task_id": "task-context",
            },
        )
        assert status == 409, p
        helper.stop(process, log)
        process = log = None
        process, log, base = helper.start(data)
        assert listed(base, plan["task_plan_id"])["task_plans"][-1] == plan
        assert listed(base)["task_plans"][-1]["tasks"][1]["dependencies"] == [
            "task-context"
        ]
        print("Spec 135 ST2 provider-neutral task plan E2E: PASS")
    finally:
        helper.SCOPE = oldh
        wb.SCOPE = oldw
        if process is not None:
            helper.stop(process, log)


if __name__ == "__main__":
    main()
