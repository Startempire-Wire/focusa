//! `GET /v1/license/status` \u2014 LicenseGuard plane surface (bead focusa-nbai.1).
//!
//! Returns current tier, capability posture, key fingerprint, and expiry.

use axum::{Json, Router, routing::get};
use focusa_license::{Capability, CapabilityCheck, LicenseGuard};
use serde::Serialize;
use std::sync::{Arc, OnceLock};

static GUARD: OnceLock<LicenseGuard> = OnceLock::new();

/// Initialize the daemon's LicenseGuard (called once at startup).
pub fn init_guard(guard: LicenseGuard) {
    let _ = GUARD.set(guard);
}

fn current_guard() -> LicenseGuard {
    GUARD
        .get()
        .cloned()
        .unwrap_or_else(|| LicenseGuard::eval(7))
}

pub fn router() -> Router<Arc<crate::server::AppState>> {
    Router::new().route("/v1/license/status", get(license_status))
}

#[derive(Debug, Serialize)]
struct CapabilityPosture {
    capability: String,
    outcome: &'static str,
    reason: Option<String>,
}

async fn license_status() -> Json<serde_json::Value> {
    let g = current_guard();
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

    Json(serde_json::json!({
        "status": if g.is_expired() { "expired" } else { "ok" },
        "tier": g.tier.label(),
        "issued_at": g.issued_at.to_rfc3339(),
        "expires_at": g.expires_at.map(|d| d.to_rfc3339()),
        "bsl_change_date": g.bsl_change_date.to_rfc3339(),
        "customer_email": g.customer_email,
        "key_hash": g.key_hash,
        "expired": g.is_expired(),
        "capabilities": posture,
        "summary": format!(
            "tier={} capabilities={}",
            g.tier.label(),
            posture.iter().filter(|p| p.outcome == "denied").count()
        ),
        "next_action": if g.is_expired() {
            "renew or purchase commercial license"
        } else {
            "license plane ready"
        },
    }))
}
