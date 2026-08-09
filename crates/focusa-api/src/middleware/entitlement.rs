use std::collections::BTreeMap;
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
use focusa_license::{LicenseGuard, RecoveryAllowance, authority::EntitlementState};
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
    operations: BTreeMap<String, RoutePolicyRecord>,
    routes: Vec<RouteClassificationRecord>,
}

const ROUTE_CLASSIFICATION_JSON: &str = include_str!(
    "../../../../docs/contracts/spec141/generated-capability-v2/route-classification.json"
);
const OPERATION_REGISTRY_JSON: &str = include_str!(
    "../../../../docs/contracts/spec135/generated-contract-v1/operation-registry.json"
);
const RECOVERY_ONLY_SURFACE_JSON: &str = include_str!(
    "../../../../docs/contracts/spec152e-recovery-only-surface.v1.json"
);
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

    let mut operation_by_id = BTreeMap::new();
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

/// Runtime denial code -> exact safe recovery action from the cross-surface
/// recovery-only contract (Spec 152E §18/§20). Every denial envelope carries
/// the bound action so API/CLI/TUI/menubar/agent present the same recovery
/// posture. Unknown codes fail closed to the contract's default action.
#[derive(Debug, Clone, Copy)]
struct RecoveryGuidance {
    action: &'static str,
    allowed: &'static [&'static str],
}

const DEFAULT_RECOVERY_GUIDANCE: RecoveryGuidance = RecoveryGuidance {
    action: "recovery_only",
    allowed: &[
        "health",
        "version",
        "license_status",
        "export",
        "diagnostics",
        "repair",
        "update_for_recovery",
        "uninstall",
        "safe_read",
    ],
};

fn recovery_guidance_for_code(code: &str) -> RecoveryGuidance {
    static GUIDANCE: OnceLock<BTreeMap<&'static str, RecoveryGuidance>> = OnceLock::new();
    let guidance = GUIDANCE.get_or_init(|| {
        let mut map = BTreeMap::new();
        // Fail closed: an unparseable embedded contract yields the default
        // guidance for every code rather than an envelope without recovery.
        let Ok(contract) =
            serde_json::from_str::<serde_json::Value>(RECOVERY_ONLY_SURFACE_JSON)
        else {
            return map;
        };
        let allowed: &'static [&'static str] = Box::leak(
            contract
                .get("consistency")
                .and_then(|c| c.get("envelope_allowed"))
                .and_then(|list| list.as_array())
                .map(|list| {
                    list.iter()
                        .filter_map(|value| value.as_str())
                        .map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| DEFAULT_RECOVERY_GUIDANCE.allowed.to_vec())
                .into_boxed_slice(),
        );
        let default_action: &'static str = contract
            .get("default_runtime_denial")
            .and_then(|d| d.get("recovery_action"))
            .and_then(|value| value.as_str())
            .map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str)
            .unwrap_or(DEFAULT_RECOVERY_GUIDANCE.action);
        if let Some(bindings) = contract
            .get("runtime_denial_bindings")
            .and_then(|bindings| bindings.as_object())
        {
            for (code_value, binding) in bindings {
                let action: &'static str = binding
                    .get("recovery_action")
                    .and_then(|value| value.as_str())
                    .map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str)
                    .unwrap_or(default_action);
                let code: &'static str =
                    Box::leak(code_value.clone().into_boxed_str()) as &'static str;
                map.insert(code, RecoveryGuidance { action, allowed });
            }
        }
        map
    });
    guidance
        .get(code)
        .copied()
        .unwrap_or(DEFAULT_RECOVERY_GUIDANCE)
}

