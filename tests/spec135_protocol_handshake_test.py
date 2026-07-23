#!/usr/bin/env python3
"""Validate scope-exact capabilities and fail-closed protocol/version handshake."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "docs/contracts/spec135/generated-contract-v1"
lock = json.loads((BUNDLE / "compatibility-lock.yaml").read_text())
accepted = json.loads((BUNDLE / "protocol-handshake.accepted.fixture.json").read_text())
blocked = json.loads((BUNDLE / "protocol-handshake.blocked.fixture.json").read_text())
capabilities = json.loads((BUNDLE / "ui-capability-snapshot.fixture.json").read_text())
openapi = json.loads((BUNDLE / "openapi-3.0.3.json").read_text())
source = (ROOT / "crates/focusa-api/src/routes/agent_capabilities.rs").read_text()
ts = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()
go = (ROOT / "packages/generated/spec135/go/client.gen.go").read_text()

assert lock["schema"] == "focusa.compatibility_lock.v1"
for field in (
    "focusa_runtime",
    "focusa_api",
    "operation_registry",
    "tool_result",
    "event_stream",
    "a2ui_protocol",
    "a2ui_catalog",
    "ag_ui_adapter",
    "pi_runtime",
    "uiai_engine",
    "uiai_focusa_client",
    "docling",
    "embedding_profile",
    "domain_pack_versions",
    "minimum_reader_versions",
    "minimum_writer_versions",
):
    assert field in lock
required_versions = {
    "focusa_api": "1.0.0",
    "operation_registry": "1.0.0",
    "tool_result": "1.0.0",
    "event_stream": "1.0.0",
    "openapi": "3.0.3",
    "json_schema": "2020-12",
    "a2ui_protocol": "0.9.1",
    "a2ui_catalog": "0.9.1",
}
assert required_versions.items() <= lock["minimum_reader_versions"].items()

assert accepted["schema"] == "focusa.protocol_handshake.response.v1"
assert accepted["compatible"] is True and accepted["status"] == "accepted"
assert accepted["safe_state_retained"] is True
assert accepted["server_versions"] == required_versions
assert blocked["schema"] == "focusa.tool_result.v1"
assert blocked["failure_class"] == "stale_runtime_registry"
assert blocked["raw"]["compatible"] is False
assert blocked["raw"]["safe_state_retained"] is True
assert blocked["raw"]["mismatches"]
assert all(
    {"component", "required", "actual", "upgrade_action"} <= item.keys()
    for item in blocked["raw"]["mismatches"]
)

assert capabilities["schema"] == "focusa.ui_capability_snapshot.v1"
assert capabilities["scope_validated"] is True
assert capabilities["project_root"] == "/example"
assert capabilities["continuity_id"] == "example"
assert capabilities["permissions"]["missing_scopes"] == []
assert capabilities["permissions"]["granted_scopes"]
assert all(item["status"] == "available" for item in capabilities["capabilities"])

for path, operation_id, method in (
    ("/v1/agent/compatibility-lock", "focusa.compatibility_lock.read", "get"),
    ("/v1/agent/handshake", "focusa.protocol.handshake", "post"),
):
    assert openapi["paths"][path][method]["operationId"] == operation_id
handshake_parameters = {
    (item["name"], item["in"], item["required"])
    for item in openapi["paths"]["/v1/agent/handshake"]["post"]["parameters"]
}
assert {
    ("project_root", "query", True),
    ("continuity_id", "query", True),
} <= handshake_parameters

for marker in (
    "projection_scope_error",
    "PermissionContext",
    'permissions.allows("project:read")',
    "handshake_mismatches",
    "safe_state_retained",
    "FailureClass::StaleRuntimeRegistry",
):
    assert marker in source
assert 'operations["focusa.protocol.handshake"]' in ts
assert 'operations["focusa.compatibility_lock.read"]' in ts
assert "func (c *Client) FocusaProtocolHandshake(" in go
assert "func (c *Client) FocusaCompatibilityLockRead(" in go

print(
    "Spec 135 protocol handshake: PASS (scope exact, permission projected, incompatible clients fail closed)"
)
