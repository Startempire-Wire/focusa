#!/usr/bin/env python3
"""SPEC135-C2 runtime proof: scoped FTS5 retrieval, citations, contradictions, restart, lexical fallback."""

from __future__ import annotations

import json
import os
import pathlib
import shutil
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
import urllib.parse
from jsonschema import Draft202012Validator

ROOT = pathlib.Path(__file__).resolve().parents[1]
BINARY = ROOT / "target/debug/focusa-daemon"
SCOPE = {
    "project_root": "/example/focusa",
    "continuity_id": "focusa-cont-c2",
    "attachment_id": "attachment:c2",
}


def port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def request(base: str, method: str, path: str, payload=None):
    body = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(
        base + path,
        data=body,
        method=method,
        headers={"content-type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=30) as response:
        return response.status, json.load(response)


def start(data: pathlib.Path, listen_port: int):
    env = os.environ.copy()
    env.update(
        {
            "FOCUSA_DATA_DIR": str(data),
            "FOCUSA_BIND": f"127.0.0.1:{listen_port}",
            "FOCUSA_CONTEXT_VECTOR_MODE": "disabled",
        }
    )
    process = subprocess.Popen(
        [str(BINARY)],
        cwd=ROOT,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )
    base = f"http://127.0.0.1:{listen_port}"
    for _ in range(240):
        if process.poll() is not None:
            raise RuntimeError(process.stderr.read().decode(errors="replace"))
        try:
            with urllib.request.urlopen(base + "/v1/health", timeout=1) as response:
                if response.status == 200:
                    return process, base
        except (urllib.error.URLError, TimeoutError):
            time.sleep(0.05)
    process.terminate()
    raise RuntimeError("daemon did not become healthy")


def stop(process):
    process.terminate()
    try:
        process.wait(timeout=8)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=3)


def current_version(base: str) -> int:
    query = urllib.parse.urlencode(SCOPE)
    _, body = request(base, "GET", f"/v1/context/sources?{query}")
    return int(body["state_version"])


def ingest(base: str, key: str, title: str, content: str, expected: int):
    payload = {
        **SCOPE,
        "idempotency_key": key,
        "expected_state_version": expected,
        "source_kind": "markdown",
        "title": title,
        "content": content,
        "source_locator": f"file:///{title.lower().replace(' ', '-')}.md",
        "source_revision": "git:c2",
        "mime_type": "text/markdown",
    }
    for attempt in range(4):
        try:
            status, body = request(base, "POST", "/v1/context/sources/ingest", payload)
            assert status == 200, body
            return body
        except urllib.error.HTTPError as error:
            if error.code != 409 or attempt == 3:
                raise
            payload["expected_state_version"] = current_version(base)
            time.sleep(0.05)
    raise AssertionError("unreachable ingestion retry state")


def retrieve(base: str, scope=SCOPE):
    status, body = request(
        base,
        "POST",
        "/v1/context/retrieve",
        {
            **scope,
            "query": 'release policy signed artifacts deployment " OR * NOT',
            "limit": 8,
            "mode": "hybrid",
            "include_contradictions": True,
        },
    )
    assert status == 200, body
    return body


def main():
    assert BINARY.exists(), f"missing daemon binary: {BINARY}"
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-c2-e2e-"))
    process = None
    try:
        process, base = start(data, port())
        first = ingest(
            base,
            "c2-positive",
            "Release Policy A",
            "The release policy requires signed artifacts for every deployment.",
            current_version(base),
        )
        second = ingest(
            base,
            "c2-negative",
            "Release Policy B",
            "The release policy does not require signed artifacts for every deployment.",
            first["state_version"],
        )
        assert second["state_version"] > first["state_version"]

        result = retrieve(base)
        schema = json.loads(
            (
                ROOT
                / "docs/contracts/spec135/generated-contract-v1/json-schema/focusa.context_retrieve_response.v1.json"
            ).read_text()
        )
        Draft202012Validator(schema).validate(result)
        payload = result["result"]
        assert result["canonical_sources"] is True
        assert result["evidence_ref"].startswith("evidence:context-retrieval:")
        assert result["receipt_ref"].startswith("receipt:context-retrieval:")
        assert (
            payload["mode_requested"] == "hybrid" and payload["mode_used"] == "lexical"
        )
        assert payload["capabilities"]["degraded_to_lexical"] is True
        assert payload["capabilities"]["vector_index"] == "sqlite_vec.available"
        assert payload["indexed_source_count"] == 2 and payload["result_count"] == 2
        assert len(payload["contradictions"]) == 1
        assert all(
            hit["citation"]["source_revision"] == "git:c2" for hit in payload["hits"]
        )
        assert all(hit["citation"]["line_start"] >= 1 for hit in payload["hits"])
        first_ids = [hit["chunk_id"] for hit in payload["hits"]]

        _, isolated = request(
            base,
            "POST",
            "/v1/context/retrieve",
            {
                **{**SCOPE, "continuity_id": "other-workstream"},
                "query": "release policy",
                "mode": "lexical",
            },
        )
        assert isolated["result"]["result_count"] == 0
        stop(process)
        process = None

        process, base = start(data, port())
        resumed = retrieve(base)["result"]
        assert [hit["chunk_id"] for hit in resumed["hits"]] == first_ids
        assert (
            resumed["indexed_source_count"] == 2 and len(resumed["contradictions"]) == 1
        )
        print(
            "Spec 135 C2 Context retrieval E2E: PASS (FTS5, sqlite-vec projection, citations, contradictions, scope, fallback, restart)"
        )
    finally:
        if process is not None:
            stop(process)
        shutil.rmtree(data, ignore_errors=True)


if __name__ == "__main__":
    main()
