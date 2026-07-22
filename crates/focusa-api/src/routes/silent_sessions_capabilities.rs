use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use focusa_core::silent_sessions::{
    CapabilityPreflightResult, CatalogFreshness, HARNESS_CAPABILITY_NAMES, HarnessKind,
    PreflightStatus, known_harnesses, strict_unknown_preflight, unknown_harness_catalog,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{middleware::principal::ApiRequestPrincipal, server::AppState};

use super::{
    silent_sessions::{
        ApiResponse, disclose_principal_side_effect, durable_request_principal, failure,
    },
    silent_sessions_contract::SilentSessionApiEnvelope,
};

#[derive(Debug, Deserialize)]
struct HarnessPreflightBody {
    #[serde(default = "default_strict")]
    strict: bool,
    #[serde(default)]
    required_capabilities: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ModelPreflightBody {
    model: String,
    thinking: Option<String>,
    #[serde(default = "default_strict")]
    strict: bool,
    #[serde(default)]
    require_entitlement_preflight: bool,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/silent-sessions/capabilities", get(all_capabilities))
        .route("/v1/harnesses", get(harnesses))
        .route(
            "/v1/harnesses/{harness}/capabilities",
            get(harness_capabilities),
        )
        .route("/v1/harnesses/{harness}/preflight", post(harness_preflight))
        .route("/v1/providers", get(providers))
        .route("/v1/providers/{provider}/models", get(provider_models))
        .route(
            "/v1/providers/{provider}/models/preflight",
            post(model_preflight),
        )
}

async fn all_capabilities(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResponse {
    authenticated_unknown(
        &state,
        &headers,
        "capabilities_unknown",
        json!({
            "harnesses": known_harnesses()
                .into_iter()
                .map(unknown_harness_catalog)
                .collect::<Vec<_>>(),
            "providers": [],
            "provider_catalog_freshness": CatalogFreshness::Unknown,
            "reason": "no timestamped adapter or provider probes are registered"
        }),
    )
    .await
}

async fn harnesses(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResponse {
    authenticated_unknown(
        &state,
        &headers,
        "harness_catalog_unknown",
        json!({
            "harnesses": known_harnesses().into_iter().map(|harness| json!({
                "harness": harness,
                "identity_known": true,
                "availability": "unknown",
                "freshness": CatalogFreshness::Unknown
            })).collect::<Vec<_>>()
        }),
    )
    .await
}

async fn harness_capabilities(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(harness): Path<String>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let Some(harness) = parse_harness(&harness) else {
        return after(not_found("harness"), &principal);
    };
    unknown(
        "harness_capabilities_unknown",
        json!(unknown_harness_catalog(harness)),
        &principal,
    )
}

async fn harness_preflight(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(harness): Path<String>,
    Json(body): Json<HarnessPreflightBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    let Some(harness) = parse_harness(&harness) else {
        return after(not_found("harness"), &principal);
    };
    let checks = if body.required_capabilities.is_empty() {
        vec!["adapter_registered"]
    } else {
        body.required_capabilities
            .iter()
            .map(String::as_str)
            .collect()
    };
    if checks.iter().any(|name| !valid_harness_check(name)) {
        return after(invalid_harness_requirement(), &principal);
    }
    let result = unknown_preflight(body.strict, &checks);
    unknown(
        preflight_status(&result),
        json!({"harness": harness, "result": result}),
        &principal,
    )
}

async fn providers(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResponse {
    authenticated_unknown(
        &state,
        &headers,
        "provider_catalog_unknown",
        json!({
            "providers": [],
            "freshness": CatalogFreshness::Unknown,
            "reason": "no canonical provider registry or fresh entitlement probe exists"
        }),
    )
    .await
}

async fn provider_models(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider): Path<String>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    unknown(
        "model_catalog_unknown",
        json!({
            "provider": provider,
            "provider_configured": "unknown",
            "authentication": {"available": "unknown", "type": "unknown"},
            "entitlement": "unknown",
            "models": [],
            "freshness": CatalogFreshness::Unknown,
            "reason": "no fresh canonical provider model probe exists"
        }),
        &principal,
    )
}

async fn model_preflight(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Json(body): Json<ModelPreflightBody>,
) -> ApiResponse {
    let principal = match durable_request_principal(&state, &headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    if provider.trim().is_empty() || body.model.trim().is_empty() {
        return after(invalid_request(), &principal);
    }
    let mut checks = vec![
        "provider_configured",
        "authentication_available",
        "authentication_type",
        "exact_model_availability",
        "context_window_compatibility",
        "rate_limit_posture",
        "billing_or_usage_budget_posture",
        "model_catalog_freshness",
    ];
    if body.require_entitlement_preflight {
        checks.push("subscription_or_api_entitlement");
    }
    if body.thinking.is_some() {
        checks.push("thinking_level_support");
    }
    let result = unknown_preflight(body.strict, &checks);
    unknown(
        preflight_status(&result),
        json!({
            "provider": provider,
            "model": body.model,
            "thinking": body.thinking,
            "result": result
        }),
        &principal,
    )
}

fn unknown_preflight(strict: bool, checks: &[&str]) -> CapabilityPreflightResult {
    let mut result = strict_unknown_preflight(checks);
    if !strict {
        result.strict = false;
        result.status = PreflightStatus::Unknown;
    }
    result
}

fn preflight_status(result: &CapabilityPreflightResult) -> &'static str {
    match result.status {
        PreflightStatus::Passed => "preflight_passed",
        PreflightStatus::Blocked => "preflight_blocked",
        PreflightStatus::Degraded => "preflight_degraded",
        PreflightStatus::Unknown => "preflight_unknown",
    }
}

async fn authenticated_unknown(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    status: &str,
    data: Value,
) -> ApiResponse {
    let principal = match durable_request_principal(state, headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    unknown(status, data, &principal)
}

fn unknown(status: &str, data: Value, principal: &ApiRequestPrincipal) -> ApiResponse {
    let mut envelope = SilentSessionApiEnvelope::canonical(status, data);
    envelope.degraded = true;
    envelope.failure_class = Some("probe_unavailable".into());
    envelope.recovery_hint = Some("Register and run a timestamped canonical probe.".into());
    after((StatusCode::OK, Json(envelope)), principal)
}

fn parse_harness(value: &str) -> Option<HarnessKind> {
    match value {
        "pi" => Some(HarnessKind::Pi),
        "codex" => Some(HarnessKind::Codex),
        "claude" => Some(HarnessKind::Claude),
        "opencode" => Some(HarnessKind::Opencode),
        "generic_rpc" => Some(HarnessKind::GenericRpc),
        "generic_pty" => Some(HarnessKind::GenericPty),
        _ => None,
    }
}

fn valid_harness_check(name: &str) -> bool {
    name == "adapter_registered" || HARNESS_CAPABILITY_NAMES.contains(&name)
}

fn default_strict() -> bool {
    true
}

fn invalid_harness_requirement() -> ApiResponse {
    failure(
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "validation_rejected",
        "required_capabilities contains an unknown capability name",
    )
}

fn invalid_request() -> ApiResponse {
    failure(
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "validation_rejected",
        "provider and model must be non-empty",
    )
}

fn not_found(target: &str) -> ApiResponse {
    failure(
        StatusCode::NOT_FOUND,
        "not_found",
        "not_found",
        &format!("No known {target} identity matches the requested value."),
    )
}

fn after(response: ApiResponse, principal: &ApiRequestPrincipal) -> ApiResponse {
    disclose_principal_side_effect(response, principal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_identity_parser_is_exact() {
        assert_eq!(parse_harness("pi"), Some(HarnessKind::Pi));
        assert_eq!(parse_harness("Pi"), None);
        assert_eq!(parse_harness("unknown"), None);
    }

    #[test]
    fn harness_requirement_names_are_closed_world() {
        assert!(valid_harness_check("adapter_registered"));
        assert!(valid_harness_check("model_preflight"));
        assert!(!valid_harness_check("model_magic"));
    }

    #[test]
    fn strict_unknown_blocks_but_relaxed_unknown_never_allows_mutation() {
        let strict = unknown_preflight(true, &["entitlement"]);
        let relaxed = unknown_preflight(false, &["entitlement"]);
        assert_eq!(strict.status, PreflightStatus::Blocked);
        assert_eq!(relaxed.status, PreflightStatus::Unknown);
        assert!(!strict.mutation_allowed);
        assert!(!relaxed.mutation_allowed);
    }
}
