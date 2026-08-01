use super::*;
use chrono::Utc;

fn host_scope() -> LifecycleScope {
    LifecycleScope {
        host_id: "host:test".into(),
        project_root: None,
        continuity_id: None,
    }
}

fn project_scope() -> LifecycleScope {
    LifecycleScope {
        host_id: "host:test".into(),
        project_root: Some("/project".into()),
        continuity_id: Some("continuity:test".into()),
    }
}

fn versions() -> Vec<ComponentVersion> {
    vec![ComponentVersion {
        component: "focusa".into(),
        version: "1.2.3".into(),
        compatible: true,
        evidence_refs: vec!["evidence:version".into()],
    }]
}

fn rollback(replacement_planned: bool) -> RollbackBoundary {
    RollbackBoundary {
        replacement_planned,
        prior_version_set: replacement_planned.then(versions).unwrap_or_default(),
        rollback_artifact_refs: replacement_planned
            .then(|| vec!["artifact:prior".into()])
            .unwrap_or_default(),
        rollback_trust_refs: replacement_planned
            .then(|| vec!["trust:prior".into()])
            .unwrap_or_default(),
        atomic_activation: true,
        preserves_user_data: true,
        preserves_project_data: true,
    }
}

fn preservation(action: MaintenanceAction) -> PreservationDeclaration {
    let classes = [
        LifecycleDataClass::ManagedBinaries,
        LifecycleDataClass::Services,
        LifecycleDataClass::Integrations,
        LifecycleDataClass::FocusaState,
        LifecycleDataClass::LogsCaches,
        LifecycleDataClass::LicenseState,
        LifecycleDataClass::ProviderHarnessState,
        LifecycleDataClass::ProjectFiles,
        LifecycleDataClass::ProjectTaskData,
        LifecycleDataClass::OperatorAuthoredInstructions,
    ];
    PreservationDeclaration {
        action,
        items: classes
            .into_iter()
            .map(|data_class| PreservationItem {
                data_class,
                disposition: PreservationDisposition::Preserve,
                owner_authorized: false,
                evidence_refs: vec!["evidence:preservation".into()],
            })
            .collect(),
        destructive_purge_confirmed: false,
    }
}

fn selections(project: ProjectSelection) -> LifecycleSelections {
    LifecycleSelections {
        interaction: InteractionSelection::Headless,
        authorization: AuthorizationSelection::AuthorizedDevelopment,
        channel: ChannelSelection::Stable,
        target: "x86_64-unknown-linux-gnu".into(),
        dependencies: DependencySelection::VerifyOnly,
        service: ServiceSelection::SupportedUserService,
        integrations: vec![IntegrationSelection::Pi],
        project,
        git: GitSelection::Preserve,
        task_provider: TaskProviderSelection::Preserve,
        instructions: InstructionSelection::Preserve,
        canvas: CanvasSelection::Guided,
    }
}

fn persisted(kind: LifecycleTransactionKind, scope: LifecycleScope) -> PersistedLifecycleState {
    PersistedLifecycleState {
        transaction_id: "tx:test".into(),
        transaction_kind: kind,
        scope,
        idempotency: IdempotencyRecord {
            key: "key:test".into(),
            intent_digest: "digest:test".into(),
            replay_count: 0,
        },
        current_state: LifecycleState::Preflighted,
        progress: TransactionProgress::InProgress,
        last_completed_action: Some("preflight".into()),
        transition_refs: vec!["transition:preflight".into()],
        stored_receipt_ref: None,
        completion_known: true,
        recovery: None,
        rollback: rollback(false),
        updated_at: Utc::now(),
    }
}

fn trust() -> ArtifactTrustEvidence {
    ArtifactTrustEvidence {
        declared_version: "1.2.3".into(),
        declared_channel: ChannelSelection::Stable,
        target: "x86_64-unknown-linux-gnu".into(),
        metadata_complete: true,
        checksum_refs: vec!["checksum:1".into()],
        signature_refs: vec!["signature:1".into()],
        provenance_refs: vec!["provenance:1".into()],
        staged_extraction_verified: true,
    }
}

fn completion(scope: LifecycleScope, workpoint_required: bool) -> CompletionProof {
    CompletionProof {
        artifact_trust: trust(),
        version_set: versions(),
        daemon_healthy: true,
        daemon_health_refs: vec!["health:daemon".into()],
        service: ServiceHealthEvidence {
            required: true,
            healthy: true,
            posture: "running user service".into(),
            evidence_refs: vec!["health:service".into()],
        },
        expected_scope: scope.clone(),
        observed_scope: scope,
        scope_evidence_refs: vec!["scope:verified".into()],
        mutation_required_confirmation: true,
        mutation_confirmation_ref: Some("confirmation:1".into()),
        workpoint_required,
        workpoint_ref: workpoint_required.then(|| "workpoint:1".into()),
        secret_values_detected: false,
        secret_handling_refs: vec!["secrets:redacted".into()],
        rollback: rollback(true),
        preservation: preservation(MaintenanceAction::Update),
        platform_evidence: vec![PlatformEvidence {
            target: "x86_64-unknown-linux-gnu".into(),
            mechanism: "systemd-user".into(),
            evidence_refs: vec!["platform:linux".into()],
        }],
    }
}

