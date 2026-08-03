#!/usr/bin/env python3
"""SPEC135-ST4 Workpoint/Work Rail/Evidence/closure/Receipt remote E2E."""

import pathlib
import tempfile
import time
import urllib.parse
import spec135_role_profile_e2e_test as h
import spec135_spec_workbench_e2e_test as wb
import spec135_task_plan_e2e_test as tp

RAIL = "/v1/work-rail"
MUTATE = "/v1/work-rail/mutate"


def listed(base, scope, rid=None):
    q = urllib.parse.urlencode(
        {
            **scope,
            "working_subpath_id": "mission-canvas",
            **({"work_rail_id": rid} if rid else {}),
        }
    )
    status, p = h.call(base, "GET", f"{RAIL}?{q}")
    assert status == 200, p
    return p


def mutate(base, scope, action, key, wp, item, row=None, **extra):
    for _ in range(30):
        current = listed(base, scope, row["work_rail_id"] if row else None)
        body = {
            **scope,
            "working_subpath_id": "mission-canvas",
            "idempotency_key": key,
            "expected_state_version": current["state_version"],
            "expected_rail_revision": row["state_revision"] if row else 0,
            "action": action,
            "side_effect_policy": "preview",
            "actor_ref": "operator:vsmith",
            "interaction_reason": f"E2E {action}",
            "workpoint_id": wp,
            "provider_item_id": item,
            **extra,
        }
        if row:
            body["work_rail_id"] = row["work_rail_id"]
        status, preview = h.call(base, "POST", MUTATE, body)
        assert status == 200 and not preview["committed"], preview
        body["side_effect_policy"] = "commit"
        body["preview_token"] = preview["preview_token"]
        status, p = h.call(base, "POST", MUTATE, body)
        if status == 200:
            assert p["committed"] and p["row"]["interaction_history"]
            return p
        assert status == 409, p
        time.sleep(0.05)
    raise RuntimeError("Work Rail writer busy")


