#!/usr/bin/env python3
"""SPEC135-U1 real bridge proof: bounded UIAI artifact descriptor, Evidence, scope, idempotency, restart."""

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
SCOPE = {
    "project_root": "/example/focusa",
    "continuity_id": "focusa-cont-u1",
    "attachment_id": "attachment:u1",
}


def port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def call(base, method, path, body=None):
    req = urllib.request.Request(
        base + path,
        data=None if body is None else json.dumps(body).encode(),
        method=method,
        headers={"content-type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=30) as r:
        return r.status, json.load(r)


def start(data):
    p = port()
    env = os.environ.copy()
    env.update(FOCUSA_DATA_DIR=str(data), FOCUSA_BIND=f"127.0.0.1:{p}")
    log = open(data / "daemon.log", "ab")
    proc = subprocess.Popen(
        [str(BINARY)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT
    )
    base = f"http://127.0.0.1:{p}"
    for _ in range(240):
        try:
            with urllib.request.urlopen(base + "/v1/health", timeout=1) as r:
                if r.status == 200:
                    return proc, log, base
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


def listed(base, scope=SCOPE):
    return call(
        base, "GET", "/v1/workspace/artifacts?" + urllib.parse.urlencode(scope)
    )[1]


def intake_with_writer_retry(base, body, attempts=20):
    """Respect concurrent daemon writers while preserving exact version authority."""
    for _ in range(attempts):
        body["expected_state_version"] = listed(base)["state_version"]
        try:
            return call(base, "POST", "/v1/workspace/artifacts/intake", body)
        except urllib.error.HTTPError as error:
            if error.code != 409:
                raise
            time.sleep(0.05)
    raise RuntimeError("canonical writer did not quiesce for Workspace Artifact intake")


def rejected(base, body, expected_status):
    body["expected_state_version"] = listed(base)["state_version"]
    try:
        call(base, "POST", "/v1/workspace/artifacts/intake", body)
    except urllib.error.HTTPError as error:
        assert error.code == expected_status
        return json.load(error)
    raise AssertionError(f"expected HTTP {expected_status}")


def main():
    data = pathlib.Path(tempfile.mkdtemp(prefix="focusa-u1-"))
    proc = log = None
    try:
        proc, log, base = start(data)
        preview = "console_errors=0; exceptions=0; failed_requests=0"
        sha = hashlib.sha256(preview.encode()).hexdigest()
        body = {
            **SCOPE,
            "idempotency_key": "u1-diagnostics",
            "expected_state_version": 0,
            "artifact_kind": "diagnostics",
            "mime_type": "application/json",
            "title": "UIAI retrieval diagnostics",
            "summary": "Bounded browser diagnostics for the cited Context workflow.",
            "handle_ref": "uiai-diagnostics:session=biw4d3UW:seq=8",
            "artifact_url": "https://fpv.wpuiai.com/m/sunny-summit-a09a",
            "inline_preview": preview,
            "sha256": sha,
            "size_bytes": len(preview),
            "source_system": "uiai",
            "source_ref": "uiai-browser:session=biw4d3UW",
            "source_url": "https://example.invalid/context-retrieve",
            "project_identity_ref": "project:focusa",
            "workpoint_id": "focusa-mc-u1",
            "work_item_ref": "focusa-mc-u1",
            "instance_id": "focusa-instance:u1-e2e",
            "work_surface_id": "surface:u1-research",
            "uiai_session_id": "biw4d3UW",
            "browser_context_id": "browser-context:u1",
            "browser_target_id": "browser-target:u1",
            "diagnostics_refs": ["uiai-diagnostics:session=biw4d3UW:seq=8"],
            "evidence_refs": ["browser-diagnostics:2026-07-20T23:07:04.444Z"],
            "domain_pack_refs": ["domain-pack:software-delivery"],
            "candidate_object_refs": ["candidate-object:release-policy"],
            "candidate_link_refs": ["candidate-link:release-policy-source"],
            "candidate_claim_refs": ["context-claim:d1ca46af9971217fb0d74e10"],
            "verification_policy_refs": ["verification-policy:source-cited"],
            "semantic_delta_refs": [],
            "citation_refs": ["context-citation:a248110aa8107ce7b8fa3c9d"],
            "evidence_status": "linked",
            "redaction_status": "secret_safe",
            "freshness_status": "fresh",
            "provenance_status": "verified",
            "retention_policy": "project_evidence",
            "cleanup_action": "close UIAI session independently when work surface is done",
            "preferred_renderer": "diagnostics_inspector",
            "fallback_renderer": "bounded_text_and_handle",
            "render_width": 1440,
            "render_height": 1000,
        }
        status, result = intake_with_writer_retry(base, body)
        assert status == 200
        assert (
            result["canonical_link"]
            and result["external_artifact_authority"]
            and not result["replayed"]
        )
        artifact = result["artifact"]
        assert (
            artifact["source"]["system"] == "uiai"
            and artifact["trust"]["evidence_status"] == "linked"
        )
        assert (
            artifact["origin"]["attachment_id"] == SCOPE["attachment_id"]
            and artifact["origin"]["instance_id"] == "focusa-instance:u1-e2e"
            and artifact["diagnostics_refs"]
            and artifact["evidence_refs"]
            and artifact["semantic"]["citation_refs"]
            and artifact["render"]["width"] == 1440
            and artifact["retention"]["policy"] == "project_evidence"
            and artifact["revision"] == 1
        )
        body["expected_state_version"] = result["state_version"]
        _, replay = intake_with_writer_retry(base, body)
        assert (
            replay["replayed"]
            and replay["artifact"]["artifact_id"] == artifact["artifact_id"]
            and replay["receipt_ref"] == result["receipt_ref"]
        )

        updated_body = dict(body)
        updated_body["idempotency_key"] = "u1-diagnostics-rehydrated"
        updated_body["artifact_url"] = (
            "https://fpv.wpuiai.com/m/sunny-summit-rehydrated"
        )
        _, updated = intake_with_writer_retry(base, updated_body)
        assert (
            not updated["replayed"]
            and updated["artifact"]["artifact_id"] == artifact["artifact_id"]
            and updated["artifact"]["revision"] == 2
            and updated["artifact"]["linked_at"] == artifact["linked_at"]
            and updated["artifact"]["content"]["artifact_url"].endswith("rehydrated")
        )

        research_preview = (
            "# Signed release research\n\n"
            "The cited source requires signed artifacts for every deployment."
        )
        research_body = dict(body)
        research_body.update(
            idempotency_key="u1-research-packet",
            artifact_kind="markdown",
            mime_type="text/markdown",
            title="Cited release policy research",
            summary="Bounded UIAI research packet with source-preserving citation.",
            handle_ref="uiai-research-packet:release-policy",
            artifact_url="https://example.invalid/research/release-policy",
            inline_preview=research_preview,
            sha256=hashlib.sha256(research_preview.encode()).hexdigest(),
            size_bytes=len(research_preview),
            source_ref="uiai-research:session=biw4d3UW:release-policy",
            citation_refs=[
                "source:https://example.invalid/release-policy#signed-artifacts"
            ],
            preferred_renderer="cited_markdown_reader",
            fallback_renderer="bounded_text_and_handle",
            render_width=900,
            render_height=700,
        )
        _, research = intake_with_writer_retry(base, research_body)
        assert (
            research["artifact"]["artifact_kind"] == "markdown"
            and research["artifact"]["semantic"]["citation_refs"]
            and research["artifact"]["content"]["inline_preview"] == research_preview
            and research["artifact"]["revision"] == 1
        )

        invalid_cases = []
        missing_evidence = dict(body)
        missing_evidence.update(idempotency_key="invalid-evidence", evidence_refs=[])
        invalid_cases.append(missing_evidence)
        oversized = dict(body)
        oversized.update(idempotency_key="invalid-preview", inline_preview="x" * 2001)
        invalid_cases.append(oversized)
        missing_origin = dict(body)
        missing_origin.update(
            idempotency_key="invalid-uiai-origin", uiai_session_id=None
        )
        invalid_cases.append(missing_origin)
        missing_diagnostics = dict(body)
        missing_diagnostics.update(
            idempotency_key="invalid-diagnostics", diagnostics_refs=[]
        )
        invalid_cases.append(missing_diagnostics)
        markdown_without_citation = dict(body)
        markdown_without_citation.update(
            idempotency_key="invalid-markdown-citation",
            artifact_kind="markdown",
            mime_type="text/markdown",
            diagnostics_refs=[],
            citation_refs=[],
        )
        invalid_cases.append(markdown_without_citation)
        for invalid in invalid_cases:
            error = rejected(base, invalid, 422)
            assert error["status"] == "validation_rejected"

        stale = dict(body)
        stale["idempotency_key"] = "invalid-stale-version"
        stale["expected_state_version"] = 0
        try:
            call(base, "POST", "/v1/workspace/artifacts/intake", stale)
        except urllib.error.HTTPError as error:
            assert error.code == 409
        else:
            raise AssertionError("stale state version was accepted")

        assert listed(base, {**SCOPE, "continuity_id": "other"})["artifacts"] == []
        stop(proc, log)
        proc = log = None
        proc, log, base = start(data)
        resumed = listed(base)
        resumed_by_kind = {item["artifact_kind"]: item for item in resumed["artifacts"]}
        assert (
            len(resumed["artifacts"]) == 2
            and resumed_by_kind["diagnostics"]["artifact_id"] == artifact["artifact_id"]
            and resumed_by_kind["diagnostics"]["revision"] == 2
            and resumed_by_kind["diagnostics"]["content"]["artifact_url"].endswith(
                "rehydrated"
            )
            and resumed_by_kind["markdown"]["semantic"]["citation_refs"]
        )
        serialized = json.dumps(resumed).lower()
        assert "cookies" not in serialized and "local_storage" not in serialized
        print(
            "Spec 135 U1 Workspace Artifact E2E: PASS (diagnostics + cited research, full descriptor, negative gates, rehydration, replay, restart)"
        )
    finally:
        if proc is not None:
            stop(proc, log)
        shutil.rmtree(data, ignore_errors=True)


if __name__ == "__main__":
    main()
