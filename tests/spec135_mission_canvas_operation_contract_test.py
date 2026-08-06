#!/usr/bin/env python3
"""Contract gate for the promoted domain-pack install operation."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OPERATION = "focusa.mission_canvas.domain_pack.install"
REGISTRY = ROOT / "docs/contracts/spec135/mission-canvas-v1/operation-registry.json"
OPENAPI = ROOT / "docs/contracts/spec135/mission-canvas-v1/openapi-3.0.3.json"
CLIENT = ROOT / "docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated.ts"
TRANSPORT = ROOT / "apps/desktop/src/lib/mission-canvas/http-transport.ts"
ROUTE = ROOT / "crates/focusa-api/src/routes/mission_canvas.rs"
SERVICE = ROOT / "crates/focusa-core/src/mission_canvas/domain_pack.rs"
PERSISTENCE = ROOT / "crates/focusa-core/src/mission_canvas/persistence.rs"
CONSUMER = ROOT / "apps/desktop/src/lib/mission-canvas/domain-pack-install-controller.ts"

parser = argparse.ArgumentParser()
parser.add_argument("--operation", required=True)
args = parser.parse_args()
assert args.operation == OPERATION, args.operation

registry = json.loads(REGISTRY.read_text())
operation = next(item for item in registry["operations"] if item["operation_id"] == OPERATION)
assert operation["schema"] == "focusa.mission_canvas.operation_descriptor.v1"
assert operation["operation_version"] == "1.0.0"
assert operation["method"] == "POST"
assert operation["path"] == "/v1/mission-canvas/domain-packs/install"
assert operation["availability"] == "available"
assert operation["confirmation"] == "explicit"
assert operation["permissions_required"] == ["mission_canvas:write"]
assert operation["requires_idempotency_key"] is True
assert operation["requires_if_match_revision"] is False
assert operation["receipt_required"] is True
assert operation["scope_required"] == ["workstream"]
assert operation["response_schema_ref"] == "DomainPackInstallReceipt"

openapi = json.loads(OPENAPI.read_text())
route = openapi["paths"][operation["path"]][operation["method"].lower()]
assert route["operationId"] == OPERATION
assert route["x-focusa-permissions"] == ["mission_canvas:write"]
assert route["x-focusa-receipt-required"] is True
assert route["x-focusa-scope-required"] == ["workstream"]
request_schema = route["requestBody"]["content"]["application/json"]["schema"]
assert "workstream" in request_schema["required"]
assert request_schema["properties"]["workstream"]["$ref"] == "#/components/schemas/WorkstreamKey"
assert route["responses"]["200"]["content"]["application/json"]["schema"]["$ref"].endswith(
    "/DomainPackInstallReceipt"
)

client = CLIENT.read_text()
assert "domain_packInstall(input: MissionCanvasOperationInput): Promise<DomainPackInstallReceipt>" in client
assert f'"{OPERATION}"' in client
transport = TRANSPORT.read_text()
for marker in (
    "registry.operations",
    "validateMissionCanvasContract('WorkstreamAuthorityContext'",
    "idempotency_key_required",
    "explicit_confirmation_required",
    "foreign_receipt_scope",
    "response_schema_ref",
):
    assert marker in transport, marker

route_text = ROUTE.read_text()
for marker in (
    "post(install_domain_pack)",
    "require_permission_with_state",
    "validate_authority",
    "DomainPackInstallCommand",
    "DomainPackInstallService",
    "WorkstreamContext::extract",
    "x-focusa-capabilities",
    "MissingAuthority",
    "confirmation",
    "idempotency_key",
):
    assert marker in route_text, marker

service_text = SERVICE.read_text()
for marker in (
    "DOMAIN_PACK_INSTALL_OPERATION",
    "DOMAIN_PACK_INSTALL_CAPABILITY",
    "DOMAIN_PACK_INSTALL_PERMISSION",
    "WorkstreamContext",
    "ConfirmationRequired",
    "CapabilityUnavailable",
    "PermissionDenied",
    "request_digest",
    "install_domain_pack(",
):
    assert marker in service_text, marker
persistence_text = PERSISTENCE.read_text()
for marker in (
    "mission_canvas_domain_pack_installations",
    "DomainPackIdempotencyConflict",
    "transaction.commit()",
    "receipt_json",
    "receipt_refs",
):
    assert marker in persistence_text, marker

consumer = CONSUMER.read_text()
for marker in (
    "domain_packInstall",
    "awaiting_confirmation",
    "confirmation: 'confirm'",
    "authorityRef",
    "sameWorkstreamKey",
    "foreign_receipt_scope",
):
    assert marker in consumer, marker

print(f"Spec 135 Mission Canvas operation contract: PASS ({OPERATION})")
