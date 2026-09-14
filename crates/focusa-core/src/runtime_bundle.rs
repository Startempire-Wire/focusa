//! Harness-neutral runtime bundle manifest — slice 1 (#257).
//!
//! A runtime bundle is the typed unit of distribution across harnesses and
//! machines: manifest identity, digest-anchored content, target-harness
//! compatibility, workspace placement rules, and activation/rollback
//! references. Bundles never carry authority — the controller decides
//! placement and activation (docs/162 invariants).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const RUNTIME_BUNDLE_SCHEMA: &str = "focusa.runtime_bundle.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeBundleManifest {
    pub schema: String,
    pub bundle_id: String,
    pub version: String,
    pub targets: Vec<BundleTarget>,
    pub placement: BundlePlacement,
    pub content: Vec<BundleArtifact>,
    pub activation: ActivationRef,
    pub rollback: RollbackRef,
    pub digest: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleTarget {
    pub harness: String,
    pub platform: String,
    pub min_focusa_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundlePlacement {
    /// Where the bundle lands on the target machine.
    pub install_root: String,
    /// Compatibility symlink policy (one canonical package rule).
    pub canonical_symlink: String,
    /// Never auto-place on remote hosts; controller decides.
    pub controller_owned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleArtifact {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationRef {
    /// Transaction id from the OTA/install lifecycle (#309).
    pub transaction_ref: String,
    pub requires_operator_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackRef {
    /// Previous bundle id to restore on rollback.
    pub restore_bundle_id: Option<String>,
    pub rollback_transaction_ref: String,
}

impl RuntimeBundleManifest {
    /// Content digest over the ordered artifact list — the anchor every
    /// verification checks before placement or activation.
    pub fn compute_digest(&self) -> String {
        let mut hasher = Sha256::new();
        for artifact in &self.content {
            hasher.update(artifact.path.as_bytes());
            hasher.update(artifact.sha256.as_bytes());
        }
        format!("sha256:{}", hex(&hasher.finalize()))
    }

    /// Verify the manifest is internally consistent: schema, digest match,
    /// targets non-empty, and placement is controller-owned.
    pub fn verify(&self) -> Result<(), String> {
        if self.schema != RUNTIME_BUNDLE_SCHEMA {
            return Err(format!("unexpected schema {}", self.schema));
        }
        if self.bundle_id.trim().is_empty() || self.version.trim().is_empty() {
            return Err("bundle_id and version must be non-empty".to_string());
        }
        if self.targets.is_empty() {
            return Err("at least one target required".to_string());
        }
        for artifact in &self.content {
            if artifact.path.is_empty()
                || artifact.sha256.len() != 64
                || !artifact.sha256.chars().all(|c| c.is_ascii_hexdigit())
            {
                return Err(format!("artifact {} has invalid digest", artifact.path));
            }
        }
        if self.content.is_empty() {
            return Err("at least one artifact required".to_string());
        }
        let computed = self.compute_digest();
        if computed != self.digest {
            return Err(format!(
                "digest mismatch: manifest {0} computed {1}",
                self.digest, computed
            ));
        }
        if !self.placement.controller_owned {
            return Err("placement must be controller-owned".to_string());
        }
        Ok(())
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RuntimeBundleManifest {
        RuntimeBundleManifest {
            schema: RUNTIME_BUNDLE_SCHEMA.to_string(),
            bundle_id: "b1".to_string(),
            version: "0.9.153".to_string(),
            targets: vec![BundleTarget {
                harness: "pi".to_string(),
                platform: "linux-x64".to_string(),
                min_focusa_version: "0.9.121".to_string(),
            }],
            placement: BundlePlacement {
                install_root: "/usr/local/lib/focusa".to_string(),
                canonical_symlink: "/usr/local/bin/focusa".to_string(),
                controller_owned: true,
            },
            content: vec![
                BundleArtifact {
                    path: "bin/focusa".to_string(),
                    sha256: "a".repeat(64),
                    size_bytes: 1_000_000,
                },
                BundleArtifact {
                    path: "bin/focusa-daemon".to_string(),
                    sha256: "b".repeat(64),
                    size_bytes: 2_000_000,
                },
            ],
            activation: ActivationRef {
                transaction_ref: "tx-1".to_string(),
                requires_operator_confirmation: true,
            },
            rollback: RollbackRef {
                restore_bundle_id: Some("b0".to_string()),
                rollback_transaction_ref: "tx-rollback-1".to_string(),
            },
            digest: String::new(),
            created_at: "2026-08-16T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn verify_requires_matching_digest() {
        let mut manifest = sample();
        manifest.digest = manifest.compute_digest();
        assert_eq!(manifest.verify(), Ok(()));
        manifest.digest = "sha256:deadbeef".to_string();
        assert!(manifest.verify().is_err());
    }

    #[test]
    fn verify_rejects_non_controller_placement() {
        let mut manifest = sample();
        manifest.digest = manifest.compute_digest();
        manifest.placement.controller_owned = false;
        assert!(manifest.verify().unwrap_err().contains("controller-owned"));
    }

    #[test]
    fn verify_rejects_bad_artifact_digests() {
        let mut manifest = sample();
        manifest.digest = manifest.compute_digest();
        manifest.content[0].sha256 = "zz".repeat(32);
        assert!(manifest.verify().unwrap_err().contains("invalid digest"));
    }

    #[test]
    fn digest_is_order_sensitive_and_stable() {
        let mut manifest = sample();
        manifest.digest = manifest.compute_digest();
        let first = manifest.compute_digest();
        manifest.content.reverse();
        let reversed = manifest.compute_digest();
        assert_ne!(first, reversed);
        manifest.content.reverse();
        assert_eq!(manifest.compute_digest(), first);
    }
}
