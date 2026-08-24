//! Canonical contextual capability authorization (Workforce Full AC2).
//!
//! Every caller supplies server-resolved principal grants, one grounded
//! capability, and the exact execution context. `can` is pure and its returned
//! decision is the only record an audit store may persist.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CAPABILITY_AUTHORIZATION_SCHEMA: &str = "focusa.capability_authorization.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityEffect {
    Read,
    Write,
    Control,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityPrincipal {
    pub principal_id: String,
    pub source: String,
    pub authenticated: bool,
    /// Grants are resolved from daemon-owned token state, never request headers.
    pub grants: BTreeSet<String>,
    #[serde(default)]
    pub workstream_keys: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundedCapability {
    pub name: String,
    pub required_scope: String,
    pub effect: CapabilityEffect,
    pub assignable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityContext {
    pub request_id: String,
    pub workstream_key: Option<String>,
    pub workset_id: Option<String>,
    pub work_item_id: Option<String>,
    pub frame_id: Option<String>,
    pub risk: CapabilityRisk,
    /// True only after the daemon entitlement middleware accepted this request.
    pub entitlement_satisfied: bool,
    /// Legacy header values are requests, never grants. Any attempted elevation
    /// is denied and preserved in the decision audit record.
    #[serde(default)]
    pub requested_scopes: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityAuthorizationDecision {
    pub schema: String,
    pub decision_id: String,
    pub request_id: String,
    pub principal_id: String,
    pub capability: String,
    pub required_scope: String,
    pub allowed: bool,
    pub reason_code: String,
    pub context: CapabilityContext,
}

pub fn can(
    principal: &CapabilityPrincipal,
    capability: &GroundedCapability,
    context: &CapabilityContext,
) -> CapabilityAuthorizationDecision {
    let reason = if !capability.assignable {
        Err("CAPABILITY_UNAVAILABLE")
    } else if !principal.authenticated && !capability.required_scope.starts_with("public:") {
        Err("PRINCIPAL_UNAUTHENTICATED")
    } else if !context.entitlement_satisfied && capability.effect != CapabilityEffect::Read {
        Err("ENTITLEMENT_REQUIRED")
    } else if !requested_scopes_are_bounded(principal, capability, context) {
        Err("CLIENT_SCOPE_ELEVATION_DENIED")
    } else if !workstream_is_bounded(principal, context) {
        Err("WORKSTREAM_SCOPE_DENIED")
    } else if context.risk == CapabilityRisk::High
        && !principal.grants.contains("risk:high")
        && !principal.grants.contains("admin:*")
    {
        Err("HIGH_RISK_GRANT_REQUIRED")
    } else if grants_capability(principal, capability) {
        Ok("AUTHORIZED")
    } else {
        Err("CAPABILITY_SCOPE_DENIED")
    };

    let (allowed, reason_code) = match reason {
        Ok(code) => (true, code),
        Err(code) => (false, code),
    };
    let decision_id = decision_digest(principal, capability, context, allowed, reason_code);
    CapabilityAuthorizationDecision {
        schema: CAPABILITY_AUTHORIZATION_SCHEMA.into(),
        decision_id,
        request_id: context.request_id.clone(),
        principal_id: principal.principal_id.clone(),
        capability: capability.name.clone(),
        required_scope: capability.required_scope.clone(),
        allowed,
        reason_code: reason_code.into(),
        context: context.clone(),
    }
}

fn requested_scopes_are_bounded(
    principal: &CapabilityPrincipal,
    capability: &GroundedCapability,
    context: &CapabilityContext,
) -> bool {
    context.requested_scopes.iter().all(|requested| {
        if requested == "admin:*" {
            return principal.grants.contains("admin:*")
                && matches!(principal.source.as_str(), "admin_token" | "local_loopback");
        }
        if requested == "read:*" {
            return principal.grants.contains("read:*");
        }
        if requested == "write:*" {
            return principal.grants.contains("write:*");
        }
        if principal.grants.contains(requested) {
            return true;
        }
        if principal.grants.contains("admin:*")
            && matches!(principal.source.as_str(), "admin_token" | "local_loopback")
        {
            return true;
        }
        let read_alias = requested.ends_with(":read") || requested.ends_with(":stream");
        let write_alias = requested.ends_with(":write")
            || requested.ends_with(":create")
            || requested.ends_with(":control")
            || requested.ends_with(":config");
        (read_alias && principal.grants.contains("read:*"))
            || (write_alias && principal.grants.contains("write:*"))
    })
}

fn workstream_is_bounded(principal: &CapabilityPrincipal, context: &CapabilityContext) -> bool {
    if principal.workstream_keys.is_empty() {
        return true;
    }
    context
        .workstream_key
        .as_ref()
        .is_some_and(|key| principal.workstream_keys.contains(key))
}

fn grants_capability(principal: &CapabilityPrincipal, capability: &GroundedCapability) -> bool {
    capability.required_scope.starts_with("public:")
        || principal
            .grants
            .contains(&format!("capability:{}", capability.name))
        || grant_matches(
            principal,
            &capability.required_scope,
            capability.effect,
            CapabilityRisk::Low,
        )
}

fn grant_matches(
    principal: &CapabilityPrincipal,
    required: &str,
    effect: CapabilityEffect,
    risk: CapabilityRisk,
) -> bool {
    if principal.grants.contains(required) {
        return true;
    }
    if principal.grants.contains("admin:*") {
        return principal.source == "admin_token" || principal.source == "local_loopback";
    }
    if effect == CapabilityEffect::Read && principal.grants.contains("read:*") {
        return true;
    }
    if matches!(effect, CapabilityEffect::Write | CapabilityEffect::Control)
        && risk != CapabilityRisk::High
        && !required.ends_with(":admin")
        && !required.ends_with(":forensics")
        && principal.grants.contains("write:*")
    {
        return true;
    }
    false
}

fn decision_digest(
    principal: &CapabilityPrincipal,
    capability: &GroundedCapability,
    context: &CapabilityContext,
    allowed: bool,
    reason_code: &str,
) -> String {
    let bytes = serde_json::to_vec(&(
        CAPABILITY_AUTHORIZATION_SCHEMA,
        principal,
        capability,
        context,
        allowed,
        reason_code,
    ))
    .expect("serializing authorization inputs cannot fail");
    format!("can_{}", hex::encode(Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn principal(source: &str, grants: &[&str]) -> CapabilityPrincipal {
        CapabilityPrincipal {
            principal_id: "principal:test".into(),
            source: source.into(),
            authenticated: true,
            grants: grants.iter().map(|grant| (*grant).into()).collect(),
            workstream_keys: BTreeSet::new(),
        }
    }

    fn capability(scope: &str, effect: CapabilityEffect) -> GroundedCapability {
        GroundedCapability {
            name: "focusa_test".into(),
            required_scope: scope.into(),
            effect,
            assignable: true,
        }
    }

    fn context() -> CapabilityContext {
        CapabilityContext {
            request_id: "request:1".into(),
            workstream_key: None,
            workset_id: None,
            work_item_id: None,
            frame_id: None,
            risk: CapabilityRisk::Low,
            entitlement_satisfied: true,
            requested_scopes: BTreeSet::new(),
        }
    }

    #[test]
    fn authorization_matrix_is_fail_closed_and_contextual() {
        let cases = [
            (
                principal("paired_device", &["read:*"]),
                capability("state:read", CapabilityEffect::Read),
                true,
            ),
            (
                principal("paired_device", &["read:*"]),
                capability("state:write", CapabilityEffect::Write),
                false,
            ),
            (
                principal("paired_device", &["write:*"]),
                capability("state:write", CapabilityEffect::Write),
                true,
            ),
            (
                principal("paired_device", &["write:*"]),
                capability("sync:admin", CapabilityEffect::Admin),
                false,
            ),
            (
                principal("admin_token", &["admin:*"]),
                capability("sync:admin", CapabilityEffect::Admin),
                true,
            ),
            (
                principal("paired_device", &["admin:*"]),
                capability("sync:admin", CapabilityEffect::Admin),
                false,
            ),
        ];
        for (principal, capability, expected) in cases {
            assert_eq!(can(&principal, &capability, &context()).allowed, expected);
        }
    }

    #[test]
    fn client_asserted_admin_scope_never_elevates() {
        let mut request = context();
        request.requested_scopes.insert("admin:*".into());
        let decision = can(
            &principal("paired_device", &["read:*", "write:*"]),
            &capability("state:write", CapabilityEffect::Write),
            &request,
        );
        assert!(!decision.allowed);
        assert_eq!(decision.reason_code, "CLIENT_SCOPE_ELEVATION_DENIED");

        let mut bounded = context();
        bounded
            .requested_scopes
            .insert("silent_sessions:control".into());
        assert!(
            can(
                &principal("paired_device", &["write:*"]),
                &capability("silent_sessions:control", CapabilityEffect::Control),
                &bounded,
            )
            .allowed
        );
    }

    #[test]
    fn workstream_and_high_risk_context_are_bounded() {
        let mut scoped = principal("paired_device", &["write:*"]);
        scoped.workstream_keys.insert("workstream:a".into());
        let mut request = context();
        request.workstream_key = Some("workstream:b".into());
        assert_eq!(
            can(
                &scoped,
                &capability("state:write", CapabilityEffect::Write),
                &request
            )
            .reason_code,
            "WORKSTREAM_SCOPE_DENIED"
        );
        request.workstream_key = Some("workstream:a".into());
        request.risk = CapabilityRisk::High;
        assert_eq!(
            can(
                &scoped,
                &capability("state:write", CapabilityEffect::Write),
                &request
            )
            .reason_code,
            "HIGH_RISK_GRANT_REQUIRED"
        );
    }

    #[test]
    fn grounded_catalog_is_exhaustively_covered_by_one_gate() {
        let registry: serde_json::Value = serde_json::from_str(include_str!(
            "../../../docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json"
        ))
        .unwrap();
        let principal = principal("admin_token", &["admin:*", "risk:high"]);
        let descriptors = registry["descriptors"].as_array().unwrap();
        assert_eq!(descriptors.len(), 146);
        for descriptor in descriptors {
            let name = descriptor["tool_names"]["pi"].as_str().unwrap();
            let assignable = descriptor["availability"]["assignable"].as_bool().unwrap();
            let capability = GroundedCapability {
                name: name.into(),
                required_scope: "catalog:read".into(),
                effect: CapabilityEffect::Read,
                assignable,
            };
            let decision = can(&principal, &capability, &context());
            assert_eq!(decision.allowed, assignable, "{name}");
            if !assignable {
                assert_eq!(decision.reason_code, "CAPABILITY_UNAVAILABLE");
            }
        }
    }

    #[test]
    fn persisted_audit_is_the_exact_decision() {
        let root = std::env::temp_dir().join(format!("focusa-can-audit-{}", uuid::Uuid::now_v7()));
        let config = crate::types::FocusaConfig {
            data_dir: root.display().to_string(),
            ..crate::types::FocusaConfig::default()
        };
        let persistence =
            crate::runtime::persistence_sqlite::SqlitePersistence::new(&config).unwrap();
        let decision = can(
            &principal("admin_token", &["admin:*"]),
            &capability("admin:service", CapabilityEffect::Admin),
            &context(),
        );
        persistence
            .append_capability_authorization_audit(&decision)
            .unwrap();
        persistence
            .append_capability_authorization_audit(&decision)
            .unwrap();
        assert_eq!(
            persistence
                .load_capability_authorization_audit(&decision.decision_id)
                .unwrap(),
            Some(decision)
        );
        drop(persistence);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn same_inputs_produce_same_auditable_decision() {
        let principal = principal("admin_token", &["admin:*"]);
        let capability = capability("admin:service", CapabilityEffect::Admin);
        let request = context();
        assert_eq!(
            can(&principal, &capability, &request),
            can(&principal, &capability, &request)
        );
    }
}
