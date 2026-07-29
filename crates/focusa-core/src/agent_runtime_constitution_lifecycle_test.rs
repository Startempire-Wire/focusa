use crate::agent_runtime_constitution::*;
use crate::agent_runtime_constitution_lifecycle::*;
use chrono::Utc;
use std::collections::{BTreeMap, BTreeSet};

fn version(name: &str, parent: Option<&str>) -> RuntimeConstitutionVersion {
    RuntimeConstitutionVersion {
        version: name.into(),
        parent_version: parent.map(str::to_string),
        content_sha256: "a".repeat(64),
        lifecycle: RuntimeConstitutionLifecycleState::Draft,
        created_at: Utc::now(),
    }
}

#[test]
fn immutable_activation_pinning_and_rollback_require_authority() {
    let mut registry = RuntimeConstitutionRegistry::default();
    registry.draft(version("1", None)).unwrap();
    assert_eq!(
        registry.draft(version("1", None)),
        Err("immutable_version_exists".into())
    );
    assert_eq!(
        registry.approve("1", false, &[]),
        Err("approval_requires_operator_and_evidence".into())
    );
    registry.approve("1", true, &["test:green".into()]).unwrap();
    assert_eq!(
        registry.activate("1", false),
        Err("activation_requires_operator".into())
    );
    registry.activate("1", true).unwrap();
    let pin = registry.pin_session("session-1", &"b".repeat(64)).unwrap();
    assert_eq!(pin.constitution_version, "1");
    registry.draft(version("2", Some("1"))).unwrap();
    registry
        .approve("2", true, &["test:green-2".into()])
        .unwrap();
    registry.activate("2", true).unwrap();
    assert_eq!(
        registry
            .session_pin("session-1")
            .unwrap()
            .constitution_version,
        "1"
    );
    registry.rollback("1", true).unwrap();
    assert_eq!(registry.active_version(), Some("1"));
}

#[test]
fn revocation_fails_closed_for_activation_and_rollback() {
    let mut registry = RuntimeConstitutionRegistry::default();
    registry.draft(version("bad", None)).unwrap();
    registry
        .approve("bad", true, &["security:test".into()])
        .unwrap();
    registry.revoke("bad", "prompt_injection").unwrap();
    assert_eq!(
        registry.activate("bad", true),
        Err("version_revoked".into())
    );
    assert_eq!(
        registry.rollback("bad", true),
        Err("rollback_target_revoked".into())
    );
}

fn evaluation(id: &str, score: f64, security: f64) -> PromptEvaluation {
    evaluate_prompt_variant(
        id,
        id,
        [("quality".into(), score), ("security".into(), security)]
            .into_iter()
            .collect(),
        vec![format!("evidence:{id}")],
    )
    .unwrap()
}

#[test]
fn promotion_requires_gain_and_no_hard_dimension_regression() {
    let baseline = evaluation("baseline", 0.7, 0.9);
    let better = evaluation("better", 0.9, 0.95);
    let unsafe_candidate = evaluation("unsafe", 1.0, 0.4);
    let hard: BTreeSet<_> = ["security".into()].into_iter().collect();
    assert!(matches!(
        decide_prompt_promotion(&baseline, &better, 0.05, &hard),
        PromptPromotionDecision::Promote { .. }
    ));
    assert!(matches!(
        decide_prompt_promotion(&baseline, &unsafe_candidate, 0.05, &hard),
        PromptPromotionDecision::Rollback { .. }
    ));
    let hold = evaluation("hold", 0.71, 0.91);
    assert!(matches!(
        decide_prompt_promotion(&baseline, &hold, 0.05, &hard),
        PromptPromotionDecision::Hold { .. }
    ));
}

#[test]
fn impact_assessment_escalates_permission_and_release_drift() {
    let impact = assess_contract_impact(
        "impact-1",
        vec!["AGENTS.md".into()],
        vec!["release-permission-policy".into()],
    );
    assert_eq!(impact.risk, "high");
    assert!(
        impact
            .required_checks
            .contains(&"operator_reapproval".into())
    );
    let bounded = assess_contract_impact(
        "impact-2",
        vec!["docs.md".into()],
        vec!["communication-style".into()],
    );
    assert_eq!(bounded.risk, "bounded");
}

#[test]
fn evaluation_rejects_missing_evidence_and_out_of_range_scores() {
    assert_eq!(
        evaluate_prompt_variant("e", "v", BTreeMap::new(), vec![]).unwrap_err(),
        "evaluation_requires_dimensions_and_evidence"
    );
    assert_eq!(
        evaluate_prompt_variant(
            "e",
            "v",
            [("quality".into(), 1.2)].into_iter().collect(),
            vec!["proof".into()],
        )
        .unwrap_err(),
        "evaluation_score_out_of_range"
    );
}
