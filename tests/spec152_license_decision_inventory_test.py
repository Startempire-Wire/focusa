#!/usr/bin/env python3
"""Fail-closed audit for Spec 152 license-decision ownership."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVENTORY_PATH = ROOT / "docs/contracts/spec152-license-decision-inventory.v1.json"
inventory = json.loads(INVENTORY_PATH.read_text())
assert inventory["schema"] == "focusa.spec152_license_decision_inventory.v1"
surfaces = inventory["surfaces"]
paths = {surface["path"] for surface in surfaces}

required = {
    "crates/focusa-license/src/authority.rs",
    "crates/focusa-license/src/authority_store.rs",
    "crates/focusa-license/src/lib.rs",
    "crates/focusa-license/src/authority_client.rs",
    "crates/focusa-license/src/authority_http.rs",
    "crates/focusa-license/src/authority_credentials.rs",
    "crates/focusa-api/src/middleware/entitlement.rs",
    "crates/focusa-api/src/routes/license.rs",
    "crates/focusa-api/src/server.rs",
    "crates/focusa-core/src/license.rs",
    "crates/focusa-cli/src/commands/license.rs",
    "crates/focusa-cli/src/commands/install.rs",
    "scripts/install-focusa.sh",
    "scripts/install-focusa.ps1",
    "apps/menubar/src/lib/api.ts",
    "apps/pi-extension/src/commands.ts",
}
assert not required - paths, f"unclassified production surfaces: {sorted(required - paths)}"

allowed_classes = set(inventory["classifications"])
for surface in surfaces:
    assert surface["classification"] in allowed_classes
    assert surface["decisions"], f"empty decision classification: {surface['path']}"
    assert (ROOT / surface["path"]).is_file(), f"inventoried path missing: {surface['path']}"

may_grant = [surface["path"] for surface in surfaces if surface["may_grant"]]
assert may_grant == [
    "crates/focusa-license/src/authority.rs",
    "crates/focusa-license/src/authority_store.rs",
    "crates/focusa-license/src/lib.rs",
]

store = (ROOT / "crates/focusa-license/src/authority_store.rs").read_text()
authority = (ROOT / "crates/focusa-license/src/authority.rs").read_text()
license_lib = (ROOT / "crates/focusa-license/src/lib.rs").read_text()
entitlement = (ROOT / "crates/focusa-api/src/middleware/entitlement.rs").read_text()
legacy = (ROOT / "crates/focusa-core/src/license.rs").read_text()

for rejected in ["test-root", "fixture-authority", "local-dev-root", "example-root"]:
    assert rejected in store or rejected in (ROOT / "crates/focusa-license/tests/authority_lease.rs").read_text()
assert "parse_production_trust_roots" in store
assert "recovery_only" in store.lower()
assert "verify_lease" in authority
assert "resolve_license_guard" in license_lib
resolver = license_lib[license_lib.index("pub fn resolve_license_guard()") : license_lib.index("/// Read ~/.config/focusa/license.json")]
assert "LicenseGuard::eval" not in resolver, "production resolver must not self-issue eval authority"
assert "read_license_json" not in resolver, "legacy plaintext must not enter production resolver"
assert "LicenseGuard::eval(7)" in entitlement
assert "!entitlement_allows_mutation" in entitlement
assert "Active | EntitlementState::OfflineGrace" in entitlement
assert "resolve_license_guard()" in legacy
assert "load_local_license()" not in legacy[legacy.index("pub fn load_license_status"):legacy.index("pub fn load_local_license")]

for source in [store, authority, license_lib]:
    lowered = source.lower()
    assert "allow_license_bypass" not in lowered
    assert "skip_license" not in lowered
    assert "trust_fixture_root" not in lowered

print(f"Spec152 license decision inventory: PASS ({len(surfaces)} classified surfaces; {len(may_grant)} authority modules)")
