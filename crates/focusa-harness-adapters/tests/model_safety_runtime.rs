use chrono::{Duration, Utc};
use focusa_core::silent_session::{
    ModelBinding, ModelFallbackPolicy, ModelSelectionPolicy, SilentSessionId,
    SilentSessionModelConfig, SilentSessionRun, SilentSessionRunId, SilentSessionVersions,
    WorkspaceBinding, WorkspaceStrategy,
};
use focusa_core::silent_session_protocol::CapabilitySupport;
use focusa_harness_adapters::*;
use std::path::PathBuf;

fn model(name: &str) -> ModelBinding {
    ModelBinding {
        provider: "openai-codex".into(),
        model: name.into(),
        thinking: Some("high".into()),
    }
}

fn run(requested: ModelBinding) -> SilentSessionRun {
    SilentSessionRun {
        schema: focusa_core::silent_session::SILENT_SESSION_RUN_SCHEMA.into(),
        versions: SilentSessionVersions::default(),
        run_id: SilentSessionRunId::new(),
        session_id: SilentSessionId::new(),
        generation: 1,
        runner_id: "runner:test".into(),
        adapter_id: PI_RPC_ADAPTER_ID.into(),
        process_backend_id: "posix_direct.v1".into(),
        requested_model_binding: requested,
        effective_model_binding: None,
        observed_model_binding: None,
        workspace_binding: WorkspaceBinding {
            workspace_id: "workspace:test".into(),
            root: PathBuf::from("/projects/focusa-worktree"),
            strategy: WorkspaceStrategy::IsolatedWorktree,
            branch_ref: Some("focusa/model-safety".into()),
        },
        process_identity: None,
        harness_native_session_ref: None,
        started_at: None,
        ended_at: None,
        exit_status: None,
        current_event_seq: 0,
        output_stream_refs: vec![],
        runtime_checkpoint_refs: vec![],
        workpoint_checkpoint_refs: vec![],
    }
}

fn config() -> SilentSessionModelConfig {
    SilentSessionModelConfig {
        requested: model("gpt-exact"),
        selection_policy: ModelSelectionPolicy::Exact,
        fallback_policy: ModelFallbackPolicy::Disabled,
        allowed_fallbacks: vec![],
        auth_profile_ref: "auth:operator-subscription".into(),
        require_entitlement_preflight: true,
        require_runtime_model_confirmation: true,
    }
}

fn evidence(candidate: ModelBinding, status: PreflightStatus) -> ModelPreflightEvidence {
    let now = Utc::now();
    ModelPreflightEvidence {
        schema: MODEL_PREFLIGHT_EVIDENCE_SCHEMA.into(),
        candidate,
        auth_profile_ref: "auth:operator-subscription".into(),
        checks: ALL_PROVIDER_CHECKS
            .into_iter()
            .map(|kind| ProviderCheckEvidence {
                kind,
                status,
                source_ref: format!("probe:{kind:?}"),
                observed_at: now - Duration::seconds(1),
                fresh_until: now + Duration::minutes(5),
                detail: "deterministic test probe".into(),
            })
            .collect(),
    }
}

#[test]
fn strict_preflight_blocks_unknown_entitlement_and_never_grants_mutation() {
    let now = Utc::now();
    let config = config();
    let capabilities = HarnessCapabilities::all(CapabilitySupport::Native);
    let mut evidence = evidence(config.requested.clone(), PreflightStatus::Passed);
    evidence
        .checks
        .iter_mut()
        .find(|check| check.kind == ProviderCheckKind::SubscriptionOrApiEntitlement)
        .unwrap()
        .status = PreflightStatus::Unknown;

    let verdict = evaluate_model_preflight(&config, &capabilities, &evidence, None, now).unwrap();
    assert_eq!(verdict.status, PreflightStatus::Blocked);
    assert!(!verdict.launch_allowed);
    assert!(!verdict.mutation_allowed);
    assert_eq!(verdict.event_kind, "model.preflight_blocked");
    assert_eq!(
        verdict.blocking_checks,
        vec![ProviderCheckKind::SubscriptionOrApiEntitlement]
    );
}

#[test]
fn exact_model_requires_requested_effective_and_observed_truth() {
    let now = Utc::now();
    let config = config();
    let capabilities = HarnessCapabilities::all(CapabilitySupport::Native);
    let evidence = evidence(config.requested.clone(), PreflightStatus::Passed);
    let preflight = evaluate_model_preflight(&config, &capabilities, &evidence, None, now).unwrap();
    assert_eq!(preflight.status, PreflightStatus::Passed);
    assert!(preflight.launch_allowed);
    assert!(!preflight.mutation_allowed);

    let mut run = run(config.requested.clone());
    apply_model_preflight_to_run(&mut run, &preflight).unwrap();
    assert_eq!(run.effective_model_binding, Some(config.requested.clone()));
    assert_eq!(run.observed_model_binding, None);

    let mismatch = model("ambient-default");
    let runtime = confirm_runtime_model(&RuntimeModelConfirmationRequest {
        config: &config,
        preflight: &preflight,
        effective: Some(&config.requested),
        observed: Some(&mismatch),
        harness_connected: true,
        bootstrap_verified: true,
        writer_lease_valid: true,
        context_authority_fresh: true,
    });
    assert_eq!(runtime.status, RuntimeModelStatus::Mismatch);
    assert_eq!(runtime.event_kind, "model.mismatch");
    assert!(!runtime.mutation_allowed);
    assert!(runtime.controlled_abort_required);
    assert!(runtime.blocked_state_required);
    assert!(runtime.operator_notification_required);
    apply_runtime_confirmation_to_run(&mut run, &runtime).unwrap();
    assert_eq!(run.observed_model_binding, Some(mismatch));

    let confirmed = confirm_runtime_model(&RuntimeModelConfirmationRequest {
        config: &config,
        preflight: &preflight,
        effective: Some(&config.requested),
        observed: Some(&config.requested),
        harness_connected: true,
        bootstrap_verified: true,
        writer_lease_valid: true,
        context_authority_fresh: true,
    });
    assert_eq!(confirmed.status, RuntimeModelStatus::Confirmed);
    assert!(confirmed.mutation_allowed);
    assert!(!confirmed.controlled_abort_required);
    apply_runtime_confirmation_to_run(&mut run, &confirmed).unwrap();
    assert_eq!(run.observed_model_binding, Some(config.requested.clone()));
}

