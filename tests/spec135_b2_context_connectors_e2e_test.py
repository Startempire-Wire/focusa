#!/usr/bin/env python3
"""Spec 135B-2 real-source, provenance, deduplication, and recovery E2E proof."""

import json
import socket
import sys
import tempfile
from pathlib import Path
from urllib.parse import urlencode

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tests"))
import spec135_context_ingestion_e2e_test as harness  # noqa: E402

CONTINUITY = "focusa-cont-b2-context"
ATTACHMENT = "attachment:b2-context"


def list_sources(base: str, project: Path):
    query = urlencode(
        {
            "project_root": str(project),
            "continuity_id": CONTINUITY,
            "attachment_id": ATTACHMENT,
        }
    )
    return harness.request("GET", f"{base}/v1/context/sources?{query}")


def ingest(base: str, project: Path, version: int, **overrides):
    payload = {
        "project_root": str(project),
        "continuity_id": CONTINUITY,
        "attachment_id": ATTACHMENT,
        "idempotency_key": "b2-default",
        "expected_state_version": version,
        "source_kind": "file",
        "source_locator": "docs/context.md",
        "source_revision": "file:1",
        "title": "Local project context",
        "mime_type": "text/markdown",
    }
    payload.update(overrides)
    return harness.request("POST", f"{base}/v1/context/sources/ingest", payload)


def main():
    assert harness.BINARY.exists(), "build focusa-daemon before running B2 E2E"
    temp_root = Path(tempfile.mkdtemp(prefix="focusa-b2-context-"))
    project = temp_root / "project"
    (project / "docs").mkdir(parents=True)
    local_content = "# Source-linked Context\n\nRelease evidence must remain cited."
    (project / "docs/context.md").write_text(local_content)
    (temp_root / "outside.md").write_text("must not be ingested")

    port_socket = socket.socket()
    port_socket.bind(("127.0.0.1", 0))
    port = port_socket.getsockname()[1]
    port_socket.close()
    data_dir = temp_root / "data"
    data_dir.mkdir()
    process, log = harness.start_daemon(data_dir, port, None)
    base = f"http://127.0.0.1:{port}"
    try:
        listed = list_sources(base, project)
        local = ingest(base, project, listed["state_version"])
        artifact = local["source"]["artifact"]
        assert artifact["schema"] == "focusa.project_context_artifact.v1"
        assert artifact["source_ref"] == "docs/context.md"
        assert artifact["content_sha256"] == local["source"]["content_hash"]
        assert artifact["scope"]["project_root"] == str(project)
        assert artifact["classification"]["freshness_status"] == "current"
        assert local["source"]["health"]["read_write_posture"] == "read-only"

        research = ingest(
            base,
            project,
            local["state_version"],
            idempotency_key="b2-research",
            source_kind="research",
            source_locator="https://example.com/reference",
            source_revision="etag:reference-1",
            title="Public reference",
            mime_type="text/markdown",
            content=local_content,
            source_url="https://example.com/reference",
            author="Reference author",
            page_or_message_ref="section:release",
            domain_pack_refs=["domain-pack:release"],
            verification_policy_refs=["verification-policy:public-source"],
        )
        research_artifact = research["source"]["artifact"]
        assert research_artifact["provenance"]["source_url"].startswith(
            "https://example.com/reference"
        )
        assert (
            research_artifact["duplicate_of_artifact_ref"]
            == artifact["artifact_id"]
        )

        connected = ingest(
            base,
            project,
            research["state_version"],
            idempotency_key="b2-connected",
            source_kind="connected",
            source_locator="drive://folder/file-1",
            source_revision="drive:42",
            title="Connected source",
            mime_type="text/plain",
            content="Connected source delta with bounded provenance.",
            connector_id="google-drive.readonly.v1",
            account_ref="account-ref:operator-drive",
            oauth_scopes=["drive.file.readonly"],
            sync_cursor="cursor:42",
            incremental_sync_method="changes_cursor",
            rate_limit_posture="provider_backoff",
            recovery_action="reauthorize and resume from the last durable cursor",
        )
        health = connected["source"]["health"]
        provenance = connected["source"]["artifact"]["provenance"]
        assert provenance["connector_id"] == "google-drive.readonly.v1"
        assert provenance["account_ref"] == "account-ref:operator-drive"
        assert health["cursor_state"] == "cursor:42"
        assert health["incremental_sync_method"] == "changes_cursor"
        assert health["recovery_action"].startswith("reauthorize")
        assert "revoke" in health["revocation_behavior"]

        rejected = harness.request(
            "POST",
            f"{base}/v1/context/sources/ingest",
            {
                "project_root": str(project),
                "continuity_id": CONTINUITY,
                "attachment_id": ATTACHMENT,
                "idempotency_key": "b2-path-escape",
                "expected_state_version": connected["state_version"],
                "source_kind": "file",
                "source_locator": "../outside.md",
                "source_revision": "outside:1",
                "title": "Escaped source",
                "mime_type": "text/markdown",
            },
            expected=403,
        )
        assert rejected["failure_class"] == "permission_denied"

        harness.stop(process, log)
        process, log = harness.start_daemon(data_dir, port, None)
        resumed = list_sources(base, project)
        assert len(resumed["sources"]) == 3
        assert all(source["artifact"]["artifact_id"] for source in resumed["sources"])
        assert any(
            source["health"].get("cursor_state") == "cursor:42"
            for source in resumed["sources"]
        )
    finally:
        if process.poll() is None:
            harness.stop(process, log)

    print("Spec 135 B2 Context artifacts/connectors E2E: PASS (file/web, provenance, dedup, cursor, recovery, restart)")


if __name__ == "__main__":
    main()
