use super::*;

fn policy() -> SemanticPerformancePolicy {
    SemanticPerformancePolicy {
        max_sync_nodes: 1_000,
        max_sync_memory_bytes: 16 * 1024 * 1024,
        max_affected_nodes: 100,
        allow_advisory_fallback: false,
    }
}

fn request() -> SemanticExecutionRequest {
    SemanticExecutionRequest {
        artifact_hash: "sha256:artifact".into(),
        profile_hash: "sha256:profile".into(),
        changed_node_refs: BTreeSet::from(["node:a".into()]),
        affected_node_refs: BTreeSet::from(["node:b".into()]),
        estimated_nodes: 10,
        estimated_memory_bytes: 1_024,
        whole_world_reasoning_required: false,
        required_strength: AssuranceStrength::Independent,
        accepted_work_refs: BTreeSet::from(["settlement:accepted".into()]),
    }
}

#[test]
fn cache_hit_preserves_strength_and_accepted_work() {
    let request = request();
    let key = format!("{}:{}", request.artifact_hash, request.profile_hash);
    let plan = plan_semantic_execution(
        &policy(),
        &request,
        SemanticPressureMode::LowMemory,
        &BTreeSet::from([key]),
    )
    .unwrap();
    assert_eq!(plan.mode, SemanticExecutionMode::Cached);
    assert_eq!(plan.achieved_strength, AssuranceStrength::Independent);
    assert_eq!(
        plan.preserved_accepted_work_refs,
        request.accepted_work_refs
    );
}

#[test]
fn bounded_delta_validates_only_changed_and_affected_neighborhood() {
    let request = request();
    let plan = plan_semantic_execution(
        &policy(),
        &request,
        SemanticPressureMode::Normal,
        &BTreeSet::new(),
    )
    .unwrap();
    assert_eq!(plan.mode, SemanticExecutionMode::AffectedNeighborhood);
    assert_eq!(
        plan.validation_node_refs,
        BTreeSet::from(["node:a".into(), "node:b".into()])
    );
    assert!(plan.result_limit_required && plan.cancellation_required);
}

#[test]
fn expensive_whole_world_work_is_async_and_pressure_defers_without_loss() {
    let mut request = request();
    request.whole_world_reasoning_required = true;
    assert_eq!(
        plan_semantic_execution(
            &policy(),
            &request,
            SemanticPressureMode::Normal,
            &BTreeSet::new()
        )
        .unwrap()
        .mode,
        SemanticExecutionMode::WholeWorldAsync
    );
    request.estimated_memory_bytes = 32 * 1024 * 1024;
    let deferred = plan_semantic_execution(
        &policy(),
        &request,
        SemanticPressureMode::LowMemory,
        &BTreeSet::new(),
    )
    .unwrap();
    assert_eq!(
        deferred.mode,
        SemanticExecutionMode::DeferredPreservingAcceptedWork
    );
    assert_eq!(deferred.achieved_strength, AssuranceStrength::Independent);
    assert_eq!(deferred.preserved_accepted_work_refs.len(), 1);
}

#[test]
fn oversized_neighborhood_and_configured_strength_downgrade_fail_closed() {
    let mut request = request();
    request.affected_node_refs = (0..101).map(|n| format!("node:{n}")).collect();
    assert_eq!(
        plan_semantic_execution(
            &policy(),
            &request,
            SemanticPressureMode::Normal,
            &BTreeSet::new()
        ),
        Err(SemanticPerformanceError::AffectedNeighborhoodTooLarge)
    );
    let mut risky_policy = policy();
    risky_policy.allow_advisory_fallback = true;
    let mut expensive = request();
    expensive.estimated_memory_bytes = risky_policy.max_sync_memory_bytes + 1;
    assert_eq!(
        plan_semantic_execution(
            &risky_policy,
            &expensive,
            SemanticPressureMode::Constrained,
            &BTreeSet::new()
        ),
        Err(SemanticPerformanceError::AssuranceDowngradeForbidden)
    );
}
