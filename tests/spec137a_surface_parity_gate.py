#!/usr/bin/env python3
"""Verify Spec137A API/registry/CLI/Pi/UI/TUI/notification/platform parity."""
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]
PARITY = json.loads((ROOT / "docs/contracts/spec137a-surface-parity.v1.yaml").read_text())
CORE = (ROOT / "crates/focusa-core/src/temporal_conformance.rs").read_text()
API = (
    (ROOT / "crates/focusa-api/src/routes/temporal.rs").read_text()
    + (ROOT / "crates/focusa-api/src/routes/temporal_conformance.rs").read_text()
)
CLI = (ROOT / "crates/focusa-cli/src/commands/temporal.rs").read_text()
PI = (ROOT / "apps/pi-extension/src/tools.ts").read_text()
UI = (ROOT / "apps/menubar/src/lib/components/TemporalAuthorityPeek.svelte").read_text()
TUI = (ROOT / "crates/focusa-tui/src/mission_control.rs").read_text()
REGISTRY = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/operation-registry.json").read_text()
)
OPENAPI = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/openapi-3.0.3.json").read_text()
)
UI_BINDINGS = json.loads(
    (ROOT / "docs/contracts/spec135/generated-contract-v1/ui-action-bindings.fixture.json").read_text()
)
TYPESCRIPT = (ROOT / "packages/generated/spec135/typescript/schema.d.ts").read_text()
ADVANCED_TEMPORAL_OPERATIONS = {
    "focusa.temporal.clock.capture": "/v1/temporal/clock/capture",
    "focusa.temporal.priority.commit": "/v1/temporal/priority/commit",
    "focusa.temporal.civil.resolve": "/v1/temporal/civil/resolve",
    "focusa.temporal.high_consequence_preflight": "/v1/temporal/high-consequence/preflight",
    "focusa.temporal.migrate_signatures": "/v1/temporal/migrate-signatures",
    "focusa.temporal.settle_closure": "/v1/temporal/settle-closure",
}
required = set(PARITY["required_surfaces"])
records = {row["surface"]: row for row in PARITY["records"]}
assert set(records) == required and len(records) == 9
for name, row in records.items():
    assert row["state"] in {"implemented", "degraded", "unsupported"}, name
    assert row["behavior_ref"] and row["evidence_refs"], name
    if row["state"] != "implemented": assert row["recovery_ref"], name
for symbol in ("SurfaceParityRecord", "SurfaceConformanceState", "validate_surface_parity", "UnknownSurface", "MissingRecovery"):
    assert symbol in CORE, symbol
assert "spec137a_conformance_surface" in API and '"conformance":spec137a_conformance_surface' in API
assert "TemporalCmd::Status" in CLI
assert 'name: "focusa_temporal_authority"' in PI and 'Type.Literal("status")' in PI
assert "<dt>Conformance</dt>" in UI and "conformance.warnings" in UI
assert "conformance={conformance}" in TUI
registry_operations = {row["operation_id"]: row for row in REGISTRY["operations"]}
ui_operation_ids = {row["action_id"] for row in UI_BINDINGS["bindings"]}
assert "focusa.temporal.status" in registry_operations
for operation_id, path in ADVANCED_TEMPORAL_OPERATIONS.items():
    assert registry_operations[operation_id]["path"] == path
    assert OPENAPI["paths"][path]["post"]["operationId"] == operation_id
    assert f'operations["{operation_id}"]' in TYPESCRIPT
    assert operation_id in ui_operation_ids
assert {
    "project_root",
    "continuity_id",
    "idempotency_key",
    "confirm",
    "subject_ref",
    "outcome",
    "evidence_refs",
    "receipt_ref",
}.issubset(
    OPENAPI["components"]["schemas"]["focusa_temporal_settle_closure_request_v1"][
        "required"
    ]
)
assert PARITY["full_conformance_status"] in {
    "proof_pending",
    "blocked_live_proof_required",
    "verified_complete",
}
if PARITY["full_conformance_status"] != "verified_complete":
    assert PARITY["warnings"]
else:
    assert PARITY["status"] == "verified_complete"
print("Spec137A surface parity gate: PASS (9 explicit surfaces; degraded/unsupported/unknown fail closed)")