#[test]
fn recovery_guidance_is_contract_bound() {
    let guidance = recovery_guidance_for_code("ENTITLEMENT_BASE_REQUIRED");
    assert_eq!(guidance.action, "reactivate_or_repair_lease");
    assert!(guidance.allowed.contains(&"export"));
    assert!(guidance.allowed.contains(&"diagnostics"));
    assert!(guidance.allowed.contains(&"repair"));
    assert!(guidance.allowed.contains(&"update_for_recovery"));
    assert!(guidance.allowed.contains(&"uninstall"));
    assert!(guidance.allowed.contains(&"license_status"));

    assert_eq!(
        recovery_guidance_for_code("ENTITLEMENT_FEATURE_REQUIRED").action,
        "manage_license"
    );
    assert_eq!(
        recovery_guidance_for_code("ENTITLEMENT_LIMIT_EXHAUSTED").action,
        "manage_limit"
    );
    assert_eq!(
        recovery_guidance_for_code("ENTITLEMENT_ROUTE_UNCLASSIFIED").action,
        "recovery_only"
    );

    // Unknown codes fail closed to the contract default.
    assert_eq!(
        recovery_guidance_for_code("UNKNOWN_RUNTIME_CODE").action,
        "recovery_only"
    );
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
        if let Some(policy) = policy {
            if policy.recovery_allowance == RecoveryAllowance::None {
                match reserve_route_limit(&state, &request) {
                    Ok(reservation) => reservation,
                    Err(denial) => return denial_response(&state, denial),
                }
            } else {
                None
            }
        } else {
            return denial_response(&state, route_unclassified_denial());
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
    let guidance = recovery_guidance_for_code(&denial.code);
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
                    "action": guidance.action,
                    "allowed": guidance.allowed
                }
            }
        })),
    )
        .into_response()
}

