#!/usr/bin/env python3
"""C1 real Markdown/code/PDF Context ingestion, incremental update, health, and restart proof."""
from __future__ import annotations

import base64
import json
import os
import shutil
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "target/debug/focusa-daemon"
PROJECT = "/example/focusa-c1"
CONTINUITY = "focusa-cont-c1-ingestion"
ATTACHMENT = "attachment:c1-context"


def request(method: str, url: str, payload=None, expected=200):
    data = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(url, data=data, method=method, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=240) as response:
            body = json.loads(response.read())
            assert response.status == expected, (response.status, body)
            return body
    except urllib.error.HTTPError as error:
        body = json.loads(error.read())
        assert error.code == expected, (error.code, body)
        return body


def wait_health(base: str):
    for _ in range(180):
        try:
            request("GET", f"{base}/v1/health")
            return
        except Exception:
            time.sleep(0.1)
    raise AssertionError("daemon did not become healthy")


def start_daemon(data_dir: Path, port: int, docling_url: str | None):
    env = os.environ.copy()
    env.update({"FOCUSA_BIND": f"127.0.0.1:{port}", "FOCUSA_DATA_DIR": str(data_dir)})
    if docling_url:
        env["FOCUSA_DOCLING_BASE_URL"] = docling_url
    else:
        env.pop("FOCUSA_DOCLING_BASE_URL", None)
    log = open(data_dir / "daemon.log", "ab")
    process = subprocess.Popen([str(BINARY)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
    wait_health(f"http://127.0.0.1:{port}")
    return process, log


def stop(process, log):
    process.send_signal(signal.SIGTERM)
    try:
        process.wait(timeout=15)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)
    log.close()


def minimal_pdf(text: str) -> bytes:
    escaped = text.replace("\\", "\\\\").replace("(", "\\(").replace(")", "\\)")
    stream = f"BT /F1 18 Tf 72 720 Td ({escaped}) Tj ET\n".encode()
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
        f"<< /Length {len(stream)} >>\nstream\n".encode() + stream + b"endstream",
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    ]
    out = bytearray(b"%PDF-1.4\n")
    offsets = [0]
    for index, obj in enumerate(objects, 1):
        offsets.append(len(out))
        out.extend(f"{index} 0 obj\n".encode() + obj + b"\nendobj\n")
    xref = len(out)
    out.extend(f"xref\n0 {len(objects)+1}\n0000000000 65535 f \n".encode())
    for offset in offsets[1:]:
        out.extend(f"{offset:010d} 00000 n \n".encode())
    out.extend(f"trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode())
    return bytes(out)


def scope_query():
    return urllib.parse.urlencode({"project_root": PROJECT, "continuity_id": CONTINUITY, "attachment_id": ATTACHMENT})


def ingest_request(base: str, payload: dict):
    for _ in range(6):
        try:
            return request("POST", f"{base}/v1/context/sources/ingest", payload)
        except AssertionError as error:
            observed = error.args[0] if error.args else None
            if not isinstance(observed, tuple) or observed[0] != 409:
                raise
            payload["expected_state_version"] = request("GET", f"{base}/v1/context/sources?{scope_query()}")["state_version"]
    raise AssertionError("Context ingestion did not settle after bounded writer-conflict retries")


def ingest_payload(version: int, **overrides):
    payload = {
        "project_root": PROJECT,
        "continuity_id": CONTINUITY,
        "attachment_id": ATTACHMENT,
        "idempotency_key": "c1-markdown-r1",
        "expected_state_version": version,
        "source_kind": "markdown",
        "source_locator": "README.md",
        "source_revision": "git:1111111",
        "title": "Mission Context",
        "mime_type": "text/markdown",
        "content": "# Mission Context\n\nCanonical Markdown ingestion.",
    }
    payload.update(overrides)
    return payload


def main():
    assert BINARY.exists(), "build focusa-daemon before running C1 E2E"
    data_dir = Path(tempfile.mkdtemp(prefix="focusa-c1-ingestion-"))
    port_socket = socket.socket()
    port_socket.bind(("127.0.0.1", 0))
    port = port_socket.getsockname()[1]
    port_socket.close()
    docling_url = os.environ.get("FOCUSA_TEST_DOCLING_URL")
    process, log = start_daemon(data_dir, port, docling_url)
    base = f"http://127.0.0.1:{port}"
    try:
        listed = request("GET", f"{base}/v1/context/sources?{scope_query()}")
        version = listed["state_version"]

        markdown_payload = ingest_payload(version)
        markdown = ingest_request(base, markdown_payload)
        assert markdown["canonical"] and markdown["source"]["adapter_id"] == "focusa.local_text.v1"
        assert markdown["source"]["health"]["status"] == "healthy"
        assert markdown["source"]["revision"] == 1
        version = markdown["state_version"]
        replay = ingest_request(base, markdown_payload)
        assert replay["replayed"] and replay["tool_result"]["status"] == "no_op"
        version = request("GET", f"{base}/v1/context/sources?{scope_query()}")["state_version"]

        code = ingest_request(base, ingest_payload(
            version,
            idempotency_key="c1-code-r1",
            source_kind="code",
            source_locator="src/lib.rs",
            source_revision="git:2222222",
            title="Core source",
            mime_type="text/x-rust",
            content="pub fn mission() -> &'static str { \"ready\" }",
        ))
        assert code["source"]["revision"] == 1
        code_source_id = code["source"]["source_id"]
        version = request("GET", f"{base}/v1/context/sources?{scope_query()}")["state_version"]
        updated = ingest_request(base, ingest_payload(
            version,
            idempotency_key="c1-code-r2",
            source_kind="code",
            source_locator="src/lib.rs",
            source_revision="git:3333333",
            title="Core source",
            mime_type="text/x-rust",
            content="pub fn mission() -> &'static str { \"resumed\" }",
        ))
        assert updated["source"]["source_id"] == code_source_id
        assert updated["source"]["revision"] == 2
        version = updated["state_version"]
        listed = request("GET", f"{base}/v1/context/sources?{scope_query()}")
        assert len(listed["sources"]) == 2

        version = listed["state_version"]
        adapter_health = request("GET", f"{base}/v1/context/adapters/docling/health")
        if docling_url:
            assert adapter_health["configured"] and adapter_health["status"] == "healthy", adapter_health
            pdf = minimal_pdf("Mission Canvas PDF Context")
            converted = ingest_request(base, ingest_payload(
                version,
                idempotency_key="c1-pdf-r1",
                source_kind="pdf",
                source_locator="mission-context.pdf",
                source_revision="sha256:pdf-r1",
                title="Mission PDF Context",
                mime_type="application/pdf",
                content=None,
                content_base64=base64.b64encode(pdf).decode(),
            ))
            assert converted["source"]["adapter_id"] == "docling-serve.v1"
            assert converted["source"]["ingestion_status"] == "completed"
            assert converted["source"]["content"].strip()
            assert any(x.startswith("docling_status=") for x in converted["source"]["extraction_diagnostics"])
            version = converted["state_version"]
        else:
            assert not adapter_health["configured"] and adapter_health["status"] == "offline"
            failed = request("POST", f"{base}/v1/context/sources/ingest", ingest_payload(
                version,
                idempotency_key="c1-pdf-offline",
                source_kind="pdf",
                source_locator="mission-context.pdf",
                source_revision="sha256:pdf-offline",
                title="Mission PDF Context",
                mime_type="application/pdf",
                content=None,
                content_base64=base64.b64encode(minimal_pdf("Offline PDF")).decode(),
            ), expected=503)
            assert failed["status"] == "offline" and "FOCUSA_DOCLING_BASE_URL" in failed["summary"]

        stop(process, log)
        process, log = start_daemon(data_dir, port, docling_url)
        resumed = request("GET", f"{base}/v1/context/sources?{scope_query()}")
        expected_count = 3 if docling_url else 2
        assert len(resumed["sources"]) == expected_count
        assert any(source["source_id"] == code_source_id and source["revision"] == 2 for source in resumed["sources"])
        assert resumed["state_version"] >= version
        print(f"Spec 135 C1 Context ingestion E2E: PASS (sources={expected_count}, docling={'real' if docling_url else 'offline fail-closed'}, restart=resumed)")
    finally:
        if process.poll() is None:
            stop(process, log)
        shutil.rmtree(data_dir, ignore_errors=True)


if __name__ == "__main__":
    main()
