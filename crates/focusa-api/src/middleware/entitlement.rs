use std::sync::Arc;

use axum::{
    Json,
    extract::{Request, State},
    http::Method,
    middleware::Next,
    response::{IntoResponse, Response},
};
use focusa_license::authority::EntitlementState;

use crate::{middleware::entitlement_routes::requirement_for_path, server::AppState};

pub async fn entitlement_gate_layer(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    if !route_requires_entitlement(request.method(), request.uri().path()) {
        return next.run(request).await;
    }
    let Some(denial) = route_entitlement_denial(&state.license_guard, request.uri().path()) else {
        return next.run(request).await;
    };

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
                "code": denial.code,
                "message": denial.message,
                "state": state_label,
                "required_feature": denial.required_feature,
                "limit_bucket": denial.limit_bucket,
                "recovery": {
                    "status_path": "/v1/license/status",
                    "allowed": ["health", "version", "license_recovery", "safe_read"]
                }
            }
        })),
    )
        .into_response()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RouteEntitlementDenial {
    code: &'static str,
    message: &'static str,
    required_feature: Option<&'static str>,
    limit_bucket: Option<&'static str>,
}

fn route_entitlement_denial(
    guard: &focusa_license::LicenseGuard,
    path: &str,
) -> Option<RouteEntitlementDenial> {
    if !entitlement_allows_mutation(guard) {
        return Some(RouteEntitlementDenial {
            code: "ENTITLEMENT_REQUIRED",
            message: "A valid signed Focusa authority lease is required for this operation.",
            required_feature: requirement_for_path(path).map(|requirement| requirement.feature),
            limit_bucket: requirement_for_path(path)
                .and_then(|requirement| requirement.limit_bucket),
        });
    }
    let Some(requirement) = requirement_for_path(path) else {
        return Some(RouteEntitlementDenial {
            code: "ENTITLEMENT_ROUTE_UNCLASSIFIED",
            message: "This mutation route has no exact entitlement descriptor and is blocked fail-closed.",
            required_feature: None,
            limit_bucket: None,
        });
    };
    let snapshot = guard.entitlement.as_ref()?;
    if !snapshot
        .features
        .get(requirement.feature)
        .copied()
        .unwrap_or(false)
    {
        return Some(RouteEntitlementDenial {
            code: "ENTITLEMENT_FEATURE_REQUIRED",
            message: "The signed authority lease does not grant this exact feature.",
            required_feature: Some(requirement.feature),
            limit_bucket: requirement.limit_bucket,
        });
    }
    if let Some(bucket) = requirement.limit_bucket {
        if snapshot.limits.get(bucket).copied().unwrap_or(0) == 0 {
            return Some(RouteEntitlementDenial {
                code: "ENTITLEMENT_LIMIT_EXHAUSTED",
                message: "The signed authority limit for this operation is unavailable or exhausted.",
                required_feature: Some(requirement.feature),
                limit_bucket: Some(bucket),
            });
        }
    }
    None
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
        || path == "/v1/update/check"
        || path == "/v1/update/plan"
        || path == "/v1/update/rollback"
        || is_recovery_export(path)
        || path.starts_with("/v1/license/");
    !recovery_path
}

fn is_recovery_export(path: &str) -> bool {
    let segments: Vec<_> = path.trim_matches('/').split('/').collect();
    matches!(segments.as_slice(), ["v1", "silent-sessions", session_id, "export"] if !session_id.is_empty())
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
        assert!(route_requires_entitlement(
            &Method::POST,
            "/v1/connect/room/create"
        ));
        assert!(route_requires_entitlement(
            &Method::POST,
            "/v1/device/pair/start"
        ));
        assert!(!route_requires_entitlement(&Method::GET, "/v1/health"));
        assert!(!route_requires_entitlement(
            &Method::POST,
            "/v1/update/check"
        ));
        assert!(!route_requires_entitlement(
            &Method::POST,
            "/v1/update/plan"
        ));
        assert!(!route_requires_entitlement(
            &Method::POST,
            "/v1/update/rollback"
        ));
        assert!(route_requires_entitlement(
            &Method::POST,
            "/v1/update/apply"
        ));
        assert!(route_requires_entitlement(&Method::POST, "/v1/export/run"));
        assert!(!route_requires_entitlement(
            &Method::POST,
            "/v1/silent-sessions/session-1/export"
        ));
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

    #[test]
    fn exact_feature_and_signed_limit_are_required_before_route_handler() {
        let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node");
        snapshot.state = EntitlementState::Active;
        snapshot.expires_at = Some(chrono::Utc::now() + chrono::Duration::minutes(5));
        snapshot.lease_id = Some("lease-1".into());
        snapshot.lease_digest = Some("sha256:lease".into());
        let path = "/v1/workpoint/checkpoint";

        let guard = LicenseGuard::from_entitlement(snapshot.clone());
        assert_eq!(
            route_entitlement_denial(&guard, path).unwrap().code,
            "ENTITLEMENT_FEATURE_REQUIRED"
        );

        snapshot
            .features
            .insert("focusa.core.workpoint".into(), true);
        let guard = LicenseGuard::from_entitlement(snapshot.clone());
        assert_eq!(
            route_entitlement_denial(&guard, path).unwrap().code,
            "ENTITLEMENT_LIMIT_EXHAUSTED"
        );

        snapshot.limits.insert("workpoints".into(), 1);
        let guard = LicenseGuard::from_entitlement(snapshot);
        assert_eq!(route_entitlement_denial(&guard, path), None);
        assert_eq!(
            route_entitlement_denial(&guard, "/v1/unclassified/mutation")
                .unwrap()
                .code,
            "ENTITLEMENT_ROUTE_UNCLASSIFIED"
        );
    }
}
