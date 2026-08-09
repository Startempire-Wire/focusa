//! Spec 152 / 152A Section 3 — signed capsule manifest and provenance contract.
//!
//! A protected feature capsule is a versioned distribution unit containing one
//! or more private workers, modules, models, prompts, policies, or assets.
//! Every capsule carries a signed manifest that binds capsule identity, version,
//! product and feature namespace, platform/architecture compatibility, plaintext
//! and ciphertext digests, the minimum compatible public-shell contract,
//! required lease features and limit policy version, release channel and status,
//! revocation state, provenance, and the node-bound key envelope reference.
//!
//! Digest and signature authority live ONLY inside this signed manifest:
//! an unsigned sidecar checksum carries no authority and there is no global key.
//! The capsule-signing key and payload-encryption keys never ship in source
//! control. Modified, mixed, downgraded, unknown, withdrawn, revoked, unsigned,
//! or shell-incompatible manifests fail closed before load.
//!
//! Callers supply factual canonical-registry/delivery lookup results via
//! [`CapsuleVerificationFacts`]; they NEVER supply product, feature, limit,
//! price, channel, or commercial rights.

use std::collections::BTreeMap;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Canonical signed capsule manifest schema.
pub const CAPSULE_MANIFEST_SCHEMA: &str = "focusa.capsule_manifest.v1";

/// The only manifest schema version the verifier accepts.
pub const CAPSULE_MANIFEST_VERSION: u32 = 1;

/// Schema of the node-bound capsule key envelope referenced by a manifest.
/// The envelope itself is issued per-node and implemented by the key-envelope
/// client; the manifest binds it by reference and digest only.
pub const NODE_KEY_ENVELOPE_SCHEMA: &str = "focusa.node_capsule_key_envelope.v1";

/// Canonical limit policy version bound by manifests (Spec 152 feature/limit
/// policy registry). Unknown policy versions fail closed.
pub const KNOWN_LIMIT_POLICY_VERSION: &str = "focusa.limit_policy.v1";

/// Registered capsule platforms (Spec 152A Section 3 platform compatibility).
pub const REGISTERED_CAPSULE_PLATFORMS: [&str; 3] = ["linux", "macos", "windows"];

/// Registered capsule architectures.
pub const REGISTERED_CAPSULE_ARCHES: [&str; 2] = ["x86_64", "aarch64"];

/// Registered release channels.
pub const REGISTERED_CAPSULE_CHANNELS: [&str; 4] = ["stable", "preview", "beta", "internal"];

/// Registered release statuses. `withdrawn` is recall state and fails closed.
pub const REGISTERED_CAPSULE_RELEASE_STATUSES: [&str; 3] = ["released", "deprecated", "withdrawn"];

/// The only signature algorithm accepted by the verifier.
pub const CAPSULE_SIGNATURE_ALGORITHM: &str = "ed25519";

/// One sha256 digest value bound inside the signed manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleDigest {
    pub algorithm: String,
    pub value: String,
}

/// Plaintext and ciphertext digests of the capsule payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleDigests {
    pub plaintext: CapsuleDigest,
    pub ciphertext: CapsuleDigest,
}

/// Minimum compatible public-shell contract the capsule requires.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicShellContract {
    pub id: String,
    pub major: u32,
    pub minor: u32,
}

/// Revocation state bound inside the signed manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleRevocation {
    pub revoked: bool,
    pub revocation_id: Option<String>,
    pub revoked_at: Option<String>,
    pub reason: Option<String>,
}

/// Signed build provenance for the capsule payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleProvenance {
    pub builder_id: String,
    pub build_id: String,
    pub source_commit: String,
    pub signed_at: String,
}

/// Reference to the node-bound content-key envelope issued with this manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEnvelopeRef {
    pub schema: String,
    pub envelope_ref: String,
    pub envelope_digest_sha256: String,
}

/// ed25519 signature envelope over the canonical manifest body bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleSignature {
    pub signature_algorithm: String,
    pub signer_key_id: String,
    pub signature_b64: String,
}

