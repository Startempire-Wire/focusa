#!/usr/bin/env python3
"""SPEC135-C3 runtime proof: canonical claim graph, contradiction decisions, reactive projection, restart."""

from __future__ import annotations
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
BINARY = ROOT / "target/debug/focusa-daemon"
SCOPE = {
    "project_root": "/example/focusa",
    "continuity_id": "focusa-cont-c3",
    "attachment_id": "attachment:c3",
}


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def call(base, method, path, body=None):
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(
        base + path,
        data=data,
        method=method,
        headers={"content-type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=30) as response:
        return response.status, json.load(response)


def start(data):
    port = free_port()
    env = os.environ.copy()
    env.update(FOCUSA_DATA_DIR=str(data), FOCUSA_BIND=f"127.0.0.1:{port}")
    log = open(data / "daemon.log", "ab")
    proc = subprocess.Popen(
        [str(BINARY)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT
    )
    base = f"http://127.0.0.1:{port}"
    for _ in range(240):
        if proc.poll() is not None:
            raise RuntimeError((data / "daemon.log").read_text(errors="replace"))
        try:
            with urllib.request.urlopen(base + "/v1/health", timeout=1) as response:
                if response.status == 200:
                    return proc, log, base
        except (urllib.error.URLError, TimeoutError):
            time.sleep(0.05)
    raise RuntimeError("daemon did not become healthy")


def stop(proc, log):
    proc.send_signal(signal.SIGTERM)
    try:
        proc.wait(timeout=15)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=5)
    log.close()


def read_graph(base, scope=SCOPE):
    _, body = call(base, "GET", "/v1/context/graph?" + urllib.parse.urlencode(scope))
    return body


def mutate(base, key, action, **fields):
    payload = {
        **SCOPE,
        "idempotency_key": key,
        "expected_state_version": read_graph(base)["state_version"],
        "action": action,
        **fields,
    }
    for attempt in range(5):
        try:
            status, body = call(base, "POST", "/v1/context/graph/mutate", payload)
            assert status == 200
            return body
        except urllib.error.HTTPError as error:
            if error.code != 409 or attempt == 4:
                raise
            payload["expected_state_version"] = read_graph(base)["state_version"]
            time.sleep(0.05)
    raise AssertionError("unreachable")


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-c3-e2e-"))
    proc = log = None
    try:
        proc, log, base = start(data)
        a = mutate(
            base,
            "claim-a",
            "propose_claim",
            claim="Release artifacts require signatures.",
            source_citation_refs=["citation:source-a"],
            confidence=0.92,
        )
        claim_a = next(
            claim for claim in a["claims"] if claim["idempotency_key"] == "claim-a"
        )
        b = mutate(
            base,
            "claim-b",
            "propose_claim",
            claim="Release artifacts do not require signatures.",
            source_citation_refs=["citation:source-b"],
            confidence=0.88,
        )
        claim_b = next(
            claim for claim in b["claims"] if claim["idempotency_key"] == "claim-b"
        )
        opened = mutate(
            base,
            "edge-a-b",
            "open_contradiction",
            left_claim_id=claim_a["claim_id"],
            right_claim_id=claim_b["claim_id"],
            rationale="Sources make opposite release assertions.",
        )
        edge = opened["contradictions"][0]
        assert edge["status"] == "open"
        assert set(opened["projection"]["blocked_claim_refs"]) == {
            claim_a["claim_id"],
            claim_b["claim_id"],
        }
        assert opened["projection"]["accepted_claim_refs"] == []

        reviewed = mutate(
            base,
            "review-a",
            "review_claim",
            claim_id=claim_a["claim_id"],
            review_outcome="accept",
            actor="operator",
            rationale="Source A is the signed release policy.",
            source_citation_refs=["citation:source-a"],
        )
        assert (
            next(
                claim
                for claim in reviewed["claims"]
                if claim["claim_id"] == claim_a["claim_id"]
            )["status"]
            == "accepted"
        )
        assert reviewed["projection"]["accepted_claim_refs"] == [], (
            "open contradiction must block reactive acceptance"
        )

        resolved = mutate(
            base,
            "resolve-a-b",
            "resolve_contradiction",
            contradiction_id=edge["contradiction_id"],
            resolution="accept_left",
            selected_claim_id=claim_a["claim_id"],
            actor="operator",
            rationale="Verified signed release policy wins.",
            source_citation_refs=["citation:source-a"],
        )
        assert resolved["contradictions"][0]["status"] == "resolved"
        statuses = {claim["claim_id"]: claim["status"] for claim in resolved["claims"]}
        assert (
            statuses[claim_a["claim_id"]] == "accepted"
            and statuses[claim_b["claim_id"]] == "rejected"
        )
        projection = resolved["projection"]
        assert projection["accepted_claim_refs"] == [claim_a["claim_id"]]
        assert (
            projection["blocked_claim_refs"] == []
            and projection["unresolved_contradiction_refs"] == []
        )
        assert len(resolved["decisions"]) == 2
        receipt = resolved["receipt_ref"]
        evidence = resolved["evidence_ref"]

        replay = mutate(
            base,
            "resolve-a-b",
            "resolve_contradiction",
            contradiction_id=edge["contradiction_id"],
            resolution="accept_left",
            selected_claim_id=claim_a["claim_id"],
            actor="operator",
            rationale="Verified signed release policy wins.",
        )
        assert (
            replay["replayed"] is True
            and replay["receipt_ref"] == receipt
            and replay["evidence_ref"] == evidence
        )
        ids = [claim["claim_id"] for claim in replay["claims"]]
        stop(proc, log)
        proc = log = None

        proc, log, base = start(data)
        resumed = read_graph(base)
        assert [claim["claim_id"] for claim in resumed["claims"]] == ids
        assert resumed["projection"] == projection
        assert len(resumed["decisions"]) == 2
        isolated = read_graph(base, {**SCOPE, "continuity_id": "other"})
        assert isolated["claims"] == [] and isolated["contradictions"] == []
        print(
            "Spec 135 C3 Context claim graph E2E: PASS (claims, contradiction, approval, reactive projection, idempotency, restart)"
        )
    finally:
        if proc is not None:
            stop(proc, log)
        shutil.rmtree(data, ignore_errors=True)


if __name__ == "__main__":
    main()
