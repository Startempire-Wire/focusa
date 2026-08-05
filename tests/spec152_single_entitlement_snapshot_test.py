#!/usr/bin/env python3
"""Spec 152 single immutable entitlement projection gate."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
license_lib = (ROOT / "crates/focusa-license/src/lib.rs").read_text()
core = (ROOT / "crates/focusa-core/src/license.rs").read_text()
api = (ROOT / "crates/focusa-api/src/routes/license.rs").read_text()
server = (ROOT / "crates/focusa-api/src/server.rs").read_text()
main = (ROOT / "crates/focusa-api/src/main.rs").read_text()
cli = (ROOT / "crates/focusa-cli/src/commands/license.rs").read_text()

check = license_lib[license_lib.index("pub fn check(&self") : license_lib.index("/// Hard-require")]
assert "self.entitlement" in check
assert "feature_enabled" in check
assert "match (self.tier" not in check
assert "Tier::Licensed" not in check
assert "Tier::Open" not in check
assert "LocalEval" not in check
assert "legacy tier is migration-only" in check

resolver = license_lib[license_lib.index("pub fn resolve_license_guard()") : license_lib.index("/// Read ~/.config/focusa/license.json")]
assert "resolve_authority_state" in resolver
assert "read_license_json" not in resolver
assert "read_license_toml" not in resolver

status = core[core.index("pub fn load_license_status") : core.index("fn status_from_local")]
assert "focusa_license::resolve_license_guard()" in status
assert "load_local_license" not in status
assert "focusa_license::entitlement_projection(entitlement)" in status
assert "authority: Some(authority)" in status

assert "state.license_guard.clone()" in api
assert "focusa_license::entitlement_projection(g.entitlement.as_ref())" in api
assert '"authority": authority' in api
assert '"error": "ENTITLEMENT_SNAPSHOT_MISSING"' in api
assert "license_guard:" in server
assert "focusa_license::resolve_license_guard()" in main
assert "license_guard," in main
assert "focusa_license::entitlement_projection(guard.entitlement.as_ref())?" in cli
assert '"authority": authority' in cli

print("Spec152 single entitlement snapshot: PASS (daemon/core/API/CLI delegate to focusa-license)")