/// The signed capsule manifest (Spec 152A Section 3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleManifest {
    pub schema: String,
    pub manifest_version: u32,
    pub capsule_id: String,
    pub capsule_version: String,
    pub product: String,
    pub feature_namespace: String,
    pub platform: String,
    pub arch: String,
    pub digests: CapsuleDigests,
    pub public_shell_contract: PublicShellContract,
    pub required_features: Vec<String>,
    pub limit_policy_version: String,
    pub channel: String,
    pub release_status: String,
    pub revocation: CapsuleRevocation,
    pub provenance: CapsuleProvenance,
    pub key_envelope: KeyEnvelopeRef,
    pub signature: CapsuleSignature,
}

/// One trusted capsule-signing key (public material only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedSignerKey {
    pub key_id: String,
    pub verifying_key_b64: String,
}

/// Factual canonical-registry and delivery lookup results supplied by the
/// runtime intake. Callers report what the signed registry and the delivery
/// pipeline measured; they NEVER supply product, feature, limit, price,
/// channel, or commercial rights.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleVerificationFacts {
    /// sha256 of the plaintext payload as measured by the delivery pipeline,
    /// when a delivery measurement exists.
    pub delivered_plaintext_sha256: Option<String>,
    /// sha256 of the delivered ciphertext as measured by the delivery pipeline,
    /// when a delivery measurement exists.
    pub delivered_ciphertext_sha256: Option<String>,
    /// The public shell contract the runtime currently exposes.
    pub runtime_shell_contract: PublicShellContract,
    /// Capsule versions already installed per feature namespace (mixed-set check).
    pub installed_namespace_versions: BTreeMap<String, String>,
    /// Latest known release per feature namespace (rollback/downgrade check).
    pub known_latest_namespace_versions: BTreeMap<String, String>,
    /// Feature keys registered in the canonical feature registry.
    pub registered_feature_keys: Vec<String>,
    /// Limit policy versions the authority has published.
    pub known_limit_policy_versions: Vec<String>,
    /// Signing keys the runtime trusts for capsule manifests.
    pub trusted_signer_keys: Vec<TrustedSignerKey>,
    /// Capsule ids the authority has revoked (independent of manifest claims).
    pub revoked_capsule_ids: Vec<String>,
}

impl Default for CapsuleVerificationFacts {
    fn default() -> Self {
        Self {
            delivered_plaintext_sha256: None,
            delivered_ciphertext_sha256: None,
            runtime_shell_contract: PublicShellContract {
                id: "focusa.public_shell".to_string(),
                major: 1,
                minor: 0,
            },
            installed_namespace_versions: BTreeMap::new(),
            known_latest_namespace_versions: BTreeMap::new(),
            registered_feature_keys: Vec::new(),
            known_limit_policy_versions: vec![KNOWN_LIMIT_POLICY_VERSION.to_string()],
            trusted_signer_keys: Vec::new(),
            revoked_capsule_ids: Vec::new(),
        }
    }
}

/// Fail-closed trust decision for one signed capsule manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleVerificationDecision {
    /// The manifest is signed by a trusted key and every bound claim matches
    /// the canonical registry and delivery measurements.
    Verified,
    /// No well-formed ed25519 signature envelope is present.
    RejectedUnsigned,
    /// The signing key id is not in the trusted signer set.
    RejectedUnknownSigner,
    /// The signature does not verify over the canonical manifest body bytes.
    RejectedInvalidSignature,
    /// Unknown schema, manifest version, capsule id, namespace, or malformed
    /// capsule version.
    RejectedUnknownManifest,
    /// Product is not a registered product owner.
    RejectedUnknownProduct,
    /// Platform is not a registered capsule platform.
    RejectedUnknownPlatform,
    /// Architecture is not a registered capsule architecture.
    RejectedUnknownArch,
    /// Channel is not a registered release channel.
    RejectedUnknownChannel,
    /// Release status is not a registered status.
    RejectedUnknownReleaseStatus,
    /// A required feature is not registered in the canonical feature registry.
    RejectedUnknownFeature,
    /// The limit policy version is not a published authority policy.
    RejectedUnknownLimitPolicy,
    /// The node-bound key envelope reference is missing, unknown, or malformed.
    RejectedMissingKeyEnvelope,
    /// The capsule or its payload was modified after signing/delivery.
    RejectedModified,
    /// A different capsule version for the same feature namespace is installed.
    RejectedMixedSet,
    /// The manifest version is older than the latest known release for the
    /// namespace (rollback protection).
    RejectedDowngraded,
    /// The runtime public-shell contract cannot satisfy the manifest minimum.
    RejectedIncompatibleShell,
    /// The capsule is revoked by manifest claim or by authority facts.
    RejectedRevoked,
    /// The capsule was withdrawn by the publisher.
    RejectedWithdrawn,
}

