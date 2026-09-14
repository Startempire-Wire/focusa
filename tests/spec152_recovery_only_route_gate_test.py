#!/usr/bin/env python3
"""Spec 152 recovery-only route classification and pre-side-effect gate."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
gate = (ROOT / "crates/focusa-api/src/middleware/entitlement.rs").read_text()
server = (ROOT / "crates/focusa-api/src/server.rs").read_text()

for route in [
    "/v1/workpoint/checkpoint",
    "/v1/evidence/capture",
    "/v1/turn",
    "/v1/silent-sessions/start",
    "/v1/update/apply",
    "/v1/export/run",
]:
    assert route in gate

for recovery in [
    'path == "/health"',
    'path == "/v1/health"',
    'path == "/v1/version"',
    'path == "/v1/update/check"',
    'path == "/v1/update/plan"',
    'path == "/v1/update/rollback"',
    'path.starts_with("/v1/license/")',
    "is_recovery_export(path)",
]:
    assert recovery in gate

# Pairing and connection are authentication conveniences, not entitlement or
# recovery authority. Spec 152 requires entitlement before pairing/execution.
assert 'path.starts_with("/v1/connect/")' not in gate
assert 'path.starts_with("/v1/device/pair/")' not in gate

assert "matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)" in gate
assert "snapshot.expires_at.is_some_and" in gate
assert "offline_grace_until" in gate
assert "lease_id" in gate
assert 'value.starts_with("sha256:")' in gate
assert "EntitlementState::Unactivated | EntitlementState::RecoveryOnly => false" in gate

entitlement_layer = server.index("middleware::entitlement::entitlement_gate_layer")
route_merge = server.index(".merge(routes::workpoint::router())")
assert entitlement_layer > route_merge, "entitlement middleware must wrap merged route handlers"
assert server.index("middleware::auth::auth_layer") > entitlement_layer

print("Spec152 recovery-only route gate: PASS (safe reads/recovery public; six mutation families pre-gated)")
