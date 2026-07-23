#!/usr/bin/env python3
"""SPEC135-RI3 durable Interview state, close/reopen, deferred branch, restart proof."""

import pathlib
import tempfile
import time

import spec135_role_profile_e2e_test as ri1

SCOPE = {
    "project_root": "/tmp/focusa-spec135-ri3-project",
    "continuity_id": "focusa-cont-spec135-ri3",
    "attachment_id": "attachment-spec135-ri3",
}
LIST = "/v1/interviews/sessions"
MUTATE = "/v1/interviews/sessions/mutate"


def listed(base, session_id=None, scope=SCOPE):
    query = "&".join(f"{key}={value}" for key, value in scope.items())
    if session_id:
        query += f"&interview_session_id={session_id}"
    status, payload = ri1.call(base, "GET", f"{LIST}?{query}")
    assert status == 200, payload
    return payload


def mutate(base, action, key, session=None, **extra):
    for _ in range(30):
        current = listed(base, session["interview_session_id"] if session else None)
        body = {
            **SCOPE,
            "idempotency_key": key,
            "expected_state_version": current["state_version"],
            "expected_session_revision": session["state_revision"] if session else 0,
            "action": action,
            **extra,
        }
        if session:
            body["interview_session_id"] = session["interview_session_id"]
        status, payload = ri1.call(base, "POST", MUTATE, body)
        if status == 200:
            return payload
        assert status == 409, payload
        time.sleep(0.05)
    raise RuntimeError("Interview writer did not quiesce")


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-ri3-session-"))
    process = log = None
    original_scope = ri1.SCOPE
    try:
        ri1.SCOPE = SCOPE
        process, log, base = ri1.start(data)
        source = ri1.commit_context(base)["source"]
        source_id = source["source_id"]
        draft = ri1.mutate(
            base,
            "/v1/roles/profiles/draft",
            ri1.draft_body(source_id, "ri3-role-draft"),
        )["profile"]
        role = ri1.mutate(
            base,
            "/v1/roles/profiles/review",
            ri1.review_body(draft, "ri3-role-approve", "approve"),
        )["profile"]

        opened = mutate(
            base,
            "open",
            "ri3-open",
            approved_role_profile_ref=role["role_profile_id"],
        )
        session = opened["session"]
        assert session["state_revision"] == 1 and session["status"] == "active"
        assert opened["exact_resume"] is True
        status, replay = ri1.call(
            base,
            "POST",
            MUTATE,
            {
                **SCOPE,
                "idempotency_key": "ri3-open",
                "expected_state_version": 0,
                "expected_session_revision": 0,
                "action": "open",
                "approved_role_profile_ref": role["role_profile_id"],
            },
        )
        assert status == 200 and replay["replayed"] is True
        assert replay["session"] == session

        session = mutate(
            base,
            "upsert_branch",
            "ri3-branch-scope",
            session,
            branch={
                "decision_branch_id": "scope",
                "tranche": "boundary",
                "label": "Scope and non-goals",
            },
        )["session"]
        session = mutate(
            base,
            "upsert_branch",
            "ri3-branch-risks",
            session,
            branch={
                "decision_branch_id": "risks",
                "tranche": "failure",
                "label": "Known failure boundaries",
            },
        )["session"]
        session = mutate(
            base,
            "defer_branch",
            "ri3-defer-risks",
            session,
            decision_branch_id="risks",
            deferred_reason="Operator will provide compliance owner next session.",
        )["session"]
        deferred = next(
            branch
            for branch in session["branches"]
            if branch["decision_branch_id"] == "risks"
        )
        assert deferred["status"] == "deferred" and deferred["deferred_reason"]

        session = mutate(
            base,
            "queue_question",
            "ri3-question-scope",
            session,
            question={
                "decision_branch_id": "scope",
                "question": "Should Alpha 2 exclude operational permission grants?",
                "reason_for_asking": "The operator owns the acceptance boundary.",
                "triggering_gap": "Alpha 2 non-goal is not explicitly accepted.",
                "recommendation": "Keep permission grants outside Alpha 2.",
                "recommendation_basis_refs": [source_id],
                "environment_facts_checked": [source_id],
                "contradiction_refs": [],
                "linked_context_refs": [source_id],
                "linked_spec_sections": ["135H §4"],
                "decision_required": True,
                "priority": "blocker",
                "answer_type": "boolean",
                "sensitivity": "consequential",
                "readiness_effect": "Closes the permission-boundary blocker.",
                "stop_condition": "Operator accepts or rejects the stated non-goal.",
            },
        )["session"]
        question_id = session["current_question_id"]
        assert question_id and session["active_branch_id"] == "scope"
        exact_pointer = (session["active_branch_id"], session["current_question_id"])

        session = mutate(base, "pause", "ri3-pause", session)["session"]
        assert session["status"] == "paused"
        assert (
            session["active_branch_id"],
            session["current_question_id"],
        ) == exact_pointer
        session = mutate(base, "close", "ri3-close", session)["session"]
        assert session["status"] == "closed" and session["closed_at"]
        assert (
            session["active_branch_id"],
            session["current_question_id"],
        ) == exact_pointer

        ri1.stop(process, log)
        process = log = None
        process, log, base = ri1.start(data)
        history = listed(base, session["interview_session_id"])["sessions"]
        latest = history[-1]
        assert latest == session
        assert len(history) == 7
        assert (
            next(
                branch
                for branch in latest["branches"]
                if branch["decision_branch_id"] == "risks"
            )["status"]
            == "deferred"
        )

        session = mutate(base, "reopen", "ri3-reopen", latest)["session"]
        assert session["status"] == "active" and session.get("closed_at") is None
        assert (
            session["active_branch_id"],
            session["current_question_id"],
        ) == exact_pointer
        session = mutate(
            base,
            "record_answer",
            "ri3-answer",
            session,
            answer={
                "question_id": question_id,
                "answer": True,
                "operator_id": "operator:vsmith",
                "confidence": 1.0,
                "notes": "Accepted as an explicit Alpha 2 non-goal.",
            },
        )["session"]
        assert session.get("current_question_id") is None
        assert session["questions"][-1]["status"] == "answered"
        assert session["answers"][-1]["answer"] is True
        assert session["answers"][-1]["operator_id"] == "operator:vsmith"

        other = {**SCOPE, "attachment_id": "attachment-unrelated"}
        assert listed(base, scope=other)["sessions"] == []
        ri1.stop(process, log)
        process = log = None
        process, log, base = ri1.start(data)
        persisted = listed(base, session["interview_session_id"])["sessions"][-1]
        assert persisted == session
        print("Spec 135 RI3 durable Interview session E2E: PASS")
    finally:
        ri1.SCOPE = original_scope
        if process is not None:
            ri1.stop(process, log)


if __name__ == "__main__":
    main()