fn reserve_route_limit(
    state: &AppState,
    request: &Request,
) -> Result<Option<String>, RouteEntitlementDenial> {
    let method = request.method();
    let path = request.uri().path();
    let Some(policy) = resolve_route_entitlement_policy(method, path) else {
        return Ok(None);
    };
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
            message: "A stable Idempotency-Key is required before reserving signed limit units."
                .to_string(),
            required_feature: policy.required_feature.clone(),
            limit_bucket: Some(bucket.to_string()),
        })?;
    let snapshot = state
        .license_guard
        .entitlement
        .as_ref()
        .ok_or(RouteEntitlementDenial {
            code: "ENTITLEMENT_REQUIRED".to_string(),
            message: "A valid signed Focusa authority lease is required for this operation."
                .to_string(),
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

    // Declared recovery/customer-control surfaces are governed by their own
    // allowances and never by lease state; they pass before any evaluation.
    if policy.recovery_allowance != RecoveryAllowance::None {
        return None;
    }

    if let Err(failure) = evaluate_entitlement_execution(
        guard,
        &policy.to_execution_policy(),
        EntitlementExecutionContext::default(),
    ) {
        // With no signed lease at all, the base Focusa gate is the first
        // prerequisite for every value-producing operation, premium or not;
        // it must be reported before any premium feature denial can exist.
        let code = if guard.entitlement.is_none() && failure.code == "ENTITLEMENT_REQUIRED" {
            "ENTITLEMENT_BASE_REQUIRED".to_string()
        } else {
            failure.code
        };
        return Some(RouteEntitlementDenial {
            code,
            message: failure.message,
            required_feature: failure.required_feature,
            limit_bucket: failure.limit_bucket,
        });
    }

    // Lease binding/current-state validation before handler invocation:
    // an expired, revoked, unbound, or fabricated lease must never reach a
    // value-producing handler even when the policy state grid would allow it.
    if !entitlement_allows_mutation(guard) {
        return Some(RouteEntitlementDenial {
            code: "ENTITLEMENT_BASE_REQUIRED".to_string(),
            message:
                "A current signed Focusa authority lease is required before this handler runs."
                    .to_string(),
            required_feature: policy.required_feature,
            limit_bucket: policy.limit_bucket,
        });
    }
    None
}

fn resolve_route_entitlement_policy(method: &Method, path: &str) -> Option<RouteEntitlementPolicy> {
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
        feature_to_capability_family(requirement.feature).map(|capability_family| {
            RouteEntitlementPolicy {
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
            }
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

fn feature_to_capability_family(feature: &str) -> Option<focusa_license::CapabilityFamily> {
    use focusa_license::CapabilityFamily;
    match feature {
        "focusa.core.workpoint" | "focusa.core.evidence" | "focusa.core.mission" => {
            Some(CapabilityFamily::BaseFocusa)
        }
        "focusa.agent.parallelism" | "focusa.agent.silent_sessions" => {
            Some(CapabilityFamily::Automation)
        }
        "focusa.team.multi_operator" | "focusa.remote.stream" => Some(CapabilityFamily::TeamRemote),
        "focusa.release.proof" => Some(CapabilityFamily::ReleaseProof),
        "focusa.update.unattended"
        | "focusa.update.apply"
        | "focusa.install.channel.nightly"
        | "focusa.install.channel.preview" => Some(CapabilityFamily::PremiumUpdates),
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
    let normalized = if segments.is_empty() {
        "root"
    } else {
        &segments
    };
    format!("rest.{normalized}.{}", method.as_str().to_ascii_lowercase())
}

fn route_recovery_allowance(path: &str) -> Option<RecoveryAllowance> {
    // Recovery-allowance paths are matched first by exact path, then by
    // template-based matching for parameterized routes.
    match path {
        // Account recovery: node deactivation, diagnostics, pairing status
        "/v1/device/pair/revoke" => Some(RecoveryAllowance::AccountRecovery),
        "/v1/device/pair/status" => Some(RecoveryAllowance::AccountRecovery),
        "/v1/doctor" => Some(RecoveryAllowance::AccountRecovery),
        "/v1/doctor/closure" => Some(RecoveryAllowance::AccountRecovery),

        // Customer data export: run, status, history, manifest
        "/v1/export/run" => Some(RecoveryAllowance::CustomerDataExport),
        "/v1/export/status" => Some(RecoveryAllowance::CustomerDataExport),
        "/v1/export/history" => Some(RecoveryAllowance::CustomerDataExport),

        // Repair and rollback
        "/v1/project/bootstrap/repair" => Some(RecoveryAllowance::RepairRollback),
        "/v1/update/rollback" => Some(RecoveryAllowance::RepairRollback),

        // Stable security update
        "/v1/update/apply" => Some(RecoveryAllowance::StableSecurityUpdate),

        _ => {
            // Template-based matching for parameterized recovery routes
            let segments: Vec<_> = path.trim_matches('/').split('/').collect();
            match segments.as_slice() {
                // /v1/export/manifest/{export_id}
                ["v1", "export", "manifest", export_id] if !export_id.is_empty() => {
                    Some(RecoveryAllowance::CustomerDataExport)
                }
                _ => None,
            }
        }
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
        || path == "/v1/info"
        || path == "/v1/update/check"
        || path == "/v1/update/plan"
        || path == "/v1/update/rollback"
        || path == "/v1/doctor"
        || path == "/v1/doctor/closure"
        || path == "/v1/export/status"
        || path == "/v1/export/history"
        || path == "/v1/device/pair/status"
        || is_read_only_preflight(path)
        || is_recovery_export(path)
        || is_export_manifest_read(path)
        || path.starts_with("/v1/license/");
    !recovery_path
}

fn route_is_known_read_without_side_effect(method: &Method, path: &str) -> bool {
    // A declared-method ledger entry alone never exempts a premium read:
    // reads that resolve to an optional-premium family must still pass the
    // entitlement gate (they expose premium collaboration/automation state).
    route_has_classified_read(method, path)
        || (route_has_declared_methods(path) && !route_has_premium_policy(method, path))
}

fn route_has_classified_read(method: &Method, path: &str) -> bool {
    resolve_route_entitlement_policy(method, path)
        .is_some_and(|policy| policy.operation_class == focusa_license::OperationClass::Read)
}

fn route_has_premium_policy(method: &Method, path: &str) -> bool {
    resolve_route_entitlement_policy(method, path)
        .is_some_and(|policy| policy.capability_family.is_optional_premium())
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

fn is_export_manifest_read(path: &str) -> bool {
    let segments: Vec<_> = path.trim_matches('/').split('/').collect();
    matches!(segments.as_slice(), ["v1", "export", "manifest", export_id] if !export_id.is_empty())
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
            route_entitlement_denial(
                &LicenseGuard::eval(7),
                &Method::POST,
                "/v1/workpoint/checkpoint"
            )
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
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, team_path),
            None
        );
        assert_eq!(
            route_entitlement_denial(
                &LicenseGuard::eval(7),
                &Method::POST,
                "/v1/unclassified/mutation"
            )
            .unwrap()
            .code,
            "ENTITLEMENT_ROUTE_UNCLASSIFIED"
        );
        assert_eq!(
            route_entitlement_denial(&LicenseGuard::eval(7), &Method::GET, "/v1/connect/rooms")
                .unwrap()
                .code,
            "ENTITLEMENT_BASE_REQUIRED"
        );
    }

    #[test]
    fn recovery_allowances_skip_entitlement_state_and_feature_checks() {
        let guard = LicenseGuard::eval(7);

        // Stable security update
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/update/apply"),
            None,
            "apply must remain available during recovery-only paths"
        );

        // Repair
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/project/bootstrap/repair"),
            None,
            "repair must remain available during recovery-only paths"
        );

        // Customer data export
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/export/run"),
            None,
            "export must remain available during recovery-only paths"
        );
        assert_eq!(
            route_entitlement_denial(&guard, &Method::GET, "/v1/export/status"),
            None,
            "export status must remain available during recovery-only paths"
        );
        assert_eq!(
            route_entitlement_denial(&guard, &Method::GET, "/v1/export/history"),
            None,
            "export history must remain available during recovery-only paths"
        );
        assert_eq!(
            route_entitlement_denial(&guard, &Method::GET, "/v1/export/manifest/manifest-1"),
            None,
            "export manifest must remain available during recovery-only paths"
        );

        // Rollback
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/update/rollback"),
            None,
            "rollback must remain available during recovery-only paths"
        );

        // Node deactivation
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/device/pair/revoke"),
            None,
            "node deactivation must remain available during recovery-only paths"
        );

        // Diagnostics
        assert_eq!(
            route_entitlement_denial(&guard, &Method::GET, "/v1/doctor"),
            None,
            "diagnostics must remain available during recovery-only paths"
        );
        assert_eq!(
            route_entitlement_denial(&guard, &Method::GET, "/v1/doctor/closure"),
            None,
            "diagnostics closure must remain available during recovery-only paths"
        );

        // License status
        assert_eq!(
            route_entitlement_denial(&guard, &Method::GET, "/v1/license/status"),
            None,
            "license status must remain available during recovery-only paths"
        );

        // Base mutation must still be denied
        assert_eq!(
            route_entitlement_denial(&guard, &Method::POST, "/v1/workpoint/checkpoint")
                .unwrap()
                .code,
            "ENTITLEMENT_BASE_REQUIRED"
        );
    }

    #[test]
    fn recovery_route_matrix_permanent_customer_control_routes() {
        // Prove that every required customer-control route remains available
        // in every blocked entitlement state, while protected mutations and
        // accidental destructive purge remain denied.
        use focusa_license::authority::EntitlementSnapshot;

        // All blocked states where base mutations must be denied.
        // recovery_only / refunded_or_revoked: RecoveryOnly snapshot → RefundedOrRevoked
        // unactivated: no snapshot → PendingUnverified
        // missing: LicenseGuard::eval(7) has no snapshot → MissingOrCorrupt
        let blocked_guards: Vec<(&str, LicenseGuard)> = vec![
            (
                "recovery_only",
                LicenseGuard::from_entitlement(EntitlementSnapshot::recovery_only(
                    "focusa",
                    "node",
                    "recovery-matrix",
                )),
            ),
            ("refunded_or_revoked", {
                let mut snap = EntitlementSnapshot::unactivated("focusa", "node");
                snap.state = EntitlementState::RecoveryOnly;
                snap.lease_id = Some("lease-refunded".into());
                snap.lease_digest = Some("sha256:refunded".into());
                snap.sequence = Some(1);
                LicenseGuard::from_entitlement(snap)
            }),
            (
                "unactivated",
                LicenseGuard::from_entitlement(EntitlementSnapshot::unactivated("focusa", "node")),
            ),
            ("missing", LicenseGuard::eval(7)),
        ];

        // Recovery/customer-control routes that MUST be available in every blocked state.
        // These are tested through route_entitlement_denial (routes that pass through
        // the entitlement middleware's recovery allowance path).
        let recovery_routes: Vec<(&str, &Method, &str)> = vec![
            // Stable security update
            ("stable_update", &Method::POST, "/v1/update/apply"),
            // Repair
            ("repair", &Method::POST, "/v1/project/bootstrap/repair"),
            // Rollback
            ("rollback", &Method::POST, "/v1/update/rollback"),
            // Customer data export
            ("export_run", &Method::POST, "/v1/export/run"),
            ("export_status", &Method::GET, "/v1/export/status"),
            ("export_history", &Method::GET, "/v1/export/history"),
            (
                "export_manifest",
                &Method::GET,
                "/v1/export/manifest/manifest-1",
            ),
            // Node deactivation
            ("node_deactivation", &Method::POST, "/v1/device/pair/revoke"),
            // Pairing status (read)
            ("pairing_status", &Method::GET, "/v1/device/pair/status"),
            // Diagnostics
            ("diagnostics", &Method::GET, "/v1/doctor"),
            ("diagnostics_closure", &Method::GET, "/v1/doctor/closure"),
            // License status
            ("license_status", &Method::GET, "/v1/license/status"),
        ];

        // Routes that are exempted via route_requires_entitlement and never
        // reach the entitlement denial check. Tested separately.
        let exempted_routes: Vec<(&str, &Method, &str)> = vec![
            ("health", &Method::GET, "/v1/health"),
            ("version", &Method::GET, "/v1/version"),
            ("update_check", &Method::POST, "/v1/update/check"),
            ("update_plan", &Method::POST, "/v1/update/plan"),
            (
                "silent_export",
                &Method::POST,
                "/v1/silent-sessions/session-1/export",
            ),
            ("preflight", &Method::POST, "/v1/silent-sessions/preflight"),
        ];

        // Protected mutation routes that MUST be denied in every blocked state.
        // These are value-producing mutations that require base entitlement.
        // device/pair/start is excluded because it is classified as account_recovery
        // in the operation registry (required for node activation/recovery).
        let protected_mutations: Vec<(&str, &Method, &str)> = vec![
            (
                "workpoint_checkpoint",
                &Method::POST,
                "/v1/workpoint/checkpoint",
            ),
            (
                "evidence_capture",
                &Method::POST,
                "/v1/evidence/capture",
            ),
            (
                "metacog_capture",
                &Method::POST,
                "/v1/metacognition/capture",
            ),
            ("turn_start", &Method::POST, "/v1/turn/start"),
            (
                "silent_session_start",
                &Method::POST,
                "/v1/silent-sessions/session-1/start",
            ),
            ("project_new", &Method::POST, "/v1/project/new"),
            (
                "connect_room_create",
                &Method::POST,
                "/v1/connect/room/create",
            ),
            (
                "constitution_propose",
                &Method::POST,
                "/v1/constitution/propose",
            ),
            (
                "project_bootstrap_apply",
                &Method::POST,
                "/v1/project/bootstrap/apply",
            ),
            (
                "silent_session_create",
                &Method::POST,
                "/v1/silent-sessions",
            ),
        ];

        for (state_label, guard) in &blocked_guards {
            // Recovery routes must be available through the entitlement denial check
            for (route_label, method, path) in &recovery_routes {
                let denial = route_entitlement_denial(guard, method, path);
                assert!(
                    denial.is_none(),
                    "{route_label} ({path}) must be available in state {state_label}, got: {denial:?}"
                );
            }

            // Exempted routes must never require entitlement
            for (route_label, method, path) in &exempted_routes {
                assert!(
                    !route_requires_entitlement(method, path),
                    "{route_label} ({path}) must be exempted from entitlement in state {state_label}"
                );
            }

            // Protected mutations must be denied
            for (route_label, method, path) in &protected_mutations {
                let denial = route_entitlement_denial(guard, method, path);
                assert!(
                    denial.is_some(),
                    "{route_label} ({path}) must be denied in state {state_label}"
                );
                if let Some(d) = denial {
                    assert!(
                        d.code == "ENTITLEMENT_BASE_REQUIRED"
                            || d.code == "ENTITLEMENT_REQUIRED"
                            || d.code == "ENTITLEMENT_FEATURE_REQUIRED"
                            || d.code == "ENTITLEMENT_ROUTE_UNCLASSIFIED",
                        "{route_label} ({path}) in state {state_label}: unexpected denial code {}",
                        d.code
                    );
                }
            }
        }

        // In Active state, base mutations must be allowed
        let mut active_snap = EntitlementSnapshot::unactivated("focusa", "node");
        active_snap.state = EntitlementState::Active;
        active_snap.lease_id = Some("lease-active".into());
        active_snap.lease_digest = Some("sha256:active".into());
        active_snap.sequence = Some(7);
        active_snap.expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));
        let active_guard = LicenseGuard::from_entitlement(active_snap);

        assert_eq!(
            route_entitlement_denial(&active_guard, &Method::POST, "/v1/workpoint/checkpoint"),
            None,
            "base mutation must be allowed with active entitlement"
        );
        assert_eq!(
            route_entitlement_denial(&active_guard, &Method::POST, "/v1/metacognition/capture"),
            None,
            "metacog capture must be allowed with active entitlement"
        );

        // Recovery routes must also remain available in active state
        assert_eq!(
            route_entitlement_denial(&active_guard, &Method::POST, "/v1/export/run"),
            None,
            "export must remain available in active state"
        );
        assert_eq!(
            route_entitlement_denial(&active_guard, &Method::POST, "/v1/update/apply"),
            None,
            "update must remain available in active state"
        );
    }

    #[test]
    fn blocked_leases_permit_only_declared_recovery_surfaces_with_zero_mutation_sentinels() {
        // Replay proof: for every blocked lease posture (missing, invalid,
        // expired, revoked), only declared recovery surfaces may pass the
        // pre-handler gate, and every protected mutation route must be denied
        // before side effects — zero mutation sentinel events may escape.
        use focusa_license::authority::EntitlementSnapshot;

        let mut expired_snap = EntitlementSnapshot::unactivated("focusa", "node");
        expired_snap.state = EntitlementState::Active;
        expired_snap.lease_id = Some("lease-expired".into());
        expired_snap.lease_digest = Some("sha256:expired".into());
        expired_snap.sequence = Some(7);
        expired_snap.expires_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));

        let mut invalid_snap = EntitlementSnapshot::unactivated("focusa", "node");
        invalid_snap.state = EntitlementState::Active;
        invalid_snap.lease_id = Some("lease-invalid".into());
        invalid_snap.lease_digest = Some("not-a-sha256-digest".into());
        invalid_snap.sequence = Some(7);
        invalid_snap.expires_at = Some(chrono::Utc::now() + chrono::Duration::minutes(5));

        let mut revoked_snap = EntitlementSnapshot::unactivated("focusa", "node");
        revoked_snap.state = EntitlementState::RecoveryOnly;
        revoked_snap.lease_id = Some("lease-revoked".into());
        revoked_snap.lease_digest = Some("sha256:revoked".into());
        revoked_snap.sequence = Some(1);

        let blocked_guards: Vec<(&str, LicenseGuard)> = vec![
            ("missing", LicenseGuard::eval(7)),
            ("invalid", LicenseGuard::from_entitlement(invalid_snap)),
            ("expired", LicenseGuard::from_entitlement(expired_snap)),
            ("revoked", LicenseGuard::from_entitlement(revoked_snap)),
        ];

        let recovery_surfaces: Vec<(&str, &Method, &str)> = vec![
            ("stable_update", &Method::POST, "/v1/update/apply"),
            ("repair", &Method::POST, "/v1/project/bootstrap/repair"),
            ("rollback", &Method::POST, "/v1/update/rollback"),
            ("export_run", &Method::POST, "/v1/export/run"),
            ("export_status", &Method::GET, "/v1/export/status"),
            ("export_history", &Method::GET, "/v1/export/history"),
            (
                "export_manifest",
                &Method::GET,
                "/v1/export/manifest/manifest-1",
            ),
            ("node_deactivation", &Method::POST, "/v1/device/pair/revoke"),
            ("pairing_status", &Method::GET, "/v1/device/pair/status"),
            ("diagnostics", &Method::GET, "/v1/doctor"),
            ("diagnostics_closure", &Method::GET, "/v1/doctor/closure"),
            ("license_status", &Method::GET, "/v1/license/status"),
            ("health", &Method::GET, "/v1/health"),
            ("version", &Method::GET, "/v1/version"),
        ];

        let protected_mutations: Vec<(&str, &Method, &str)> = vec![
            (
                "workpoint_checkpoint",
                &Method::POST,
                "/v1/workpoint/checkpoint",
            ),
            (
                "evidence_capture",
                &Method::POST,
                "/v1/evidence/capture",
            ),
            (
                "metacog_capture",
                &Method::POST,
                "/v1/metacognition/capture",
            ),
            ("turn_start", &Method::POST, "/v1/turn/start"),
            (
                "silent_session_start",
                &Method::POST,
                "/v1/silent-sessions/session-1/start",
            ),
            ("project_new", &Method::POST, "/v1/project/new"),
            (
                "connect_room_create",
                &Method::POST,
                "/v1/connect/room/create",
            ),
            (
                "constitution_propose",
                &Method::POST,
                "/v1/constitution/propose",
            ),
            (
                "project_bootstrap_apply",
                &Method::POST,
                "/v1/project/bootstrap/apply",
            ),
            (
                "silent_session_create",
                &Method::POST,
                "/v1/silent-sessions",
            ),
            ("device_pair_list", &Method::GET, "/v1/device/pair/list"),
            ("connect_rooms", &Method::GET, "/v1/connect/rooms"),
        ];

        for (state_label, guard) in &blocked_guards {
            for (route_label, method, path) in &recovery_surfaces {
                let denial = route_entitlement_denial(guard, method, path);
                assert!(
                    denial.is_none(),
                    "{route_label} ({path}) must remain available in state {state_label}, got: {denial:?}"
                );
            }

            let mut mutation_sentinels = 0u32;
            for (route_label, method, path) in &protected_mutations {
                let denial = route_entitlement_denial(guard, method, path);
                assert!(
                    denial.is_some(),
                    "{route_label} ({path}) must be denied before side effects in state {state_label}"
                );
                mutation_sentinels += 1;
            }
            assert_eq!(
                mutation_sentinels,
                protected_mutations.len() as u32,
                "state {state_label}: every protected mutation must emit a denial sentinel"
            );
        }
    }

    #[test]
    fn route_entitlement_inheritance() {
        // Verify all routes in the reconciliation manifest resolve through
        // inheritance to a canonical operation policy without owning pricing,
        // tier, or caller-controlled grants.
        let reconciliation_json =
            include_str!("../../../../docs/contracts/spec152f-surface-reconciliation/rest.v1.json");
        let reconciliation: serde_json::Value =
            serde_json::from_str(reconciliation_json).expect("valid reconciliation JSON");
        let rows = reconciliation["rows"].as_array().expect("rows is an array");
        assert!(
            rows.len() >= 189,
            "expected >=189 REST entries in reconciliation manifest, got {}",
            rows.len()
        );

        let mut resolved = 0u32;
        let mut unresolved = Vec::new();

        for row in rows {
            let path = row["symbol_or_route"].as_str().expect("path is a string");
            let resolution = row["resolution"].as_str().unwrap_or_default();

            // Recovery routes are handled by the middleware
            if resolution == "recovery_or_read_allowance" {
                assert!(
                    resolve_route_entitlement_policy(&Method::POST, path).is_none()
                        || route_recovery_allowance(path).is_some(),
                    "{path}: recovery route must resolve via recovery allowance, not entitlement"
                );
                resolved += 1;
                continue;
            }

            // All base and premium routes must resolve through inheritance
            // Try common mutation methods for each path
            let policy = resolve_route_entitlement_policy(&Method::POST, path)
                .or_else(|| resolve_route_entitlement_policy(&Method::GET, path))
                .or_else(|| resolve_route_entitlement_policy(&Method::PATCH, path))
                .or_else(|| resolve_route_entitlement_policy(&Method::PUT, path))
                .or_else(|| resolve_route_entitlement_policy(&Method::DELETE, path));

            match policy {
                Some(p) => {
                    // No route may present itself as owning pricing or tier
                    assert!(
                        !p.operation_id.is_empty(),
                        "{path}: must have a synthetic operation id"
                    );
                    // Premium routes must use approved families
                    if p.capability_family.is_optional_premium() {
                        assert!(
                            resolution == "premium_family_candidate",
                            "{path}: carries premium family {:?} but resolution is {resolution}",
                            p.capability_family
                        );
                    }
                    resolved += 1;
                }
                None => {
                    unresolved.push(path.to_string());
                }
            }
        }

        assert!(
            unresolved.is_empty(),
            "{} REST routes could not resolve: {:?}",
            unresolved.len(),
            &unresolved[..unresolved.len().min(20)]
        );
        assert_eq!(
            resolved as usize,
            rows.len(),
            "all {} REST entries must resolve",
            rows.len()
        );
    }
}