#[test]
fn fallback_requires_non_exact_policy_allowlist_trigger_and_notification() {
    let now = Utc::now();
    let fallback_model = model("gpt-allowed-fallback");
    let mut config = config();
    config.selection_policy = ModelSelectionPolicy::AllowList;
    config.fallback_policy = ModelFallbackPolicy::ExplicitAllowList;
    config.allowed_fallbacks = vec![fallback_model.clone()];
    let fallback = ModelFallbackAttempt {
        trigger: ModelFallbackTrigger::RateLimited,
        trigger_evidence_ref: "rate-limit:requested-model".into(),
        candidate: fallback_model.clone(),
        operator_notification_ref: "notification:model-fallback".into(),
    };
    let evidence = evidence(fallback_model.clone(), PreflightStatus::Passed);
    let verdict = evaluate_model_preflight(
        &config,
        &HarnessCapabilities::all(CapabilitySupport::Native),
        &evidence,
        Some(&fallback),
        now,
    )
    .unwrap();
    assert_eq!(verdict.selected, Some(fallback_model));
    assert_eq!(verdict.fallback, Some(fallback));
    assert!(verdict.operator_notification_required);
    assert!(!verdict.mutation_allowed);

    let mut denied = config.clone();
    denied.allowed_fallbacks = vec![model("different-model")];
    assert_eq!(
        evaluate_model_preflight(
            &denied,
            &HarnessCapabilities::all(CapabilitySupport::Native),
            &evidence,
            verdict.fallback.as_ref(),
            now,
        ),
        Err(ModelSafetyError::FallbackNotAllowlisted)
    );

    let mut exact = config;
    exact.selection_policy = ModelSelectionPolicy::Exact;
    assert_eq!(
        evaluate_model_preflight(
            &exact,
            &HarnessCapabilities::all(CapabilitySupport::Native),
            &evidence,
            verdict.fallback.as_ref(),
            now,
        ),
        Err(ModelSafetyError::ExactSelectionForbidsFallback)
    );
}

#[test]
fn preflight_rejects_missing_or_stale_evidence() {
    let now = Utc::now();
    let config = config();
    let capabilities = HarnessCapabilities::all(CapabilitySupport::Native);
    let mut incomplete = evidence(config.requested.clone(), PreflightStatus::Passed);
    incomplete.checks.pop();
    assert_eq!(
        evaluate_model_preflight(&config, &capabilities, &incomplete, None, now),
        Err(ModelSafetyError::IncompleteEvidence)
    );

    let mut stale = evidence(config.requested.clone(), PreflightStatus::Passed);
    stale.checks[0].fresh_until = now - Duration::seconds(1);
    assert_eq!(
        evaluate_model_preflight(&config, &capabilities, &stale, None, now),
        Err(ModelSafetyError::InvalidEvidenceFreshness)
    );
}

#[test]
fn model_switch_requires_checkpoint_revision_reverification_and_generation() {
    let proof = ModelSwitchProof {
        schema: MODEL_SWITCH_PROOF_SCHEMA.into(),
        checkpoint_ref: "workpoint-checkpoint:model-switch".into(),
        checkpoint_reason: "model_switch".into(),
        config_revision_ref: "config-revision:2".into(),
        prior_generation: 4,
        next_generation: 5,
        safe_in_place_switch_proof_ref: None,
        preflight_ref: "model-preflight:2".into(),
        refreshed_bootstrap_ref: "bootstrap:2".into(),
        runtime_confirmation_ref: "model-confirmation:2".into(),
        event_ref: "event:model-switched".into(),
        receipt_ref: "receipt:model-switch".into(),
    };
    assert_eq!(validate_model_switch_proof(&proof), Ok(()));

    let mut missing_generation = proof.clone();
    missing_generation.next_generation = missing_generation.prior_generation;
    assert_eq!(
        validate_model_switch_proof(&missing_generation),
        Err(ModelSwitchProofError::GenerationNotAdvanced)
    );

    missing_generation.safe_in_place_switch_proof_ref = Some("adapter-proof:safe-switch".into());
    assert_eq!(validate_model_switch_proof(&missing_generation), Ok(()));

    let mut missing_receipt = proof;
    missing_receipt.receipt_ref.clear();
    assert_eq!(
        validate_model_switch_proof(&missing_receipt),
        Err(ModelSwitchProofError::MissingLinkage)
    );
}
