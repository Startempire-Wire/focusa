//! Proves the runtime verification path accepts the protected production
//! issuer: the compile-time-embedded trust root must verify the live issuer's
//! key-set envelope (sequence 8, protected key IDs, fixture roots retired).

use base64::Engine as _;
use focusa_license::authority::AuthorityLeaseVerifier;
use focusa_license::authority_store::embedded_production_trust_roots;

const LIVE_ENVELOPE: &str = include_str!("fixtures/live-protected-keyset-envelope.json");
const PROTECTED_ROOT_ID: &str = "authority-root-20260907-0055c689";
const PROTECTED_LEASE_ID: &str = "authority-lease-20260907-0055c689";

#[test]
fn embedded_trust_root_verifies_live_protected_key_set() {
    let roots = embedded_production_trust_roots()
        .expect("production trust roots must be embedded at compile time");
    assert!(
        roots.contains_key(PROTECTED_ROOT_ID),
        "protected production root must be embedded"
    );
    let envelope: focusa_license::authority::SignedEnvelope =
        serde_json::from_str(LIVE_ENVELOPE).expect("live key-set envelope parses");
    assert_eq!(envelope.signer_key_id, PROTECTED_ROOT_ID);
    let verifier = AuthorityLeaseVerifier::from_signed_key_set(
        &envelope,
        &roots,
        chrono::Utc::now(),
        Some(8),
    )
    .expect("live key-set envelope must verify against the embedded protected root");
    let _ = verifier; // verifier construction itself proves signature/root/sequence/expiry
    let payload: serde_json::Value = serde_json::from_slice(&base64::engine::general_purpose::STANDARD
        .decode(&envelope.payload_b64)
        .expect("payload base64 decodes"))
    .expect("key-set payload parses");
    assert_eq!(payload["sequence"], 8);
    let keys = payload["keys"].as_array().expect("keys array");
    assert!(keys.iter().any(|key| key["key_id"] == PROTECTED_LEASE_ID
        && key["status"] == "active"),
        "protected lease key must be active in the verified key set");
}
