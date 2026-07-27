#!/usr/bin/env python3
"""Static release/OTA architecture gate for Spec145 and GitHub #56."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE = (ROOT / "crates/focusa-core/src/release_cycle.rs").read_text()
INTELLIGENCE = (ROOT / "crates/focusa-core/src/release_intelligence.rs").read_text()
UPDATE = (ROOT / "crates/focusa-cli/src/commands/update.rs").read_text()
RELEASE_CLI = (ROOT / "crates/focusa-cli/src/commands/release.rs").read_text()
TAG_SCRIPT = (ROOT / "scripts/create-dev-release-tag.sh").read_text()
CI = (ROOT / ".github/workflows/ci.yml").read_text()
RELEASE = (ROOT / ".github/workflows/release.yml").read_text()
SPEC132 = (ROOT / ".github/workflows/spec132-terminal-matrix.yml").read_text()
DOC = (ROOT / "docs/145-focusa-canonical-core-release-cycle-fast-release-architecture.md").read_text()
OPERATIONS = (ROOT / "docs/146-focusa-canonical-release-cycle-operations-and-proof-runbook.md").read_text()
TOPOLOGY = (ROOT / "config/focusa-release-topology.json").read_text()


def require(body: str, needles: list[str], label: str) -> None:
    missing = [needle for needle in needles if needle not in body]
    assert not missing, f"{label} missing: {missing}"


require(
    CORE,
    [
        "RELEASE_TOPOLOGY_SCHEMA",
        "ReleaseTopology",
        "ReleaseCandidate",
        "ReleaseStage",
        "ReleaseEvidence",
        "ReleaseFixLane",
        "ReleaseBenchmark",
        "has_cycle",
        "illegal release transition",
        "release evidence SHA differs",
    ],
    "release kernel",
)
require(
    INTELLIGENCE,
    [
        "ReleaseIntelligencePacket",
        "ReleaseArtifactTruth",
        "render_markdown",
        "publishable release contains unproven checks",
        "Artifact truth",
        "Release benchmark",
    ],
    "release intelligence",
)
require(
    TAG_SCRIPT,
    [
        "push_candidate_main_with_auto_rebase",
        "Waiting for exact stamped-candidate preflight before immutable tag",
        'git push origin "${TAG}"',
    ],
    "exact candidate pre-tag flow",
)
assert TAG_SCRIPT.index("  push_candidate_main_with_auto_rebase\n") < TAG_SCRIPT.rindex('git tag "${TAG}" HEAD'), "tag created before candidate preflight"
require(
    RELEASE_CLI,
    [
        "ReleaseCycleCmd",
        "ValidateTopology",
        "RenderIntelligence",
        "focusa.release_topology_validation.v1",
        "focusa.release_intelligence_render.v1",
        "immutable release pages are never overwritten",
    ],
    "release CLI entry",
)
require(
    UPDATE,
    [
        '"ls-files", "--"',
        "Some(extension_root),\n                None",
        'apply.status == "failed_rolled_back"',
        "update apply failed and rollback was applied",
        'Environment=\"PATH={runtime_path}\"',
        "base_seconds: 120",
        "RandomizedDelaySec=24s",
    ],
    "OTA truth",
)
require(
    CI,
    ["concurrency:", "cancel-in-progress: true", "Swatinem/rust-cache@v2"],
    "CI speed controls",
)
require(
    RELEASE,
    [
        "tags:",
        "'v*-dev'",
        "release-${{ github.ref }}",
        "Swatinem/rust-cache@v2",
        "Lock exact release candidate",
        "focusa.release_candidate.v1",
        "Upload release candidate lock",
        "Release blocked by release-scoped pull requests",
        "unrelated open pull requests remain queued outside the locked candidate",
        "Require exact candidate-SHA preflight receipts",
        "Exact tag CI proof",
        "tag-ci-proof",
        "needs: [tauri-build, rust-release, pi-extension-release, tag-ci-proof]",
        "shared-key: release-target-${{ matrix.target }}",
        "actions/workflows/ci.yml/runs",
        "2>/dev/null || echo '[]'",
    ],
    "Release trigger/cache controls",
)
assert "Release cargo test" not in RELEASE, "Release duplicates source/tag CI cargo tests on the critical path"
assert "Release clippy" not in RELEASE, "Release duplicates source/tag CI clippy on the critical path"
require(
    SPEC132,
    [
        "push:\n    branches: [main]",
        "pull_request:",
        "crates/focusa-cli/src/commands/update.rs",
        "crates/focusa-core/src/silent_sessions/**",
        "crates/focusa-session-runner/**",
        "spec132-${{ github.event.pull_request.number || github.ref }}",
        "Swatinem/rust-cache@v2",
        "shared-key: release-target-${{ matrix.target }}",
        "toolchain: nightly-2026-01-08",
    ],
    "Spec132 ownership",
)
require(
    DOC,
    ["Canonical state machine", "Call-stack design", "GitHub Actions adapter DAG"],
    "Spec145 architecture",
)
require(
    OPERATIONS,
    [
        "Automatic OTA architecture",
        "Baseline benchmark: v0.9.127-dev",
        "Detailed acceptance",
        "Rollback",
    ],
    "Spec146 operations and proof",
)
require(
    TOPOLOGY,
    [
        '"schema": "focusa.release_topology.v1"',
        '"surface_id": "daemon"',
        '"surface_id": "pi_extension"',
        '"surface_id": "agent_context"',
    ],
    "Focusa topology fixture",
)

print("PASS: Spec145 canonical release kernel, OTA truth, speed controls, topology, and architecture present")
