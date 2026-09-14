//! Project-scoped Credential Authority — #299 slice 1 (docs/156).
//!
//! Typed, secret-free contracts: provider descriptors, credential roles,
//! requirements, autonomy policies, use grants, and secret bindings.
//! Invariant from the spec: model/public projections carry NO account
//! identifier and NO secret value — agents request ROLES, never vault
//! item names or raw values unless the operator deliberately permits it.

use serde::{Deserialize, Serialize};

pub const CREDENTIAL_PROVIDER_DESCRIPTOR_SCHEMA: &str = "focusa.credential_provider_descriptor.v1";
pub const CREDENTIAL_ROLE_SCHEMA: &str = "focusa.credential_role.v1";
pub const CREDENTIAL_REQUIREMENT_SCHEMA: &str = "focusa.credential_requirement.v1";
pub const CREDENTIAL_USE_GRANT_SCHEMA: &str = "focusa.credential_use_grant.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialProviderDescriptor {
    pub schema: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub adapter_version: String,
    pub custody_location_ref: String,
    pub supported_secret_classes: Vec<String>,
    pub supported_operations: Vec<String>,
    pub supported_auth_methods: Vec<String>,
    pub supports_machine_identity: bool,
    pub supports_dynamic_secrets: bool,
    pub supports_leases: bool,
    pub supports_rotation: bool,
    pub supports_revocation: bool,
    pub supports_audit: bool,
    pub supports_blind_use: bool,
    pub supports_process_injection: bool,
    pub supports_browser_injection: bool,
    pub availability: String,
    pub freshness: String,
    #[serde(default)]
    pub health_evidence_refs: Vec<String>,
    #[serde(default)]
    pub private_configuration_ref: Option<String>,
    pub content_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRole {
    pub schema: String,
    pub credential_role_id: String,
    pub human_label: String,
    pub project_scope_ref: String,
    pub purpose: String,
    pub secret_class: String,
    pub provider_binding_ref: String,
    #[serde(default)]
    pub allowed_target_refs: Vec<String>,
    #[serde(default)]
    pub allowed_origin_refs: Vec<String>,
    #[serde(default)]
    pub allowed_host_process_route_refs: Vec<String>,
    pub default_exposure_mode: String,
    #[serde(default)]
    pub rotation_policy_ref: Option<String>,
    pub owner_ref: String,
    pub status: String,
    pub metadata_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialOperation {
    Use,
    Reveal,
    Manage,
    Rotate,
    Revoke,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRequirement {
    pub schema: String,
    pub requirement_id: String,
    pub project_scope_ref: String,
    pub workstream_ref: String,
    pub callgraph_frame_ref: String,
    pub attempt_generation: u32,
    pub credential_role_ref: String,
    pub required_operation: CredentialOperation,
    pub required_exposure_mode: String,
    #[serde(default)]
    pub exact_target_refs: Vec<String>,
    pub exact_consumer_ref: String,
    #[serde(default)]
    pub required_auth_challenge_support: Vec<String>,
    #[serde(default)]
    pub precondition_refs: Vec<String>,
    pub validity_minimum_seconds: u64,
    pub use_count_required: u32,
    #[serde(default)]
    pub evidence_requirement_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialUseGrant {
    pub schema: String,
    pub grant_id: String,
    pub credential_role_ref: String,
    pub operation: CredentialOperation,
    pub exposure_mode: String,
    #[serde(default)]
    pub exact_target_refs: Vec<String>,
    pub consumer_ref: String,
    pub granted_at: String,
    pub expires_at: String,
    pub use_count_allowed: u32,
    pub use_count_used: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrantVerdict {
    pub satisfied: bool,
    pub reasons: Vec<String>,
}

/// Deterministic grant resolution: a requirement is satisfied only when a
/// grant matches role + operation + exposure + target + consumer, is
/// unexpired, and has remaining use count. No partial satisfaction.
pub fn verify_requirement(
    requirement: &CredentialRequirement,
    grants: &[CredentialUseGrant],
    now: &str,
) -> GrantVerdict {
    let mut reasons = Vec::new();
    let matching: Vec<&CredentialUseGrant> = grants
        .iter()
        .filter(|grant| {
            grant.credential_role_ref == requirement.credential_role_ref
                && grant.operation == requirement.required_operation
                && grant.exposure_mode == requirement.required_exposure_mode
                && grant.consumer_ref == requirement.exact_consumer_ref
                && requirement
                    .exact_target_refs
                    .iter()
                    .all(|target| grant.exact_target_refs.contains(target))
        })
        .collect();
    if matching.is_empty() {
        reasons.push("no grant matches the requirement".to_string());
        return GrantVerdict {
            satisfied: false,
            reasons,
        };
    }
    let usable: Vec<&&CredentialUseGrant> = matching
        .iter()
        .filter(|grant| grant.expires_at.as_str() > now)
        .collect();
    if usable.is_empty() {
        reasons.push("matching grants are expired".to_string());
    }
    let has_count = usable
        .iter()
        .any(|grant| grant.use_count_used < grant.use_count_allowed);
    if !has_count {
        reasons.push("no matching grant has remaining use count".to_string());
    }
    if requirement.use_count_required > 0 && usable.len() < requirement.use_count_required as usize
    {
        reasons.push(format!(
            "requirement needs {} grants, {} usable",
            requirement.use_count_required,
            usable.len()
        ));
    }
    GrantVerdict {
        satisfied: reasons.is_empty(),
        reasons,
    }
}

/// Redaction guard: a descriptor must not carry account identifiers or
/// secret values in model projections.
pub fn descriptor_is_redacted(descriptor: &CredentialProviderDescriptor) -> bool {
    let lower = format!(
        "{} {} {}",
        descriptor.provider_id, descriptor.adapter_version, descriptor.custody_location_ref
    )
    .to_lowercase();
    !(lower.contains("password")
        || lower.contains("secret=")
        || lower.contains("token=")
        || lower.contains("api_key")
        || lower.contains("@") && lower.contains(".com"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requirement(op: CredentialOperation, targets: &[&str]) -> CredentialRequirement {
        CredentialRequirement {
            schema: CREDENTIAL_REQUIREMENT_SCHEMA.to_string(),
            requirement_id: "req-1".to_string(),
            project_scope_ref: "/root/proj".to_string(),
            workstream_ref: "ws-1".to_string(),
            callgraph_frame_ref: "frame-1".to_string(),
            attempt_generation: 1,
            credential_role_ref: "role-dns".to_string(),
            required_operation: op,
            required_exposure_mode: "blind".to_string(),
            exact_target_refs: targets.iter().map(|t| t.to_string()).collect(),
            exact_consumer_ref: "consumer-1".to_string(),
            required_auth_challenge_support: vec![],
            precondition_refs: vec![],
            validity_minimum_seconds: 60,
            use_count_required: 1,
            evidence_requirement_refs: vec![],
        }
    }

    fn grant(expires: &str, used: u32, allowed: u32) -> CredentialUseGrant {
        CredentialUseGrant {
            schema: CREDENTIAL_USE_GRANT_SCHEMA.to_string(),
            grant_id: "grant-1".to_string(),
            credential_role_ref: "role-dns".to_string(),
            operation: CredentialOperation::Use,
            exposure_mode: "blind".to_string(),
            exact_target_refs: vec!["dns.example.com".to_string()],
            consumer_ref: "consumer-1".to_string(),
            granted_at: "2026-08-16T00:00:00Z".to_string(),
            expires_at: expires.to_string(),
            use_count_allowed: allowed,
            use_count_used: used,
        }
    }

    #[test]
    fn matching_unexpired_grant_satisfies() {
        let verdict = verify_requirement(
            &requirement(CredentialOperation::Use, &["dns.example.com"]),
            &[grant("2026-08-17T00:00:00Z", 0, 3)],
            "2026-08-16T12:00:00Z",
        );
        assert!(verdict.satisfied, "reasons: {:?}", verdict.reasons);
    }

    #[test]
    fn expired_or_exhausted_grants_fail_with_reasons() {
        let verdict = verify_requirement(
            &requirement(CredentialOperation::Use, &["dns.example.com"]),
            &[grant("2026-08-16T11:00:00Z", 0, 3)],
            "2026-08-16T12:00:00Z",
        );
        assert!(!verdict.satisfied);
        assert!(verdict.reasons.iter().any(|r| r.contains("expired")));
        let verdict = verify_requirement(
            &requirement(CredentialOperation::Use, &["dns.example.com"]),
            &[grant("2026-08-17T00:00:00Z", 3, 3)],
            "2026-08-16T12:00:00Z",
        );
        assert!(!verdict.satisfied);
        assert!(verdict.reasons.iter().any(|r| r.contains("use count")));
    }

    #[test]
    fn wrong_operation_or_target_fails() {
        let verdict = verify_requirement(
            &requirement(CredentialOperation::Reveal, &["dns.example.com"]),
            &[grant("2026-08-17T00:00:00Z", 0, 3)],
            "2026-08-16T12:00:00Z",
        );
        assert!(!verdict.satisfied);
        let verdict = verify_requirement(
            &requirement(CredentialOperation::Use, &["other.example.com"]),
            &[grant("2026-08-17T00:00:00Z", 0, 3)],
            "2026-08-16T12:00:00Z",
        );
        assert!(!verdict.satisfied);
    }

    #[test]
    fn redaction_guard_rejects_account_ids_and_secrets() {
        let clean = CredentialProviderDescriptor {
            schema: CREDENTIAL_PROVIDER_DESCRIPTOR_SCHEMA.to_string(),
            provider_id: "rbw-main".to_string(),
            provider_kind: "rbw".to_string(),
            adapter_version: "1.15.0".to_string(),
            custody_location_ref: "ref:opaque:1".to_string(),
            supported_secret_classes: vec![],
            supported_operations: vec![],
            supported_auth_methods: vec![],
            supports_machine_identity: false,
            supports_dynamic_secrets: false,
            supports_leases: false,
            supports_rotation: false,
            supports_revocation: false,
            supports_audit: true,
            supports_blind_use: true,
            supports_process_injection: true,
            supports_browser_injection: false,
            availability: "available".to_string(),
            freshness: "current".to_string(),
            health_evidence_refs: vec![],
            private_configuration_ref: None,
            content_digest: "sha256:abc".to_string(),
        };
        assert!(descriptor_is_redacted(&clean));
        let mut leaked = clean.clone();
        leaked.custody_location_ref = "ref:verious.smith@gmail.com".to_string();
        assert!(!descriptor_is_redacted(&leaked));
    }
}

/// Grant lifecycle (§docs/156): grants move through a bounded state
/// machine; expiry and exhaustion are deterministic transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantState {
    Issued,
    Active,
    Exhausted,
    Expired,
    Revoked,
}

pub fn grant_state(grant: &CredentialUseGrant, now: &str) -> GrantState {
    if grant.expires_at.as_str() <= now {
        return GrantState::Expired;
    }
    if grant.use_count_used >= grant.use_count_allowed {
        return GrantState::Exhausted;
    }
    GrantState::Active
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;

    fn grant(expires: &str, used: u32, allowed: u32) -> CredentialUseGrant {
        CredentialUseGrant {
            schema: CREDENTIAL_USE_GRANT_SCHEMA.to_string(),
            grant_id: "grant-1".to_string(),
            credential_role_ref: "role-dns".to_string(),
            operation: CredentialOperation::Use,
            exposure_mode: "blind".to_string(),
            exact_target_refs: vec!["dns.example.com".to_string()],
            consumer_ref: "consumer-1".to_string(),
            granted_at: "2026-08-16T00:00:00Z".to_string(),
            expires_at: expires.to_string(),
            use_count_allowed: allowed,
            use_count_used: used,
        }
    }

    #[test]
    fn grant_state_tracks_expiry_and_exhaustion() {
        let mut grant = grant("2026-08-17T00:00:00Z", 0, 3);
        assert_eq!(
            grant_state(&grant, "2026-08-16T12:00:00Z"),
            GrantState::Active
        );
        assert_eq!(
            grant_state(&grant, "2026-08-17T00:00:01Z"),
            GrantState::Expired
        );
        grant.use_count_used = 3;
        assert_eq!(
            grant_state(&grant, "2026-08-16T12:00:00Z"),
            GrantState::Exhausted
        );
    }
}
