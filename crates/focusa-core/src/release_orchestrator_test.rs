use super::*;
use crate::release_cycle::{RELEASE_CANDIDATE_SCHEMA, RELEASE_TOPOLOGY_SCHEMA, ReleaseSurface};
use std::sync::Mutex;

fn topology(canary: bool) -> ReleaseTopology {
    ReleaseTopology {
        schema: RELEASE_TOPOLOGY_SCHEMA.into(),
        project_id: "example".into(),
        profile: "portable".into(),
        provider: "mock".into(),
        global_gates: vec!["scope_lock".into(), "exact_sha".into()],
        surfaces: vec![
            ReleaseSurface {
                surface_id: "library".into(),
                kind: ReleaseSurfaceKind::Library,
                depends_on: vec![],
                required_gates: vec!["test".into()],
                artifact_identity: "sha256".into(),
                deployment_target: None,
                canary_required: false,
                rollback_required: false,
            },
            ReleaseSurface {
                surface_id: "service".into(),
                kind: ReleaseSurfaceKind::Service,
                depends_on: vec!["library".into()],
                required_gates: vec!["health".into()],
                artifact_identity: "sha256".into(),
                deployment_target: Some("production".into()),
                canary_required: canary,
                rollback_required: true,
            },
        ],
    }
}

fn candidate() -> ReleaseCandidate {
    ReleaseCandidate {
        schema: RELEASE_CANDIDATE_SCHEMA.into(),
        candidate_id: "release:example:1.0.0".into(),
        project_root: "/srv/example".into(),
        continuity_id: "release-main".into(),
        workpoint_id: "work-1".into(),
        version: "1.0.0".into(),
        exact_sha: "0123456789abcdef".into(),
        topology_ref: "release-topology.json".into(),
        stage: ReleaseStage::Plan,
        locked_scope_refs: vec!["issue:1".into()],
        evidence: vec![],
        admitted_fixes: vec![],
        benchmark: None,
    }
}

fn descriptor() -> ReleaseAdapterDescriptor {
    ReleaseAdapterDescriptor {
        schema: RELEASE_ADAPTER_SCHEMA.into(),
        adapter_id: "mock".into(),
        adapter_version: "1".into(),
        supported_profiles: vec!["portable".into()],
        supported_surface_kinds: vec![ReleaseSurfaceKind::Library, ReleaseSurfaceKind::Service],
        supported_stages: vec![
            ReleaseStage::Locked,
            ReleaseStage::CandidateSnapshotted,
            ReleaseStage::Preflighted,
            ReleaseStage::Built,
            ReleaseStage::Packaged,
            ReleaseStage::Provenanced,
            ReleaseStage::DraftPublished,
            ReleaseStage::CanaryDeployed,
            ReleaseStage::Verified,
            ReleaseStage::Promoted,
            ReleaseStage::Closed,
        ],
        supports_canary: true,
        supports_rollback: true,
    }
}

struct MockAdapter {
    blocked: Option<ReleaseStage>,
    calls: Mutex<Vec<ReleaseStage>>,
}

#[async_trait]
impl ReleaseAdapter for MockAdapter {
    fn descriptor(&self) -> ReleaseAdapterDescriptor {
        descriptor()
    }

    async fn execute(&self, request: ReleaseStageRequest) -> anyhow::Result<ReleaseStageReceipt> {
        self.calls.lock().unwrap().push(request.stage);
        let blocked = self.blocked == Some(request.stage);
        Ok(ReleaseStageReceipt {
            stage: request.stage,
            outcome: if blocked {
                AdapterOutcome::Blocked
            } else {
                AdapterOutcome::Passed
            },
            evidence: ReleaseEvidence {
                stage: request.stage,
                exact_sha: request.exact_sha.clone(),
                observed_at: "2026-01-01T00:00:00Z".into(),
                evidence_refs: vec![format!("mock:{:?}", request.stage)],
                invalidates: vec![],
            },
            adapter_id: "mock".into(),
            artifact_set_id: matches!(
                request.stage,
                ReleaseStage::Built | ReleaseStage::Packaged | ReleaseStage::Provenanced
            )
            .then(|| format!("sha256:{}", request.exact_sha)),
            rollback_ref: (request.stage == ReleaseStage::Promoted)
                .then(|| "mock:rollback:1".into()),
            elapsed_ms: 10,
            queue_ms: 1,
            retry_ms: 0,
            reason_codes: blocked
                .then(|| "mock_gate_failed".into())
                .into_iter()
                .collect(),
        })
    }
}

fn authority(mutations: bool) -> ReleaseAuthority {
    ReleaseAuthority {
        project_root: "/srv/example".into(),
        continuity_id: "release-main".into(),
        operator_confirmed: true,
        mutation_allowed: mutations,
        approval_refs: vec!["operator:release-once".into()],
    }
}

#[test]
fn topology_waves_are_deterministic() {
    assert_eq!(
        surface_waves(&topology(true)).unwrap(),
        vec![vec!["library".to_string()], vec!["service".to_string()]]
    );
}

#[test]
fn canvas_terminal_and_headless_share_one_canonical_plan() {
    let candidate = candidate();
    let topology = topology(true);
    let tuning = ReleasePlanTuning::default();
    let evidence = BTreeMap::new();
    let plans: Vec<_> = [
        ReleaseInvocationSurface::Canvas,
        ReleaseInvocationSurface::Terminal,
        ReleaseInvocationSurface::Headless,
    ]
    .into_iter()
    .map(|surface| {
        MasterReleaseOrchestrator::plan_for_surface(
            &candidate,
            &topology,
            &descriptor(),
            &evidence,
            &tuning,
            surface,
        )
        .unwrap()
    })
    .collect();
    assert!(plans.windows(2).all(|pair| pair[0].stages == pair[1].stages
        && pair[0].surface_waves == pair[1].surface_waves
        && pair[0].tuning == pair[1].tuning));
}

