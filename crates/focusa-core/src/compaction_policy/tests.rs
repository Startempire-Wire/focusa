use super::*;
use chrono::{Duration, Utc};

fn facts(provider: &str, endpoint: &str, continuity: &str) -> CompactionRuntimeFacts {
    CompactionRuntimeFacts {
        provider_raw: Some(provider.into()),
        api: Some("responses".into()),
        model_id_raw: Some("model-x".into()),
        response_model: None,
        endpoint_class: Some(endpoint.into()),
        api_version: Some("v1".into()),
        beta_features: vec![],
        adapter_revision: "adapter-1".into(),
        capability_evidence_revision: "caps-1".into(),
        context_window: Some(200_000),
        max_output_tokens: Some(16_384),
        reasoning_enabled: Some(true),
        transport: Some("https".into()),
        state_mode: Some("stateful".into()),
        cache_mode: Some("explicit".into()),
        harness_mode: Some("pi".into()),
        objective_profile: Some("daily_driver".into()),
        session_id: "session-1".into(),
        attachment_id: "attachment-1".into(),
        project_root: Some("/srv/project".into()),
        continuity_id: Some(continuity.into()),
    }
}

fn evidence(name: &str, expires_after: Option<chrono::DateTime<Utc>>) -> CapabilityEvidence {
    CapabilityEvidence {
        capability: name.into(),
        state: CapabilityState::Proven,
        source: "adapter_conformance".into(),
        adapter_revision: "adapter-1".into(),
        proof_ref: Some(format!("evidence:{name}")),
        proof_digest: Some("sha256:proof".into()),
        verified_at: Some(Utc::now()),
        expires_after,
    }
}

#[test]
fn identity_segments_gateway_and_workstream_without_raw_scope() {
    let first = resolve_runtime_fingerprint(facts("openai", "first-party", "continuity-a"));
    let gateway =
        resolve_runtime_fingerprint(facts("openai", "compatible-gateway", "continuity-a"));
    let other_workstream =
        resolve_runtime_fingerprint(facts("openai", "first-party", "continuity-b"));
    assert_eq!(first.provider_canonical.as_deref(), Some("openai"));
    assert_eq!(
        gateway.provider_canonical.as_deref(),
        Some("gateway:openai")
    );
    assert_ne!(first.segment_key, gateway.segment_key);
    assert_eq!(first.segment_key, other_workstream.segment_key);
    assert_ne!(
        first.continuity_id_hash,
        other_workstream.continuity_id_hash
    );
    let encoded = serde_json::to_string(&first).unwrap();
    assert!(!encoded.contains("/srv/project"));
    assert!(!encoded.contains("continuity-a"));
}

#[test]
fn capability_mask_is_evidence_gated_and_expiry_safe() {
    let fingerprint = resolve_runtime_fingerprint(facts("openai", "first-party", "continuity-a"));
    let now = Utc::now();
    let unknown = legal_action_mask(&fingerprint, &[], now);
    assert!(!unknown.contains(&ContextManagementAction::ProviderNativeCompaction));
    let names = [
        "openai_opaque_compaction_request",
        "openai_opaque_compaction_item_round_trip",
        "reasoning_state_round_trip",
        "continuation_survives_process_resume",
        "continuation_survives_transport_fallback",
    ];
    let valid: Vec<_> = names
        .iter()
        .map(|name| evidence(name, Some(now + Duration::hours(1))))
        .collect();
    assert!(
        legal_action_mask(&fingerprint, &valid, now)
            .contains(&ContextManagementAction::ProviderNativeCompaction)
    );
    let expired: Vec<_> = names
        .iter()
        .map(|name| evidence(name, Some(now - Duration::seconds(1))))
        .collect();
    assert!(
        !legal_action_mask(&fingerprint, &expired, now)
            .contains(&ContextManagementAction::ProviderNativeCompaction)
    );
}

#[test]
fn finite_lattice_preserves_exact_legacy_and_hard_floor() {
    let legal = std::collections::BTreeSet::from([
        ContextManagementAction::CheckpointOnly,
        ContextManagementAction::PiStructuredCompaction,
    ]);
    let lattice = compile_policy_lattice(200_000, &legal, "tightwad", Some(140_000));
    let legacy = lattice
        .iter()
        .find(|policy| policy.policy_id == "legacy_current_v1")
        .unwrap();
    assert_eq!(legacy.compact_at_tokens, Some(140_000));
    assert_eq!(legacy.hard_at_tokens, 170_000);
    assert_eq!(legacy.attempt_cooldown_ms, 60_000);
    assert_eq!(legacy.successful_compaction_cooldown_ms, 180_000);
    assert!(lattice.len() <= 8);
    assert!(
        lattice
            .iter()
            .all(|policy| policy.hard_at_tokens == 170_000)
    );
    assert!(
        lattice
            .iter()
            .filter_map(|policy| policy.compact_at_tokens)
            .all(|trigger| trigger <= 160_000)
    );
}

