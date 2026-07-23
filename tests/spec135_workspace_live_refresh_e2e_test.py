#!/usr/bin/env python3
"""SPEC135-U2 durable, exact-scope Workspace invalidation SSE proof."""

import hashlib
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
SCOPE_A = {
    "project_root": "/example/focusa",
    "continuity_id": "focusa-cont-u2",
    "attachment_id": "attachment:u2-a",
}
SCOPE_B = {**SCOPE_A, "attachment_id": "attachment:u2-b"}


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
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.status, json.load(response)


def start(data_dir):
    selected_port = free_port()
    env = os.environ.copy()
    env.update(
        FOCUSA_DATA_DIR=str(data_dir),
        FOCUSA_BIND=f"127.0.0.1:{selected_port}",
    )
    log = open(data_dir / "daemon.log", "ab")
    process = subprocess.Popen(
        [str(BINARY)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT
    )
    base = f"http://127.0.0.1:{selected_port}"
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


def stop(process, log_file):
    process.send_signal(signal.SIGTERM)
    try:
        process.wait(15)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(5)
    log_file.close()


def listed(base, scope):
    return call(
        base, "GET", "/v1/workspace/artifacts?" + urllib.parse.urlencode(scope)
    )[1]


def artifact_body(scope, idempotency_key, artifact_url):
    preview = "workspace live refresh proof"
    return {
        **scope,
        "idempotency_key": idempotency_key,
        "expected_state_version": 0,
        "artifact_kind": "image",
        "mime_type": "image/png",
        "title": "Workspace live refresh artifact",
        "summary": "Bounded artifact used to prove targeted invalidation.",
        "handle_ref": "uiai-screenshot:session=u2:artifact-a",
        "artifact_url": artifact_url,
        "inline_preview": preview,
        "sha256": hashlib.sha256(preview.encode()).hexdigest(),
        "size_bytes": len(preview),
        "source_system": "uiai",
        "source_ref": "uiai-browser:session=u2:artifact-a",
        "source_url": "https://example.invalid/u2",
        "project_identity_ref": "project:focusa",
        "workpoint_id": "focusa-mc-u2",
        "work_item_ref": "focusa-mc-u2",
        "instance_id": "focusa-instance:u2",
        "focusa_session_id": "focusa-session:u2",
        "work_surface_id": f"surface:{scope['attachment_id']}",
        "uiai_session_id": "uiai-session:u2",
        "browser_context_id": "browser-context:u2",
        "browser_target_id": "browser-target:u2",
        "diagnostics_refs": ["uiai-diagnostics:session=u2:seq=1"],
        "evidence_refs": ["evidence:workspace-live-refresh:u2"],
        "domain_pack_refs": [],
        "candidate_object_refs": [],
        "candidate_link_refs": [],
        "candidate_claim_refs": [],
        "verification_policy_refs": [],
        "semantic_delta_refs": [],
        "citation_refs": ["source:https://example.invalid/u2"],
        "evidence_status": "verified",
        "redaction_status": "secret_safe",
        "freshness_status": "fresh",
        "provenance_status": "verified",
        "retention_policy": "project_evidence",
        "cleanup_action": "close UIAI session asynchronously",
        "preferred_renderer": "image_preview",
        "fallback_renderer": "artifact_card_and_open",
        "render_width": 1440,
        "render_height": 1000,
    }


def intake(base, body, attempts=20):
    for _ in range(attempts):
        body["expected_state_version"] = listed(
            base,
            {
                "project_root": body["project_root"],
                "continuity_id": body["continuity_id"],
                "attachment_id": body["attachment_id"],
            },
        )["state_version"]
        try:
            return call(base, "POST", "/v1/workspace/artifacts/intake", body)[1]
        except urllib.error.HTTPError as error:
            if error.code != 409:
                raise
            time.sleep(0.05)
    raise RuntimeError("canonical writer did not quiesce")


def stream_url(base, scope, cursor):
    query = urllib.parse.urlencode({"cursor": cursor, **scope})
    return f"{base}/v1/events/stream?{query}"


def curl_stream(url, seconds=2):
    result = subprocess.run(
        ["curl", "-sS", "-N", "--max-time", str(seconds), url],
        text=True,
        capture_output=True,
        timeout=seconds + 3,
        check=False,
    )
    return result.stdout


def stream_process(url, seconds=4):
    return subprocess.Popen(
        ["curl", "-sS", "-N", "--max-time", str(seconds), url],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def envelopes(output):
    found = []
    for line in output.splitlines():
        if line.startswith("data:"):
            value = json.loads(line.removeprefix("data:").strip())
            if value.get("schema") == "focusa.stream_event.v1":
                found.append(value)
    return found


def workspace_event(output):
    matches = [
        event
        for event in envelopes(output)
        if event["event_type"] == "workspace_artifact_linked"
    ]
    assert matches, output
    return matches[-1]


def assert_bounded(event, scope, expected_revision):
    assert event["scope"]["project_root"] == scope["project_root"]
    assert event["scope"]["continuity_id"] == scope["continuity_id"]
    assert event["scope"]["attachment_id"] == scope["attachment_id"]
    assert event["source_state_revision"] == expected_revision
    assert event["payload"]["schema"] == "focusa.workspace_event.v1"
    assert event["payload"]["semantic_authority"] is False
    assert event["payload_ref"] == event["payload"]["artifact_id"]
    serialized = json.dumps(event).lower()
    for forbidden in (
        "inline_preview",
        "artifact_path",
        "sha256",
        "cookies",
        "browser_storage",
        "semantic_delta_refs",
    ):
        assert forbidden not in serialized


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-u2-"))
    process = log = None
    try:
        process, log, base = start(data)
        first = intake(
            base,
            artifact_body(SCOPE_A, "u2-link-1", "https://example.invalid/u2/one"),
        )
        first_event = workspace_event(curl_stream(stream_url(base, SCOPE_A, 0)))
        assert_bounded(first_event, SCOPE_A, first["state_version"])
        assert first_event["invalidate"] == sorted(
            [
                "mission_canvas.surface_detail:surface:attachment:u2-a",
                "workpoint.current",
                "workspace.artifacts:attachment:u2-a",
                "workspace.history:attachment:u2-a",
                "workspace.sidebar.proof",
            ]
        )
        first_cursor = first_event["cursor"]

        second_body = artifact_body(
            SCOPE_A, "u2-link-2", "https://example.invalid/u2/two"
        )
        second = intake(base, second_body)
        second_event = workspace_event(
            curl_stream(stream_url(base, SCOPE_A, first_cursor))
        )
        assert_bounded(second_event, SCOPE_A, second["state_version"])
        assert int(second_event["cursor"]) > int(first_cursor)
        assert (
            second_event["payload"]["artifact_id"]
            == first_event["payload"]["artifact_id"]
        )
        second_cursor = second_event["cursor"]

        stop(process, log)
        process = log = None
        process, log, base = start(data)
        replayed = workspace_event(curl_stream(stream_url(base, SCOPE_A, first_cursor)))
        assert replayed["event_id"] == second_event["event_id"]
        assert replayed["cursor"] == second_cursor

        live = stream_process(stream_url(base, SCOPE_A, second_cursor), seconds=4)
        time.sleep(0.4)
        third = intake(
            base,
            artifact_body(SCOPE_A, "u2-link-3", "https://example.invalid/u2/three"),
        )
        live_output, _ = live.communicate(timeout=8)
        live_event = workspace_event(live_output)
        assert_bounded(live_event, SCOPE_A, third["state_version"])
        assert int(live_event["cursor"]) > int(second_cursor)

        other = intake(
            base,
            artifact_body(SCOPE_B, "u2-other", "https://example.invalid/u2/other"),
        )
        assert other["artifact"]["origin"]["attachment_id"] == SCOPE_B["attachment_id"]
        unrelated_output = curl_stream(
            stream_url(base, SCOPE_A, live_event["cursor"]), seconds=1
        )
        assert not [
            event
            for event in envelopes(unrelated_output)
            if event["event_type"] == "workspace_artifact_linked"
        ]

        applied_event_ids = {
            first_event["event_id"],
            second_event["event_id"],
            live_event["event_id"],
        }
        assert len(applied_event_ids) == 3
        print(
            "Spec 135 U2 live refresh E2E: PASS (bounded event, exact filter, reconnect replay, restart, live tail, unrelated suppression)"
        )
    finally:
        if process is not None:
            stop(process, log)
        shutil.rmtree(data, ignore_errors=True)


if __name__ == "__main__":
    main()
