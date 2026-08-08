use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use axum::{
    Json,
    extract::{Request, State},
    http::Method,
    middleware::Next,
    response::{IntoResponse, Response},
};
use focusa_core::{
    entitlement_execution_guard::{
        EntitlementExecutionContext, EntitlementExecutionPolicy, evaluate_entitlement_execution,
    },
    runtime::persistence_sqlite::EntitlementLimitReservationOutcome,
};
use focusa_license::{
    authority::EntitlementState,
    LicenseGuard,
    RecoveryAllowance,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{middleware::entitlement_routes::requirement_for_path, server::AppState};

#[derive(Debug)]
struct RouteEntitlementPolicy {
    operation_id: String,
    operation_class: focusa_license::OperationClass,
    capability_family: focusa_license::CapabilityFamily,
    required_feature: Option<String>,
    limit_bucket: Option<String>,
    recovery_allowance: RecoveryAllowance,
}

impl RouteEntitlementPolicy {
    fn to_execution_policy(&self) -> EntitlementExecutionPolicy {
        EntitlementExecutionPolicy::new(
            &self.operation_id,
            self.operation_class,
            self.capability_family,
            self.required_feature.as_deref(),
            self.limit_bucket.as_deref(),
            self.recovery_allowance,
        )
    }
}

#[derive(Debug, Deserialize)]
struct RoutePolicyDocument {
    operations: Vec<RoutePolicyRecord>,
}

#[derive(Debug, Deserialize)]
struct RoutePolicyRecord {
    operation_id: String,
    operation_class: focusa_license::OperationClass,
    capability_family: focusa_license::CapabilityFamily,
    required_feature: Option<String>,
    limit_bucket: Option<String>,
    recovery_allowance: RecoveryAllowance,
}

#[derive(Debug, Deserialize)]
struct RouteClassificationRecord {
    path: String,
    #[serde(default)]
    methods: Vec<String>,
    #[serde(default)]
    operation_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RouteClassificationDocument {
    routes: Vec<RouteClassificationRecord>,
}

#[derive(Debug)]
struct EntitlementMetadata {
    operations: HashMap<String, RoutePolicyRecord>,
    routes: Vec<RouteClassificationRecord>,
}

const ROUTE_CLASSIFICATION_JSON: &str =
    include_str!("../../../../docs/contracts/spec141/generated-capability-v2/route-classification.json");
const OPERATION_REGISTRY_JSON: &str =
    include_str!("../../../../docs/contracts/spec135/generated-contract-v1/operation-registry.json");
const ROUTE_UNCLASSIFIED_ERROR: &str =
    "This mutation route has no exact entitlement descriptor and is blocked fail-closed.";

fn entitlement_metadata() -> Option<&'static EntitlementMetadata> {
    static ENTITLEMENT_METADATA: OnceLock<Result<EntitlementMetadata, String>> = OnceLock::new();
    match ENTITLEMENT_METADATA.get_or_init(load_entitlement_metadata) {
        Ok(metadata) => Some(metadata),
        Err(_) => None,
    }
}

fn load_entitlement_metadata() -> Result<EntitlementMetadata, String> {
    let operations: RoutePolicyDocument = serde_json::from_str(OPERATION_REGISTRY_JSON)
        .map_err(|error| format!("failed parsing operation-registry.json: {error}"))?;
    let routes: RouteClassificationDocument = serde_json::from_str(ROUTE_CLASSIFICATION_JSON)
        .map_err(|error| format!("failed parsing route-classification.json: {error}"))?;

    let mut operation_by_id = HashMap::with_capacity(operations.operations.len());
    for operation in operations.operations {
        operation_by_id.insert(operation.operation_id.clone(), operation);
    }

    Ok(EntitlementMetadata {
        operations: operation_by_id,
        routes: routes.routes,
    })
}

fn route_unclassified_denial() -> RouteEntitlementDenial {
    RouteEntitlementDenial {
        code: "ENTITLEMENT_ROUTE_UNCLASSIFIED".to_string(),
        message: ROUTE_UNCLASSIFIED_ERROR.to_string(),
        required_feature: None,
        limit_bucket: None,
    }
}

pub async fn entitlement_gate_layer(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path();
    let requires_entitlement = route_requires_entitlement(&method, path);
    let policy = resolve_route_entitlement_policy(&method, path);

    if let Some(denial) = route_entitlement_denial(&state.license_guard, &method, path) {
        return denial_response(&state, denial);
    }

    let reservation = if requires_entitlement {
        let Some(policy) = policy else {
            return denial_response(&state, route_unclassified_denial());
        };
        if policy.recovery_allowance == RecoveryAllowance::None {
            match reserve_route_limit(&state, &request, &policy) {
                Ok(reservation) => reservation,
                Err(denial) => return denial_response(&state, denial),
            }
        } else {
            None
        }
    } else {
        None
    };

    let response = next.run(request).await;
    if let Some(reservation_id) = reservation {
        let _ = state
            .persistence
            .settle_entitlement_limit(&reservation_id, response.status().is_success());
    }
    response
}

fn denial_response(state: &AppState, denial: RouteEntitlementDenial) -> Response {
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

fn reserve_route_limit(
    state: &AppState,
    request: &Request,
    policy: &RouteEntitlementPolicy,
) -> Result<Option<String>, RouteEntitlementDenial> {
    let Some(bucket) = policy.limit_bucket.as_deref() else {
        return Ok(None);
    };
    let idempotency_key = request
        .headers()
        .get("Idempotency-Key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .ok_or(RouteEntitlementDenial {
            code: "ENTITLEMENT_IDEMPOTENCY_REQUIRED".to_string(),
            message: "A stable Idempotency-Key is required before reserving signed limit units.".to_string(),
            required_feature: policy.required_feature.clone(),
            limit_bucket: Some(bucket.to_string()),
        })?;
    let snapshot = state
        .license_guard
        .entitlement
        .as_ref()
        .ok_or(RouteEntitlementDenial {
            code: "ENTITLEMENT_REQUIRED".to_string(),
            message: "A valid signed Focusa authority lease is required for this operation.".to_string(),
            required_feature: policy.required_feature.clone(),
            limit_bucket: Some(bucket.to_string()),
        })?;
    let lease_id = snapshot.lease_id.as_deref().unwrap_or_default();
    let lease_sequence = snapshot.sequence.unwrap_or_default();
    let available = snapshot.limits.get(bucket).copied().unwrap_or(0);
    let reservation_id = format!(
        "sha256:{:x}",
        Sha256::digest(format!(
            "{lease_id}\0{lease_sequence}\0{bucket}\0{idempotency_key}"
        ))
    );
    match state.persistence.reserve_entitlement_limit(
        &reservation_id,
        lease_id,
        lease_sequence,
        bucket,
        1,
        available,
    ) {
        Ok(
            EntitlementLimitReservationOutcome::Reserved
            | EntitlementLimitReservationOutcome::IdempotentReplay,
        ) => Ok(Some(reservation_id)),
        Ok(EntitlementLimitReservationOutcome::Exhausted) => Err(RouteEntitlementDenial {
            code: "ENTITLEMENT_LIMIT_EXHAUSTED".to_string(),
            message: "The signed authority limit for this operation is exhausted.".to_string(),
            required_feature: policy.required_feature.clone(),
            limit_bucket: Some(bucket.to_string()),
        }),
        Err(_) => Err(RouteEntitlementDenial {
            code: "ENTITLEMENT_RESERVATION_FAILED".to_string(),
            message: "The durable entitlement limit reservation could not be recorded.".to_string(),
            required_feature: policy.required_feature.clone(),
            limit_bucket: Some(bucket.to_string()),
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RouteEntitlementDenial {
    code: String,
    message: String,
    required_feature: Option<String>,
    limit_bucket: Option<String>,
}

fn route_entitlement_denial(
    guard: &LicenseGuard,
    method: &Method,
    path: &str,
) -> Option<RouteEntitlementDenial> {
    let Some(policy) = resolve_route_entitlement_policy(method, path) else {
        if route_requires_entitlement(method, path) {
            return Some(route_unclassified_denial());
        }
        return None;
    };

    if let Err(failure) = evaluate_entitlement_execution(
        guard,
        &policy.to_execution_policy(),
        EntitlementExecutionContext::default(),
    ) {
        return Some(RouteEntitlementDenial {
            code: failure.code,
            message: failure.message,
            required_feature: failure.required_feature,
            limit_bucket: failure.limit_bucket,
        });
    }
    None
}

fn resolve_route_entitlement_policy(
    method: &Method,
    path: &str,
) -> Option<RouteEntitlementPolicy> {
    if let Some(allowance) = route_recovery_allowance(path) {
        let Some(capability_family) = allowance.implied_family() else {
            return None;
        };
        return Some(RouteEntitlementPolicy {
            operation_id: synthetic_operation_id(method, path),
            operation_class: focusa_license::OperationClass::Recovery,
            capability_family,
            required_feature: None,
            limit_bucket: None,
            recovery_allowance: allowance,
        });
    }

    if let Some(policy) = route_entitlement_policy_from_classification(method, path) {
        return Some(policy);
    }

    requirement_for_path(path).and_then(|requirement| {
        feature_to_capability_family(requirement.feature).map(|capability_family| RouteEntitlementPolicy {
            operation_id: synthetic_operation_id(method, path),
            operation_class: focusa_license::OperationClass::ValueMutation,
            capability_family,
            required_feature: if capability_family.is_optional_premium() {
                Some(requirement.feature.to_string())
            } else {
                None
            },
            limit_bucket: requirement.limit_bucket.map(|bucket| bucket.to_string()),
            recovery_allowance: RecoveryAllowance::None,
        })
    })
}

fn route_entitlement_policy_from_classification(
    method: &Method,
    path: &str,
) -> Option<RouteEntitlementPolicy> {
    let metadata = entitlement_metadata()?;

    for route in &metadata.routes {
        if !method_matches(method, &route.methods) {
            continue;
        }
        if !path_template_matches(&route.path, path) {
            continue;
        }
        let Some(operation_id) = route.operation_refs.first() else {
            continue;
        };
        let operation = metadata.operations.get(operation_id)?;
        return Some(RouteEntitlementPolicy {
            operation_id: operation.operation_id.clone(),
            operation_class: operation.operation_class,
            capability_family: operation.capability_family,
            required_feature: operation.required_feature.clone(),
            limit_bucket: operation.limit_bucket.clone(),
            recovery_allowance: operation.recovery_allowance,
        });
    }

    None
}

fn feature_to_capability_family(
    feature: &str,
) -> Option<focusa_license::CapabilityFamily> {
    use focusa_license::CapabilityFamily;
    match feature {
        "focusa.core.workpoint" | "focusa.core.evidence" | "focusa.core.mission" => {
            Some(CapabilityFamily::BaseFocusa)
        }
        "focusa.agent.parallelism" | "focusa.agent.silent_sessions" => {
            Some(CapabilityFamily::Automation)
        }
        "focusa.team.multi_operator" | "focusa.remote.stream" => {
            Some(CapabilityFamily::TeamRemote)
        }
        "focusa.release.proof" => Some(CapabilityFamily::ReleaseProof),
        "focusa.update.unattended" | "focusa.update.apply"
        | "focusa.install.channel.nightly" | "focusa.install.channel.preview" => {
            Some(CapabilityFamily::PremiumUpdates)
        }
        "focusa.export.packaged" => Some(CapabilityFamily::CustomerDataExport),
        _ => None,
    }
}

fn method_matches(method: &Method, allowed_methods: &[String]) -> bool {
    if allowed_methods.is_empty() {
        return true;
    }
    allowed_methods
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(method.as_str()))
}

fn path_template_matches(template: &str, path: &str) -> bool {
    if template == path {
        return true;
    }
    let template_segments: Vec<_> = template.trim_matches('/').split('/').collect();
    let path_segments: Vec<_> = path.trim_matches('/').split('/').collect();
    if template_segments.len() != path_segments.len() {
        return false;
    }
    for (template_segment, path_segment) in template_segments.iter().zip(path_segments.iter()) {
        if template_segment.starts_with('{') && template_segment.ends_with('}') {
            if path_segment.is_empty() {
                return false;
            }
            continue;
        }
        if template_segment != path_segment {
            return false;
        }
    }
    true
}

fn synthetic_operation_id(method: &Method, path: &str) -> String {
    let segments = path
        .trim_matches('/')
        .replace('/', ".")
        .replace('{', "")
        .replace('}', "");
    let normalized = if segments.is_empty() { "root" } else { &segments };
    format!("rest.{normalized}.{}", method.as_str().to_ascii_lowercase())
}

fn route_recovery_allowance(path: &str) -> Option<RecoveryAllowance> {
    match path {
        "/v1/device/pair/revoke" => Some(RecoveryAllowance::AccountRecovery),
        "/v1/export/run" => Some(RecoveryAllowance::CustomerDataExport),
        "/v1/project/bootstrap/repair" => Some(RecoveryAllowance::RepairRollback),
        "/v1/update/apply" => Some(RecoveryAllowance::StableSecurityUpdate),
        "/v1/update/rollback" => Some(RecoveryAllowance::RepairRollback),
        _ => None,
    }
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
        if path == "/health"
            || path == "/v1/health"
            || path == "/v1/version"
            || path == "/v1/update/check"
            || path == "/v1/update/plan"
            || path == "/v1/update/rollback"
            || is_read_only_preflight(path)
            || is_recovery_export(path)
            || path.starts_with("/v1/license/")
            || route_is_known_read_without_side_effect(method, path)
        {
            return false;
        }
        return true;
    }
    let recovery_path = path == "/health"
        || path == "/v1/health"
        || path == "/v1/version"
        || path == "/v1/update/check"
        || path == "/v1/update/plan"
        || path == "/v1/update/rollback"
        || is_read_only_preflight(path)
        || is_recovery_export(path)
        || path.starts_with("/v1/license/");
    !recovery_path
}

fn route_is_known_read_without_side_effect(method: &Method, path: &str) -> bool {
    route_has_classified_read(method, path) || route_has_declared_methods(path)
}

fn route_has_classified_read(method: &Method, path: &str) -> bool {
    resolve_route_entitlement_policy(method, path)
        .is_some_and(|policy| policy.operation_class == focusa_license::OperationClass::Read)
}

fn route_has_declared_methods(path: &str) -> bool {
    entitlement_metadata().is_some_and(|metadata| {
        metadata
            .routes
            .iter()
            .any(|route| !route.methods.is_empty() && path_template_matches(&route.path, path))
    })
}

fn is_read_only_preflight(path: &str) -> bool {
    let segments: Vec<_> = path.trim_matches('/').split('/').collect();
    path == "/v1/silent-sessions/preflight"
        || path == "/v1/silent-sessions/config/resolve"
        || matches!(segments.as_slice(), ["v1", "harnesses", harness, "preflight"] if !harness.is_empty())
        || matches!(segments.as_slice(), ["v1", "providers", provider, "models", "preflight"] if !provider.is_empty())
        || matches!(segments.as_slice(), ["v1", "silent-sessions", session_id, "config", "preview"] if !session_id.is_empty())
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
        assert!(!route_requires_entitlement(&Method::GET, "/v1/version"));
        assert!(!route_requires_entitlement(
            &Method::GET,
            "/v1/device/pair/status"
        ));
        assert!(route_requires_entitlement(
            &Method::GET,
            "/v1/device/pair/list"
        ));
        assert!(route_requires_entitlement(
            &Method::GET,
            "/v1/connect/rooms"
        ));
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
        assert!(route_requires_entitlement(
            &Method::POST,
            "/v1/project/bootstrap/repair"
        ));
        assert!(route_requires_entitlement(&Method::POST, "/v1/export/run"));
        assert!(!route_requires_entitlement(
            &Method::POST,
            "/v1/silent-sessions/session-1/export"
        ));
        for path in [
            "/v1/silent-sessions/preflight",
            "/v1/silent-sessions/config/resolve",
            "/v1/harnesses/pi/preflight",
            "/v1/providers/pi-runtime/models/preflight",
            "/v1/silent-sessions/session-1/config/preview",
        ] {
            assert!(!route_requires_entitlement(&Method::POST, path), "{path}");
        }
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
        assert_eq!(
            route_entitlement_denial(&LicenseGuard::eval(7), &Method::POST, "/v1/workpoint/checkpoint")
                .unwrap()
                .code,
            "ENTITLEMENT_BASE_REQUIRED"
        );

        let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node");
        snapshot.state = EntitlementState::Active;
        snapshot.expires_at = Some(chrono::Utc::now() + chrono::Duration::minutes(5));
        snapshot.sequence = Some(7);
        snapshot.lease_id = Some("lease-1".into());
        snapshot.lease_digest = Some("sha256:lease".into());
        let guard = LicenseGuard::from_entitlement(snapshot.clone());
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/workpoint/checkpoint"),
            None
        );

        let team_path = "/v1/connect/room/create";
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, team_path)
                .unwrap()
                .code,
            "ENTITLEMENT_FEATURE_REQUIRED"
        );
        snapshot
            .features
            .insert("focusa.team.multi_operator".into(), true);
        let guard = LicenseGuard::from_entitlement(snapshot);
        assert_eq!(route_entitlement_denial(&guard, &Method::POST, team_path), None);
        assert_eq!(
            route_entitlement_denial(&LicenseGuard::eval(7), &Method::POST, "/v1/unclassified/mutation")
                .unwrap()
                .code,
            "ENTITLEMENT_ROUTE_UNCLASSIFIED"
        );
        assert_eq!(
            route_entitlement_denial(&LicenseGuard::eval(7), &Method::GET, "/v1/connect/rooms")
                .unwrap()
                .code,
            "ENTITLEMENT_ROUTE_UNCLASSIFIED"
        );
    }

    #[test]
    fn recovery_allowances_skip_entitlement_state_and_feature_checks() {
        let guard = LicenseGuard::eval(7);
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/update/apply"),
            None,
            "apply must remain available during recovery-only paths"
        );
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/project/bootstrap/repair"),
            None,
            "repair must remain available during recovery-only paths"
        );
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/export/run"),
            None,
            "export must remain available during recovery-only paths"
        );
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/update/rollback"),
            None,
            "rollback must remain available during recovery-only paths"
        );
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/device/pair/revoke"),
            None,
            "node deactivation must remain available during recovery-only paths"
        );
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/workpoint/checkpoint")
                .unwrap()
                .code,
            "ENTITLEMENT_BASE_REQUIRED"
        );
    }
}
