//! Public typed compiler consumer; no dispatcher or runtime admission is implied.
use focusa_core::procedure_compiler::{
    EffectClass, ExecutionPlan, PROCEDURE_SCHEMA, Procedure, Step, StepKind, compile_plan,
};
use sha2::{Digest, Sha256};

fn fixture() -> Procedure {
    Procedure {
        schema: PROCEDURE_SCHEMA.into(),
        procedure_id: "digest-fixture".into(),
        version: 1,
        steps: vec![Step {
            step_id: "a".into(),
            kind: StepKind::Shell,
            effect_class: EffectClass::Local,
            require_confirmation: false,
            rollback_step: None,
            command: "run a".into(),
        }],
    }
}

#[test]
fn typed_consumer_binds_version_safety_and_golden_v2_encoding() {
    let base = fixture();
    let plan = compile_plan(&base).unwrap();
    assert_eq!(
        plan.digest, "sha256:5f068c6abb4a2d2f67231fcda8aa108e05cc31aa5da1b6922177fd0af6290cec",
        "procedure digest omits its versioned typed encoding"
    );
    type ProcedureMutation = fn(&mut Procedure);
    let changes: [(&str, ProcedureMutation); 5] = [
        ("version", |p| p.version += 1),
        ("kind", |p| p.steps[0].kind = StepKind::HumanHandoff),
        ("effect", |p| {
            p.steps[0].effect_class = EffectClass::Security
        }),
        ("confirmation", |p| p.steps[0].require_confirmation = true),
        ("rollback", |p| {
            p.steps[0].rollback_step = Some("undo-a".into())
        }),
    ];
    for (label, change) in changes {
        let mut changed = base.clone();
        change(&mut changed);
        let encoded = serde_json::to_vec(&changed).unwrap();
        let decoded: Procedure = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, changed);
        assert_ne!(
            compile_plan(&decoded).unwrap().digest,
            plan.digest,
            "{label} aliases another plan"
        );
    }
    let mut unsupported = base.clone();
    unsupported.schema = "focusa.procedure.unsupported".into();
    assert!(compile_plan(&unsupported).is_err());
}

#[test]
fn typed_consumer_does_not_alias_unframed_step_boundaries() {
    let mut first = fixture();
    first.steps[0].step_id = "a".into();
    first.steps[0].command = "bc".into();
    let mut second = first.clone();
    second.steps[0].step_id = "ab".into();
    second.steps[0].command = "c".into();
    assert_ne!(
        compile_plan(&first).unwrap().digest,
        compile_plan(&second).unwrap().digest,
        "unframed step IDs and commands alias before hashing"
    );
}

#[test]
fn legacy_plan_remains_readable_but_cannot_match_the_recompiled_v2_identity() {
    let source = fixture();
    let current = compile_plan(&source).unwrap();
    let mut legacy_hash = Sha256::new();
    legacy_hash.update(source.procedure_id.as_bytes());
    for step in &source.steps {
        legacy_hash.update(step.step_id.as_bytes());
        legacy_hash.update(step.command.as_bytes());
    }
    let old_digest = format!("sha256:{:x}", legacy_hash.finalize());
    let mut saved = serde_json::to_value(&current).unwrap();
    saved["digest"] = serde_json::json!(old_digest);
    let before = serde_json::to_vec(&saved).unwrap();
    let legacy: ExecutionPlan = serde_json::from_slice(&before).unwrap();
    assert_eq!(legacy.procedure_id, current.procedure_id);
    assert_eq!(legacy.plan_steps, current.plan_steps);
    assert_ne!(
        legacy.digest, current.digest,
        "legacy unframed identity was silently accepted as v2"
    );
    assert_eq!(serde_json::to_vec(&saved).unwrap(), before);
}
