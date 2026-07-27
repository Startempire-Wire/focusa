use super::*;
use crate::release_cycle::{RELEASE_CANDIDATE_SCHEMA, ReleaseCandidate};
use crate::release_orchestrator::{MasterReleaseOrchestrator, ReleaseAuthority, ReleaseRunMode};
use std::collections::BTreeMap;

fn fixture(topology: &str, manifest: &str) -> (ReleaseTopology, ReleaseAdapterManifest) {
    let topology: ReleaseTopology = serde_json::from_str(topology).unwrap();
    let manifest: ReleaseAdapterManifest = serde_json::from_str(manifest).unwrap();
    manifest.validate(&topology).unwrap();
    (topology, manifest)
}

#[test]
fn all_reference_adapter_manifests_conform() {
    let fixtures = [
        (
            include_str!("../../../config/focusa-release-topology.json"),
            include_str!("../../../config/release-adapters/focusa.json"),
        ),
        (
            include_str!("../../../config/release-topologies/uiai-engine.json"),
            include_str!("../../../config/release-adapters/uiai-engine.json"),
        ),
        (
            include_str!("../../../config/release-topologies/single-package.json"),
            include_str!("../../../config/release-adapters/single-package.json"),
        ),
        (
            include_str!("../../../config/release-topologies/service-container-web.json"),
            include_str!("../../../config/release-adapters/service-container-web.json"),
        ),
    ];
    for (topology, manifest) in fixtures {
        fixture(topology, manifest);
    }
}

#[derive(Clone)]
struct FakeExecutor {
    id: String,
}

#[async_trait]
impl ReleaseOperationExecutor for FakeExecutor {
    fn executor_id(&self) -> &str {
        &self.id
    }

    async fn execute(
        &self,
        operation: &ReleaseOperation,
        request: &ReleaseStageRequest,
    ) -> anyhow::Result<ReleaseOperationReceipt> {
        Ok(ReleaseOperationReceipt {
            operation_id: operation.operation_id.clone(),
            executor_id: self.id.clone(),
            exact_sha: request.exact_sha.clone(),
            outcome: AdapterOutcome::Passed,
            observed_at: "2026-01-01T00:00:00Z".into(),
            evidence_refs: vec![format!("fixture:{}:{}", self.id, operation.operation_id)],
            artifact_set_id: matches!(
                request.stage,
                ReleaseStage::Built | ReleaseStage::Packaged | ReleaseStage::Provenanced
            )
            .then(|| format!("artifact:{}:{}", self.id, request.exact_sha)),
            rollback_ref: (request.stage == ReleaseStage::Promoted)
                .then(|| format!("rollback:{}:1", self.id)),
            elapsed_ms: 5,
            queue_ms: 0,
            retry_ms: 0,
            reason_codes: vec![],
        })
    }
}

fn candidate(topology: &ReleaseTopology) -> ReleaseCandidate {
    ReleaseCandidate {
        schema: RELEASE_CANDIDATE_SCHEMA.into(),
        candidate_id: format!("release:{}:1.0.0", topology.project_id),
        project_root: format!("/projects/{}", topology.project_id),
        continuity_id: "release-main".into(),
        workpoint_id: "work-1".into(),
        version: "1.0.0".into(),
        exact_sha: "0123456789abcdef".into(),
        topology_ref: "fixture-topology".into(),
        stage: ReleaseStage::Plan,
        locked_scope_refs: vec!["fixture:scope".into()],
        evidence: vec![],
        admitted_fixes: vec![],
        benchmark: None,
    }
}

