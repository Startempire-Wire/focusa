#!/usr/bin/env python3
"""Spec 135 RI2 runtime proof: retrieval-first Grill strategy over canonical Context and Role."""

import copy
import pathlib
import tempfile

import spec135_role_profile_e2e_test as ri1

PROJECT = "/tmp/focusa-spec135-ri2-project"
CONTINUITY = "focusa-cont-spec135-ri2"
ATTACHMENT = "attachment-spec135-ri2"
SCOPE = {
    "project_root": PROJECT,
    "continuity_id": CONTINUITY,
    "attachment_id": ATTACHMENT,
}
ENDPOINT = "/v1/interview/strategy/grill-with-docs/next-question"
TRANCHES = [
    "discovery",
    "boundary",
    "failure",
    "evidence",
    "architecture",
    "spec_readiness",
]


def gap(source_id, tranche, gap_id, branch, priority="normal", dependencies=1):
    return {
        "gap_id": gap_id,
        "tranche": tranche,
        "decision_branch_id": branch,
        "question": f"Which bounded option should govern {gap_id}?",
        "reason_for_asking": "The remaining tradeoff is operator-owned and cannot be discovered from project sources.",
        "triggering_gap": f"Unresolved operator decision: {gap_id}",
        "recommendation": "Choose the smallest reversible option that preserves operator authority.",
        "recommendation_basis_refs": [source_id],
        "environment_facts_checked": [source_id],
        "contradiction_refs": [],
        "linked_context_refs": [source_id],
        "linked_spec_sections": ["135H §4"],
        "domain_term_candidates": [],
        "architecture_decision_candidates": [],
        "decision_required": True,
        "priority": priority,
        "answer_type": "select",
        "readiness_effect": "Closes one readiness dependency.",
        "stop_condition": "The operator selects an option or explicitly defers the branch.",
        "downstream_dependency_count": dependencies,
        "resolved": False,
    }


def strategy_context(source_id, role_id):
    gaps = [
        gap(source_id, "discovery", "desired-outcome", "purpose", "blocker", 9),
        gap(source_id, "boundary", "scope-boundary", "scope", "normal", 2),
        gap(source_id, "boundary", "non-goal", "scope", "high", 6),
        gap(source_id, "failure", "known-failure", "failure", "high", 4),
        gap(source_id, "evidence", "acceptance-proof", "evidence", "high", 3),
        gap(source_id, "architecture", "adapter-boundary", "architecture", "normal", 2),
        gap(source_id, "spec_readiness", "approval-gate", "readiness", "normal", 1),
    ]
    return {
        **SCOPE,
        "session_id": "interview-session-ri2",
        "approved_role_profile_ref": role_id,
        "active_branch_id": "scope",
        "completed_tranches": [],
        "gaps": gaps,
    }


def expect_rejected(base, body, contains):
    status, payload = ri1.call(base, "POST", ENDPOINT, body)
    assert status == 422, payload
    assert contains.lower() in payload["summary"].lower(), payload


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-ri2-strategy-"))
    process = log = None
    original_scope = ri1.SCOPE
    try:
        ri1.SCOPE = SCOPE
        process, log, base = ri1.start(data)
        source = ri1.commit_context(base)["source"]
        source_id = source["source_id"]
        drafted = ri1.mutate(
            base,
            "/v1/roles/profiles/draft",
            ri1.draft_body(source_id, "ri2-role-draft"),
        )["profile"]
        approved = ri1.mutate(
            base,
            "/v1/roles/profiles/review",
            ri1.review_body(drafted, "ri2-role-approve", "approve"),
        )["profile"]

        body = strategy_context(source_id, approved["role_profile_id"])
        status, response = ri1.call(base, "POST", ENDPOINT, body)
        assert status == 200, response
        assert response["schema"] == "focusa.grill_interview_strategy_response.v1"
        assert response["advisory_strategy"] is True
        assert response["canonical_inputs_verified"] is True
        assert response["interview_state_authority"] == "Focusa Interview Engine"
        result = response["result"]
        assert result["strategy_id"] == "focusa.interview.strategy.grill-with-docs.v1"
        assert result["retrieval_performed_before_question"] is True
        assert result["one_question_only"] is True
        assert result["all_core_tranches_accounted_for"] is True
        assert result["ready_for_spec"] is False
        proposal = result["proposal"]
        assert proposal["tranche"] == "boundary"
        assert proposal["triggering_gap"].endswith("non-goal")
        assert proposal["decision_branch_id"] == "scope"
        assert proposal["recommendation_basis_refs"] == [source_id]
        assert proposal["environment_facts_checked"] == [source_id]
        assert proposal["linked_context_refs"] == [source_id]
        assert proposal["operator_answer_is_authoritative"] is True
        assert "2 unresolved" in proposal["branch_progress"]

        missing_tranche = copy.deepcopy(body)
        missing_tranche["gaps"] = [
            g for g in missing_tranche["gaps"] if g["tranche"] != "failure"
        ]
        expect_rejected(base, missing_tranche, "six core")

        uncited = copy.deepcopy(body)
        uncited["gaps"][0]["recommendation_basis_refs"] = []
        expect_rejected(base, uncited, "cited basis")

        unknown_ref = copy.deepcopy(body)
        unknown_ref["gaps"][0]["linked_context_refs"] = ["context-source:other-scope"]
        expect_rejected(base, unknown_ref, "not canonical in exact scope")

        unapproved = copy.deepcopy(body)
        unapproved["approved_role_profile_ref"] = (
            drafted["role_profile_id"] + ":missing"
        )
        expect_rejected(base, unapproved, "approved role profile")

        all_resolved = copy.deepcopy(body)
        all_resolved["active_branch_id"] = None
        for candidate in all_resolved["gaps"]:
            candidate["resolved"] = True
        status, ready = ri1.call(base, "POST", ENDPOINT, all_resolved)
        assert status == 200 and ready["result"]["ready_for_spec"] is True
        assert "proposal" not in ready["result"]

        ri1.stop(process, log)
        process = log = None
        process, log, base = ri1.start(data)
        status, replayed = ri1.call(base, "POST", ENDPOINT, body)
        assert status == 200, replayed
        assert replayed["result"] == result
        assert replayed["tool_result"]["status"] == "completed"
        print("Spec 135 RI2 Grill Interview strategy E2E: PASS")
    finally:
        ri1.SCOPE = original_scope
        if process is not None:
            ri1.stop(process, log)


if __name__ == "__main__":
    main()
