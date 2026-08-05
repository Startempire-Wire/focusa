//! `GET /v1/license/status` \u2014 LicenseGuard plane surface (bead focusa-nbai.1).
//!
//! Returns current tier, capability posture, key fingerprint, and expiry.

use axum::{Json, Router, extract::State, routing::get};
use focusa_license::{Capability, CapabilityCheck, authority::EntitlementState};
use serde::Serialize;
use std::sync::Arc;

pub fn router() -> Router<Arc<crate::server::AppState>> {
    Router::new().route("/v1/license/status", get(license_status))
}

#[derive(Debug, Serialize)]
struct CapabilityPosture {
    capability: String,
    outcome: &'static str,
    reason: Option<String>,
}

async fn license_status(
    State(state): State<Arc<crate::server::AppState>>,
) -> Json<serde_json::Value> {
    let g = state.license_guard.clone();
    let caps = [
        Capability::CommercialUse,
        Capability::HostedMode,
        Capability::ProductEmbedding,
        Capability::TelemetrySend,
        Capability::LocalEval,
    ];
    let posture: Vec<CapabilityPosture> = caps
        .iter()
        .map(|c| {
            let check = g.check(*c);
            let (outcome, reason) = match &check {
                CapabilityCheck::Permitted => ("permitted", None),
                CapabilityCheck::PermittedWithWarning { warning } => {
                    ("permitted_with_warning", Some(warning.clone()))
                }
                CapabilityCheck::Denied { reason } => ("denied", Some(reason.clone())),
            };
            CapabilityPosture {
                capability: c.label().to_string(),
                outcome,
                reason,
            }
        })
        .collect();

    let authority_state = g.entitlement.as_ref().map(|snapshot| snapshot.state);
    let status = match authority_state {
        Some(EntitlementState::Unactivated) => "unactivated",
        Some(EntitlementState::RecoveryOnly) => "recovery_only",
        Some(EntitlementState::OfflineGrace) => "offline_grace",
        Some(EntitlementState::Active) => "active",
        None if g.is_expired() => "expired",
        None => "legacy_migration_only",
    };
    let next_action = match authority_state {
        Some(EntitlementState::Unactivated) => "begin authority device-code activation",
        Some(EntitlementState::RecoveryOnly) => "repair or refresh signed authority lease",
        Some(EntitlementState::OfflineGrace) => {
            "refresh signed authority lease before grace expiry"
        }
        Some(EntitlementState::Active) => "authority entitlement ready",
        None => "migrate legacy license through authority activation",
    };

    Json(serde_json::json!({
        "status": status,
        "tier": g.tier.label(),
        "issued_at": g.issued_at.to_rfc3339(),
        "expires_at": g.expires_at.map(|d| d.to_rfc3339()),
        "bsl_change_date": g.bsl_change_date.to_rfc3339(),
        "masked_identity": mask_identity(&g.customer_email),
        "expired": g.is_expired(),
        "authority": g.entitlement,
        "capabilities": posture,
        "summary": format!(
            "tier={} capabilities={}",
            g.tier.label(),
            posture.iter().filter(|p| p.outcome == "denied").count()
        ),
        "next_action": next_action,
    }))
}

fn mask_identity(value: &str) -> Option<String> {
    let (local, domain) = value.trim().split_once('@')?;
    if local.is_empty() || domain.is_empty() {
        return None;
    }
    let first = local.chars().next()?;
    Some(format!("{first}***@{domain}"))
}

#[cfg(test)]
mod tests {
    use super::mask_identity;

    #[test]
    fn identity_is_masked_and_invalid_values_are_omitted() {
        assert_eq!(
            mask_identity("operator@example.com").as_deref(),
            Some("o***@example.com")
        );
        assert_eq!(mask_identity(""), None);
        assert_eq!(mask_identity("raw-token-without-domain"), None);
    }
}
