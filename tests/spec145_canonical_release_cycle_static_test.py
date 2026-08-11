#!/usr/bin/env python3
"""Static release/OTA architecture gate for Spec145 and GitHub #56."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE = (ROOT / "crates/focusa-core/src/release_cycle.rs").read_text()
INTELLIGENCE = (ROOT / "crates/focusa-core/src/release_intelligence.rs").read_text()
ORCHESTRATOR = "\n".join((ROOT / f"crates/focusa-core/src/{name}").read_text() for name in ["release_orchestrator.rs", "release_planner.rs", "release_protocol.rs"])
LEDGER = (ROOT / "crates/focusa-core/src/release_ledger.rs").read_text()
ADAPTERS = (ROOT / "crates/focusa-core/src/release_adapters.rs").read_text()
CALIBRATION = (ROOT / "crates/focusa-core/src/release_calibration.rs").read_text()
REFERENCE_ADAPTERS = "\n".join(path.read_text() for path in sorted((ROOT / "config/release-adapters").glob("*.json")))
REFERENCE_TOPOLOGIES = "\n".join(path.read_text() for path in sorted((ROOT / "config/release-topologies").glob("*.json")))
UPDATE = (ROOT / "crates/focusa-cli/src/commands/update.rs").read_text()
RELEASE_CLI = "\n".join((ROOT / f"crates/focusa-cli/src/commands/{name}").read_text() for name in ["release.rs", "release_master.rs"])
TAG_SCRIPT = (ROOT / "scripts/create-dev-release-tag.sh").read_text()
VERSION_SELECTOR = (ROOT / "scripts/select-release-version.py").read_text()
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
        "Release surfaces already stamped ${VERSION}; preserving exact retry SHA.",
        "STAMPED_SOURCE_SHA=\"$(git rev-parse HEAD)\"",
        "scripts/generate-locked-release-candidate-ancestry.py",
        "scripts/generate-locked-release-governance-receipt.py",
        "ensure_source_workflow \"Spec 132 terminal matrix\" \"$HEAD_SHA\"",
        "source_gate_dispatch_blocked",
        'git push origin "${TAG}"',
        "scripts/select-release-version.py",
        'stage "version-selection"',
        "VERSION_SELECTION_DETAILS",
        "normalize_release_channel",
        '--prerelease=true --latest=false',
        '--prerelease=false --latest=true',
        'stage "release-channel"',
    ],
    "exact candidate pre-tag flow",
)
require(
    VERSION_SELECTOR,
    [
        "focusa.release_version_selection.v1",
        "highest_patch",
        "channel_maxima",
        "release version regression",
        "selected_patch > highest_patch",
    ],
    "monotonic release version selection",
)
proof_reseal = TAG_SCRIPT.index("python3 scripts/generate-locked-release-candidate-ancestry.py")
stamp_commit = TAG_SCRIPT.index('git commit -m "chore: stamp release surfaces ${VERSION}"')
candidate_push = TAG_SCRIPT.index("  push_candidate_main_with_auto_rebase\n", proof_reseal)
assert stamp_commit < proof_reseal < candidate_push, "stamped candidate proof is not resealed before source CI"
assert candidate_push < TAG_SCRIPT.rindex('git tag "${TAG}" HEAD'), "tag created before candidate preflight"
assert TAG_SCRIPT.index('ensure_source_workflow "Spec 132 terminal matrix" "$HEAD_SHA"') < TAG_SCRIPT.index('wait_for_source_workflow "Spec 132 terminal matrix" "$HEAD_SHA"'), "Spec132 wait begins before missing-run dispatch"
require(
    RELEASE_CLI,
    [
        "ReleaseCycleCmd",
        "ValidateTopology",
        "ValidateAdapter",
        "Plan",
        "Execute",
        "Calibrate",
        "ReleaseRunInput",
        "run_with_checkpoint_sink",
        "RenderIntelligence",
        "focusa.release_topology_validation.v1",
        "focusa.release_intelligence_render.v1",
        "immutable release pages are never overwritten",
    ],
    "release CLI entry",
)
require(
    ORCHESTRATOR,
    [
        "MasterReleaseOrchestrator",
        "ReleaseAdapter",
        "ReleaseInvocationSurface",
        "ReleaseRunInput",
        "run_with_checkpoint_sink",
        "exact_sha_evidence_reused",
        "idempotency_key",
        "immutable artifact set changed between release stages",
        'status: "rolled_back".into()',
        "mutation_authority_missing",
    ],
    "provider-neutral master orchestrator",
)
require(
    LEDGER,
    [
        "ReleaseRunCheckpoint",
        "JsonlReleaseRunLedger",
        "ReleaseCheckpointSink",
        "release checkpoint sequence mismatch",
        "release ledger SHA mismatch",
    ],
    "interruption-safe release ledger",
)
require(
    ADAPTERS,
    [
        "ReleaseAdapterManifest",
        "ReleaseOperationExecutor",
        "JsonProcessReleaseExecutor",
        "focusa.release_plugin_envelope.v1",
        "env_clear()",
        "release plugin receipt exceeds 1 MiB",
    ],
    "pluggable adapter boundary",
)
require(
    CALIBRATION,
    [
        "ReleaseCalibrator",
        "ReleaseCalibrationLedger",
        "CalibrationOutcome",
        "parallelize_independent_topology_waves",
        "RolledBack",
        "crosses project/profile authority",
    ],
    "continual release calibration",
)
require(
    REFERENCE_ADAPTERS + REFERENCE_TOPOLOGIES,
    [
        '"manifest_id": "focusa-github-actions-v1"',
        '"manifest_id":"portable-cli-library-v1"',
        '"manifest_id": "uiai-engine-v1"',
        '"profile": "single_package"',
        '"profile": "service_container_web"',
    ],
    "cross-software reference adapters",
)
require(
    UPDATE,
    [
        '"ls-files", "--"',
        "Some(extension_root),\n                None",
        'apply.status == "failed_rolled_back"',
        "update apply failed and rollback was applied",
        'Environment=\"PATH={runtime_path}\"',
        'Environment=\"FOCUSA_FOCUSA_PATH=/usr/local/bin/focusa\"',
        'Environment=\"FOCUSA_FOCUSA_TUI_PATH=/usr/local/bin/focusa-tui\"',
        "Never let a private root install shadow globally executable binaries",
        "base_seconds: 120",
        "RandomizedDelaySec=24s",
    ],
    "OTA truth",
)
require(
    CI,
    [
        "concurrency:",
        "cancel-in-progress: ${{ github.event_name == 'pull_request' }}",
        "Swatinem/rust-cache@v2",
    ],
    "CI speed controls",
)
require(
    RELEASE,
    [
        "tags:",
        "'v*'",
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

print("PASS: provider-neutral Master Release Cycle, adapters, calibration, OTA truth, topology, and architecture present")
