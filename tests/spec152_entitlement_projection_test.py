#!/usr/bin/env python3
"""Build-independent Spec 152 canonical entitlement projection gate."""

from __future__ import annotations

import base64
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = json.loads(
    (ROOT / "docs/contracts/spec152-entitlement-projection-fixture.v1.json").read_text()
)
GOLDEN = json.loads(
    (
        ROOT
        / "crates/focusa-license/tests/fixtures/spec152-authority-golden-vector.json"
    ).read_text()
)
EXPECTED = FIXTURE["expected_projection"]
LEASE = json.loads(base64.b64decode(GOLDEN["lease_envelope"]["payload_b64"]))

projected = {
    "schema": "focusa.entitlement_projection.v1",
    "state": LEASE["status"],
    "product": LEASE["product"],
    "node_id": LEASE["node_id"],
    "lease_id": LEASE["lease_id"],
    "lease_sequence": LEASE["sequence"],
    "lease_digest": GOLDEN["expected_lease_digest"],
    "expires_at": LEASE["expires_at"].replace("Z", "+00:00"),
    "offline_grace_until": LEASE["offline_grace_until"].replace("Z", "+00:00"),
    "features": LEASE["features"],
    "limits": LEASE["limits"],
    "recovery_reason": None,
}
assert projected == EXPECTED, "projection fixture drifted from signed authority golden vector"

license_source = (ROOT / "crates/focusa-license/src/lib.rs").read_text()
core_source = (ROOT / "crates/focusa-core/src/license.rs").read_text()
api_source = (ROOT / "crates/focusa-api/src/routes/license.rs").read_text()
cli_source = (ROOT / "crates/focusa-cli/src/commands/license.rs").read_text()

for field in EXPECTED:
    assert f"pub {field}:" in license_source, f"canonical projection missing {field}"
assert "pub fn entitlement_projection(" in license_source
assert "snapshot.ok_or(LicenseError::EntitlementSnapshotMissing)?" in license_source
assert "ENTITLEMENT_SNAPSHOT_MISSING" in license_source

assert "pub authority: Option<focusa_license::EntitlementProjection>" in core_source
assert "focusa_license::entitlement_projection(entitlement)" in core_source
assert '"authority": authority' in api_source
assert "StatusCode::SERVICE_UNAVAILABLE" in api_source
assert '"error": "ENTITLEMENT_SNAPSHOT_MISSING"' in api_source
assert "focusa_license::entitlement_projection(guard.entitlement.as_ref())?" in cli_source
assert '"authority": authority' in cli_source

run_status = cli_source.split("async fn run_status", 1)[1].split("async fn run_deactivate", 1)[0]
for forbidden in (
    "snapshot.map(",
    "snapshot.and_then(",
    "features.is_empty()",
    "FOCUSA_REQUIRE_REAL_LICENSE",
):
    assert forbidden not in run_status, f"CLI projection retains forbidden fallback: {forbidden}"

print("Spec152 canonical entitlement projection: PASS (golden vector + core/API/CLI parity)")
