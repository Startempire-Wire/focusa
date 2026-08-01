use super::*;

fn definition(status: ReflexRuntimeStatus) -> SemanticReflexDefinition {
    SemanticReflexDefinition {
        reflex_id: "reroute_on_new_finding".into(),
        trigger_types: BTreeSet::from(["finding.recorded".into()]),
        required_context_keys: BTreeSet::from(["finding_id".into(), "snapshot_hash".into()]),
        action_type: "reroute_verification".into(),
        evidence_types: BTreeSet::from(["reroute_receipt".into()]),
        escalation_boundary: "operator_on_critical".into(),
        authority_scope: "project:/project:continuity-1".into(),
        requirement_ids: BTreeSet::from(["spec144-23-reflex".into()]),
        max_actions: 1,
        timeout_ms: 1_000,
        failure_envelope: "blocked_with_recovery".into(),
        runtime_status: status,
    }
}

fn invocation() -> SemanticReflexInvocation {
    SemanticReflexInvocation {
        reflex_id: "reroute_on_new_finding".into(),
        trigger_type: "finding.recorded".into(),
        project_root: "/project".into(),
        continuity_id: "continuity-1".into(),
        authority_scope: "project:/project:continuity-1".into(),
        context: BTreeMap::from([
            ("finding_id".into(), "finding-1".into()),
            ("snapshot_hash".into(), "sha256:snapshot".into()),
        ]),
        requested_actions: 1,
        mutation_requested: false,
        operator_confirmed: false,
    }
}

#[test]
fn executable_reflex_preserves_trigger_context_action_evidence_and_escalation() {
    let mut invocation = invocation();
    invocation.context.insert("critical".into(), "true".into());
    let outcome = execute_semantic_reflex(
        &definition(ReflexRuntimeStatus::Executable),
        &invocation,
        vec!["evidence:reroute".into()],
    )
    .unwrap();
    assert_eq!(outcome.action_type, "reroute_verification");
    assert_eq!(outcome.evidence_refs, vec!["evidence:reroute"]);
    assert!(outcome.escalation_required);
}

#[test]
fn registry_only_reflex_never_claims_runtime_implementation() {
    assert_eq!(
        execute_semantic_reflex(
            &definition(ReflexRuntimeStatus::SchemaOnly),
            &invocation(),
            vec!["evidence:any".into()]
        ),
        Err(SemanticReflexError::SchemaOnly)
    );
}

#[test]
fn authority_context_budget_confirmation_and_evidence_fail_closed() {
    let definition = definition(ReflexRuntimeStatus::Executable);
    let mut wrong_scope = invocation();
    wrong_scope.authority_scope = "foreign".into();
    assert_eq!(
        execute_semantic_reflex(&definition, &wrong_scope, vec!["evidence:1".into()]),
        Err(SemanticReflexError::AuthorityMismatch)
    );
    let mut missing = invocation();
    missing.context.remove("snapshot_hash");
    assert_eq!(
        execute_semantic_reflex(&definition, &missing, vec!["evidence:1".into()]),
        Err(SemanticReflexError::MissingContext("snapshot_hash".into()))
    );
    let mut over_budget = invocation();
    over_budget.requested_actions = 2;
    assert_eq!(
        execute_semantic_reflex(&definition, &over_budget, vec!["evidence:1".into()]),
        Err(SemanticReflexError::BudgetExceeded)
    );
    let mut mutation = invocation();
    mutation.mutation_requested = true;
    assert_eq!(
        execute_semantic_reflex(&definition, &mutation, vec!["evidence:1".into()]),
        Err(SemanticReflexError::ConfirmationRequired)
    );
    assert_eq!(
        execute_semantic_reflex(&definition, &invocation(), vec![]),
        Err(SemanticReflexError::EvidenceRequired)
    );
}

#[test]
fn shared_catalog_requires_every_normative_reflex_name() {
    let definitions: Vec<_> = SHARED_SEMANTIC_REFLEXES
        .iter()
        .map(|id| {
            let mut item = definition(ReflexRuntimeStatus::Executable);
            item.reflex_id = (*id).into();
            item
        })
        .collect();
    assert!(shared_reflex_catalog_is_complete(&definitions));
    assert!(!shared_reflex_catalog_is_complete(&definitions[1..]));
}
