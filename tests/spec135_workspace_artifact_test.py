#!/usr/bin/env python3
"""SPEC135-U1 static contract proof for UIAI Workspace Artifact bridge."""

import json
from pathlib import Path
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
B = ROOT / "docs/contracts/spec135/generated-contract-v1"
types = (ROOT / "crates/focusa-core/src/types.rs").read_text()
reducer = (ROOT / "crates/focusa-core/src/reducer.rs").read_text()
api = (ROOT / "crates/focusa-api/src/routes/workspace_artifacts.rs").read_text()
registry = json.loads((B / "operation-registry.json").read_text())
openapi = json.loads((B / "openapi-3.0.3.json").read_text())
bindings = json.loads((B / "ui-action-bindings.fixture.json").read_text())
dag = json.loads((ROOT / "docs/contracts/spec135-delivery-dag.v1.yaml").read_text())
proof = json.loads((B / "spec135-u1-workspace-artifact-proof.json").read_text())
request = json.loads(
    (B / "json-schema/focusa.workspace_artifact_intake.request.v1.json").read_text()
)
result = json.loads(
    (B / "json-schema/focusa.workspace_artifact_intake_result.v1.json").read_text()
)
listing = json.loads(
    (B / "json-schema/focusa.workspace_artifact_list.v1.json").read_text()
)
ts = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()
go = (ROOT / "packages/generated/spec135/go/client.gen.go").read_text()
ui = (ROOT / "packages/a2ui-renderer/proof/workspace-artifact.ts").read_text()
ui_html = (ROOT / "packages/a2ui-renderer/proof/workspace-artifact.html").read_text()
for m in [
    "WorkspaceArtifactContent",
    "WorkspaceArtifactSource",
    "WorkspaceArtifactScope",
    "WorkspaceArtifactOrigin",
    "WorkspaceArtifactTrust",
    "WorkspaceArtifactEvidenceStatus",
    "WorkspaceArtifactSemantic",
    "WorkspaceArtifactRetention",
    "WorkspaceArtifactRender",
    "WorkspaceArtifactRecord",
    "WorkspaceArtifactLinked",
    "workspace_artifacts",
]:
    assert m in types, m
for m in [
    "inline_preview",
    "2000",
    "diagnostics_refs",
    "evidence_refs",
    "citation_refs",
    "instance_id",
    "canonical artifact intake requires linked or verified Evidence",
    "external_artifact_authority",
    "uiai source requires uiai_session_id",
    "write_serial_lock",
    "drop(_writer)",
]:
    assert m in api, m
assert "WorkspaceArtifactLinked" in reducer
assert "invalid Workspace Artifact projection revision" in reducer
ops = {o["operation_id"]: o for o in registry["operations"]}
for oid, path in [
    ("focusa.workspace.artifact.list", "/v1/workspace/artifacts"),
    ("focusa.workspace.artifact.intake", "/v1/workspace/artifacts/intake"),
]:
    assert ops[oid]["path"] == path and path in openapi["paths"]
    assert all(
        ops[oid]["scope"][key]
        for key in ["project_scoped", "workstream_scoped", "attachment_scoped"]
    )
    assert any(b["action_id"] == oid for b in bindings["bindings"])
for s in [request, result, listing]:
    Draft202012Validator.check_schema(s)
assert request["properties"]["inline_preview"]["maxLength"] == 2000
assert request["properties"]["evidence_refs"]["minItems"] == 1
assert result["properties"]["external_artifact_authority"]["const"] is True
artifact = result["properties"]["artifact"]
assert artifact["properties"]["origin"]["properties"]["uiai_session_id"]
assert artifact["properties"]["origin"]["properties"]["silent_session_id"]
assert (
    "linked" in artifact["properties"]["trust"]["properties"]["evidence_status"]["enum"]
)
assert (
    artifact["properties"]["semantic"]["properties"]["citation_refs"]["maxItems"] == 64
)
assert artifact["properties"]["render"]["properties"]["width"]["maximum"] == 16384
assert artifact["properties"]["revision"]["minimum"] == 1
alpha1 = next(
    item
    for item in dag["critical_path_contract"]["slices"]
    if item["slice_id"] == "SPEC135-ALPHA1"
)
u1_path = ["SPEC135-F12", "SPEC135-U1", "SPEC135-U2", "SPEC135-ALPHA1"]
assert u1_path in alpha1["feeder_paths"]
assert proof["critical_path"] == u1_path
assert (
    "focusa_workspace_artifact_intake_request_v1" in ts
    and "FocusaWorkspaceArtifactIntake" in go
)
for marker in [
    "focusa.workspace.artifact.intake",
    "FocusaSourceConnectorCard",
    "FocusaEvidenceSummary",
    "FocusaReceiptCard",
    "FocusaAdvancedDetails",
    "external_artifact_authority",
    "uiai_session_id",
    "citation_refs",
    "artifact.render.preferred_renderer",
    "artifact.retention.cleanup_action",
    "browser-diagnostics:",
    "evidence:context-retrieve:",
    "image_preview",
    "bounded_metadata_and_handle",
    "close the UIAI session independently",
]:
    assert marker in ui, marker
assert "Workspace Artifact bridge" in ui_html
assert "cookies" not in ui.lower() and "localstorage" not in ui.lower()
print(
    f"Spec 135 U1 Workspace Artifact bridge: PASS ({len(ops)} operations; bounded/external/scoped/evidenced)"
)