#[test]
fn typed_transactions_validate_and_resume_idempotently() {
    let preflight = PreflightReport {
        host_id: "host:test".into(),
        os: "linux".into(),
        architecture: "x86_64".into(),
        user_home_boundary: "/home/test".into(),
        shell: "bash".into(),
        tty_present: false,
        supported_target: Some("x86_64-unknown-linux-gnu".into()),
        existing_version_set: vec![],
        writable_user_targets: vec!["/home/test/.local/bin".into()],
        network_available: true,
        offline_allowed: false,
        artifact_metadata_reachable: true,
        license_posture: LicensePosture::AuthorizedDevelopment,
        explicit_project_path: None,
        inspected_project_path: None,
        findings: vec![PreflightFinding {
            finding_id: "target".into(),
            subject: PreflightSubject::HostTarget,
            disposition: PreflightFindingDisposition::AlreadySatisfied,
            summary: "supported".into(),
            evidence_refs: vec!["target:evidence".into()],
        }],
        inspected_at: Utc::now(),
    };
    let mut host = HostInstallTransaction {
        intent: HostInstallIntent {
            selections: selections(ProjectSelection::Skip),
            preflight,
            artifact: trust(),
        },
        persisted: persisted(LifecycleTransactionKind::HostInstall, host_scope()),
    };
    assert_eq!(host.validate(), Ok(()));
    assert_eq!(
        host.persisted.replay("key:test", "digest:test"),
        Ok(ReplayDisposition::Resume)
    );
    host.intent.selections.project = ProjectSelection::ExistingPath("/project".into());
    assert_eq!(
        host.validate(),
        Err(InstallLifecycleValidationError::ProjectSelectionForbiddenForHostInstall)
    );

    let project = ProjectOnboardingTransaction {
        intent: ProjectOnboardingIntent {
            selections: selections(ProjectSelection::ExistingPath("/project".into())),
            exact_scope: project_scope(),
            bootstrap_preview_ref: "preview:1".into(),
            mutation_confirmation_ref: Some("confirmation:1".into()),
        },
        persisted: persisted(LifecycleTransactionKind::ProjectOnboarding, project_scope()),
    };
    assert_eq!(project.validate(), Ok(()));

    let maintenance = LifecycleMaintenanceTransaction {
        intent: LifecycleMaintenanceIntent {
            action: MaintenanceAction::Update,
            selections: selections(ProjectSelection::Skip),
            preservation: preservation(MaintenanceAction::Update),
        },
        persisted: persisted(LifecycleTransactionKind::LifecycleMaintenance, host_scope()),
    };
    assert_eq!(maintenance.validate(), Ok(()));
}

#[test]
fn replay_conflicts_and_unknown_completion_fail_closed() {
    let mut state = persisted(LifecycleTransactionKind::LifecycleMaintenance, host_scope());
    assert_eq!(
        state.replay("key:test", "changed"),
        Err(InstallLifecycleValidationError::IdempotencyConflict)
    );
    state.completion_known = false;
    assert_eq!(
        state.replay("key:test", "digest:test"),
        Ok(ReplayDisposition::InspectBeforeResume)
    );
    state.progress = TransactionProgress::Complete;
    assert_eq!(
        state.validate(),
        Err(InstallLifecycleValidationError::CompletedTransactionRequiresReceipt)
    );
}

#[test]
fn preflight_never_inspects_an_undeclared_project() {
    let report = PreflightReport {
        host_id: "host:test".into(),
        os: "linux".into(),
        architecture: "x86_64".into(),
        user_home_boundary: "/home/test".into(),
        shell: "bash".into(),
        tty_present: true,
        supported_target: Some("target".into()),
        existing_version_set: vec![],
        writable_user_targets: vec![],
        network_available: false,
        offline_allowed: true,
        artifact_metadata_reachable: false,
        license_posture: LicensePosture::Unavailable,
        explicit_project_path: None,
        inspected_project_path: Some("/inferred-cwd".into()),
        findings: vec![],
        inspected_at: Utc::now(),
    };
    assert_eq!(
        report.validate(),
        Err(InstallLifecycleValidationError::ProjectInspectionWithoutExactScope)
    );
}

