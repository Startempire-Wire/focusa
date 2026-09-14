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
        "Publish immutable candidate prerelease",
        "gh release edit",
        "--draft=false",
        "--prerelease",
        "--latest=false",
    ]
    for token in required_workflow_tokens:
        assert token in WORKFLOW, token
    assert WORKFLOW.index("Generate typed release intelligence packet and page") < WORKFLOW.index(
        "Generate detached signatures, manifest, provenance, and trust metadata"
    )
    assert WORKFLOW.index("Upload trusted OTA metadata and detached signatures") < WORKFLOW.index(
        "Publish immutable candidate prerelease"
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