impl CapsuleVerificationDecision {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::RejectedUnsigned => "rejected_unsigned",
            Self::RejectedUnknownSigner => "rejected_unknown_signer",
            Self::RejectedInvalidSignature => "rejected_invalid_signature",
            Self::RejectedUnknownManifest => "rejected_unknown_manifest",
            Self::RejectedUnknownProduct => "rejected_unknown_product",
            Self::RejectedUnknownPlatform => "rejected_unknown_platform",
            Self::RejectedUnknownArch => "rejected_unknown_arch",
            Self::RejectedUnknownChannel => "rejected_unknown_channel",
            Self::RejectedUnknownReleaseStatus => "rejected_unknown_release_status",
            Self::RejectedUnknownFeature => "rejected_unknown_feature",
            Self::RejectedUnknownLimitPolicy => "rejected_unknown_limit_policy",
            Self::RejectedMissingKeyEnvelope => "rejected_missing_key_envelope",
            Self::RejectedModified => "rejected_modified",
            Self::RejectedMixedSet => "rejected_mixed_set",
            Self::RejectedDowngraded => "rejected_downgraded",
            Self::RejectedIncompatibleShell => "rejected_incompatible_shell",
            Self::RejectedRevoked => "rejected_revoked",
            Self::RejectedWithdrawn => "rejected_withdrawn",
        }
    }

    pub const fn is_verified(self) -> bool {
        matches!(self, Self::Verified)
    }

    pub const fn is_rejected(self) -> bool {
        !matches!(self, Self::Verified)
    }

    /// Stable Spec 172 Section 21 error code. Rejected manifests never load and
    /// never become limited/paid by client metadata.
    pub const fn stable_error(self) -> &'static str {
        match self {
            Self::Verified => "",
            _ => super::dynamic_operation_manifest::ENTITLEMENT_POLICY_UNKNOWN,
        }
    }
}

/// Deterministic canonicalization of a capsule manifest body: every field
/// except the signature envelope, serialized with sorted keys and compact
/// separators (`json.dumps(body, sort_keys=True, separators=(',', ':'))`).
///
/// This is the exact byte sequence the authority signs; replaying it from the
/// pinned commit reproduces the same digest for every vector.
pub fn canonical_capsule_manifest_bytes(
    manifest: &CapsuleManifest,
) -> Result<Vec<u8>, serde_json::Error> {
    let mut value = serde_json::to_value(manifest)?;
    if let Some(object) = value.as_object_mut() {
        object.remove("signature");
    }
    serde_json::to_string(&value).map(String::into_bytes)
}

/// sha256 (hex) of the deterministic canonical manifest body.
pub fn capsule_manifest_sha256(manifest: &CapsuleManifest) -> Result<String, serde_json::Error> {
    Ok(format!("{:x}", Sha256::digest(canonical_capsule_manifest_bytes(manifest)?)))
}

