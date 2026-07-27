use super::*;

fn topology() -> ReleaseTopology {
    ReleaseTopology {
        schema: RELEASE_TOPOLOGY_SCHEMA.into(),
        project_id: "focusa".into(),
        profile: "multi_surface".into(),
        provider: "github_actions".into(),
        global_gates: vec!["scope_lock".into()],
        surfaces: vec![
            ReleaseSurface {
                surface_id: "cli".into(),
                kind: ReleaseSurfaceKind::Cli,
                depends_on: vec![],
                required_gates: vec!["test".into()],
                artifact_identity: "sha256".into(),
                deployment_target: None,
                canary_required: false,
                rollback_required: false,
            },
            ReleaseSurface {
                surface_id: "daemon".into(),
                kind: ReleaseSurfaceKind::Daemon,
                depends_on: vec!["cli".into()],
                required_gates: vec!["health".into()],
                artifact_identity: "sha256".into(),
                deployment_target: Some("production".into()),
                canary_required: true,
                rollback_required: true,
            },
        ],
    }
}

fn candidate() -> ReleaseCandidate {
    ReleaseCandidate {
        schema: RELEASE_CANDIDATE_SCHEMA.into(),
        candidate_id: "release:focusa:v1".into(),
        project_root: "/project".into(),
        continuity_id: "main".into(),
        workpoint_id: "work-1".into(),
        version: "1.0.0".into(),
        exact_sha: "0123456789abcdef".into(),
        topology_ref: "topology.json".into(),
        stage: ReleaseStage::Plan,
        locked_scope_refs: vec!["issue:1".into()],
        evidence: vec![],
        admitted_fixes: vec![],
        benchmark: None,
    }
}

fn evidence(stage: ReleaseStage) -> ReleaseEvidence {
    ReleaseEvidence {
        stage,
        exact_sha: "0123456789abcdef".into(),
        observed_at: "2026-01-01T00:00:00Z".into(),
        evidence_refs: vec!["actions:1".into()],
        invalidates: vec![],
    }
}

#[test]
fn validates_topology_and_rejects_cycles() {
    let mut value = topology();
    value.validate().unwrap();
    value.surfaces[0].depends_on.push("daemon".into());
    assert!(value.validate().unwrap_err().to_string().contains("cycle"));
}

#[test]
fn focusa_release_topology_fixture_is_valid() {
    let value: ReleaseTopology =
        serde_json::from_str(include_str!("../../../config/focusa-release-topology.json")).unwrap();
    value.validate().unwrap();
    assert_eq!(value.surfaces.len(), 8);
}

#[test]
fn release_candidate_advances_only_one_evidenced_stage() {
    let mut value = candidate();
    assert!(
        value
            .advance(ReleaseStage::Built, evidence(ReleaseStage::Built))
            .is_err()
    );
    value
        .advance(ReleaseStage::Locked, evidence(ReleaseStage::Locked))
        .unwrap();
    assert_eq!(value.stage, ReleaseStage::Locked);
    assert_eq!(value.evidence.len(), 1);
}

#[test]
fn bounded_fix_lane_rejects_unknown_surface() {
    let mut value = candidate();
    let fix = ReleaseFixLane {
        failed_gate: "windows-build".into(),
        affected_surfaces: vec!["unknown".into()],
        expected_proof: vec!["actions:2".into()],
        invalidated_evidence: vec![],
        new_candidate_required: true,
        operator_amendment_ref: None,
    };
    assert!(value.admit_fix(&topology(), fix).is_err());
}
