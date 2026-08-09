//! `GET /v1/license/status` \u2014 LicenseGuard plane surface (bead focusa-nbai.1).
//!
//! Returns current tier, capability posture, key fingerprint, and expiry.

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use focusa_license::{
    ActivationRegistration, Capability, CapabilityCheck, AgentActivationEnvelope,
    authority::EntitlementState,
};
use serde::Serialize;
use std::sync::Arc;

pub fn router() -> Router<Arc<crate::server::AppState>> {
    Router::new()
        .route("/v1/license/status", get(license_status))
        .route("/v1/activation/status", get(activation_status))
}

#[derive(Debug, Serialize)]
struct CapabilityPosture {
    capability: String,
    outcome: &'static str,
    reason: Option<String>,
}

async fn license_status(
    State(state): State<Arc<crate::server::AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let g = state.license_guard.clone();
    let authority =
        focusa_license::entitlement_projection(g.entitlement.as_ref()).map_err(|error| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "ENTITLEMENT_SNAPSHOT_MISSING",
                    "message": error.to_string(),
                    "recovery_policy": "recovery, export, repair, and uninstall remain available",
                })),
            )
        })?;
    let entitlement_decision = focusa_license::entitlement_decision_projection(g.entitlement.as_ref())
        .map_err(|error| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "ENTITLEMENT_SNAPSHOT_MISSING",
                    "message": error.to_string(),
                    "recovery_policy": "recovery, export, repair, and uninstall remain available",
                })),
            )
        })?;
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

    Ok(Json(serde_json::json!({
        "status": status,
        "tier": g.tier.label(),
        "issued_at": g.issued_at.to_rfc3339(),
        "expires_at": g.expires_at.map(|d| d.to_rfc3339()),
        "bsl_change_date": g.bsl_change_date.to_rfc3339(),
        "masked_identity": g.customer_email.as_deref().and_then(mask_identity),
        "expired": g.is_expired(),
        "authority": authority,
        "entitlement_decision": entitlement_decision,
        "capabilities": posture,
        "summary": format!(
            "tier={} capabilities={}",
            g.tier.label(),
            posture.iter().filter(|p| p.outcome == "denied").count()
        ),
        "next_action": next_action,
    })))
}

fn mask_identity(value: &str) -> Option<String> {
    let (local, domain) = value.trim().split_once('@')?;
    if local.is_empty() || domain.is_empty() {
        return None;
    }
    let first = local.chars().next()?;
    Some(format!("{first}***@{domain}"))
}

/// Read presenter-safe registration snapshots from the activation directory.
/// Only `focusa.activation_registration.v1` snapshots are accepted; unknown
/// schemas and malformed files fail closed (skipped, never granted).
fn read_registration_snapshots(directory: &std::path::Path) -> Vec<ActivationRegistration> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut paths: Vec<std::path::PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "json").unwrap_or(false))
        .collect();
    paths.sort();
    let mut registrations = Vec::new();
    for path in paths {
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(registration) = serde_json::from_str::<ActivationRegistration>(&raw) else {
            continue;
        };
        if registration.schema == "focusa.activation_registration.v1" {
            registrations.push(registration);
        }
    }
    registrations
}

