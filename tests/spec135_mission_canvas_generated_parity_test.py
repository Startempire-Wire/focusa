#!/usr/bin/env python3
"""Generated artifact parity, compatibility, and determinism gate for Mission Canvas."""
from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/contracts/spec135/mission-canvas-v1"


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


commands = [
    ["python3", "scripts/generate-spec135-mission-canvas-schemas.py", "--check"],
    ["python3", "scripts/generate-spec135-mission-canvas-operations.py", "--check"],
    ["python3", "scripts/generate-spec135-mission-canvas-derived-contracts.py", "--check"],
]
for command in commands:
    subprocess.run(command, cwd=ROOT, check=True)

registry_path = OUT / "operation-registry.json"
bundle_path = ROOT / "schemas/spec135/mission-canvas/composition-bundle.v1.schema.json"
bundle_defs = json.loads(bundle_path.read_text())["$defs"]
openapi_path = OUT / "openapi-3.0.3.json"
registry = json.loads(registry_path.read_text())
openapi = json.loads(openapi_path.read_text())
capabilities = json.loads((OUT / "capability-snapshot.json").read_text())
bindings = json.loads((OUT / "ui-action-bindings.json").read_text())
lock = json.loads((OUT / "compatibility-lock.json").read_text())
handshake = json.loads((OUT / "protocol-handshake.json").read_text())
client = (OUT / "typescript/mission-canvas-client.generated.ts").read_text()
types = (OUT / "typescript/mission-canvas-types.generated.ts").read_text()
validators = (OUT / "typescript/mission-canvas-validators.generated.ts").read_text()

registry_ids = {entry["operation_id"] for entry in registry["operations"]}
openapi_ids = {
    operation["operationId"]
    for path in openapi["paths"].values()
    for method, operation in path.items()
    if method in {"get", "post", "put", "patch", "delete"}
}
capability_ids = {entry["operation_id"] for entry in capabilities["operations"]}
binding_ids = {entry["operation_id"] for entry in bindings["bindings"]}
assert registry_ids == openapi_ids == capability_ids == binding_ids
assert openapi["openapi"] == "3.0.3"
for capability in capabilities["operations"]:
    operation = next(item for item in registry["operations"] if item["operation_id"] == capability["operation_id"])
    assert capability["scope_required"] == ["workstream"]
    assert capability["scope_optional"] == operation["scope_optional"]
    assert capability["authority_chain"] == operation["authority_chain"]
identity_defs = {
    "ProjectRootKey",
    "ScopeRef",
    "WorkstreamId",
    "WorkstreamKey",
    "ContinuityId",
    "AttachmentKey",
    "SessionId",
    "InstanceId",
    "WorkspaceBindingId",
    "RuntimeObjectRef",
    "WorkSurfaceId",
    "WorkstreamAuthorityContext",
}
assert identity_defs.issubset(openapi["components"]["schemas"])
assert "ExactScope" not in openapi["components"]["schemas"]
assert "export type WorkstreamKey" in types
assert "export type AttachmentKey" in types
assert "export type WorkSurfaceId" in types
assert "sameWorkstreamAuthorityContext" in validators
assert "MissionCanvasOperationInput" in client
assert "WorkstreamAuthorityContext" in client
for entry in registry["operations"]:
    assert entry["scope_required"] == ["workstream"]
    assert "project_root" not in entry["scope_required"]
    assert entry["authority_chain"] == [
        "scope_ref", "project_root_key", "workstream_id", "continuity_id",
        "attachment_key", "session_id", "instance_id", "workspace_binding_id",
        "runtime_object", "work_surface_id",
    ]
for path_item in openapi["paths"].values():
    for operation in path_item.values():
        if isinstance(operation, dict) and "operationId" in operation:
            assert operation["x-focusa-scope-required"] == ["workstream"]
            assert "project_root" not in operation["x-focusa-scope-required"]
            if "requestBody" in operation:
                request_schema = operation["requestBody"]["content"]["application/json"]["schema"]
                if "required" in request_schema:
                    assert "workstream" in request_schema["required"]
                    assert request_schema["properties"]["workstream"]["$ref"] == "#/components/schemas/WorkstreamKey"
                else:
                    schema_name = request_schema["$ref"].rsplit("/", 1)[-1]
                    assert "workstream" in bundle_defs[schema_name]["required"]
assert capabilities["runtime_promoted"]
assert capabilities["all_operations_promoted"]
assert all(entry["status"] == "available" for entry in capabilities["operations"])
assert all(entry["enabled"] for entry in bindings["bindings"])
assert client.count("return this.transport.request<") == registry["operation_count"]
assert "export type ResolvedWorkspaceProjection" in types
assert "export type HostLifecycleState" in types
assert "validateMissionCanvasContract" in validators

assert lock["schema_bundle_sha256"] == digest(bundle_path)
assert lock["operation_registry_sha256"] == digest(registry_path)
for relative, expected in lock["derived_artifacts"].items():
    assert expected == "sha256:" + digest(OUT / relative), relative
assert handshake["schema_bundle_digest"] == "sha256:" + lock["schema_bundle_sha256"]
assert handshake["operation_registry_digest"] == "sha256:" + lock["operation_registry_sha256"]
assert handshake["required_scope_keys"] == ["workstream"]
assert handshake["authority_chain"] == [
    "scope_ref", "project_root_key", "workstream_id", "continuity_id",
    "attachment_key", "session_id", "instance_id", "workspace_binding_id",
    "runtime_object", "work_surface_id",
]
assert handshake["runtime_promotion_required"] is False

tracked = [bundle_path, registry_path, *[OUT / relative for relative in lock["derived_artifacts"]], OUT / "compatibility-lock.json", OUT / "protocol-handshake.json"]
before = {path: digest(path) for path in tracked}
subprocess.run(["python3", "scripts/generate-spec135-mission-canvas-schemas.py"], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
subprocess.run(["python3", "scripts/generate-spec135-mission-canvas-operations.py"], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
subprocess.run(["python3", "scripts/generate-spec135-mission-canvas-derived-contracts.py"], cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
after = {path: digest(path) for path in tracked}
assert before == after

print("Spec 135 Mission Canvas generated parity and determinism: PASS")
