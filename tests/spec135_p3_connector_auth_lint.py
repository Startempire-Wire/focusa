#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
source = (R / "crates/focusa-core/src/connector_auth.rs").read_text()
root_manifest = (R / "Cargo.toml").read_text()
core_manifest = (R / "crates/focusa-core/Cargo.toml").read_text()
contract = json.loads(
    (R / "docs/contracts/spec135-p3-connector-auth-lifecycle.v1.yaml").read_text()
)
for marker in (
    "ConnectorOAuthConfig",
    "ConnectorAuthLifecycle",
    "PkceCodeChallenge::new_random_sha256",
    'entry("access")',
    "delete_credential",
    "authorization_required",
    "repair_os_keyring",
):
    assert marker in source
assert 'oauth2 = { version = "5"' in root_manifest
assert 'keyring = "3"' in root_manifest
assert "oauth2 = { workspace = true }" in core_manifest
assert "keyring = { workspace = true }" in core_manifest
for forbidden in ("println!", "dbg!", "access_token: String", "client_secret: String"):
    assert forbidden not in source
assert contract["secret_boundary"]["storage"] == "operating_system_keyring"
assert contract["secret_boundary"]["serializable_records_contain_secrets"] is False
print("Spec 135 P3 OAuth2/keyring connector lifecycle strict lint: PASS")
