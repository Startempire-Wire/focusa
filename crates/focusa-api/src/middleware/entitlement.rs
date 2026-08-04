use std::sync::Arc;

use axum::{
    Json,
    extract::{Request, State},
    http::Method,
    middleware::Next,
    response::{IntoResponse, Response},
};
use focusa_license::authority::EntitlementState;

use crate::server::AppState;

pub async fn entitlement_gate_layer(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    if !route_requires_entitlement(request.method(), request.uri().path())
        || entitlement_allows_mutation(&state.license_guard)
    {
        return next.run(request).await;
    }

    let authority_state = state
        .license_guard
        .entitlement
        .as_ref()
        .map(|snapshot| snapshot.state)
        .unwrap_or(EntitlementState::RecoveryOnly);
    let state_label = match authority_state {
        EntitlementState::Unactivated => "unactivated",
        EntitlementState::RecoveryOnly => "recovery_only",
        EntitlementState::Active => "active",
        EntitlementState::OfflineGrace => "offline_grace",
    };
    (
        axum::http::StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "status": "blocked",
            "error": {
                "code": "ENTITLEMENT_REQUIRED",
                "message": "A valid signed Focusa authority lease is required for this operation.",
                "state": state_label,
                "recovery": {
                    "status_path": "/v1/license/status",
                    "allowed": ["health", "version", "license_recovery", "safe_read"]
                }
            }
        })),
    )
        .into_response()
}

pub(crate) fn entitlement_allows_mutation(guard: &focusa_license::LicenseGuard) -> bool {
    let now = chrono::Utc::now();
    guard.entitlement.as_ref().is_some_and(|snapshot| {
        let bound = snapshot
            .lease_id
            .as_deref()
            .is_some_and(|value| !value.is_empty())
            && snapshot
                .lease_digest
                .as_deref()
                .is_some_and(|value| value.starts_with("sha256:"));
        bound
            && match snapshot.state {
                EntitlementState::Active => snapshot.expires_at.is_some_and(|expiry| expiry > now),
                EntitlementState::OfflineGrace => snapshot
                    .offline_grace_until
                    .is_some_and(|grace_until| grace_until > now),
                EntitlementState::Unactivated | EntitlementState::RecoveryOnly => false,
            }
    })
}

pub(crate) fn route_requires_entitlement(method: &Method, path: &str) -> bool {
    if matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS) {
        return false;
    }
    let recovery_path = path == "/health"
        || path == "/v1/health"
        || path == "/v1/version"
        || path.starts_with("/v1/license/")
        || path.starts_with("/v1/connect/")
        || path.starts_with("/v1/device/pair/");
    !recovery_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_license::{LicenseGuard, authority::EntitlementSnapshot};

    #[test]
    fn mutation_routes_require_entitlement_before_handlers() {
        for path in [
            "/v1/workpoint/checkpoint",
            "/v1/evidence/capture",
            "/v1/turn",
            "/v1/silent-sessions/start",
            "/v1/update/apply",
            "/v1/export/run",
        ] {
            assert!(route_requires_entitlement(&Method::POST, path), "{path}");
        }
        assert!(!route_requires_entitlement(
            &Method::GET,
            "/v1/workpoint/current"
        ));
        assert!(!route_requires_entitlement(
            &Method::POST,
            "/v1/license/refresh"
        ));
        assert!(!route_requires_entitlement(
            &Method::POST,
            "/v1/connect/room/create"
        ));
        assert!(!route_requires_entitlement(
            &Method::POST,
            "/v1/device/pair/start"
        ));
        assert!(!route_requires_entitlement(&Method::GET, "/v1/health"));
    }

    #[test]
    fn only_signed_active_or_grace_snapshot_allows_mutation() {
        assert!(!entitlement_allows_mutation(&LicenseGuard::eval(7)));
        let bind = |snapshot: &mut EntitlementSnapshot| {
            snapshot.lease_id = Some("lease-1".into());
            snapshot.lease_digest = Some("sha256:lease".into());
        };
        let mut active = EntitlementSnapshot::unactivated("focusa", "node");
        active.state = EntitlementState::Active;
        active.expires_at = Some(chrono::Utc::now() + chrono::Duration::minutes(5));
        bind(&mut active);
        assert!(entitlement_allows_mutation(
            &LicenseGuard::from_entitlement(active.clone())
        ));
        active.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));
        assert!(!entitlement_allows_mutation(
            &LicenseGuard::from_entitlement(active)
        ));

        let mut grace = EntitlementSnapshot::unactivated("focusa", "node");
        grace.state = EntitlementState::OfflineGrace;
        grace.offline_grace_until = Some(chrono::Utc::now() + chrono::Duration::minutes(5));
        bind(&mut grace);
        assert!(entitlement_allows_mutation(
            &LicenseGuard::from_entitlement(grace.clone())
        ));
        grace.offline_grace_until = Some(chrono::Utc::now() - chrono::Duration::seconds(1));
        assert!(!entitlement_allows_mutation(
            &LicenseGuard::from_entitlement(grace)
        ));

        let recovery = EntitlementSnapshot::recovery_only("focusa", "node", "invalid");
        assert!(!entitlement_allows_mutation(
            &LicenseGuard::from_entitlement(recovery)
        ));
    }
}
