use serde_json::json;

use crate::silent_sessions::*;

pub(super) fn requested() -> SilentSessionConfig {
    SilentSessionConfig::new(
        IdentityConfig {
            display_name: "proof".into(),
            project_root: "/srv/focusa".into(),
            continuity_id: "proof-continuity".into(),
            work_item_ref: Some("focusa-proof".into()),
            mission: "prove config".into(),
            agent_identity_ref: "agent:pi".into(),
            role_profile_ref: None,
        },
        HarnessConfig {
            kind: HarnessKind::Pi,
            adapter_version: "1".into(),
            native_resume_policy: NativeResumePolicy::Prefer,
        },
        ModelConfig {
            provider: "provider-a".into(),
            model: "model-a".into(),
            thinking: None,
            selection_policy: ModelSelectionPolicy::Exact,
            fallback_policy: ModelFallbackPolicy::Disabled,
            allowed_fallbacks: Vec::new(),
            auth_profile_ref: "auth:proof".into(),
            require_entitlement_preflight: true,
            require_runtime_model_confirmation: true,
        },
    )
}

fn layer(kind: ConfigLayerKind, source: &str, values: serde_json::Value) -> ConfigLayer {
    ConfigLayer {
        kind,
        source_ref: source.into(),
        values,
        locks: Vec::new(),
    }
}

#[test]
fn precedence_provenance_hash_and_mutation_classes_are_deterministic() {
    let layers = vec![
        layer(
            ConfigLayerKind::ExecutionProfile,
            "profile:local-pi",
            json!({"model":{"provider":"provider-b"}}),
        ),
        layer(
            ConfigLayerKind::BehavioralPreset,
            "preset:audit",
            json!({"notifications":{"channels":["operator"]}}),
        ),
        layer(
            ConfigLayerKind::SessionRequest,
            "request:1",
            json!({"model":{"model":"model-b"},"output":{"operator_projection_budget":2048}}),
        ),
    ];
    let first = resolve_silent_session_config(requested(), layers.clone()).unwrap();
    let second = resolve_silent_session_config(requested(), layers).unwrap();
    assert_eq!(first.redacted_config_hash, second.redacted_config_hash);
    assert_eq!(first.resolved_effective_config.model.provider, "provider-b");
    assert_eq!(first.resolved_effective_config.model.model, "model-b");
    assert_eq!(
        first.field_provenance["model.model"].layer,
        ConfigLayerKind::SessionRequest
    );
    assert!(
        first
            .restart_required_fields
            .contains(&"model.provider".into())
    );
    assert_eq!(
        mutation_class("notifications.channels"),
        ConfigMutationClass::HotMutable
    );
    assert_eq!(
        mutation_class("model.model"),
        ConfigMutationClass::RestartRequired
    );
    assert_eq!(
        mutation_class("identity.continuity_id"),
        ConfigMutationClass::Immutable
    );
}

#[test]
fn locks_precedence_and_secret_values_fail_closed() {
    let mut locked = layer(
        ConfigLayerKind::ProjectPolicy,
        "project:policy",
        json!({"model":{"provider":"provider-a"}}),
    );
    locked.locks.push(ConfigPolicyLock {
        field_path: "model.provider".into(),
        expected_value: json!("provider-a"),
        reason: "exact provider required".into(),
    });
    let override_layer = layer(
        ConfigLayerKind::SessionRequest,
        "request:unsafe",
        json!({"model":{"provider":"provider-c"}}),
    );
    assert!(matches!(
        resolve_silent_session_config(requested(), vec![locked, override_layer]),
        Err(ConfigResolutionError::PolicyLockViolation { .. })
    ));

    let raw_secret = layer(
        ConfigLayerKind::SessionRequest,
        "request:secret",
        json!({"model":{"api_token":"plaintext"}}),
    );
    assert!(matches!(
        resolve_silent_session_config(requested(), vec![raw_secret]),
        Err(ConfigResolutionError::InvalidEffectiveConfig(_))
    ));

    let unsorted = vec![
        layer(ConfigLayerKind::SessionRequest, "request", json!({})),
        layer(ConfigLayerKind::ExecutionProfile, "profile", json!({})),
    ];
    assert_eq!(
        resolve_silent_session_config(requested(), unsorted),
        Err(ConfigResolutionError::PrecedenceOrder)
    );
}
