use std::{collections::BTreeMap, env, path::Path as FsPath, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use chrono::{Duration, Utc};
use focusa_core::silent_sessions::{
    CapabilityFact, CapabilityFactState, CapabilityPreflightResult, CatalogFreshness,
    HARNESS_CAPABILITY_NAMES, HarnessCapabilityCatalog, HarnessKind, PreflightCheck,
    PreflightStatus, known_harnesses, strict_unknown_preflight, unknown_harness_catalog,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{middleware::principal::ApiRequestPrincipal, server::AppState};

use super::{
    silent_sessions::{
        ApiResponse, disclose_principal_side_effect, durable_request_principal, failure,
    },
    silent_sessions_contract::{RetryDirective, SilentSessionApiEnvelope},
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

const PI_OBSERVED_CAPABILITIES: &[&str] = &[
    "structured_events",
    "model_preflight",
    "native_session_resume",
    "prompt_delivery",
    "steering",
    "followup_queue",
    "native_abort",
];

fn command_available(name: &str) -> bool {
    command_available_on(
        &env::split_paths(&env::var_os("PATH").unwrap_or_default()).collect::<Vec<_>>(),
        name,
    )
}

/// Pure probe: is `name` an executable anywhere on the given path list?
/// Split out so the #195 catalog-probe behavior is deterministically
/// testable without mutating process env.
fn command_available_on(paths: &[std::path::PathBuf], name: &str) -> bool {
    paths.iter().any(|directory| {
        let candidate = directory.join(name);
        candidate.is_file() && executable(&candidate)
    })
}

#[cfg(unix)]
fn executable(path: &FsPath) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn executable(path: &FsPath) -> bool {
    path.is_file()
}

fn observed_fact(state: CapabilityFactState, source: &str) -> CapabilityFact {
    let observed_at = Utc::now();
    CapabilityFact {
        state,
        source: source.to_string(),
        observed_at: Some(observed_at),
        expires_at: Some(observed_at + Duration::minutes(5)),
        freshness: CatalogFreshness::Fresh,
    }
}

fn observed_pi_catalog() -> HarnessCapabilityCatalog {
    HarnessCapabilityCatalog {
        harness: HarnessKind::Pi,
        adapter_registered: observed_fact(CapabilityFactState::Supported, "local_path_probe"),
        capabilities: HARNESS_CAPABILITY_NAMES
            .iter()
            .map(|name| {
                let state = if PI_OBSERVED_CAPABILITIES.contains(name) {
                    CapabilityFactState::Supported
                } else {
                    CapabilityFactState::Unknown
                };
                (
                    (*name).to_string(),
                    observed_fact(state, "focusa_pi_rpc_adapter"),
                )
            })
            .collect::<BTreeMap<_, _>>(),
        catalog_freshness: CatalogFreshness::Fresh,
    }
}

fn observed_harness_preflight(strict: bool, checks: &[&str]) -> CapabilityPreflightResult {
    let rows = checks
        .iter()
        .map(|name| {
            let supported =
                *name == "adapter_registered" || PI_OBSERVED_CAPABILITIES.contains(name);
            PreflightCheck {
                name: (*name).to_string(),
                state: if supported {
                    CapabilityFactState::Supported
                } else {
                    CapabilityFactState::Unknown
                },
                required: true,
                reason: if supported {
                    "observed local Pi RPC adapter capability".into()
                } else {
                    "capability is not established by the bounded local probe".into()
                },
            }
        })
        .collect::<Vec<_>>();
    let all_supported = rows
        .iter()
        .all(|row| row.state == CapabilityFactState::Supported);
    CapabilityPreflightResult {
        status: if all_supported {
            PreflightStatus::Passed
        } else if strict {
            PreflightStatus::Blocked
        } else {
            PreflightStatus::Unknown
        },
        strict,
        checks: rows,
        catalog_freshness: CatalogFreshness::Fresh,
        mutation_allowed: all_supported,
    }
}

fn configured_models() -> Vec<String> {
    env::var("FOCUSA_PI_SUPPORTED_MODELS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .map(str::to_string)
        .collect()
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
    if command_available("pi") {
        return authenticated_observed(
            &state,
            &headers,
            "capabilities_observed",
            json!({
                "harnesses": [observed_pi_catalog()],
                "providers": [{
                    "provider": "pi-runtime",
                    "catalog_status": "ready",
                    "auth_status": "delegated_to_harness",
                    "entitlement_status": "model_preflight_required",
                    "capability_status": "supported"
                }],
                "provider_catalog_freshness": CatalogFreshness::Fresh
            }),
        )
        .await;
    }
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
            "reason": "Pi RPC adapter is not executable on the daemon PATH"
        }),
    )
    .await
}

async fn harnesses(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResponse {
    if command_available("pi") {
        return authenticated_observed(
            &state,
            &headers,
            "harness_catalog_observed",
            json!({"harnesses": [{
                "harness": "pi",
                "identity_known": true,
                "availability": "available",
                "freshness": CatalogFreshness::Fresh
            }]}),
        )
        .await;
    }
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
    if harness == HarnessKind::Pi && command_available("pi") {
        return observed(
            "harness_capabilities_observed",
            json!(observed_pi_catalog()),
            &principal,
        );
    }
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
    if harness == HarnessKind::Pi && command_available("pi") {
        let result = observed_harness_preflight(body.strict, &checks);
        return observed(
            preflight_status(&result),
            json!({"harness": harness, "result": result}),
            &principal,
        );
    }
    let result = unknown_preflight(body.strict, &checks);
    unknown(
        preflight_status(&result),
        json!({"harness": harness, "result": result}),
        &principal,
    )
}

async fn providers(State(state): State<Arc<AppState>>, headers: HeaderMap) -> ApiResponse {
    if command_available("pi") {
        return authenticated_observed(
            &state,
            &headers,
            "provider_catalog_observed",
            json!({
                "providers": [{
                    "provider": "pi-runtime",
                    "catalog_status": "ready",
                    "auth_status": "delegated_to_harness",
                    "entitlement_status": "model_preflight_required",
                    "capability_status": "supported"
                }],
                "freshness": CatalogFreshness::Fresh
            }),
        )
        .await;
    }
    authenticated_unknown(
        &state,
        &headers,
        "provider_catalog_unknown",
        json!({
            "providers": [],
            "freshness": CatalogFreshness::Unknown,
            "reason": "Pi RPC adapter is not executable on the daemon PATH"
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
    if provider != "pi-runtime" || !command_available("pi") {
        return model_unsupported(&provider, None, &principal);
    }
    observed(
        "model_catalog_observed",
        json!({
            "provider": provider,
            "provider_configured": true,
            "authentication": {"available": "delegated", "type": "pi_harness"},
            "entitlement": "preflight_required",
            "models": configured_models(),
            "freshness": CatalogFreshness::Fresh
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
    let supported = provider == "pi-runtime"
        && command_available("pi")
        && configured_models()
            .iter()
            .any(|model| model.eq_ignore_ascii_case(body.model.trim()));
    if !supported {
        return model_unsupported(&provider, Some(&body.model), &principal);
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
    let result = CapabilityPreflightResult {
        status: PreflightStatus::Passed,
        strict: body.strict,
        checks: checks
            .into_iter()
            .map(|name| PreflightCheck {
                name: name.to_string(),
                state: CapabilityFactState::Supported,
                required: true,
                reason: "server-owned Pi model allowlist matched".into(),
            })
            .collect(),
        catalog_freshness: CatalogFreshness::Fresh,
        mutation_allowed: true,
    };
    observed(
        "preflight_passed",
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

async fn authenticated_observed(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    status: &str,
    data: Value,
) -> ApiResponse {
    let principal = match durable_request_principal(state, headers).await {
        Ok(principal) => principal,
        Err(response) => return *response,
    };
    observed(status, data, &principal)
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

fn observed(status: &str, data: Value, principal: &ApiRequestPrincipal) -> ApiResponse {
    after(
        (
            StatusCode::OK,
            Json(SilentSessionApiEnvelope::canonical(status, data)),
        ),
        principal,
    )
}

fn unknown(status: &str, data: Value, principal: &ApiRequestPrincipal) -> ApiResponse {
    let mut envelope = SilentSessionApiEnvelope::canonical(status, data);
    envelope.degraded = true;
    envelope.failure_class = Some("probe_unavailable".into());
    envelope.recovery_hint = Some("Register and run a timestamped canonical probe.".into());
    after((StatusCode::OK, Json(envelope)), principal)
}

fn model_unsupported(
    provider: &str,
    model: Option<&str>,
    principal: &ApiRequestPrincipal,
) -> ApiResponse {
    let mut envelope = SilentSessionApiEnvelope::<Value>::failure(
        "model_unsupported",
        "unsupported_model",
        RetryDirective {
            retryable: false,
            after_ms: None,
            idempotency_key_required: false,
        },
    );
    envelope.recovery_hint = Some(
        "Select a model from GET /v1/providers/pi-runtime/models or configure the server-owned FOCUSA_PI_SUPPORTED_MODELS allowlist."
            .into(),
    );
    envelope.misuse_hint = Some(format!(
        "Provider/model is not available to this runtime: {provider}/{}",
        model.unwrap_or("<catalog>")
    ));
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

    #[test]
    fn catalog_probe_finds_harness_only_on_executable_path() {
        // #195 regression: the daemon catalog is probe-derived, so a harness
        // on the daemon PATH must resolve to observed, not unknown.
        let dir = std::env::temp_dir().join(format!("focusa-probe-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("fake-harness");
        std::fs::write(&bin, "#!/bin/sh\nexit 0").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert!(command_available_on(
            std::slice::from_ref(&dir),
            "fake-harness"
        ));
        // A non-executable sibling must NOT satisfy the probe.
        let plain = dir.join("plain-harness");
        std::fs::write(&plain, "#!/bin/sh\nexit 0").unwrap();
        assert!(!command_available_on(
            std::slice::from_ref(&dir),
            "plain-harness"
        ));
        assert!(!command_available_on(
            std::slice::from_ref(&dir),
            "missing-harness"
        ));
        assert!(!command_available_on(&[], "fake-harness"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn observed_pi_catalog_reports_supported_adapter_and_capabilities() {
        let catalog = observed_pi_catalog();
        assert_eq!(catalog.harness, HarnessKind::Pi);
        assert_eq!(
            catalog.adapter_registered.state,
            CapabilityFactState::Supported
        );
        for name in PI_OBSERVED_CAPABILITIES {
            assert_eq!(
                catalog.capabilities.get(*name).map(|fact| fact.state),
                Some(CapabilityFactState::Supported),
                "capability {name} must probe as supported"
            );
        }
        assert_eq!(catalog.catalog_freshness, CatalogFreshness::Fresh);
    }
}
