#!/usr/bin/env python3
"""Spec 135B-2 source-linked Context artifact and connector contract proof."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = (ROOT / "docs/135b-crist-project-genesis-context-role-interview-spec-tasks.md").read_text()
TYPES = (ROOT / "crates/focusa-core/src/types.rs").read_text()
ROUTE = (ROOT / "crates/focusa-api/src/routes/context_sources.rs").read_text()
CONNECTORS = (ROOT / "crates/focusa-core/src/connectors.rs").read_text()
AUTH = (ROOT / "crates/focusa-core/src/connector_auth.rs").read_text()
OPENAPI = (ROOT / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json").read_text()
GENERATOR = (ROOT / "scripts/generate-spec135-context-artifact-contracts.py").read_text()

for token in (
    "focusa.project_context_artifact.v1",
    "source_revision",
    "content_sha256",
    "freshness_status",
    "verification_policy_refs",
    "duplicate_of_artifact_ref",
    "ProjectContextArtifactProvenance",
    "ProjectContextArtifactClassification",
):
    assert token in TYPES or token in ROUTE, token

for source_kind in (
    '"file"',
    '"web"',
    '"research"',
    '"connected"',
    '"focusa_native"',
):
    assert source_kind in ROUTE, source_kind

for token in (
    "local_source_path",
    "canonical.starts_with(&root)",
    "validated_public_source_url",
    "read_write_posture",
    "oauth_scopes",
    "incremental_sync_method",
    "cursor_state",
    "rate_limit_posture",
    "revocation_behavior",
    "recovery_action",
):
    assert token in ROUTE or token in TYPES, token

for method in ("health", "execute"):
    assert f"async fn {method}" in CONNECTORS, method
for method in ("begin_authorization", "store_access_token", "revoke"):
    assert f"pub fn {method}" in AUTH, method

for token in (
    "focusa_project_context_artifact_v1",
    "focusa_context_source_record_v1",
    "duplicate_of_artifact_ref",
    "cursor_state",
):
    assert token in OPENAPI, token
assert "artifact_schema" in GENERATOR and "health_schema" in GENERATOR

assert "bounded retrieval" in SPEC.lower()
assert "persist incremental cursors" in SPEC
assert "keep secrets out of project files and model context" in SPEC

print("Spec 135 B2 Context artifacts/connectors: PASS")