fn parse_version(value: &str) -> Option<[u64; 3]> {
    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let mut out = [0u64; 3];
    for (index, part) in parts.iter().enumerate() {
        out[index] = part.parse::<u64>().ok()?;
    }
    Some(out)
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Verify one signed capsule manifest against canonical facts. Fail-closed
/// gates run in a fixed order; the first violation rejects the manifest.
///
/// 1. Unknown/malformed manifest identity (schema, version, capsule id,
///    namespace, capsule version shape).
/// 2. No well-formed ed25519 signature envelope.
/// 3. Signing key id not trusted.
/// 4. Signature does not verify over the canonical manifest body bytes.
/// 5. Unknown platform / arch / channel / release status.
/// 6. Unknown product owner.
/// 7. Required feature or limit policy version not registered.
/// 8. Missing or unknown node-bound key envelope reference.
/// 9. Revoked or withdrawn state (manifest claim or authority facts).
/// 10. Delivered payload digest mismatch (modified payload).
/// 11. Installed version conflict (mixed capsule set).
/// 12. Version older than the latest known release (downgrade).
/// 13. Runtime public-shell contract below the manifest minimum.
pub fn verify_capsule_manifest(
    manifest: &CapsuleManifest,
    facts: &CapsuleVerificationFacts,
) -> CapsuleVerificationDecision {
    if manifest.schema != CAPSULE_MANIFEST_SCHEMA || manifest.manifest_version != CAPSULE_MANIFEST_VERSION
    {
        return CapsuleVerificationDecision::RejectedUnknownManifest;
    }
    if manifest.capsule_id.is_empty() || manifest.feature_namespace.is_empty() {
        return CapsuleVerificationDecision::RejectedUnknownManifest;
    }
    if parse_version(&manifest.capsule_version).is_none() {
        return CapsuleVerificationDecision::RejectedUnknownManifest;
    }

    if manifest.signature.signature_algorithm != CAPSULE_SIGNATURE_ALGORITHM
        || manifest.signature.signature_b64.is_empty()
    {
        return CapsuleVerificationDecision::RejectedUnsigned;
    }
    let Ok(signature_bytes) = BASE64.decode(&manifest.signature.signature_b64) else {
        return CapsuleVerificationDecision::RejectedUnsigned;
    };
    if signature_bytes.len() != 64 {
        return CapsuleVerificationDecision::RejectedUnsigned;
    }
    let Some(trusted) = facts
        .trusted_signer_keys
        .iter()
        .find(|candidate| candidate.key_id == manifest.signature.signer_key_id)
    else {
        return CapsuleVerificationDecision::RejectedUnknownSigner;
    };
    let Ok(verifying_key_bytes) = BASE64.decode(&trusted.verifying_key_b64) else {
        return CapsuleVerificationDecision::RejectedInvalidSignature;
    };
    let Ok(verifying_key_bytes): Result<[u8; 32], _> = verifying_key_bytes.try_into() else {
        return CapsuleVerificationDecision::RejectedInvalidSignature;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&verifying_key_bytes) else {
        return CapsuleVerificationDecision::RejectedInvalidSignature;
    };
    let Ok(canonical) = canonical_capsule_manifest_bytes(manifest) else {
        return CapsuleVerificationDecision::RejectedInvalidSignature;
    };
    let Ok(signature) = Signature::from_slice(&signature_bytes) else {
        return CapsuleVerificationDecision::RejectedInvalidSignature;
    };
    if verifying_key.verify(&canonical, &signature).is_err() {
        return CapsuleVerificationDecision::RejectedInvalidSignature;
    }

    if !REGISTERED_CAPSULE_PLATFORMS.contains(&manifest.platform.as_str()) {
        return CapsuleVerificationDecision::RejectedUnknownPlatform;
    }
    if !REGISTERED_CAPSULE_ARCHES.contains(&manifest.arch.as_str()) {
        return CapsuleVerificationDecision::RejectedUnknownArch;
    }
    if !REGISTERED_CAPSULE_CHANNELS.contains(&manifest.channel.as_str()) {
        return CapsuleVerificationDecision::RejectedUnknownChannel;
    }
    if !REGISTERED_CAPSULE_RELEASE_STATUSES.contains(&manifest.release_status.as_str()) {
        return CapsuleVerificationDecision::RejectedUnknownReleaseStatus;
    }
    if !super::dynamic_operation_manifest::REGISTERED_PRODUCT_OWNERS
        .contains(&manifest.product.as_str())
    {
        return CapsuleVerificationDecision::RejectedUnknownProduct;
    }
    if manifest
        .required_features
        .iter()
        .any(|feature| !facts.registered_feature_keys.contains(feature))
    {
        return CapsuleVerificationDecision::RejectedUnknownFeature;
    }
    if !facts
        .known_limit_policy_versions
        .contains(&manifest.limit_policy_version)
    {
        return CapsuleVerificationDecision::RejectedUnknownLimitPolicy;
    }

    if manifest.key_envelope.schema != NODE_KEY_ENVELOPE_SCHEMA
        || manifest.key_envelope.envelope_ref.is_empty()
        || !is_sha256_hex(&manifest.key_envelope.envelope_digest_sha256)
    {
        return CapsuleVerificationDecision::RejectedMissingKeyEnvelope;
    }
    if !is_sha256_hex(&manifest.digests.plaintext.value)
        || !is_sha256_hex(&manifest.digests.ciphertext.value)
    {
        return CapsuleVerificationDecision::RejectedModified;
    }

    if manifest.revocation.revoked || facts.revoked_capsule_ids.contains(&manifest.capsule_id) {
        return CapsuleVerificationDecision::RejectedRevoked;
    }
    if manifest.release_status == "withdrawn" {
        return CapsuleVerificationDecision::RejectedWithdrawn;
    }

    if let Some(delivered) = &facts.delivered_plaintext_sha256 {
        if delivered != &manifest.digests.plaintext.value {
            return CapsuleVerificationDecision::RejectedModified;
        }
    }
    if let Some(delivered) = &facts.delivered_ciphertext_sha256 {
        if delivered != &manifest.digests.ciphertext.value {
            return CapsuleVerificationDecision::RejectedModified;
        }
    }

    if let Some(installed) = facts
        .installed_namespace_versions
        .get(&manifest.feature_namespace)
    {
        if installed != &manifest.capsule_version {
            return CapsuleVerificationDecision::RejectedMixedSet;
        }
    }
    if let Some(latest) = facts
        .known_latest_namespace_versions
        .get(&manifest.feature_namespace)
    {
        let (Some(latest_version), Some(this_version)) = (
            parse_version(latest),
            parse_version(&manifest.capsule_version),
        ) else {
            return CapsuleVerificationDecision::RejectedUnknownManifest;
        };
        if this_version < latest_version {
            return CapsuleVerificationDecision::RejectedDowngraded;
        }
    }

    let runtime = &facts.runtime_shell_contract;
    let required = &manifest.public_shell_contract;
    if runtime.id != required.id
        || runtime.major != required.major
        || runtime.minor < required.minor
    {
        return CapsuleVerificationDecision::RejectedIncompatibleShell;
    }

    CapsuleVerificationDecision::Verified
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    const TEST_SIGNER_KEY_ID: &str = "test-capsule-signer-2026-01";

    fn test_signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn sample_manifest() -> CapsuleManifest {
        let mut manifest = CapsuleManifest {
            schema: CAPSULE_MANIFEST_SCHEMA.to_string(),
            manifest_version: 1,
            capsule_id: "capsule_0UNITTESTCAPSULE0000000000".to_string(),
            capsule_version: "1.2.3".to_string(),
            product: "focusa".to_string(),
            feature_namespace: "focusa.worker.premium".to_string(),
            platform: "linux".to_string(),
            arch: "x86_64".to_string(),
            digests: CapsuleDigests {
                plaintext: CapsuleDigest {
                    algorithm: "sha256".to_string(),
                    value: "1111111111111111111111111111111111111111111111111111111111111111"
                        .to_string(),
                },
                ciphertext: CapsuleDigest {
                    algorithm: "sha256".to_string(),
                    value: "2222222222222222222222222222222222222222222222222222222222222222"
                        .to_string(),
                },
            },
            public_shell_contract: PublicShellContract {
                id: "focusa.public_shell".to_string(),
                major: 1,
                minor: 4,
            },
            required_features: vec!["focusa.worker.premium".to_string()],
            limit_policy_version: KNOWN_LIMIT_POLICY_VERSION.to_string(),
            channel: "stable".to_string(),
            release_status: "released".to_string(),
            revocation: CapsuleRevocation {
                revoked: false,
                revocation_id: None,
                revoked_at: None,
                reason: None,
            },
            provenance: CapsuleProvenance {
                builder_id: "focusa-signing-service".to_string(),
                build_id: "release-2026-08-01-01".to_string(),
                source_commit: "abc123def456abc123def456abc123def456abcd".to_string(),
                signed_at: "2026-08-01T00:00:00Z".to_string(),
            },
            key_envelope: KeyEnvelopeRef {
                schema: NODE_KEY_ENVELOPE_SCHEMA.to_string(),
                envelope_ref: "envelope_0UNITTESTENVELOPE0000000".to_string(),
                envelope_digest_sha256:
                    "3333333333333333333333333333333333333333333333333333333333333333".to_string(),
            },
            signature: CapsuleSignature {
                signature_algorithm: CAPSULE_SIGNATURE_ALGORITHM.to_string(),
                signer_key_id: TEST_SIGNER_KEY_ID.to_string(),
                signature_b64: String::new(),
            },
        };
        let canonical = canonical_capsule_manifest_bytes(&manifest).expect("canonical body");
        let signature = test_signing_key().sign(&canonical);
        manifest.signature.signature_b64 = BASE64.encode(signature.to_bytes());
        manifest
    }

    /// Re-sign the current body bytes with the trusted test key. Used when a
    /// scenario mutates the body after the initial signature (the authority
    /// would sign the exact mutated body in a real downgrade replay).
    fn resign(manifest: &CapsuleManifest) -> CapsuleManifest {
        let mut signed = manifest.clone();
        let canonical = canonical_capsule_manifest_bytes(&signed).expect("canonical body");
        let signature = test_signing_key().sign(&canonical);
        signed.signature.signature_b64 = BASE64.encode(signature.to_bytes());
        signed
    }

    fn trusted_facts() -> CapsuleVerificationFacts {
        let signing_key = test_signing_key();
        let verifying_key = signing_key.verifying_key();
        CapsuleVerificationFacts {
            runtime_shell_contract: PublicShellContract {
                id: "focusa.public_shell".to_string(),
                major: 1,
                minor: 4,
            },
            installed_namespace_versions: BTreeMap::from([(
                "focusa.worker.premium".to_string(),
                "1.2.3".to_string(),
            )]),
            known_latest_namespace_versions: BTreeMap::from([(
                "focusa.worker.premium".to_string(),
                "1.2.3".to_string(),
            )]),
            registered_feature_keys: vec!["focusa.worker.premium".to_string()],
            known_limit_policy_versions: vec![KNOWN_LIMIT_POLICY_VERSION.to_string()],
            trusted_signer_keys: vec![TrustedSignerKey {
                key_id: TEST_SIGNER_KEY_ID.to_string(),
                verifying_key_b64: BASE64.encode(verifying_key.to_bytes()),
            }],
            revoked_capsule_ids: Vec::new(),
            ..CapsuleVerificationFacts::default()
        }
    }

    #[test]
    fn spec152_capsule_manifest_decision_labels_are_stable_and_unique() {
        let variants = [
            CapsuleVerificationDecision::Verified,
            CapsuleVerificationDecision::RejectedUnsigned,
            CapsuleVerificationDecision::RejectedUnknownSigner,
            CapsuleVerificationDecision::RejectedInvalidSignature,
            CapsuleVerificationDecision::RejectedUnknownManifest,
            CapsuleVerificationDecision::RejectedUnknownProduct,
            CapsuleVerificationDecision::RejectedUnknownPlatform,
            CapsuleVerificationDecision::RejectedUnknownArch,
            CapsuleVerificationDecision::RejectedUnknownChannel,
            CapsuleVerificationDecision::RejectedUnknownReleaseStatus,
            CapsuleVerificationDecision::RejectedUnknownFeature,
            CapsuleVerificationDecision::RejectedUnknownLimitPolicy,
            CapsuleVerificationDecision::RejectedMissingKeyEnvelope,
            CapsuleVerificationDecision::RejectedModified,
            CapsuleVerificationDecision::RejectedMixedSet,
            CapsuleVerificationDecision::RejectedDowngraded,
            CapsuleVerificationDecision::RejectedIncompatibleShell,
            CapsuleVerificationDecision::RejectedRevoked,
            CapsuleVerificationDecision::RejectedWithdrawn,
        ];
        let mut seen = std::collections::HashSet::new();
        for variant in variants {
            let label = variant.label();
            assert!(!label.is_empty(), "empty label for {variant:?}");
            assert!(seen.insert(label), "duplicate label {label}");
        }
        assert!(CapsuleVerificationDecision::Verified.is_verified());
        assert!(!CapsuleVerificationDecision::Verified.is_rejected());
        for variant in variants[1..].iter() {
            assert!(variant.is_rejected());
            assert_eq!(
                variant.stable_error(),
                super::super::dynamic_operation_manifest::ENTITLEMENT_POLICY_UNKNOWN
            );
        }
        assert_eq!(CapsuleVerificationDecision::Verified.stable_error(), "");
    }

    #[test]
    fn spec152_capsule_manifest_registered_vocabularies_are_exact() {
        assert_eq!(REGISTERED_CAPSULE_PLATFORMS, ["linux", "macos", "windows"]);
        assert_eq!(REGISTERED_CAPSULE_ARCHES, ["x86_64", "aarch64"]);
        assert_eq!(
            REGISTERED_CAPSULE_CHANNELS,
            ["stable", "preview", "beta", "internal"]
        );
        assert_eq!(
            REGISTERED_CAPSULE_RELEASE_STATUSES,
            ["released", "deprecated", "withdrawn"]
        );
        assert_eq!(NODE_KEY_ENVELOPE_SCHEMA, "focusa.node_capsule_key_envelope.v1");
        assert_eq!(KNOWN_LIMIT_POLICY_VERSION, "focusa.limit_policy.v1");
        assert_eq!(CAPSULE_MANIFEST_SCHEMA, "focusa.capsule_manifest.v1");
        assert_eq!(CAPSULE_SIGNATURE_ALGORITHM, "ed25519");
    }

    #[test]
    fn spec152_capsule_manifest_canonicalization_is_deterministic() {
        let manifest = sample_manifest();
        let first = capsule_manifest_sha256(&manifest).expect("digest");
        let second = capsule_manifest_sha256(&manifest).expect("digest");
        assert_eq!(first, second);

        // Key order is irrelevant: re-serialize through JSON and re-parse.
        let json = serde_json::to_string(&manifest).unwrap();
        let reordered: CapsuleManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(capsule_manifest_sha256(&reordered).unwrap(), first);

        // The signature envelope is excluded from the canonical body.
        let canonical = canonical_capsule_manifest_bytes(&manifest).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&canonical).unwrap();
        assert!(parsed.get("signature").is_none());
        assert_eq!(parsed["schema"], serde_json::json!("focusa.capsule_manifest.v1"));
    }

    #[test]
    fn spec152_capsule_manifest_signed_manifest_verifies_and_fails_closed() {
        let manifest = sample_manifest();
        let facts = trusted_facts();
        assert_eq!(
            verify_capsule_manifest(&manifest, &facts),
            CapsuleVerificationDecision::Verified
        );

        // Unsigned (empty signature) fails closed.
        let mut unsigned = manifest.clone();
        unsigned.signature.signature_b64 = String::new();
        assert_eq!(
            verify_capsule_manifest(&unsigned, &facts),
            CapsuleVerificationDecision::RejectedUnsigned
        );

        // Unknown signer key fails closed.
        let mut unknown_signer = manifest.clone();
        unknown_signer.signature.signer_key_id = "forged-signer-1999".to_string();
        assert_eq!(
            verify_capsule_manifest(&unknown_signer, &facts),
            CapsuleVerificationDecision::RejectedUnknownSigner
        );

        // Modified body after signing fails closed (signature no longer verifies).
        let mut modified = manifest.clone();
        modified.platform = "macos".to_string();
        assert_eq!(
            verify_capsule_manifest(&modified, &facts),
            CapsuleVerificationDecision::RejectedInvalidSignature
        );

        // Unknown manifest version fails closed.
        let mut unknown_version = manifest.clone();
        unknown_version.manifest_version = 2;
        assert_eq!(
            verify_capsule_manifest(&unknown_version, &facts),
            CapsuleVerificationDecision::RejectedUnknownManifest
        );

        // Downgrade fails closed (re-signed old release replayed over a newer
        // known-latest release).
        let mut downgrade = manifest.clone();
        downgrade.capsule_version = "1.2.2".to_string();
        downgrade = resign(&downgrade);
        let mut downgrade_facts = facts.clone();
        downgrade_facts
            .known_latest_namespace_versions
            .insert("focusa.worker.premium".to_string(), "1.2.4".to_string());
        downgrade_facts
            .installed_namespace_versions
            .insert("focusa.worker.premium".to_string(), "1.2.2".to_string());
        assert_eq!(
            verify_capsule_manifest(&downgrade, &downgrade_facts),
            CapsuleVerificationDecision::RejectedDowngraded
        );

        // Incompatible shell fails closed.
        let mut shell_facts = facts.clone();
        shell_facts.runtime_shell_contract = PublicShellContract {
            id: "focusa.public_shell".to_string(),
            major: 1,
            minor: 3,
        };
        assert_eq!(
            verify_capsule_manifest(&manifest, &shell_facts),
            CapsuleVerificationDecision::RejectedIncompatibleShell
        );

        // Revoked by authority facts fails closed.
        let mut revoked_facts = facts.clone();
        revoked_facts
            .revoked_capsule_ids
            .push(manifest.capsule_id.clone());
        assert_eq!(
            verify_capsule_manifest(&manifest, &revoked_facts),
            CapsuleVerificationDecision::RejectedRevoked
        );
    }
}