/// `GET /v1/activation/status` — Spec 152E §14.2 agent/JSON activation and
/// resume protocol (daemon/API operation surface).
///
/// Returns the presenter-safe activation envelopes for every persisted
/// registration snapshot (the resumable registration handles) plus the
/// canonical entitlement posture. Poll credentials, raw emails, one-time key
/// envelopes, and signed leases are never present in snapshots by
/// construction and never appear here.
async fn activation_status(
    State(state): State<Arc<crate::server::AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let g = state.license_guard.clone();
    let authority =
        focusa_license::entitlement_projection(g.entitlement.as_ref()).map_err(|error| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "ENTITLEMENT_SNAPSHOT_MISSING",
                    "message": error.to_string(),
                    "recovery_policy": "recovery, export, repair, and uninstall remain available",
                })),
            )
        })?;

    // Read the persisted presenter-safe registration snapshots (never poll
    // credentials; the snapshot schema structurally excludes them).
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/root"));
    let registrations = read_registration_snapshots(&home.join(".config/focusa/activation"));

    let envelopes: Vec<AgentActivationEnvelope> = registrations
        .iter()
        .map(AgentActivationEnvelope::from_registration)
        .collect();

    Ok(Json(serde_json::json!({
        "schema": "focusa.agent_activation_status.v1",
        "entitlement": authority,
        "registrations": envelopes,
        "resumable_handles": envelopes
            .iter()
            .filter(|envelope| !envelope.terminal)
            .map(|envelope| envelope.registration_id.clone())
            .collect::<Vec<String>>(),
        "privacy": {
            "raw_email_present": false,
            "raw_key_present": false,
            "poll_credential_present": false,
            "card_data_present": false,
        },
    })))
}

#[cfg(test)]
mod tests {
    use super::{mask_identity, read_registration_snapshots};
    use focusa_license::ActivationRegistration;

    #[test]
    fn identity_is_masked_and_invalid_values_are_omitted() {
        assert_eq!(
            mask_identity("operator@example.com").as_deref(),
            Some("o***@example.com")
        );
        assert_eq!(mask_identity(""), None);
        assert_eq!(mask_identity("raw-token-without-domain"), None);
    }

    #[test]
    fn snapshot_scan_accepts_only_canonical_schemas_and_stays_deterministic() {
        let directory = std::env::temp_dir().join(format!(
            "focusa-api-activation-{}",
            uuid::Uuid::now_v7()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let snapshot = serde_json::json!({
            "schema": "focusa.activation_registration.v1",
            "registration_id": "registration-0002",
            "facade_id": "focusa-cli",
            "presenter": "cli",
            "install_channel": "source_build",
            "state": "checkout_pending",
            "masked_email": "c***@example.com",
            "poll_count": 3,
            "max_polls": 40,
        });
        std::fs::write(directory.join("registration-0002.json"), snapshot.to_string()).unwrap();
        let unrelated = serde_json::json!({"schema": "other.schema.v1", "value": 1});
        std::fs::write(directory.join("unrelated.json"), unrelated.to_string()).unwrap();
        std::fs::write(directory.join("malformed.json"), "{not json").unwrap();

        let registrations = read_registration_snapshots(&directory);
        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].registration_id, "registration-0002");
        assert_eq!(registrations[0].state.label(), "checkout_pending");
        // The scan is deterministic: run twice, same result.
        assert_eq!(
            read_registration_snapshots(&directory),
            registrations
        );
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn agent_envelope_projection_from_snapshot_is_private_and_typed() {
        let registration: ActivationRegistration = serde_json::from_value(serde_json::json!({
            "schema": "focusa.activation_registration.v1",
            "registration_id": "registration-0002",
            "facade_id": "focusa-cli",
            "presenter": "cli",
            "install_channel": "source_build",
            "state": "checkout_pending",
            "masked_email": "c***@example.com",
            "poll_count": 3,
            "max_polls": 40,
        }))
        .expect("snapshot parses");
        let envelope = focusa_license::AgentActivationEnvelope::from_registration(&registration);
        assert_eq!(envelope.schema, "focusa.agent_activation_envelope.v1");
        assert_eq!(envelope.state, "payment_pending");
        assert!(envelope.human_action_required);
        assert_eq!(envelope.human_action.as_deref(), Some("complete_payment_then_poll"));
        assert!(!envelope.key_visible);
        let body = serde_json::to_string(&envelope).unwrap();
        assert!(!body.contains("raw_email"));
        assert!(!body.contains("full_license_key"));
        assert!(!body.contains("poll_credential"));
        assert!(body.contains("c***@example.com"));
    }
}
