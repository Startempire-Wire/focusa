#!/usr/bin/env python3
"""Spec 152 node identity and protected credential leakage gate."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
credentials = (ROOT / "crates/focusa-license/src/authority_credentials.rs").read_text()
client = (ROOT / "crates/focusa-license/src/authority_client.rs").read_text()
http = (ROOT / "crates/focusa-license/src/authority_http.rs").read_text()

assert 'schema: "focusa.node_identity.v1"' in credentials
assert "Uuid::now_v7()" in credentials
assert "atomic_private_write" in credentials
assert "Permissions::from_mode(0o600)" in credentials
for platform in ["MacOsKeychain", "LinuxSecretService", "WindowsCredentialManager"]:
    assert platform in credentials
assert "KeyringCredentialStore" in credentials
assert "rotate_refresh_credential" in credentials
assert "secret_persisted_in_receipt: false" in credentials
assert 'formatter.write_str("SensitiveCredential([REDACTED])")' in client
assert 'formatter.write_str("[REDACTED]")' in client

for type_name in ["LeaseRefreshRequest", "NodeListRequest", "DeactivateNodeRequest"]:
    start = http.index(f"pub struct {type_name}")
    derive = http.rfind("#[derive", 0, start)
    assert "Debug" not in http[derive:start], f"{type_name} must not expose credentials via Debug"

for forbidden in ["println!(refresh_credential", "format!(refresh_credential", "args([refresh_credential"]:
    assert forbidden not in (credentials + client + http)

print("Spec152 protected credentials: PASS (private node identity; macOS/Linux/Windows protected backends; no secret Debug/receipt)")
