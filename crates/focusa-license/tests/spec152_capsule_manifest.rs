//! Spec 152 / 152A — signed capsule manifest contract vectors.
//!
//! Every synthetic vector in `docs/contracts/spec152-capsule-manifest-vectors.v1.json`
//! is replayed through the public verifier: valid manifests verify; modified,
//! mixed, downgraded, unknown, withdrawn, revoked, unsigned, and
//! shell-incompatible manifests fail closed with their exact stable label.
//! Deterministic canonicalization is proven by re-deriving every stored digest.

use std::{collections::HashSet, fs, path::PathBuf};

use focusa_license::{
    canonical_capsule_manifest_bytes, capsule_manifest_sha256, verify_capsule_manifest,
    CapsuleManifest, CapsuleVerificationDecision, CapsuleVerificationFacts,
};

fn vectors_path() -> PathBuf {
    [
        env!("CARGO_MANIFEST_DIR"),
        "../../docs/contracts/spec152-capsule-manifest-vectors.v1.json",
    ]
    .iter()
    .collect()
}

fn load_vectors() -> serde_json::Value {
    let payload = fs::read_to_string(vectors_path()).expect("capsule vectors file should exist");
    serde_json::from_str(&payload).expect("capsule vectors file should be valid JSON")
}

#[test]
fn spec152_capsule_manifest_vectors_fail_closed_exactly() {
    let vectors = load_vectors();
    let expected = vectors["canonicalization"]["algorithm"].as_str().unwrap();
    assert_eq!(expected, "sha256");

    let mut decisions = Vec::new();
    for group in ["valid_manifests", "invalid_manifests"] {
        for vector in vectors[group].as_array().expect("vector group") {
            let name = vector["name"].as_str().expect("vector name");
            let manifest: CapsuleManifest = serde_json::from_value(vector["manifest"].clone())
                .unwrap_or_else(|error| panic!("{name}: manifest deserialize: {error}"));
            let facts: CapsuleVerificationFacts = serde_json::from_value(vector["facts"].clone())
                .unwrap_or_else(|error| panic!("{name}: facts deserialize: {error}"));

            let digest = capsule_manifest_sha256(&manifest)
                .unwrap_or_else(|error| panic!("{name}: canonical digest: {error}"));
            let expected_digest = vector["canonical_digest_sha256"].as_str().unwrap();
            assert_eq!(digest, expected_digest, "{name}: canonical digest mismatch");

            let decision = verify_capsule_manifest(&manifest, &facts);
            let expected_decision = vector["expected_decision"].as_str().unwrap();
            assert_eq!(
                decision.label(),
                expected_decision,
                "{name}: verifier decision mismatch"
            );
            decisions.push((name.to_string(), decision));
        }
    }

    // Every stored vector name is unique.
    let names: Vec<&str> = decisions.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names.len(), HashSet::<&str>::from_iter(names.iter().copied()).len());

    // All named positive checks verify; every negative check is rejected.
    let verified = decisions
        .iter()
        .filter(|(_, decision)| decision.is_verified())
        .count();
    let rejected = decisions
        .iter()
        .filter(|(_, decision)| decision.is_rejected())
        .count();
    assert_eq!(verified, vectors["valid_manifests"].as_array().unwrap().len());
    assert_eq!(
        rejected,
        vectors["invalid_manifests"].as_array().unwrap().len()
    );
    assert!(verified >= 4);
    assert!(rejected >= 20);
}

#[test]
fn spec152_capsule_manifest_vectors_signature_binds_canonical_bytes() {
    let vectors = load_vectors();
    let mut checked = 0;
    for group in ["valid_manifests", "invalid_manifests"] {
        for vector in vectors[group].as_array().expect("vector group") {
            let name = vector["name"].as_str().unwrap();
            let manifest: CapsuleManifest = serde_json::from_value(vector["manifest"].clone())
                .unwrap_or_else(|error| panic!("{name}: {error}"));
            let canonical =
                canonical_capsule_manifest_bytes(&manifest).expect("canonical body bytes");
            // The signature envelope must never be part of the signed bytes.
            let parsed: serde_json::Value = serde_json::from_slice(&canonical).unwrap();
            assert!(parsed.get("signature").is_none(), "{name}");
            assert_eq!(
                manifest.digests.plaintext.algorithm, "sha256",
                "{name}: plaintext digest algorithm"
            );
            assert_eq!(
                manifest.digests.ciphertext.algorithm, "sha256",
                "{name}: ciphertext digest algorithm"
            );
            assert_eq!(
                manifest.signature.signature_algorithm, "ed25519",
                "{name}: signature algorithm"
            );
            if vector["expected_decision"] != "rejected_missing_key_envelope" {
                assert_eq!(
                    manifest.key_envelope.schema, "focusa.node_capsule_key_envelope.v1",
                    "{name}: key envelope schema"
                );
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 26);
}

#[test]
fn spec152_capsule_manifest_vectors_cover_all_stable_rejections() {
    let vectors = load_vectors();
    let labels: HashSet<&str> = vectors["invalid_manifests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|vector| vector["expected_decision"].as_str().unwrap())
        .collect();
    for required in [
        "rejected_unknown_manifest",
        "rejected_unknown_product",
        "rejected_unknown_platform",
        "rejected_unknown_arch",
        "rejected_unknown_channel",
        "rejected_unknown_release_status",
        "rejected_unknown_feature",
        "rejected_unknown_limit_policy",
        "rejected_unknown_signer",
        "rejected_invalid_signature",
        "rejected_modified",
        "rejected_mixed_set",
        "rejected_downgraded",
        "rejected_incompatible_shell",
        "rejected_revoked",
        "rejected_withdrawn",
        "rejected_missing_key_envelope",
    ] {
        assert!(labels.contains(required), "missing rejection vector: {required}");
    }
}
