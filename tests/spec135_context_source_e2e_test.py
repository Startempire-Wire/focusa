#!/usr/bin/env python3
"""F12: real canonical Context commit, idempotency, event, restart, exact resume."""

from __future__ import annotations

import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
from urllib.parse import urlencode
from urllib.error import HTTPError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("FOCUSA_DAEMON_BIN", ROOT / "target/debug/focusa-daemon"))
assert BINARY.exists(), f"build daemon first: cargo build -p focusa-api ({BINARY})"

scope = {
    "project_root": "/example/focusa",
    "continuity_id": "focusa-cont-f12-e2e",
    "attachment_id": "attachment:f12-e2e",
}


def free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def get_json(url: str) -> dict:
    with urlopen(url, timeout=5) as response:
        return json.load(response)


def post_json(url: str, body: dict) -> dict:
    request = Request(
        url,
        data=json.dumps(body).encode(),
        headers={"content-type": "application/json"},
        method="POST",
    )
    with urlopen(request, timeout=10) as response:
        return json.load(response)


with tempfile.TemporaryDirectory(prefix="focusa-f12-e2e-") as data_dir:
    port = free_port()
    base = f"http://127.0.0.1:{port}"
    query = urlencode(scope)
    process: subprocess.Popen[bytes] | None = None

    def start() -> None:
        nonlocal_process = subprocess.Popen(
            [str(BINARY)],
            cwd=ROOT,
            env={
                **os.environ,
                "FOCUSA_BIND": f"127.0.0.1:{port}",
                "FOCUSA_DATA_DIR": data_dir,
            },
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        globals_holder[0] = nonlocal_process
        deadline = time.time() + 30
        while time.time() < deadline:
            try:
                get_json(f"{base}/v1/health")
                return
            except Exception:
                time.sleep(0.1)
        raise AssertionError("daemon did not become healthy")

    def stop() -> None:
        current = globals_holder[0]
        if current is not None:
            current.terminate()
            try:
                current.wait(timeout=10)
            except subprocess.TimeoutExpired:
                current.kill()
            globals_holder[0] = None

    globals_holder: list[subprocess.Popen[bytes] | None] = [process]
    try:
        start()
        body = {
            **scope,
            "idempotency_key": "f12-e2e-idempotency-v1",
            "expected_state_version": 0,
            "source_kind": "markdown",
            "title": "F12 end-to-end Context",
            "content": "# Context\n\nCanonical reducer, durable event, Evidence, Receipt.",
        }
        committed = None
        for _ in range(10):
            before = get_json(f"{base}/v1/context/sources?{query}")
            body["expected_state_version"] = before["state_version"]
            try:
                committed = post_json(f"{base}/v1/context/sources/commit?{query}", body)
                break
            except HTTPError as error:
                if error.code != 409:
                    raise
                time.sleep(0.05)
        assert committed is not None, "Context commit remained writer-conflicted"
        replayed = post_json(f"{base}/v1/context/sources/commit?{query}", body)
        after = get_json(f"{base}/v1/context/sources?{query}")
        events = get_json(
            f"{base}/v1/events/recent?limit=20&event_type=ContextSourceCommitted"
        )

        assert committed["canonical"] is True and committed["replayed"] is False
        assert committed["tool_result"]["schema"] == "focusa.tool_result.v1"
        assert committed["tool_result"]["status"] == "completed"
        assert committed["evidence_ref"].startswith("evidence:context-source:")
        assert committed["receipt_ref"].startswith("receipt:context-source:")
        assert (
            committed["source"]["receipt"]["after_state_version"]
            == committed["state_version"]
        )
        assert (
            replayed["replayed"] is True
            and replayed["tool_result"]["status"] == "no_op"
        )
        assert len(after["sources"]) == 1
        assert len(events["events"]) == 1

        stop()
        start()
        resumed = get_json(f"{base}/v1/context/sources?{query}")
        assert len(resumed["sources"]) == 1
        assert resumed["sources"][0]["source_id"] == committed["source"]["source_id"]
        assert (
            resumed["sources"][0]["receipt"]["receipt_ref"] == committed["receipt_ref"]
        )
        assert (
            resumed["sources"][0]["evidence"]["evidence_ref"]
            == committed["evidence_ref"]
        )
    finally:
        stop()

print(
    "Spec 135 F12 Context source E2E: PASS (canonical commit, idempotency, event, restart/resume, Evidence/Receipt)"
)
