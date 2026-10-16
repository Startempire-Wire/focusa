//! Consumer boundary: persisted plans are untrusted even when their digest is retained.
use focusa_core::procedure_compiler::{
    compile_plan, verify_plan, EffectClass, ExecutionPlan, Procedure, Step, StepKind,
    PROCEDURE_SCHEMA,
};

fn source() -> Procedure {
    Procedure {
        schema: PROCEDURE_SCHEMA.into(),
        procedure_id: "consumer-integrity".into(),
        version: 1,
        steps: vec![
            Step {
                step_id: "transfer".into(),
                kind: StepKind::Api,
                effect_class: EffectClass::Financial,
                require_confirmation: false,
                rollback_step: Some("reverse-transfer".into()),
                command: "provider.transfer".into(),
            },
            Step {
                step_id: "handoff".into(),
                kind: StepKind::HumanHandoff,
                effect_class: EffectClass::None,
                require_confirmation: false,
                rollback_step: None,
                command: "owner review".into(),
            },
        ],
    }
}

#[test]
fn received_plan_round_trip_qualifies_exact_source_only() {
    let source = source();
    let plan = compile_plan(&source).unwrap();
    let received: ExecutionPlan =
        serde_json::from_slice(&serde_json::to_vec(&plan).unwrap()).unwrap();
    assert!(verify_plan(&source, &received).is_ok());
    let mut changed = source.clone();
    changed.version += 1;
    assert!(verify_plan(&changed, &received).is_err());
    let mut changed = source;
    changed.steps[0].command = "another provider operation".into();
    assert!(verify_plan(&changed, &received).is_err());
}

#[test]
fn matching_digest_cannot_hide_tampered_safety_or_rollback_projection() {
    let source = source();
    let plan = compile_plan(&source).unwrap();
    assert!(plan.plan_steps[0].confirmation_required);
    assert!(plan.plan_steps[1].handoff_required);
    let mut changed = plan.clone();
    changed.plan_steps[0].confirmation_required = false;
    assert!(verify_plan(&source, &changed).is_err());
    let mut changed = plan.clone();
    changed.plan_steps[1].handoff_required = false;
    assert!(verify_plan(&source, &changed).is_err());
    let mut changed = plan.clone();
    changed.plan_steps[0].effect_class = EffectClass::Local;
    assert!(verify_plan(&source, &changed).is_err());
    let mut changed = plan.clone();
    changed.rollback_order[0] = "unrelated-step".into();
    assert!(verify_plan(&source, &changed).is_err());
    let mut changed = plan.clone();
    changed.plan_steps[0].rollback_of = None;
    assert!(verify_plan(&source, &changed).is_err());
    let mut changed = plan;
    changed.plan_steps.reverse();
    assert!(verify_plan(&source, &changed).is_err());
}

#[test]
fn legacy_wire_plan_still_decodes_but_cannot_qualify_v2() {
    let source = source();
    let mut plan = compile_plan(&source).unwrap();
    // Reproduce the published pre-v2 encoding independently at the consumer.
    use sha2::{Digest, Sha256};
    let mut legacy = Sha256::new();
    legacy.update(source.procedure_id.as_bytes());
    for step in &source.steps {
        legacy.update(step.step_id.as_bytes());
        legacy.update(step.command.as_bytes());
    }
    plan.digest = format!("sha256:{:x}", legacy.finalize());
    let received: ExecutionPlan =
        serde_json::from_slice(&serde_json::to_vec(&plan).unwrap()).unwrap();
    assert!(verify_plan(&source, &received).is_err());
}

#[test]
fn unsupported_source_and_truncated_plan_fail_without_authority() {
    let mut source = source();
    let mut plan = compile_plan(&source).unwrap();
    plan.plan_steps.pop();
    assert!(verify_plan(&source, &plan).is_err());
    source.schema = "focusa.procedure.unsupported".into();
    assert!(verify_plan(&source, &plan).is_err());
}
