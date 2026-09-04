#!/usr/bin/env python3
"""Canonical Focusa distribution-manifest component and digest contract.

`stamp-menubar-version.py` is the only writer. This module owns deterministic
component discovery/digests and read-only validation for release and audit gates.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any, Iterable

SCHEMA = "focusa.distribution_manifest.v1"
TREE_ALGORITHM = "sha256-tree-v1"
MANIFEST_REL = Path(
    "docs/contracts/spec141/generated-capability-v2/distribution-manifest.json"
)
EXCLUDED_DIRS = {
    ".git",
    ".beads",
    "node_modules",
    "target",
    "dist",
    ".svelte-kit",
    "__pycache__",
}

COMPONENT_PATHS: dict[str, tuple[str, ...]] = {
    "rust_runtime": ("Cargo.toml", "Cargo.lock", "crates"),
    "pi_extension": ("apps/pi-extension",),
    "agent_skills": (".pi/skills", "scripts/focusa-skill-doctor"),
    "documentation": (
        "AGENTS.md",
        "README.md",
        "docs/current",
        "docs/contracts/spec141/generated-capability-v2",
        "docs/07-reference-store.md",
        "docs/82-focusa-memory-optimization-spec.md",
        "docs/94-focusa-intent-preserving-memory-rpc-optimization-sow.md",
        "docs/canonical-live-release-pipeline.md",
    ),
    "generated_clients": (
        "packages/generated/spec135",
        "docs/contracts/spec135/generated-contract-v1",
    ),
    "installers": (
        "scripts/install-focusa.sh",
        "scripts/install-focusa.ps1",
        "scripts/install-daemon.sh",
    ),
    "release_tooling": (
        ".appveyor.yml",
        "codemagic.yaml",
        ".github/workflows/ci.yml",
        ".github/workflows/release.yml",
        ".github/workflows/nightly.yml",
        ".github/workflows/deploy-live-daemon.yml",
        ".github/workflows/windows-ota-e2e.yml",
        "scripts/distribution_manifest.py",
        "scripts/audit-distribution-parity.mjs",
        "scripts/stamp-menubar-version.py",
        "scripts/local-release-preflight.sh",
        "scripts/verify-canonical-release-assets.py",
        "scripts/wait-for-external-release-assets.py",
        "scripts/release-trust-metadata.py",
        "scripts/release-deploy-proof.py",
        "scripts/generate-release-intelligence-packet.py",
        "tests/154-focusa-canonical-all-platform-release-test.sh",
        "tests/154_focusa_canonical_release_assets_test.py",
        "tests/distribution_manifest_contract_test.py",
        "tests/release_deploy_automation_static_test.sh",
        "tests/release_authority_root_embedding_test.py",
        "tests/release_deploy_proof_test.py",
        "tests/release_trust_metadata_static_test.sh",
        "tests/spec177-nightly-contract-static-test.sh",
        "tests/spec146_release_intelligence_workflow_gate.py",
    ),
}

LEGACY_ARTIFACT_PATHS = (
    "Cargo.toml",
    "apps/pi-extension/package.json",
    "crates/focusa-core/src/hlt_extrapolation.rs",
    "docs/contracts/spec141/generated-capability-v2/agent-card.json",
    "docs/contracts/spec141/generated-capability-v2/mcp-tools.json",
    "docs/contracts/spec141/generated-capability-v2/pi-tools.json",
    "crates/focusa-core/src/lib.rs",
)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return "sha256:" + digest.hexdigest()


def iter_component_files(root: Path, entries: Iterable[str]) -> list[Path]:
    files: set[Path] = set()
    for entry in entries:
        path = root / entry
        if not path.exists():
            raise ValueError(f"distribution component path is missing: {entry}")
        if path.is_symlink():
            raise ValueError(f"distribution component path is a symlink: {entry}")
        if not path.is_file() and not path.is_dir():
            raise ValueError(f"distribution component path is not a regular entry: {entry}")
        candidates = [path] if path.is_file() else path.rglob("*")
        for candidate in candidates:
            relative = candidate.relative_to(root)
            if any(part in EXCLUDED_DIRS for part in relative.parts):
                continue
            if relative == MANIFEST_REL:
                continue
            if candidate.is_symlink():
                raise ValueError(f"distribution component contains a symlink: {relative}")
            if candidate.is_file():
                files.add(relative)
            elif not candidate.is_dir():
                raise ValueError(
                    f"distribution component contains a special entry: {relative}"
                )
    return sorted(files, key=lambda path: path.as_posix())


def tree_contract(root: Path, entries: Iterable[str]) -> dict[str, Any]:
    files = iter_component_files(root, entries)
    digest = hashlib.sha256()
    for relative in files:
        file_digest = bytes.fromhex(sha256_file(root / relative).removeprefix("sha256:"))
        digest.update(relative.as_posix().encode("utf-8"))
        digest.update(b"\0")
        digest.update(file_digest)
        digest.update(b"\0")
    return {
        "algorithm": TREE_ALGORITHM,
        "sha256": "sha256:" + digest.hexdigest(),
        "file_count": len(files),
        "source_paths": list(entries),
    }


def contract_count(registry: Any) -> int:
    if isinstance(registry, list):
        return len(registry)
    if isinstance(registry, dict):
        for key in ("tools", "contracts", "entries"):
            if isinstance(registry.get(key), list):
                return len(registry[key])
    raise ValueError("capability registry has no canonical contract list")


def source_components(root: Path) -> dict[str, Any]:
    components = {
        name: tree_contract(root, entries) for name, entries in COMPONENT_PATHS.items()
    }
    registry_path = root / "docs/current/focusa-tool-contracts.json"
    registry = json.loads(registry_path.read_text(encoding="utf-8"))
    capability_dir = root / "docs/contracts/spec141/generated-capability-v2"
    components["capability_surfaces"] = {
        "contract_count": contract_count(registry),
        "registry_sha256": sha256_file(registry_path),
        "agent_card_sha256": sha256_file(capability_dir / "agent-card.json"),
        "pi_tools_sha256": sha256_file(capability_dir / "pi-tools.json"),
        "mcp_tools_sha256": sha256_file(capability_dir / "mcp-tools.json"),
        "openai_tools_sha256": sha256_file(capability_dir / "openai-tools.json"),
        "openapi_sha256": sha256_file(
            root / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json"
        ),
    }
    components["runtime_contract"] = {
        "system_state_root": "/usr/local/lib/focusa",
        "installed_manifest_path": "/usr/local/lib/focusa/distribution-manifest.json",
        "manifest_required_from": "0.9.188",
        "binary_paths": {
            "cli": "/usr/local/bin/focusa",
            "daemon": "/usr/local/bin/focusa-daemon",
            "tui": "/usr/local/bin/focusa-tui",
            "session_runner": "/usr/local/bin/focusa-session-runner",
        },
        "daemon_health_path": "/v1/health",
        "callgraph_validation_path": "/v1/callgraphs/validate",
        "pi_extension_package": "focusa-pi-bridge",
        "capability_registry_schema": "focusa.tool_contracts.v1",
    }
    return components


def artifacts(root: Path, existing: dict[str, Any] | None = None) -> dict[str, str]:
    paths = set(LEGACY_ARTIFACT_PATHS)
    if existing:
        paths.update(existing)
    result: dict[str, str] = {}
    for relative in sorted(paths):
        relative_path = Path(relative)
        if relative_path.is_absolute() or ".." in relative_path.parts:
            raise ValueError(f"distribution artifact path is unsafe: {relative}")
        path = root / relative_path
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"distribution artifact is missing or non-regular: {relative}")
        result[relative_path.as_posix()] = sha256_file(path)
    return result


def build_manifest(
    root: Path,
    current: dict[str, Any],
    version: str,
    source_commit: str,
    generated_at: str,
) -> dict[str, Any]:
    return {
        "schema": SCHEMA,
        "release_version": version,
        "source_commit": source_commit,
        "generated_at": generated_at,
        "digest_contract": TREE_ALGORITHM,
        "artifacts": artifacts(root, current.get("artifacts")),
        "components": source_components(root),
        "compatibility_status": current.get("compatibility_status", "compatible"),
        "drift_findings": current.get("drift_findings", []),
        "evidence_ref": "scripts/distribution_manifest.py --check",
    }


def workspace_version(root: Path) -> str:
    for line in (root / "Cargo.toml").read_text(encoding="utf-8").splitlines():
        if line.startswith("version = "):
            return line.split('"', 2)[1]
    raise ValueError("workspace package version is missing")


def verify_manifest(root: Path, manifest: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    if manifest.get("schema") != SCHEMA:
        failures.append("schema mismatch")
    if manifest.get("release_version") != workspace_version(root):
        failures.append("release_version does not match Cargo workspace")
    if manifest.get("digest_contract") != TREE_ALGORITHM:
        failures.append("digest contract mismatch")
    if not re.fullmatch(r"[0-9a-f]{7,40}", str(manifest.get("source_commit", ""))):
        failures.append("source_commit is not a Git object identifier")
    if not re.fullmatch(
        r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z",
        str(manifest.get("generated_at", "")),
    ):
        failures.append("generated_at is not canonical UTC")
    try:
        expected_artifacts = artifacts(root, manifest.get("artifacts"))
        if manifest.get("artifacts") != expected_artifacts:
            failures.append("artifact SHA-256 map is stale")
        expected_components = source_components(root)
        if manifest.get("components") != expected_components:
            failures.append("component digest/runtime contract is stale")
    except (OSError, ValueError, json.JSONDecodeError) as error:
        failures.append(str(error))
    return failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    manifest_path = args.manifest or root / MANIFEST_REL
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    failures = verify_manifest(root, manifest)
    result = {
        "schema": "focusa.distribution_manifest_check.v1",
        "ok": not failures,
        "manifest": str(manifest_path),
        "source_commit": manifest.get("source_commit"),
        "release_version": manifest.get("release_version"),
        "failures": failures,
    }
    if args.json:
        print(json.dumps(result, sort_keys=True))
    else:
        print("distribution_manifest=PASS" if not failures else "distribution_manifest=FAIL")
        for failure in failures:
            print(f"failure={failure}")
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
