//! Compiles the canonical validator on every platform, independently of systemd.
#[path = "../src/commands/distribution_manifest.rs"]
mod distribution_manifest;

use distribution_manifest::validate_distribution_manifest;
use serde_json::{Value, json};
use std::path::PathBuf;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("focusa-manifest-配布-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> PathBuf {
        self.0.join("manifest.json")
    }
    fn write(&self, bytes: &[u8]) {
        std::fs::write(self.path(), bytes).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn manifest() -> Value {
    serde_json::from_str(include_str!(
        "../../../docs/contracts/spec141/generated-capability-v2/distribution-manifest.json"
    ))
    .unwrap()
}

#[test]
fn portable_validator_preserves_exact_bytes_and_accepts_tag_or_version() {
    let fixture = Fixture::new();
    let value = manifest();
    let version = value["release_version"].as_str().unwrap();
    let bytes = serde_json::to_vec_pretty(&value).unwrap();
    fixture.write(&bytes);
    for expected in [version.to_owned(), format!("v{version}")] {
        assert_eq!(
            validate_distribution_manifest(&fixture.path(), &expected).unwrap(),
            bytes
        );
    }
}

#[test]
fn portable_validator_rejects_missing_or_changed_contract_fields() {
    let fixture = Fixture::new();
    let original = manifest();
    let tag = format!("v{}", original["release_version"].as_str().unwrap());
    for pointer in [
        "/schema",
        "/release_version",
        "/digest_contract",
        "/components/runtime_contract/installed_manifest_path",
        "/components/runtime_contract/manifest_required_from",
        "/components/runtime_contract/binary_paths/cli",
        "/components/runtime_contract/binary_paths/daemon",
        "/components/runtime_contract/binary_paths/tui",
        "/components/runtime_contract/binary_paths/session_runner",
    ] {
        for invalid in [Value::Null, json!("invalid-contract-value")] {
            let mut value = original.clone();
            *value.pointer_mut(pointer).unwrap() = invalid;
            fixture.write(&serde_json::to_vec(&value).unwrap());
            assert!(
                validate_distribution_manifest(&fixture.path(), &tag).is_err(),
                "accepted {pointer}"
            );
        }
    }
}

#[test]
fn portable_validator_rejects_unreadable_or_malformed_manifest() {
    let fixture = Fixture::new();
    assert!(validate_distribution_manifest(&fixture.path(), "v0.9.189").is_err());
    fixture.write(b"{bad json");
    assert!(validate_distribution_manifest(&fixture.path(), "v0.9.189").is_err());
}

#[test]
fn installer_and_linux_transaction_use_one_unconditional_validator() {
    let modules = include_str!("../src/commands/mod.rs");
    assert!(modules.contains(
        "pub mod device_pairing;\npub(crate) mod distribution_manifest;\npub mod doctor;"
    ));
    let installer = include_str!("../src/commands/install.rs");
    assert!(
        installer
            .contains("crate::commands::distribution_manifest::validate_distribution_manifest")
    );
    assert!(!installer.contains("crate::commands::system_service::validate_distribution_manifest"));
    let transaction = include_str!("../src/commands/system_service_manifest.rs");
    assert!(
        transaction.contains(
            "use crate::commands::distribution_manifest::validate_distribution_manifest;"
        )
    );
    assert!(!transaction.contains("fn validate_distribution_manifest("));
}
