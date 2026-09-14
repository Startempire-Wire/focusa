#!/usr/bin/env python3
"""Static acceptance gate for Spec 150A/152 installer entitlement ordering."""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INSTALL = (ROOT / "crates/focusa-cli/src/commands/install.rs").read_text()
CLIENT = (ROOT / "crates/focusa-license/src/authority_client.rs").read_text()
STORE = (ROOT / "crates/focusa-license/src/authority_store.rs").read_text()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


require('return Ok("eval".to_string())' not in INSTALL, "self-issued eval bypass remains")
require("persist_eval_license" not in INSTALL, "installer persists a local evaluation license")
require(
    "raw license keys cannot authorize installation" in INSTALL,
    "legacy raw-key path is not fail-closed",
)
require(
    "DeviceAuthorizationSession::new" in INSTALL
    and "AuthorityHttpClient::new" in INSTALL,
    "device-code start/poll orchestration is missing",
)
require(
    "key_set_envelope" in CLIENT,
    "authorized response cannot carry the signed authority key set",
)
require(
    "PersistedAuthorityState::from_verified_envelopes" in INSTALL,
    "issued lease is not verified before persistence",
)
require(
    "rotate_refresh_credential" in INSTALL and "KeyringCredentialStore" in INSTALL,
    "refresh credential does not use native protected storage",
)
require(
    "pub fn write_atomic" in STORE and "file.sync_all()" in STORE,
    "authority state persistence is not atomic and durable",
)

run_start = INSTALL.index("pub async fn run(args: InstallArgs)")
dry_run = INSTALL.index("if dry_run {", run_start)
real_install = INSTALL.index("execute_real_install(", dry_run)
require(dry_run < real_install, "dry-run no longer exits before installation")

execute_start = INSTALL.index("async fn execute_real_install(")
license_gate = INSTALL.index("phase_license(args, channel).await?", execute_start)
asset_download = INSTALL.index("phase_asset_download(", execute_start)
require(license_gate < asset_download, "assets can download before entitlement authorization")

acquire_start = INSTALL.index("async fn acquire_installer_entitlement(")
protected_write = INSTALL.index("rotate_refresh_credential(", acquire_start)
state_write = INSTALL.index(".write_atomic(&config_dir.join(AUTHORITY_STATE_FILE))", acquire_start)
require(protected_write < state_write, "authority state can persist before protected credential")

print("Focusa installer entitlement activation gate: PASS")
