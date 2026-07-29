#!/usr/bin/env python3
"""SPEC135-ST1 canonical Spec 120 Workbench lifecycle and exact resume proof."""

import pathlib
import tempfile
import time
import urllib.parse

import spec135_role_profile_e2e_test as helper

SCOPE = {
    "project_root": "/tmp/focusa-spec135-st1",
    "continuity_id": "focusa-cont-st1",
    "attachment_id": "attachment-st1",
}
LIST = "/v1/spec-workbench/sessions"
MUTATE = "/v1/spec-workbench/session/mutate"


def listed(base, sid=None, scope=SCOPE):
    q = urllib.parse.urlencode(
        {**scope, **({"workbench_session_id": sid} if sid else {})}
    )
    status, p = helper.call(base, "GET", f"{LIST}?{q}")
    assert status == 200, p
    return p


def mutate(base, action, key, session=None, **extra):
    for _ in range(30):
        current = listed(base, session["workbench_session_id"] if session else None)
        body = {
            **SCOPE,
            "idempotency_key": key,
            "expected_state_version": current["state_version"],
            "expected_session_revision": session["state_revision"] if session else 0,
            "action": action,
            **extra,
        }
        if session:
            body["workbench_session_id"] = session["workbench_session_id"]
        status, p = helper.call(base, "POST", MUTATE, body)
        if status == 200:
            return p
        assert status == 409, p
        time.sleep(0.05)
    raise RuntimeError("Workbench writer busy")


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-st1-"))
    process = log = None
    old = helper.SCOPE
    try:
        helper.SCOPE = SCOPE
        process, log, base = helper.start(data)
        source = helper.commit_context(base)["source"]
        ref = source["source_id"]
        session = mutate(
            base,
            "open",
            "st1-open",
            current_ask="Produce an approved Alpha 3 provider-neutral task materialization spec.",
        )["session"]
        assert (
            session["state_revision"] == 1
            and session["canonical"]
            and session["advisory_agents"]
            and session["operator_required"]
        )
        session = mutate(
            base,
            "upsert_section",
            "st1-section",
            session,
            section={
                "title": "Task materialization authority",
                "section_kind": "architecture",
                "order_index": 1,
                "content": "Approved task graphs materialize through provider-neutral policy into the shared Beads database only.",
                "context_refs": [ref],
                "evidence_refs": ["evidence:st1:grounding"],
                "codebase_refs": [
                    "docs/120-adversarial-spec-workbench-and-operator-approval-gates.md"
                ],
                "research_refs": [],
                "docs_only": False,
            },
        )["session"]
        section = session["sections"][0]
        sid = section["section_id"]
        assert section["status"] == "grounded"
        session = mutate(
            base,
            "add_round",
            "st1-round",
            session,
            round={
                "section_id": sid,
                "round_kind": "adversarial",
                "output_refs": ["artifact:proposer", "artifact:challenger"],
                "transcript_ref": "transcript:st1-round-1",
                "verdict": "changes_required",
            },
        )["session"]
        rid = session["rounds"][0]["round_id"]
        session = mutate(
            base,
            "add_objection",
            "st1-objection",
            session,
            objection={
                "section_id": sid,
                "round_id": rid,
                "actor_role": "adversarial_challenger",
                "claim": "The section does not explicitly prohibit a worktree-local Beads database.",
                "reasoning_summary": "Shared-database authority must be explicit before task generation.",
                "evidence_refs": ["evidence:st1:shared-beads"],
                "confidence": 0.98,
            },
        )["session"]
        oid = session["objections"][0]["objection_id"]
        assert session["sections"][0]["status"] == "challenged"
        body = {
            **SCOPE,
            "workbench_session_id": session["workbench_session_id"],
            "idempotency_key": "st1-early-approve",
            "expected_state_version": listed(base)["state_version"],
            "expected_session_revision": session["state_revision"],
            "action": "approve_section",
            "decision": {
                "section_id": sid,
                "rationale": "too early",
                "decided_by": "operator:vsmith",
                "evidence_refs": ["evidence:st1:gate"],
            },
        }
        status, p = helper.call(base, "POST", MUTATE, body)
        assert status == 422, p
        session = mutate(
            base,
            "resolve_objection",
            "st1-resolve",
            session,
            objection_id=oid,
            resolution="Added explicit shared parent Beads authority and prohibited local databases.",
        )["session"]
        assert session["objections"][0]["status"] == "resolved"
        session = mutate(
            base,
            "approve_section",
            "st1-approve-v1",
            session,
            decision={
                "section_id": sid,
                "rationale": "Grounded and objection resolved.",
                "decided_by": "operator:vsmith",
                "approval_scope": "section_revision",
                "evidence_refs": ["evidence:st1:approval-v1"],
            },
        )["session"]
        assert session["sections"][0]["status"] == "approved"
        session = mutate(
            base,
            "amend_section",
            "st1-amend",
            session,
            amendment={
                "section_id": sid,
                "content": "Approved task graphs materialize only through provider-neutral policy into canonical shared parent Beads; worktree-local databases are prohibited.",
                "reason": "Encode resolved shared-database objection.",
                "changed_by": "operator:vsmith",
                "evidence_refs": ["evidence:st1:amendment"],
            },
        )["session"]
        assert (
            session["sections"][0]["revision"] == 2
            and session["sections"][0]["status"] == "amended"
        )
        session = mutate(
            base,
            "approve_section",
            "st1-approve-v2",
            session,
            decision={
                "section_id": sid,
                "rationale": "Amended revision preserves grounding and resolves objection.",
                "decided_by": "operator:vsmith",
                "evidence_refs": ["evidence:st1:approval-v2"],
            },
        )["session"]
        pointer = session["current_section_id"]
        session = mutate(base, "close", "st1-close", session)["session"]
        assert (
            session["status"] == "closed" and session["current_section_id"] == pointer
        )
        time.sleep(1.0)
        helper.stop(process, log)
        process = log = None
        process, log, base = helper.start(data)
        latest = max(
            listed(base, session["workbench_session_id"])["sessions"],
            key=lambda candidate: candidate["state_revision"],
        )
        assert latest == session
        assert (
            latest["objections"][0]["status"] == "resolved"
            and latest["amendments"][0]["after_revision"] == 2
        )
        session = mutate(base, "reopen", "st1-reopen", latest)["session"]
        assert (
            session["status"] == "active" and session["current_section_id"] == pointer
        )
        session = mutate(base, "final_approve", "st1-final", session)["session"]
        assert session["status"] == "final_approved" and session["final_spec_id"]
        time.sleep(1.0)
        helper.stop(process, log)
        process = log = None
        process, log, base = helper.start(data)
        resumed = max(
            listed(base, session["workbench_session_id"])["sessions"],
            key=lambda candidate: candidate["state_revision"],
        )
        assert resumed == session
        other = {**SCOPE, "attachment_id": "unrelated"}
        assert listed(base, scope=other)["sessions"] == []
        print("Spec 135 ST1 Spec Workbench E2E: PASS")
    finally:
        helper.SCOPE = old
        if process is not None:
            helper.stop(process, log)


if __name__ == "__main__":
    main()