#[test]
fn calibrated_parallelism_changes_next_plan_waves() {
    let mut value = topology(true);
    value.surfaces[1].depends_on.clear();
    let serial = bounded_surface_waves(&value, 1).unwrap();
    let parallel = bounded_surface_waves(&value, 2).unwrap();
    assert_eq!(serial.len(), 2);
    assert_eq!(parallel.len(), 1);
}

#[tokio::test]
async fn plan_mode_never_calls_adapter() {
    let adapter = MockAdapter {
        blocked: None,
        calls: Mutex::new(Vec::new()),
    };
    let result = MasterReleaseOrchestrator::run(
        candidate(),
        topology(true),
        &adapter,
        authority(false),
        ReleaseRunMode::Plan,
        "2026-01-01T00:00:00Z",
        BTreeMap::new(),
    )
    .await
    .unwrap();
    assert_eq!(result.status, "planned");
    assert!(result.receipts.is_empty());
    assert!(adapter.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn execution_requires_mutation_authority() {
    let adapter = MockAdapter {
        blocked: None,
        calls: Mutex::new(Vec::new()),
    };
    let result = MasterReleaseOrchestrator::run(
        candidate(),
        topology(true),
        &adapter,
        authority(false),
        ReleaseRunMode::Execute,
        "2026-01-01T00:00:00Z",
        BTreeMap::new(),
    )
    .await
    .unwrap();
    assert_eq!(result.status, "blocked");
    assert_eq!(result.blocked_reasons, vec!["mutation_authority_missing"]);
    assert!(adapter.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn provider_neutral_cycle_closes_in_canonical_order() {
    let adapter = MockAdapter {
        blocked: None,
        calls: Mutex::new(Vec::new()),
    };
    let result = MasterReleaseOrchestrator::run(
        candidate(),
        topology(true),
        &adapter,
        authority(true),
        ReleaseRunMode::Execute,
        "2026-01-01T00:00:00Z",
        BTreeMap::new(),
    )
    .await
    .unwrap();
    assert_eq!(result.status, "closed");
    assert_eq!(result.candidate.stage, ReleaseStage::Closed);
    assert_eq!(result.receipts.len(), 11);
    assert_eq!(
        adapter.calls.lock().unwrap().as_slice(),
        result.plan.stages.as_slice()
    );
}

#[tokio::test]
async fn exact_sha_evidence_is_reused_without_adapter_call() {
    let adapter = MockAdapter {
        blocked: None,
        calls: Mutex::new(Vec::new()),
    };
    let mut reusable = BTreeMap::new();
    reusable.insert(
        ReleaseStage::Preflighted,
        ReleaseEvidence {
            stage: ReleaseStage::Preflighted,
            exact_sha: "0123456789abcdef".into(),
            observed_at: "2026-01-01T00:00:00Z".into(),
            evidence_refs: vec!["ci:123".into()],
            invalidates: vec![],
        },
    );
    let result = MasterReleaseOrchestrator::run(
        candidate(),
        topology(false),
        &adapter,
        authority(true),
        ReleaseRunMode::Execute,
        "2026-01-01T00:00:00Z",
        reusable,
    )
    .await
    .unwrap();
    assert_eq!(result.status, "closed");
    assert!(
        result
            .receipts
            .iter()
            .any(|receipt| receipt.stage == ReleaseStage::Preflighted
                && receipt.reason_codes == vec!["exact_sha_evidence_reused"])
    );
    assert!(
        result
            .receipts
            .iter()
            .any(|receipt| receipt.stage == ReleaseStage::CanaryDeployed
                && receipt.outcome == AdapterOutcome::Skipped)
    );
    assert!(
        !adapter
            .calls
            .lock()
            .unwrap()
            .contains(&ReleaseStage::Preflighted)
    );
    assert!(
        !adapter
            .calls
            .lock()
            .unwrap()
            .contains(&ReleaseStage::CanaryDeployed)
    );
}

#[tokio::test]
async fn adapter_block_stops_before_promotion() {
    let adapter = MockAdapter {
        blocked: Some(ReleaseStage::Verified),
        calls: Mutex::new(Vec::new()),
    };
    let result = MasterReleaseOrchestrator::run(
        candidate(),
        topology(true),
        &adapter,
        authority(true),
        ReleaseRunMode::Execute,
        "2026-01-01T00:00:00Z",
        BTreeMap::new(),
    )
    .await
    .unwrap();
    assert_eq!(result.status, "blocked");
    assert_eq!(result.blocked_stage, Some(ReleaseStage::Verified));
    assert_eq!(result.candidate.stage, ReleaseStage::CanaryDeployed);
    assert!(
        !adapter
            .calls
            .lock()
            .unwrap()
            .contains(&ReleaseStage::Promoted)
    );
}

#[test]
fn descriptor_rejects_missing_required_capability() {
    let mut adapter = descriptor();
    adapter.supports_rollback = false;
    assert!(
        adapter
            .validate_for(&topology(true))
            .unwrap_err()
            .to_string()
            .contains("rollback")
    );
}
