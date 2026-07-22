#!/usr/bin/env python3
"""SPEC135-RI1 canonical grounded Role Profile lifecycle proof."""

import json
import os
import pathlib
import shutil
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]
BINARY = pathlib.Path(
    os.environ.get("FOCUSA_DAEMON_BIN", ROOT / "target/debug/focusa-daemon")
)
SCOPE = {
    "project_root": "/example/focusa",
    "continuity_id": "focusa-cont-ri1",
    "attachment_id": "attachment:ri1-role",
}


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def call(base, method, path, body=None):
    request = urllib.request.Request(
        base + path,
        data=None if body is None else json.dumps(body).encode(),
        method=method,
        headers={"content-type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return response.status, json.load(response)
    except urllib.error.HTTPError as error:
        payload = json.load(error)
        return error.code, payload


def start(data_dir):
    port = free_port()
    env = os.environ.copy()
    env.update(FOCUSA_DATA_DIR=str(data_dir), FOCUSA_BIND=f"127.0.0.1:{port}")
    log = open(data_dir / "daemon.log", "ab")
    process = subprocess.Popen(
        [str(BINARY)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT
    )
    base = f"http://127.0.0.1:{port}"
    for _ in range(300):
        if process.poll() is not None:
            raise RuntimeError((data_dir / "daemon.log").read_text())
        try:
            with urllib.request.urlopen(base + "/v1/health", timeout=1) as response:
                if response.status == 200:
                    return process, log, base
        except (urllib.error.URLError, TimeoutError):
            time.sleep(0.05)
    raise RuntimeError("daemon unavailable")


def stop(process, log):
    process.send_signal(signal.SIGTERM)
    try:
        process.wait(15)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(5)
    log.close()


def listed(base, scope=SCOPE):
    query = urllib.parse.urlencode(scope)
    return call(base, "GET", f"/v1/roles/profiles?{query}")[1]


def context_list(base):
    query = urllib.parse.urlencode(SCOPE)
    return call(base, "GET", f"/v1/context/sources?{query}")[1]


def commit_context(base):
    body = {
        **SCOPE,
        "idempotency_key": "ri1-context-source",
        "expected_state_version": 0,
        "source_kind": "markdown",
        "title": "Role grounding source",
        "content": "# Mission\nBuild a restart-safe governed Focusa workspace with explicit evidence and operator approval.",
    }
    for _ in range(20):
        body["expected_state_version"] = context_list(base)["state_version"]
        status, payload = call(base, "POST", "/v1/context/sources/commit", body)
        if status == 200:
            return payload
        assert status == 409, payload
        time.sleep(0.05)
    raise RuntimeError("context writer did not quiesce")


def draft_body(source_id, key, purpose=None, redlines=None, permission_assertions=None):
    return {
        **SCOPE,
        "idempotency_key": key,
        "expected_state_version": 0,
        "original_seed": "Act as the evidence-grounded Focusa Mission Canvas delivery lead.",
        "title": "Focusa Mission Canvas delivery lead",
        "purpose": purpose
        or "Translate accepted Context into safe, test-backed delivery while preserving operator authority.",
        "expertise": [
            "Focusa canonical reducers",
            "generated UI",
            "evidence governance",
        ],
        "primary_responsibilities": [
            "Implement the accepted Spec 135 critical path",
            "Preserve exact project and workstream scope",
        ],
        "secondary_responsibilities": ["Explain recovery paths to the operator"],
        "expected_deliverables": [
            "Verified Focusa capabilities with Evidence and Receipts"
        ],
        "quality_standards": ["Restart-safe", "citation-preserving", "lint-clean"],
        "decision_principles": ["Operator steering outranks inferred intent"],
        "evidence_expectations": ["Every closure links focused runtime and UIAI proof"],
        "evidence_behavior": "Distinguish proposal, observed evidence, and canonical acceptance.",
        "communication_posture": "Concise, transparent, and recovery-oriented.",
        "stakeholder_posture": "Protect operator control and identify unresolved assumptions.",
        "non_responsibilities": [
            "Granting operational permissions",
            "Inventing semantic authority",
        ],
        "forbidden_assumptions": [
            "A role title implies permission",
            "Uncited claims are accepted",
        ],
        "escalation_triggers": [
            "Scope conflict",
            "Missing evidence",
            "Permission boundary ambiguity",
        ],
        "handoff_boundaries": [
            "UIAI owns browser execution",
            "Operator owns consequential approval",
        ],
        "tool_preferences": ["Operation Registry", "UIAI Engine"],
        "reviewer_lenses": ["security", "accessibility", "evidence quality"],
        "context_artifact_refs": [source_id],
        "context_claim_refs": [],
        "interview_answer_refs": [],
        "assumptions": [
            {
                "statement": "Spec 135 is the approved delivery contract.",
                "source_refs": [source_id],
                "status": "grounded",
            }
        ],
        "unresolved_questions": [],
        "redlines": redlines or [],
        "permission_profile_refs": ["permission-profile:operator-controlled"],
        "permission_assertions": permission_assertions or [],
    }


def mutate(base, path, body, attempts=20):
    for _ in range(attempts):
        body["expected_state_version"] = listed(base)["state_version"]
        status, payload = call(base, "POST", path, body)
        if status == 200:
            return payload
        assert status == 409, payload
        time.sleep(0.05)
    raise RuntimeError("role writer did not quiesce")


def review_body(profile, key, decision):
    return {
        **SCOPE,
        "role_profile_id": profile["role_profile_id"],
        "profile_revision": profile["revision"],
        "idempotency_key": key,
        "expected_state_version": 0,
        "decision": decision,
        "reviewed_by": "operator:vsmith",
        "rationale": f"Explicit operator {decision} after Context grounding and responsibility review.",
    }


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-ri1-"))
    process = log = None
    try:
        process, log, base = start(data)
        source = commit_context(base)["source"]
        source_id = source["source_id"]

        bad_grounding = draft_body("context-source:missing", "ri1-missing")
        bad_grounding["expected_state_version"] = listed(base)["state_version"]
        status, payload = call(base, "POST", "/v1/roles/profiles/draft", bad_grounding)
        assert status == 422 and payload["failure_class"] == "not_found", payload

        bad_permission = draft_body(
            source_id,
            "ri1-permission",
            permission_assertions=["permission to modify production"],
        )
        bad_permission["expected_state_version"] = listed(base)["state_version"]
        status, payload = call(base, "POST", "/v1/roles/profiles/draft", bad_permission)
        assert status == 422 and payload["failure_class"] == "permission_denied", (
            payload
        )

        hidden_permission = draft_body(source_id, "ri1-hidden-permission")
        hidden_permission["primary_responsibilities"] = [
            "permission to trade for the project"
        ]
        hidden_permission["expected_state_version"] = listed(base)["state_version"]
        status, payload = call(
            base, "POST", "/v1/roles/profiles/draft", hidden_permission
        )
        assert status == 422 and payload["failure_class"] == "permission_denied", (
            payload
        )

        drafted = mutate(
            base,
            "/v1/roles/profiles/draft",
            draft_body(source_id, "ri1-draft-v1"),
        )
        profile1 = drafted["profile"]
        assert profile1["revision"] == 1 and profile1["status"] == "pending_operator"
        assert profile1["grants_permissions"] is False
        assert profile1["grounding"]["context_artifact_refs"] == [source_id]
        assert profile1["assumptions"][0]["status"] == "grounded"
        assert drafted["responsibility_is_not_permission"] is True
        assert drafted["evidence_ref"] == source_id
        assert drafted["receipt_ref"].startswith("receipt:project-role-profile:")

        replay = call(
            base,
            "POST",
            "/v1/roles/profiles/draft",
            {**draft_body(source_id, "ri1-draft-v1"), "expected_state_version": 0},
        )[1]
        assert replay["replayed"] is True and replay["profile"] == profile1

        approved = mutate(
            base,
            "/v1/roles/profiles/review",
            review_body(profile1, "ri1-approve-v2", "approve"),
        )
        profile2 = approved["profile"]
        assert profile2["revision"] == 2 and profile2["status"] == "approved"
        assert profile2["review"]["decision"] == "approve"
        assert profile2["review"]["reviewed_by"] == "operator:vsmith"

        revised = mutate(
            base,
            "/v1/roles/profiles/draft",
            draft_body(
                source_id,
                "ri1-draft-v3",
                purpose="Deliver the approved critical path with explicit handoffs and bounded recovery.",
                redlines=[
                    {
                        "field": "purpose",
                        "before": profile2["purpose"],
                        "after": "Deliver the approved critical path with explicit handoffs and bounded recovery.",
                        "rationale": "Operator clarified the recovery and handoff boundary.",
                    }
                ],
            ),
        )
        profile3 = revised["profile"]
        assert profile3["revision"] == 3 and profile3["status"] == "pending_operator"
        assert profile3["redlines"][0]["field"] == "purpose"

        deferred = mutate(
            base,
            "/v1/roles/profiles/review",
            review_body(profile3, "ri1-defer-v4", "defer"),
        )
        profile4 = deferred["profile"]
        assert profile4["revision"] == 4 and profile4["status"] == "pending_operator"
        assert profile4["review"]["decision"] == "defer"

        rejected = mutate(
            base,
            "/v1/roles/profiles/review",
            review_body(profile4, "ri1-reject-v5", "reject"),
        )
        profile5 = rejected["profile"]
        assert profile5["revision"] == 5 and profile5["status"] == "superseded"
        assert profile5["review"]["decision"] == "reject"

        exact = listed(base)
        assert [item["revision"] for item in exact["profiles"]] == [1, 2, 3, 4, 5]
        assert exact["latest"]["revision"] == 5
        assert exact["approved"]["revision"] == 2
        other = listed(base, {**SCOPE, "attachment_id": "attachment:other"})
        assert other["profiles"] == [] and "latest" not in other

        stop(process, log)
        process = log = None
        process, log, base = start(data)
        resumed = listed(base)
        assert [item["revision"] for item in resumed["profiles"]] == [1, 2, 3, 4, 5]
        assert resumed["approved"]["revision"] == 2
        assert resumed["latest"]["review"]["decision"] == "reject"

        print(
            "Spec 135 RI1 Role Profile E2E: PASS (grounding, assumptions, permission separation, redline, approval/reject/defer, replay, restart)"
        )
    finally:
        if process is not None:
            stop(process, log)
        shutil.rmtree(data, ignore_errors=True)


if __name__ == "__main__":
    main()
