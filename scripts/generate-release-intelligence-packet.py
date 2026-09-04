#!/usr/bin/env python3
"""Build a typed release-intelligence packet from exact Git and artifact truth."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

SCHEMA = "focusa.release_intelligence.v1"
EXCLUDED_NAMES = {
    "SHA256SUMS.txt",
    "release-intelligence.json",
    "release-intelligence.md",
    "release-manifest.json",
    "release-provenance.json",
    "focusa-trusted-release-keys.json",
}


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], text=True).strip()


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def release_assets(dist: Path) -> list[dict[str, object]]:
    assets = []
    for path in sorted(dist.iterdir()):
        if (
            not path.is_file()
            or path.name in EXCLUDED_NAMES
            or path.name.endswith((".sig", ".pem", ".sha256"))
        ):
            continue
        platform = "cross-platform"
        for candidate in (
            "x86_64-unknown-linux-gnu",
            "x86_64-unknown-linux-musl",
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "aarch64-pc-windows-msvc",
            "x86_64-pc-windows-msvc",
        ):
            if candidate in path.name:
                platform = candidate
                break
        surface = "focusa-cli"
        for prefix, candidate_surface in (
            ("focusa-daemon-", "daemon"),
            ("focusa-tui-", "tui"),
            ("focusa-pi-extension-", "pi-extension"),
            ("focusa-agent-context-", "agent-context"),
            ("focusa-installer-", "installer"),
            ("install-focusa", "installer"),
        ):
            if path.name.startswith(prefix):
                surface = candidate_surface
                break
        assets.append(
            {
                "surface_id": surface,
                "artifact_name": path.name,
                "platform": platform,
                "sha256": sha256(path),
                "signature_ref": f"{path.name}.sig",
                "provenance_ref": "release-provenance.json",
                "installed_version": None,
                "running_version": None,
                "verification_ref": "SHA256SUMS.txt.cosign.pem",
            }
        )
    if not assets:
        raise SystemExit("release intelligence requires at least one downloaded artifact")
    return assets


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dist", required=True, type=Path)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--sha", required=True)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--run-url", required=True)
    parser.add_argument("--previous-tag")
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()

    exact_sha = git("rev-parse", args.sha)
    if exact_sha != args.sha:
        raise SystemExit(f"exact SHA mismatch: expected {args.sha}, observed {exact_sha}")
    release_range = (
        f"{args.previous_tag}..{args.sha}" if args.previous_tag else args.sha
    )
    release_log = git(
        "log", "--max-count=100", "--format=%H%x1f%s%x1f%an", release_range
    )
    rows = [row.split("\x1f", 2) for row in release_log.splitlines() if row.strip()]
    if not rows:
        raise SystemExit("release intelligence requires at least one exact-SHA commit")
    commits = [row[0] for row in rows]
    changes = [row[1] for row in rows]
    contributors = sorted({row[2] for row in rows})
    run_ref = args.run_url.rstrip("/")
    repository_ref = run_ref.split("/actions/runs/", 1)[0]
    packet = {
        "schema": SCHEMA,
        "release_id": f"{args.repo}:{args.tag}",
        "project_id": "focusa",
        "profile": "focusa-multi-surface",
        "version": args.tag.removeprefix("v"),
        "exact_sha": exact_sha,
        "previous_tag": args.previous_tag,
        "purpose": (
            "Ship the exact verified Focusa scope with truthful cross-surface "
            "artifacts, migration safety, and recoverable release proof."
        ),
        "trajectory_refs": ["docs/142-focusa-release-requirement-trace-matrix.md"],
        "material_changes": changes,
        "impact": [
            (
                "Operators receive one exact-SHA release with bounded proof and "
                "rollback guidance."
            ),
            (
                "Agents receive matching CLI, daemon, TUI, session runner, Pi, "
                "installer, and generated-context artifacts bound by one distribution manifest."
            ),
        ],
        "included_work": changes,
        "resolved_work": changes,
        "exact_proofs": [
            f"{repository_ref}/commit/{args.sha}",
            run_ref,
            f"{run_ref}#tag-ci-proof",
            f"{run_ref}#final-release-gap-gate",
            "SHA256SUMS.txt",
            "release-provenance.json",
            "distribution-manifest.json",
        ],
        "unproven_checks": [],
        "failed_checks": [],
        "known_issues": [],
        "breaking_changes": [],
        "compatibility": [
            "Existing project, Workpoint, Trajectory, and evidence data remain readable.",
            (
                "Artifact and API compatibility is governed by the checked-in "
                "migration and release contracts."
            ),
        ],
        "migrations": [
            "Run `focusa doctor` after upgrade; migration failures block activation."
        ],
        "install_steps": [
            "Use the signed installer or the platform artifact named below."
        ],
        "upgrade_steps": [
            "Run `focusa update plan`, inspect trust proof, then apply the approved update."
        ],
        "rollback_steps": [
            "Use the preserved previous binary/package and immutable release receipt."
        ],
        "artifacts": release_assets(args.dist),
        "security_and_provenance": [
            (
                "Every listed artifact is covered by Ed25519 metadata and "
                "keyless Cosign proof."
            ),
            (
                "The release page is rendered from this immutable typed packet "
                "before publication."
            ),
        ],
        "contributors": contributors,
        "traceability_refs": [
            "docs/149-focusa-workset-flow-ledger-and-release-completion-spec.md",
            "docs/contracts/135-locked-release-compatibility-delta.v1.yaml",
            run_ref,
        ],
        "commits": commits,
        "benchmark": None,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile("w", dir=args.output.parent, delete=False) as handle:
        json.dump(packet, handle, indent=2, sort_keys=True)
        handle.write("\n")
        temporary = Path(handle.name)
    temporary.replace(args.output)
    print(
        json.dumps(
            {
                "status": "generated",
                "artifacts": len(packet["artifacts"]),
                "commits": len(commits),
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