#[test]
fn project_transaction_rejects_scope_and_confirmation_gaps() {
    let mut transaction = ProjectOnboardingTransaction {
        intent: ProjectOnboardingIntent {
            selections: selections(ProjectSelection::ExistingPath("/other".into())),
            exact_scope: project_scope(),
            bootstrap_preview_ref: "preview:1".into(),
            mutation_confirmation_ref: None,
        },
        persisted: persisted(LifecycleTransactionKind::ProjectOnboarding, project_scope()),
    };
    assert_eq!(
        transaction.validate(),
        Err(InstallLifecycleValidationError::ScopeMismatch)
    );
    transaction.intent.selections.project = ProjectSelection::ExistingPath("/project".into());
    transaction.persisted.current_state = LifecycleState::ProjectBootstrapped;
    assert_eq!(
        transaction.validate(),
        Err(InstallLifecycleValidationError::MutationConfirmationRequired)
    );
}

#[test]
fn complete_receipts_prove_host_and_project_outcomes() {
    let host_proof = completion(host_scope(), false);
    let host_receipt = HostInstallReceiptPayload {
        target: "x86_64-unknown-linux-gnu".into(),
        version_set: versions(),
        artifact_trust_refs: vec!["trust:1".into()],
        daemon_health_refs: vec!["health:daemon".into()],
        service_posture: "healthy".into(),
        integration_outcomes: vec![],
        preservation: preservation(MaintenanceAction::Update),
        recovery: None,
        rollback: rollback(true),
        update_action: Some(MaintenanceAction::Update),
        uninstall_action: None,
        completion: host_proof,
    };
    assert_eq!(host_receipt.validate(), Ok(()));

    let project_receipt = ProjectOnboardingReceiptPayload {
        exact_scope: project_scope(),
        bootstrap_refs: vec!["bootstrap:1".into()],
        git: GitSelection::Preserve,
        task_provider: TaskProviderSelection::Preserve,
        instructions: InstructionSelection::Preserve,
        genesis_status: "committed".into(),
        hlt_status: "sufficient".into(),
        workpoint_ref: "workpoint:1".into(),
        canvas: CanvasSelection::Guided,
        deferred_optional_work: vec![],
        completion: completion(project_scope(), true),
    };
    assert_eq!(project_receipt.validate(), Ok(()));
}

#[test]
fn completion_validation_reports_every_false_success_dimension() {
    let mut proof = completion(project_scope(), true);
    proof.artifact_trust.signature_refs.clear();
    proof.version_set[0].compatible = false;
    proof.daemon_healthy = false;
    proof.service.healthy = false;
    proof.observed_scope.project_root = Some("/wrong".into());
    proof.mutation_confirmation_ref = None;
    proof.workpoint_ref = None;
    proof.secret_values_detected = true;
    proof.preservation.items.pop();
    proof.rollback.rollback_artifact_refs.clear();
    proof.platform_evidence.clear();

    let reasons = proof.validate().unwrap_err();
    for expected in [
        FalseCompletionReason::ArtifactTrustAbsent,
        FalseCompletionReason::VersionSetIncompatible,
        FalseCompletionReason::DaemonUnhealthy,
        FalseCompletionReason::RequiredServiceUnhealthy,
        FalseCompletionReason::ExactScopeUnproven,
        FalseCompletionReason::MutationConfirmationMissing,
        FalseCompletionReason::WorkpointMissing,
        FalseCompletionReason::SecretLeakDetected,
        FalseCompletionReason::PreservationAmbiguous,
        FalseCompletionReason::RollbackUnavailable,
        FalseCompletionReason::PlatformEvidenceMissing,
    ] {
        assert!(reasons.contains(&expected), "missing {expected:?}");
    }
}

#[test]
fn uninstall_and_recovery_boundaries_fail_closed() {
    let mut uninstall = preservation(MaintenanceAction::Uninstall);
    uninstall.items[3].disposition = PreservationDisposition::RemoveManagedArtifact;
    assert_eq!(
        uninstall.validate(),
        Err(InstallLifecycleValidationError::UninstallMustPreserveUserData)
    );

    let recovery = RecoveryInstructions {
        primary_class: RecoveryClass::UnknownCompletion,
        summary: "completion interrupted".into(),
        operator_actions: vec!["inspect persisted receipt and host state".into()],
        resume_from_state: LifecycleState::ArtifactVerified,
        inspect_before_retry: false,
        requires_confirmation: false,
        evidence_refs: vec![],
    };
    assert_eq!(
        recovery.validate(),
        Err(InstallLifecycleValidationError::UnknownCompletionRequiresInspection)
    );
}