def main():
    root = pathlib.Path(tempfile.mkdtemp(prefix="focusa-st4-parent-"))
    (root / ".git").mkdir()
    (root / ".beads").mkdir()
    (root / ".beads/issues.jsonl").touch()
    scope = {
        "project_root": str(root),
        "continuity_id": "focusa-cont-st4",
        "attachment_id": "attachment-st4",
    }
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-st4-data-"))
    process = log = None
    old = (h.SCOPE, wb.SCOPE, tp.SCOPE)
    try:
        h.SCOPE = wb.SCOPE = tp.SCOPE = scope
        process, log, base = h.start(data)
        source = h.commit_context(base)["source"]
        spec = wb.mutate(
            base, "open", "st4-spec-open", current_ask="Verify Work Rail closure."
        )["session"]
        spec = wb.mutate(
            base,
            "upsert_section",
            "st4-spec-section",
            spec,
            section={
                "title": "Verified closure",
                "section_kind": "work_rail",
                "order_index": 1,
                "content": "A Bead closes only from exact Workpoint authority with linked Evidence and a Receipt.",
                "context_refs": [source["source_id"]],
                "evidence_refs": ["evidence:st4:spec"],
                "codebase_refs": [
                    "docs/135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md"
                ],
                "research_refs": [],
                "docs_only": False,
            },
        )["session"]
        sid = spec["sections"][0]["section_id"]
        spec = wb.mutate(
            base,
            "approve_section",
            "st4-spec-approve",
            spec,
            decision={
                "section_id": sid,
                "rationale": "Exact authority and proof gate grounded.",
                "decided_by": "operator:vsmith",
                "evidence_refs": ["evidence:st4:approval"],
            },
        )["session"]
        spec = wb.mutate(base, "final_approve", "st4-spec-final", spec)["session"]
        plan = tp.mutate(
            base,
            "open",
            "st4-plan-open",
            workbench_session_id=spec["workbench_session_id"],
        )["task_plan"]
        plan = tp.mutate(
            base,
            "upsert_task",
            "st4-task",
            plan,
            task=tp.task("task-rail", "Close through Work Rail", [], sid),
        )["task_plan"]
        plan = tp.mutate(base, "preview", "st4-preview", plan)["task_plan"]
        plan = tp.mutate(
            base,
            "approve",
            "st4-approve",
            plan,
            preview_token=plan["preview_token"],
            approved_by="operator:vsmith",
        )["task_plan"]
        state = tp.listed(base)["state_version"]
        status, materialized = h.call(
            base,
            "POST",
            "/v1/task-plans/materialize/beads",
            {
                **scope,
                "task_plan_id": plan["task_plan_id"],
                "expected_state_version": state,
                "expected_plan_revision": plan["state_revision"],
                "worktree_prefix": "mcst4",
                "permission_grant_ref": "permission:operator-approved",
                "idempotency_key": "st4-materialize",
            },
        )
        assert status == 200, materialized
        item = materialized["materialization"]["tasks"][0]["provider_id"]
        status, checkpoint = h.call(
            base,
            "POST",
            "/v1/workpoint/checkpoint",
            {
                "project_root": scope["project_root"],
                "continuity_id": scope["continuity_id"],
                "working_subpath_id": "mission-canvas",
                "work_item_id": item,
                "mission": "Verify linked Work Rail closure",
                "current_action": "verify_close",
                "next_slice": "Close only after proof",
                "canonical": True,
                "idempotency_key": "st4-workpoint",
            },
        )
        assert status == 200, checkpoint
        wp = checkpoint["workpoint_id"]
        evidence = "evidence:st4:verified-output"
        status, linked = h.call(
            base,
            "POST",
            "/v1/workpoint/evidence/link",
            {
                "project_root": scope["project_root"],
                "continuity_id": scope["continuity_id"],
                "working_subpath_id": "mission-canvas",
                "workpoint_id": wp,
                "target_ref": item,
                "result": "Acceptance and provider output verified.",
                "evidence_ref": evidence,
                "attach_to_workpoint": True,
            },
        )
        assert status == 200, linked
        row = mutate(
            base,
            scope,
            "bind",
            "st4-bind",
            wp,
            item,
            title="Verified Work Rail task",
            instance_id="instance:st4",
            session_id="session:st4",
            work_surface_ids=["work-surface:st4"],
            priority=0,
            rank=1,
            change_set_ref="change-set:st4",
        )["row"]
        current = listed(base, scope, row["work_rail_id"])
        status, rejected = h.call(
            base,
            "POST",
            MUTATE,
            {
                **scope,
                "working_subpath_id": "mission-canvas",
                "idempotency_key": "st4-invalid-token",
                "expected_state_version": current["state_version"],
                "expected_rail_revision": row["state_revision"],
                "action": "activate",
                "side_effect_policy": "commit",
                "preview_token": "wrong-token",
                "actor_ref": "operator:vsmith",
                "work_rail_id": row["work_rail_id"],
                "workpoint_id": wp,
                "provider_item_id": item,
            },
        )
        assert status == 428 and rejected["failure_class"] == "approval_required"
        row = mutate(base, scope, "request_approval", "st4-approval", wp, item, row)["row"]
        assert row["focusa_status"] == "proof_missing"
        row = mutate(base, scope, "defer", "st4-defer", wp, item, row)["row"]
        assert row["focusa_status"] == "ready" and row["blockers"]
        row = mutate(base, scope, "steer", "st4-steer", wp, item, row)["row"]
        assert row["focusa_status"] == "active"
        row = mutate(base, scope, "activate", "st4-active", wp, item, row)["row"]
        closed = mutate(
            base,
            scope,
            "verify_close",
            "st4-close",
            wp,
            item,
            row,
            evidence_refs=[evidence],
            artifact_refs=["artifact:st4:output"],
            closure_claim_ref="closure-claim:st4",
        )
        row = closed["row"]
        assert (
            row["focusa_status"] == "verified_complete"
            and row["provider_status"] == "closed"
            and row["receipt_ref"]
        )
        assert '"status":"closed"' in (root / ".beads/issues.jsonl").read_text()
        row = mutate(base, scope, "reopen", "st4-reopen", wp, item, row)["row"]
        assert row["focusa_status"] == "ready" and row["provider_status"] == "open"
        assert '"status":"open"' in (root / ".beads/issues.jsonl").read_text()
        row = mutate(
            base,
            scope,
            "verify_close",
            "st4-close-final",
            wp,
            item,
            row,
            evidence_refs=[evidence],
            artifact_refs=["artifact:st4:output"],
            closure_claim_ref="closure-claim:st4-final",
        )["row"]
        actions = [entry["action"] for entry in row["interaction_history"]]
        for expected_action in (
            "bind",
            "request_approval",
            "defer",
            "steer",
            "activate",
            "verify_close",
            "reopen",
        ):
            assert expected_action in actions
        time.sleep(1.0)
        h.stop(process, log)
        process = log = None
        process, log, base = h.start(data)
        resumed = max(
            listed(base, scope, row["work_rail_id"])["rows"],
            key=lambda candidate: candidate["state_revision"],
        )
        assert resumed == row
        print("Spec 135 ST4 Work Rail verified closure E2E: PASS")
    finally:
        h.SCOPE, wb.SCOPE, tp.SCOPE = old
        if process is not None:
            h.stop(process, log)


if __name__ == "__main__":
    main()
