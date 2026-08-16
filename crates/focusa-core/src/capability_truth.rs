//! Capability Truth — slice 1 (#279): typed claims and the public-safe
//! Honesty Manifest. A capability claim is verified only when backed by a
//! typed evidence ref on the claimed surface; unverified claims are
//! reported, never silently treated as true. The manifest is
//! public-safe: internal paths/credentials never leak into it.

use serde::{Deserialize, Serialize};

pub const CAPABILITY_TRUTH_SCHEMA: &str = "focusa.capability_truth.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityTruthClaim {
    pub schema: String,
    pub capability: String,
    pub surface: String,
    pub claimed: bool,
    pub verified: bool,
    pub evidence_ref: Option<String>,
    pub verified_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HonestyManifest {
    pub schema: String,
    pub total_claims: usize,
    pub verified_claims: usize,
    pub unverified_claims: Vec<String>,
    pub public_safe: bool,
    pub redacted_evidence: Vec<String>,
}

/// Build the public-safe honesty manifest: verified claims count, every
/// unverified claim is named, and evidence refs that contain internal
/// paths are redacted (never published).
pub fn honesty_manifest(claims: &[CapabilityTruthClaim]) -> HonestyManifest {
    let total = claims.len();
    let verified = claims.iter().filter(|c| c.verified).count();
    let unverified: Vec<String> = claims
        .iter()
        .filter(|c| !c.verified)
        .map(|c| c.capability.clone())
        .collect();
    let mut redacted = Vec::new();
    let mut public_safe = true;
    for claim in claims {
        if let Some(reference) = &claim.evidence_ref {
            if looks_internal(reference) {
                redacted.push(format!(
                    "capability {}: evidence redacted (internal path)",
                    claim.capability
                ));
                public_safe = false;
            }
        }
    }
    HonestyManifest {
        schema: "focusa.honesty_manifest.v1".to_string(),
        total_claims: total,
        verified_claims: verified,
        unverified_claims: unverified,
        public_safe,
        redacted_evidence: redacted,
    }
}

/// A claim with `claimed=true` but `verified=false` and no evidence is an
/// unverifiable capability claim — never enforceable as truth.
pub fn enforceable(claim: &CapabilityTruthClaim) -> bool {
    claim.claimed && claim.verified && claim.evidence_ref.is_some()
}

fn looks_internal(reference: &str) -> bool {
    reference.contains("/home/")
        || reference.contains("/root/")
        || reference.contains("/etc/")
        || reference.contains("passwd")
        || reference.contains("token")
        || reference.contains("secret")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim(capability: &str, verified: bool, evidence: Option<&str>) -> CapabilityTruthClaim {
        CapabilityTruthClaim {
            schema: CAPABILITY_TRUTH_SCHEMA.to_string(),
            capability: capability.to_string(),
            surface: "pi".to_string(),
            claimed: true,
            verified,
            evidence_ref: evidence.map(|e| e.to_string()),
            verified_at: verified.then(|| "2026-08-16T00:00:00Z".to_string()),
        }
    }

    #[test]
    fn manifest_counts_verified_and_names_unverified() {
        let claims = vec![
            claim("focusa_bg", true, Some("docs/165-background-execution-and-completion-notification.md")),
            claim("focusa_callgraph_export", true, Some("docs/155-focusa-callgraph-workflow-and-flow-mesh-execution-integration-spec.md")),
            claim("focusa_video_render", false, None),
        ];
        let manifest = honesty_manifest(&claims);
        assert_eq!(manifest.total_claims, 3);
        assert_eq!(manifest.verified_claims, 2);
        assert_eq!(manifest.unverified_claims, vec!["focusa_video_render".to_string()]);
        assert!(manifest.public_safe);
    }

    #[test]
    fn internal_paths_are_redacted_from_public_manifests() {
        let claims = vec![claim(
            "focusa_secret_broker",
            true,
            Some("/home/wirebot/.secrets/broker-token.json"),
        )];
        let manifest = honesty_manifest(&claims);
        assert!(!manifest.public_safe);
        assert_eq!(manifest.redacted_evidence.len(), 1);
        assert!(manifest.redacted_evidence[0].contains("redacted"));
    }

    #[test]
    fn unverified_claims_are_not_enforceable() {
        assert!(enforceable(&claim("a", true, Some("docs/a.md"))));
        assert!(!enforceable(&claim("a", false, None)));
        assert!(!enforceable(&claim("a", true, None)));
    }

    #[test]
    fn manifest_roundtrips() {
        let claims = vec![claim("a", true, Some("docs/a.md"))];
        let manifest = honesty_manifest(&claims);
        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: HonestyManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, manifest);
    }
}
