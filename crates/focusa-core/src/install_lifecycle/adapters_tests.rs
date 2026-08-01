use super::*;

fn record(
    adapter: LifecycleAdapterKind,
    selection: AdapterSelection,
    capability: AdapterCapabilityState,
) -> AdapterSelectionRecord {
    AdapterSelectionRecord {
        adapter,
        selection,
        capability,
        operator_confirmed: true,
        capability_evidence_ref: format!("evidence:{adapter:?}"),
    }
}

fn request(selections: Vec<AdapterSelectionRecord>) -> LifecycleAdapterRequest {
    LifecycleAdapterRequest {
        transaction_id: "transaction-1".into(),
        transaction_receipt_id: "receipt-1".into(),
        scope: LifecycleScope {
            host_id: "host-1".into(),
            project_root: Some("/project".into()),
            continuity_id: Some("continuity-1".into()),
        },
        selections,
        provider_handoff: None,
        prior_attempt: 0,
        evidence_messages: vec!["capability checked".into()],
    }
}

#[test]
fn required_and_optional_capabilities_project_truthful_outcomes() {
    let receipt = evaluate_lifecycle_adapters(&request(vec![
        record(
            LifecycleAdapterKind::Pi,
            AdapterSelection::Required,
            AdapterCapabilityState::PresentCompatible,
        ),
        record(
            LifecycleAdapterKind::Uiai,
            AdapterSelection::OptionalEnabled,
            AdapterCapabilityState::Absent,
        ),
        record(
            LifecycleAdapterKind::MacMenubar,
            AdapterSelection::OptedOut,
            AdapterCapabilityState::Unsupported,
        ),
    ]))
    .unwrap();
    assert!(receipt.all_required_active);
    assert_eq!(
        receipt.outcomes[&LifecycleAdapterKind::Pi].state,
        AdapterOutcomeState::Active
    );
    assert_eq!(
        receipt.outcomes[&LifecycleAdapterKind::Uiai].state,
        AdapterOutcomeState::Degraded
    );
    assert_eq!(
        receipt.outcomes[&LifecycleAdapterKind::MacMenubar].state,
        AdapterOutcomeState::OptedOut
    );
}

#[test]
fn pi_busy_compacting_and_uiai_saturated_are_retryable_resume_states() {
    for (adapter, capability, expected) in [
        (
            LifecycleAdapterKind::Pi,
            AdapterCapabilityState::Busy,
            "retry_when_idle",
        ),
        (
            LifecycleAdapterKind::Pi,
            AdapterCapabilityState::Compacting,
            "resume_after_compaction",
        ),
        (
            LifecycleAdapterKind::Uiai,
            AdapterCapabilityState::Saturated,
            "retry_bounded_session",
        ),
    ] {
        let receipt = evaluate_lifecycle_adapters(&request(vec![record(
            adapter,
            AdapterSelection::OptionalEnabled,
            capability,
        )]))
        .unwrap();
        let outcome = &receipt.outcomes[&adapter];
        assert!(outcome.retryable);
        assert_eq!(outcome.resume_action.as_deref(), Some(expected));
    }
}

#[test]
fn required_absent_or_incompatible_capability_blocks_acceptance() {
    for capability in [
        AdapterCapabilityState::Absent,
        AdapterCapabilityState::Incompatible,
        AdapterCapabilityState::Unsupported,
    ] {
        let receipt = evaluate_lifecycle_adapters(&request(vec![record(
            LifecycleAdapterKind::Pi,
            AdapterSelection::Required,
            capability,
        )]))
        .unwrap();
        assert!(!receipt.all_required_active);
        assert_eq!(
            receipt.outcomes[&LifecycleAdapterKind::Pi].state,
            AdapterOutcomeState::Blocked
        );
    }
}

#[test]
fn provider_handoff_is_neutral_and_never_ingests_credentials() {
    let mut request = request(vec![record(
        LifecycleAdapterKind::ProviderAuth,
        AdapterSelection::Required,
        AdapterCapabilityState::Healthy,
    )]);
    request.provider_handoff = Some(ProviderAuthHandoff {
        provider_id: "provider-1".into(),
        handoff_url: "https://provider.example/authorize".into(),
        state_ref: "state:opaque".into(),
        credential_payload: Some("secret".into()),
    });
    assert_eq!(
        evaluate_lifecycle_adapters(&request),
        Err(LifecycleAdapterError::CredentialIngestionForbidden)
    );
    request
        .provider_handoff
        .as_mut()
        .unwrap()
        .credential_payload = None;
    assert!(evaluate_lifecycle_adapters(&request).is_ok());
}

#[test]
fn adapter_selection_is_explicit_and_evidence_is_secret_redacted() {
    let mut request = request(vec![record(
        LifecycleAdapterKind::Pi,
        AdapterSelection::Required,
        AdapterCapabilityState::PresentCompatible,
    )]);
    request.selections[0].operator_confirmed = false;
    assert_eq!(
        evaluate_lifecycle_adapters(&request),
        Err(LifecycleAdapterError::SelectionNotConfirmed)
    );
    request.selections[0].operator_confirmed = true;
    request.evidence_messages = vec!["api_key=must-not-leak".into()];
    let receipt = evaluate_lifecycle_adapters(&request).unwrap();
    assert_eq!(receipt.redacted_evidence, vec!["[REDACTED]"]);
    assert_eq!(receipt.transaction_receipt_id, "receipt-1");
}
