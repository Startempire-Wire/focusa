#!/usr/bin/env python3
"""Spec146 release-page workflow integration and packet-generation gate."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = (ROOT / ".github/workflows/release.yml").read_text()
GENERATOR = ROOT / "scripts/generate-release-intelligence-packet.py"


def main() -> None:
    required_workflow_tokens = [
        "draft: true",
        "Generate typed release intelligence packet and page",
        "generate-release-intelligence-packet.py",
        "release render-intelligence",
        "--publishable",
        "dist/release-intelligence.json",
        "dist/release-intelligence.md",
        "Publish immutable release on the channel implied by its tag",
        "gh release edit",
        "--draft=false",
        "--prerelease",
        "--latest=false",
        # Channel-aware publication: a stable tag must publish as stable and
        # become Latest, otherwise GitHub keeps resolving consumers to an
        # older release. These tokens pin that behaviour.
        "--prerelease=false",
        "release_channel=stable",
        "release_channel=candidate",
        # The OTA update pointer must be published explicitly. It is written
        # after the dist/* upload, so listing only dist/*.sig publishes
        # latest.json.sig without latest.json.
        "dist/latest.json",
        "latest.json.sig",
    ]
    for token in required_workflow_tokens:
        assert token in WORKFLOW, token
    assert WORKFLOW.index("Generate typed release intelligence packet and page") < WORKFLOW.index(
        "Generate detached signatures, manifest, provenance, and trust metadata"
    )
    assert WORKFLOW.index("Upload trusted OTA metadata and detached signatures") < WORKFLOW.index(
        "Publish immutable release on the channel implied by its tag"
    )

    # `dist/latest.json` must appear in the OTA upload step's file list, not
    # merely somewhere in the workflow. A bare substring check passes even when
    # the payload is absent from the upload, which is exactly how
    # latest.json.sig shipped without latest.json on v0.9.198.
    ota_upload = WORKFLOW.index("Upload trusted OTA metadata and detached signatures")
    ota_files_start = WORKFLOW.index("files: |", ota_upload)
    ota_files_end = WORKFLOW.index("env:", ota_files_start)
    ota_files = WORKFLOW[ota_files_start:ota_files_end]
    assert "dist/latest.json" in ota_files, (
        "the OTA upload step must publish dist/latest.json; without it the shipped "
        f"installer resolves releases/latest/download/latest.json to a 404. Got: {ota_files!r}"
    )

    # The channel decision must be a real branch, not an unconditional publish.
    channel_glob_token = 'v[0-9]*.[0-9]*.[0-9]*'
    assert channel_glob_token in WORKFLOW, (
        "the stable/candidate channel decision must use the canonical fnmatch "
        "glob form; release.yml tag patterns must not use regex '+' "
        "(structural guard mode 23)"
    )
    assert 'if [[ "$TAG" =~' not in WORKFLOW, (
        "the channel decision must not use a bash regex match; use fnmatch globs "
        "and a candidate-suffix test instead"
    )
    # The glob alone also matches candidate tags, so the suffix test is load-bearing.
    assert '[[ "$TAG" == *-* ]]' in WORKFLOW, (
        "version-shaped tags must be split on a candidate suffix; without it "
        "v0.9.198-nightly.20261001 would publish as the stable channel"
    )
    channel_branch = WORKFLOW.index(channel_glob_token)
    assert channel_branch < WORKFLOW.index('release_channel=candidate'), (
        "the candidate lane must remain reachable; a publish that is unconditionally "
        "stable would repoint Latest at a nightly"
    )

    with tempfile.TemporaryDirectory() as raw:
        directory = Path(raw)
        dist = directory / "dist"
        dist.mkdir()
        artifact = dist / "focusa-v0.0.0-test-x86_64-unknown-linux-gnu"
        artifact.write_bytes(b"exact release artifact\n")
        output = dist / "release-intelligence.json"
        sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
        subprocess.run(
            [
                "python3",
                str(GENERATOR),
                "--dist",
                str(dist),
                "--tag",
                "v0.0.0-test",
                "--sha",
                sha,
                "--repo",
                "Startempire-Wire/focusa",
                "--run-url",
                "https://github.com/Startempire-Wire/focusa/actions/runs/test",
                "--output",
                str(output),
            ],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
        packet = json.loads(output.read_text())
        assert packet["schema"] == "focusa.release_intelligence.v1"
        assert packet["exact_sha"] == sha
        assert packet["failed_checks"] == []
        assert packet["unproven_checks"] == []
        assert packet["material_changes"]
        assert packet["exact_proofs"]
        assert packet["traceability_refs"]
        assert packet["artifacts"][0]["artifact_name"] == artifact.name
        assert len(packet["artifacts"][0]["sha256"]) == 64
        assert packet["artifacts"][0]["signature_ref"].endswith(".sig")

    print("Spec146 release intelligence workflow gate: PASS")


if __name__ == "__main__":
    main()