#[tokio::test]
async fn all_reference_manifests_execute_the_same_kernel() {
    let fixtures = [
        (
            include_str!("../../../config/focusa-release-topology.json"),
            include_str!("../../../config/release-adapters/focusa.json"),
        ),
        (
            include_str!("../../../config/release-topologies/uiai-engine.json"),
            include_str!("../../../config/release-adapters/uiai-engine.json"),
        ),
        (
            include_str!("../../../config/release-topologies/single-package.json"),
            include_str!("../../../config/release-adapters/single-package.json"),
        ),
        (
            include_str!("../../../config/release-topologies/service-container-web.json"),
            include_str!("../../../config/release-adapters/service-container-web.json"),
        ),
    ];
    for (topology_body, manifest_body) in fixtures {
        let (topology, manifest) = fixture(topology_body, manifest_body);
        let executor_id = manifest.operations[0].executor_id.clone();
        let adapter = ManifestReleaseAdapter::new(
            manifest,
            topology.clone(),
            vec![FakeExecutor { id: executor_id }],
        )
        .unwrap();
        let candidate = candidate(&topology);
        let authority = ReleaseAuthority {
            project_root: candidate.project_root.clone(),
            continuity_id: candidate.continuity_id.clone(),
            operator_confirmed: true,
            mutation_allowed: true,
            approval_refs: vec!["operator:release-once".into()],
        };
        let result = MasterReleaseOrchestrator::run(
            candidate,
            topology,
            &adapter,
            authority,
            ReleaseRunMode::Execute,
            "2026-01-01T00:00:00Z",
            BTreeMap::new(),
        )
        .await
        .unwrap();
        assert_eq!(result.status, "closed");
    }
}

#[tokio::test]
async fn single_package_manifest_executes_without_focusa_or_github_assumptions() {
    let (topology, manifest) = fixture(
        include_str!("../../../config/release-topologies/single-package.json"),
        include_str!("../../../config/release-adapters/single-package.json"),
    );
    let executor = FakeExecutor {
        id: "portable".into(),
    };
    let adapter = ManifestReleaseAdapter::new(manifest, topology.clone(), vec![executor]).unwrap();
    let candidate = candidate(&topology);
    let authority = ReleaseAuthority {
        project_root: candidate.project_root.clone(),
        continuity_id: candidate.continuity_id.clone(),
        operator_confirmed: true,
        mutation_allowed: true,
        approval_refs: vec!["operator:release-once".into()],
    };
    let result = MasterReleaseOrchestrator::run(
        candidate,
        topology,
        &adapter,
        authority,
        ReleaseRunMode::Execute,
        "2026-01-01T00:00:00Z",
        BTreeMap::new(),
    )
    .await
    .unwrap();
    assert_eq!(result.status, "closed");
    assert!(
        result
            .receipts
            .iter()
            .any(|receipt| receipt.stage == ReleaseStage::CanaryDeployed
                && receipt.outcome == AdapterOutcome::Skipped)
    );
}

#[test]
fn missing_executor_fails_before_any_release_action() {
    let (topology, manifest) = fixture(
        include_str!("../../../config/release-topologies/uiai-engine.json"),
        include_str!("../../../config/release-adapters/uiai-engine.json"),
    );
    let error = match ManifestReleaseAdapter::<FakeExecutor>::new(manifest, topology, vec![]) {
        Ok(_) => panic!("missing executor unexpectedly accepted"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("missing executor"));
}

#[test]
fn operation_sha_mismatch_is_rejected() {
    let operation = ReleaseOperation {
        operation_id: "op".into(),
        stage: ReleaseStage::Preflighted,
        executor_id: "fixture".into(),
        kind: ReleaseOperationKind::Evidence,
        action: "verify".into(),
        surface_ids: vec![],
        mutates: false,
        timeout_seconds: 10,
        parallel_group: None,
        inputs: BTreeMap::new(),
    };
    let request = ReleaseStageRequest {
        candidate_id: "candidate".into(),
        exact_sha: "good".into(),
        version: "1".into(),
        project_root: "/project".into(),
        topology: serde_json::from_str(include_str!(
            "../../../config/release-topologies/single-package.json"
        ))
        .unwrap(),
        stage: ReleaseStage::Preflighted,
        surface_waves: vec![],
        approval_refs: vec![],
    };
    let receipt = ReleaseOperationReceipt {
        operation_id: "op".into(),
        executor_id: "fixture".into(),
        exact_sha: "bad".into(),
        outcome: AdapterOutcome::Passed,
        observed_at: "now".into(),
        evidence_refs: vec!["proof".into()],
        artifact_set_id: None,
        rollback_ref: None,
        elapsed_ms: 1,
        queue_ms: 0,
        retry_ms: 0,
        reason_codes: vec![],
    };
    assert!(
        receipt
            .validate(&operation, &request)
            .unwrap_err()
            .to_string()
            .contains("SHA")
    );
}
