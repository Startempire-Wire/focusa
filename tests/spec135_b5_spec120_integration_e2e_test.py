#!/usr/bin/env python3
"""Spec 135B-5 C.R.I.S.T. handoff into governed Spec 120 E2E proof."""

import pathlib
import tempfile
import time

import spec135_interview_session_e2e_test as interview
import spec135_role_profile_e2e_test as helper
import spec135_spec_workbench_e2e_test as workbench

SCOPE = {
    "project_root": "/tmp/focusa-spec135-b5",
    "continuity_id": "focusa-cont-b5",
    "attachment_id": "attachment-b5",
}

SECTIONS = [
    "Project title and one-line definition",
    "Problem or opportunity",
    "Project identity and current-state reality",
    "Long-term desired state / mandatory HLT",
    "Users and stakeholders",
    "Approved project agent role",
    "Context sources and provenance",
    "Scope",
    "Non-goals",
    "Constraints",
    "Risks",
    "Authority and approval boundaries",
    "Data, privacy, retention, and connector posture",
    "Workspace and visual profile",
    "Evidence and proof policy",
    "Core workflows",
    "Initial architecture or operating model",
    "Success criteria",
    "Milestones and Waypoints",
    "Known unknowns and open questions",
    "Initial task-decomposition policy",
    "Final approval record",
]


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-b5-"))
    process = log = None
    old_role_scope, old_interview_scope, old_workbench_scope = (
        helper.SCOPE,
        interview.SCOPE,
        workbench.SCOPE,
    )
    try:
        helper.SCOPE = interview.SCOPE = workbench.SCOPE = SCOPE
        process, log, base = helper.start(data)
        source = helper.commit_context(base)["source"]
        source_ref = source["source_id"]
        artifact_ref = source["artifact"]["artifact_id"]
        drafted = helper.mutate(
            base,
            "/v1/roles/profiles/draft",
            helper.draft_body(source_ref, "b5-role-draft"),
        )["profile"]
        approved_role = helper.mutate(
            base,
            "/v1/roles/profiles/review",
            helper.review_body(drafted, "b5-role-approve", "approve"),
        )["profile"]
        opened_interview = interview.mutate(
            base,
            "open",
            "b5-interview-open",
            approved_role_profile_ref=approved_role["role_profile_id"],
        )["session"]
        closed_interview = interview.mutate(
            base,
            "close",
            "b5-interview-close",
            opened_interview,
        )["session"]

        session = workbench.mutate(
            base,
            "open",
            "b5-workbench-open",
            current_ask="Create the governed Project Genesis specification.",
            desired_spec_template="project_genesis",
        )["session"]
        handoff = session["crist_handoff"]
        assert handoff["schema"] == "focusa.crist_spec_handoff.v1"
        assert handoff["desired_spec_template"] == "project_genesis"
        assert artifact_ref in handoff["context_pack_refs"]
        assert handoff["role_profile_ref"] == approved_role["role_profile_id"]
        assert closed_interview["interview_session_id"] in handoff["interview_session_refs"]

        current = workbench.listed(base, session["workbench_session_id"])
        status, blocked = helper.call(
            base,
            "POST",
            workbench.MUTATE,
            {
                **SCOPE,
                "workbench_session_id": session["workbench_session_id"],
                "idempotency_key": "b5-premature-final",
                "expected_state_version": current["state_version"],
                "expected_session_revision": session["state_revision"],
                "action": "final_approve",
            },
        )
        assert status == 422 and blocked["failure_class"] == "approval_required"

        for order, title in enumerate(SECTIONS, 1):
            session = workbench.mutate(
                base,
                "upsert_section",
                f"b5-section-{order}",
                session,
                section={
                    "title": title,
                    "section_kind": "project_genesis",
                    "reality_classification": "normative_target",
                    "order_index": order,
                    "content": f"{title} is grounded in accepted Project Genesis inputs.",
                    "context_refs": [artifact_ref],
                    "evidence_refs": [source["evidence"]["evidence_ref"]],
                    "codebase_refs": [],
                    "research_refs": [],
                    "docs_only": False,
                },
            )["session"]
            section = session["sections"][-1]
            assert section["reality_classification"] == "normative_target"
            session = workbench.mutate(
                base,
                "approve_section",
                f"b5-approve-{order}",
                session,
                decision={
                    "section_id": section["section_id"],
                    "rationale": "Operator reviewed grounding and reality classification.",
                    "decided_by": "operator:vsmith",
                    "evidence_refs": [source["evidence"]["evidence_ref"]],
                    "approval_scope": "section_revision",
                },
            )["session"]

        final = workbench.mutate(
            base,
            "final_approve",
            "b5-final-approve",
            session,
        )["session"]
        assert final["status"] == "final_approved" and final["final_spec_id"]
        assert len(final["sections"]) == 22
        assert all(section["status"] == "approved" for section in final["sections"])

        time.sleep(2.0)  # Allow the append-only event journal to quiesce before graceful restart.
        helper.stop(process, log)
        process = log = None
        process, log, base = helper.start(data)
        resumed = max(
            workbench.listed(base, final["workbench_session_id"], scope=SCOPE)["sessions"],
            key=lambda candidate: candidate["state_revision"],
        )
        assert resumed == final, {
            key: (final.get(key), resumed.get(key))
            for key in final.keys() | resumed.keys()
            if final.get(key) != resumed.get(key)
        }
        print("Spec 135 B5 governed Spec 120 E2E: PASS (handoff, 22 sections, reality, approval, restart)")
    finally:
        helper.SCOPE, interview.SCOPE, workbench.SCOPE = (
            old_role_scope,
            old_interview_scope,
            old_workbench_scope,
        )
        if process is not None:
            helper.stop(process, log)


if __name__ == "__main__":
    main()