#[test]
fn pressure_and_semantic_signals_are_bounded_and_least_cost() {
    let mut stats = PressureStatistics::default();
    for _ in 0..300 {
        stats.observe(12_000, 8_000, 200_000);
    }
    let prediction = stats.predict(&PressurePredictionInput {
        current_context: 150_000,
        context_window: 200_000,
        configured_reserve_floor: 16_384,
        configured_reserve_percent: 10,
        max_output_tokens: Some(12_000),
        projection_budget_tokens: 900,
        persistence_growth_allowance: 2_000,
    });
    assert_eq!(prediction.sample_count, 128);
    assert!(prediction.required_reserve >= 20_000);
    assert!(prediction.predicted_peak > 150_000);
    assert!(prediction.checkpoint_at < prediction.safe_context_limit);
    let legal = std::collections::BTreeSet::from([
        ContextManagementAction::NoAction,
        ContextManagementAction::ExternalizeToolArtifacts,
        ContextManagementAction::PiStructuredCompaction,
    ]);
    assert_eq!(
        recommend_semantic_repair(
            &SemanticPressureSignals {
                tool_output_flood: true,
                ..SemanticPressureSignals::default()
            },
            &legal
        ),
        ContextManagementAction::ExternalizeToolArtifacts
    );
}

#[test]
fn selector_is_baseline_safe_and_lease_is_deterministic() {
    let legal = std::collections::BTreeSet::new();
    let mut lattice = compile_policy_lattice(200_000, &legal, "daily_driver", None);
    let candidate = lattice
        .iter_mut()
        .find(|policy| policy.policy_id != "legacy_current_v1")
        .unwrap();
    candidate.validation = ValidationState::Validated;
    let selection = |mode, measured_confidence| PolicySelectionContext {
        mode,
        context_window: 200_000,
        sample_size: 100,
        measured_confidence: Some(measured_confidence),
        minimum_samples: 20,
        required_confidence: 0.95,
        dev_fleet_enrolled: false,
    };
    let shadow = resolve_policy(&selection(PolicyMode::Shadow, 0.99), &lattice);
    assert_eq!(shadow.selected.policy_id, "legacy_current_v1");
    let adaptive = resolve_policy(&selection(PolicyMode::Adaptive, 0.99), &lattice);
    assert_ne!(adaptive.selected.policy_id, "legacy_current_v1");
    let low_confidence = resolve_policy(&selection(PolicyMode::Adaptive, 0.5), &lattice);
    assert_eq!(low_confidence.selected.policy_id, "legacy_current_v1");
    let first = CompactionPolicyLease::freeze(&adaptive, "runtime", "caps", "features");
    let second = CompactionPolicyLease::freeze(&adaptive, "runtime", "caps", "features");
    assert_eq!(first, second);
    assert_eq!(first.fallback_policy_id, "legacy_current_v1");
}

fn observation(
    segment: &str,
    workstream: &str,
    epoch: &str,
    finding: bool,
) -> CompactionPolicyObservation {
    CompactionPolicyObservation {
        schema: "focusa.compaction_policy_observation.v1".into(),
        runtime_segment: segment.into(),
        workstream_hash: workstream.into(),
        epoch_id: epoch.into(),
        policy_id: "candidate_120000_v1".into(),
        trigger_class: "proactive".into(),
        tokens_before: 150_000,
        tokens_after: Some(70_000),
        context_release_ratio: Some(0.53),
        projection_tokens: 600,
        prepare_latency_ms: Some(3),
        compaction_latency_ms: Some(90),
        verify_latency_ms: Some(4),
        first_productive_action_ms: Some(120),
        workpoint_revision_delta: 1,
        repeat_error_delta: 0,
        rehydrate_calls: 0,
        rehydrated_bytes: 0,
        hard_findings: if finding {
            vec!["scope_mismatch".into()]
        } else {
            vec![]
        },
        rollback_triggered: finding,
    }
}

#[test]
fn registry_replay_is_idempotent_bounded_and_workstream_isolated() {
    let mut registry = CompactionPolicyRegistry::new();
    registry.observe(observation("segment", "work-a", "epoch-1", false));
    registry.observe(observation("segment", "work-a", "epoch-1", false));
    registry.observe(observation("segment", "work-b", "epoch-1", true));
    assert_eq!(
        registry
            .project("segment", "work-a")
            .unwrap()
            .observation_count,
        1
    );
    assert_eq!(
        registry
            .project("segment", "work-a")
            .unwrap()
            .hard_failure_count,
        0
    );
    assert_eq!(
        registry
            .project("segment", "work-b")
            .unwrap()
            .hard_failure_count,
        1
    );
    for index in 2..=300 {
        registry.observe(observation(
            "segment",
            "work-a",
            &format!("epoch-{index}"),
            false,
        ));
    }
    assert_eq!(
        registry
            .project("segment", "work-a")
            .unwrap()
            .observations
            .len(),
        256
    );
}
