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

fn evidence(
    fingerprint: &CompactionRuntimeFingerprint,
    name: &str,
    expires_after: Option<chrono::DateTime<Utc>>,
) -> CapabilityEvidence {
    CapabilityEvidence {
        capability: name.into(),
        runtime_segment: fingerprint.segment_key.clone(),
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
        .map(|name| evidence(&fingerprint, name, Some(now + Duration::hours(1))))
        .collect();
    assert!(
        legal_action_mask(&fingerprint, &valid, now)
            .contains(&ContextManagementAction::ProviderNativeCompaction)
    );
    let expired: Vec<_> = names
        .iter()
        .map(|name| evidence(&fingerprint, name, Some(now - Duration::seconds(1))))
        .collect();
    assert!(
        !legal_action_mask(&fingerprint, &expired, now)
            .contains(&ContextManagementAction::ProviderNativeCompaction)
    );
}

#[test]
fn provider_contracts_require_exact_proof_and_round_trip_opaque_state() {
    let fingerprint = resolve_runtime_fingerprint(facts("openai", "first-party", "continuity-a"));
    let now = Utc::now();
    assert_eq!(
        provider_strategy(&fingerprint, &[], now),
        ProviderStrategy::PiStructuredFallback
    );
    let names = [
        "first_party_openai_identity",
        "openai_opaque_compaction_request",
        "openai_opaque_compaction_item_round_trip",
        "reasoning_state_round_trip",
        "continuation_survives_process_resume",
        "continuation_survives_transport_fallback",
        "previous_response_continuation",
    ];
    let evidence: Vec<_> = names
        .iter()
        .map(|name| evidence(&fingerprint, name, None))
        .collect();
    let without_transport: Vec<_> = evidence
        .iter()
        .filter(|item| item.capability != "continuation_survives_transport_fallback")
        .cloned()
        .collect();
    assert_eq!(
        provider_strategy(&fingerprint, &without_transport, now),
        ProviderStrategy::PiStructuredFallback
    );
    assert_eq!(
        provider_strategy(&fingerprint, &evidence, now),
        ProviderStrategy::OpenAiOpaqueCompaction
    );
    let gateway =
        resolve_runtime_fingerprint(facts("openai", "compatible-gateway", "continuity-a"));
    assert_eq!(
        provider_strategy(&gateway, &evidence, now),
        ProviderStrategy::PiStructuredFallback
    );
    for provider in ["local", "unknown", "openrouter"] {
        let fallback =
            resolve_runtime_fingerprint(facts(provider, "compatible-gateway", "continuity-a"));
        assert_eq!(
            provider_strategy(&fallback, &[], now),
            ProviderStrategy::PiStructuredFallback
        );
    }
    let state = ProviderContinuationState::OpenAi(OpenAiCompactionState {
        opaque_compaction_item: vec![0, 255, 17, 33],
        encrypted_reasoning_items: vec![vec![9, 8, 7], vec![0, 1, 2]],
        previous_response_id: Some("response-1".into()),
        full_output_replay: vec![vec![6, 5, 4]],
    });
    let persisted = serde_json::to_vec(&state).unwrap();
    let replayed: ProviderContinuationState = serde_json::from_slice(&persisted).unwrap();
    assert_eq!(replayed, state);
}

#[test]
fn anthropic_and_gemini_contracts_preserve_cost_and_signature_truth() {
    let now = Utc::now();
    let fingerprint =
        resolve_runtime_fingerprint(facts("anthropic", "first-party", "continuity-a"));
    let anthropic_names = [
        "anthropic_compaction_request",
        "anthropic_compaction_block_round_trip",
        "anthropic_stop_reason_compaction",
        "anthropic_usage_iterations",
        "reasoning_state_round_trip",
        "continuation_survives_process_resume",
        "continuation_survives_transport_fallback",
    ];
    let anthropic_evidence: Vec<_> = anthropic_names
        .iter()
        .map(|name| evidence(&fingerprint, name, None))
        .collect();
    assert_eq!(
        provider_strategy(&fingerprint, &anthropic_evidence, now),
        ProviderStrategy::AnthropicServerCompaction
    );
    let usage = aggregate_anthropic_usage(&[
        ProviderUsage {
            input_tokens: 10,
            output_tokens: 3,
            cache_read_tokens: 5,
            cache_write_tokens: 2,
        },
        ProviderUsage {
            input_tokens: 20,
            output_tokens: 4,
            cache_read_tokens: 8,
            cache_write_tokens: 1,
        },
    ]);
    assert_eq!(usage.input_tokens, 30);
    assert_eq!(usage.cache_read_tokens, 13);
    let anthropic_state = ProviderContinuationState::Anthropic(AnthropicCompactionState {
        beta_revision: "compact-2026-01-12".into(),
        compaction_block: vec![0, 128, 255, 3],
        stop_reason: "compaction".into(),
        usage_iterations: vec![usage],
    });
    assert_eq!(
        serde_json::from_slice::<ProviderContinuationState>(
            &serde_json::to_vec(&anthropic_state).unwrap()
        )
        .unwrap(),
        anthropic_state
    );
    let tools = vec![
        ToolResultState {
            tool_call_id: "evidence".into(),
            tokens: 10_000,
            action_critical: false,
            evidence_critical: true,
            active_blocker: false,
        },
        ToolResultState {
            tool_call_id: "noise".into(),
            tokens: 30_000,
            action_critical: false,
            evidence_critical: false,
            active_blocker: false,
        },
    ];
    let (profitable, protected) = tool_edit_break_even(
        &tools,
        CacheCostObservation {
            clearable_tokens: 30_000,
            cache_rewrite_tokens: 5_000,
            edit_overhead_tokens: 1_000,
        },
    );
    assert!(profitable);
    assert_eq!(protected, vec!["evidence"]);
    let gemini_fingerprint =
        resolve_runtime_fingerprint(facts("google", "first-party", "continuity-a"));
    let gemini_evidence: Vec<_> = [
        "previous_interaction_continuation",
        "gemini_request_scoped_config_replay",
        "thought_signature_round_trip",
    ]
    .iter()
    .map(|name| evidence(&gemini_fingerprint, name, None))
    .collect();
    assert_eq!(
        provider_strategy(&gemini_fingerprint, &gemini_evidence, now),
        ProviderStrategy::GeminiStatefulInteraction
    );
    let gemini = ProviderContinuationState::Gemini(GeminiContinuationState {
        previous_interaction_id: Some("interaction-1".into()),
        thought_signatures: vec![vec![0, 7, 255]],
        parallel_call_signatures: std::collections::BTreeMap::from([
            ("call-a".into(), vec![1, 2]),
            ("call-b".into(), vec![3, 4]),
        ]),
        request_scoped_tools_digest: "sha256:tools".into(),
        system_instruction_digest: "sha256:system".into(),
        generation_config_digest: "sha256:generation".into(),
    });
    assert_eq!(
        serde_json::from_slice::<ProviderContinuationState>(&serde_json::to_vec(&gemini).unwrap())
            .unwrap(),
        gemini
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
