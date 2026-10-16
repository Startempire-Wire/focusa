//! Governed procedure compiler — #298 slice 1: typed procedures, the
//! deterministic execution plan, confirmation gates, and exact reverse
//! rollback ordering. Pure compiler — execution dispatch reuses the
//! CallGraph commit boundary (254) in a later slice.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PROCEDURE_SCHEMA: &str = "focusa.procedure.v1";
/// Domain-separated identity contract; legacy unframed digests are not v2.
pub const PROCEDURE_DIGEST_SCHEMA: &str = "focusa.procedure-plan.digest.v2";
pub const PROCEDURE_PREVIEW_SCHEMA: &str = "focusa.procedure_preview.v1";
pub const PROCEDURE_VERIFICATION_SCHEMA: &str = "focusa.procedure_verification.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Shell,
    Api,
    Agent,
    HumanHandoff,
    Approval,
}

// One native effect vocabulary and confirmation floor, not a second policy enum.
pub use crate::callgraph::SideEffectClass as EffectClass;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub step_id: String,
    pub kind: StepKind,
    pub effect_class: EffectClass,
    pub require_confirmation: bool,
    pub rollback_step: Option<String>,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Procedure {
    pub schema: String,
    pub procedure_id: String,
    pub version: u64,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanStep {
    pub step_id: String,
    pub effect_class: EffectClass,
    pub confirmation_required: bool,
    pub handoff_required: bool,
    pub rollback_of: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPlan {
    pub procedure_id: String,
    pub version: u64,
    pub plan_steps: Vec<PlanStep>,
    pub rollback_order: Vec<String>,
    pub digest: String,
}

/// Shared transport contract: a preview never represents admission or a grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcedurePreview {
    pub schema: String,
    pub digest_schema: String,
    pub canonical: bool,
    pub execution_authorized: bool,
    pub plan: ExecutionPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcedureVerification {
    pub schema: String,
    pub verified: bool,
    pub digest_schema: String,
    pub canonical: bool,
    pub execution_authorized: bool,
}

pub fn compile_preview(procedure: &Procedure) -> Result<ProcedurePreview, String> {
    Ok(ProcedurePreview {
        schema: PROCEDURE_PREVIEW_SCHEMA.into(),
        digest_schema: PROCEDURE_DIGEST_SCHEMA.into(),
        canonical: false,
        execution_authorized: false,
        plan: compile_plan(procedure)?,
    })
}

pub fn verify_preview(
    procedure: &Procedure,
    preview: &ProcedurePreview,
) -> Result<ProcedureVerification, String> {
    if preview.schema != PROCEDURE_PREVIEW_SCHEMA
        || preview.digest_schema != PROCEDURE_DIGEST_SCHEMA
    {
        return Err("unsupported procedure preview or digest schema".into());
    }
    if preview.canonical || preview.execution_authorized {
        return Err("procedure preview cannot claim canonical or execution authority".into());
    }
    verify_plan(procedure, &preview.plan)?;
    Ok(ProcedureVerification {
        schema: PROCEDURE_VERIFICATION_SCHEMA.into(),
        verified: true,
        digest_schema: PROCEDURE_DIGEST_SCHEMA.into(),
        canonical: false,
        execution_authorized: false,
    })
}

/// Deterministic plan compile: confirmation flags (explicit OR
/// effect-class-mandated), handoff markers, digest, and reverse
/// rollback ordering over steps that declare a rollback step.
pub fn compile_plan(procedure: &Procedure) -> Result<ExecutionPlan, String> {
    if procedure.schema != PROCEDURE_SCHEMA {
        return Err(format!("unexpected schema {}", procedure.schema));
    }
    if procedure.procedure_id.trim().is_empty() || procedure.version == 0 {
        return Err("procedure identity and positive version required".to_string());
    }
    if procedure.steps.is_empty() {
        return Err("at least one step required".to_string());
    }
    let mut ids = std::collections::HashSet::new();
    for step in &procedure.steps {
        if step.step_id.trim().is_empty() || step.command.trim().is_empty() {
            return Err("step identity and command required".to_string());
        }
        if step
            .rollback_step
            .as_ref()
            .is_some_and(|id| id.trim().is_empty())
        {
            return Err("declared rollback reference must not be blank".to_string());
        }
        if !ids.insert(step.step_id.clone()) {
            return Err(format!("duplicate step_id {}", step.step_id));
        }
    }
    let mut plan_steps = Vec::new();
    let mut rollback_order = Vec::new();
    for step in &procedure.steps {
        let confirmation_required =
            step.require_confirmation || step.effect_class.requires_confirmation();
        let handoff_required = matches!(step.kind, StepKind::HumanHandoff | StepKind::Approval);
        if step.rollback_step.is_some() {
            rollback_order.push(step.step_id.clone());
        }
        plan_steps.push(PlanStep {
            step_id: step.step_id.clone(),
            effect_class: step.effect_class,
            confirmation_required,
            handoff_required,
            rollback_of: step.rollback_step.clone(),
        });
    }
    // Reverse once: preserve exact compensation order without quadratic inserts.
    rollback_order.reverse();
    // Hash the typed source, not only the display IDs/commands. Struct field
    // order and Vec step order are deterministic; JSON frames every string.
    // This binds version, step kind, effect, explicit confirmation and rollback
    // meaning, including distinctions that derived plan projections can omit.
    let canonical = serde_json::to_vec(procedure)
        .map_err(|_| "procedure digest serialization failed".to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(PROCEDURE_DIGEST_SCHEMA.as_bytes());
    hasher.update([0]);
    hasher.update(&canonical);
    let digest = format!("sha256:{}", hex(&hasher.finalize()));
    Ok(ExecutionPlan {
        procedure_id: procedure.procedure_id.clone(),
        version: procedure.version,
        plan_steps,
        rollback_order,
        digest,
    })
}

/// Qualify a received/persisted plan against its exact typed source.
///
/// A digest alone does not authenticate the derived safety projection: callers
/// must reject altered confirmation, handoff and rollback fields as well as
/// legacy identities. This verifies integrity only, never execution authority.
pub fn verify_plan(procedure: &Procedure, plan: &ExecutionPlan) -> Result<(), String> {
    let expected = compile_plan(procedure)?;
    if plan != &expected {
        return Err("compiled procedure plan does not match its v2 source".to_string());
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest_fixture() -> Procedure {
        Procedure {
            schema: PROCEDURE_SCHEMA.to_string(),
            procedure_id: "digest-fixture".to_string(),
            version: 1,
            steps: vec![step("a", StepKind::Shell, EffectClass::Local, None)],
        }
    }

    #[test]
    fn digest_v2_matches_independent_golden_and_rejects_legacy_alias() {
        let procedure = digest_fixture();
        let plan = compile_plan(&procedure).unwrap();
        // Independently generated from the published compact JSON field order
        // plus the versioned domain separator, not from this compiler helper.
        assert_eq!(
            plan.digest,
            "sha256:5f068c6abb4a2d2f67231fcda8aa108e05cc31aa5da1b6922177fd0af6290cec"
        );
        assert_ne!(
            plan.digest,
            "sha256:8b27f6e482ed735abf012d1f457b52edbd236569efbfa07f891bc320c8bbde65"
        );
        let mut unsupported = procedure;
        unsupported.schema = "focusa.procedure.unsupported".to_string();
        assert!(compile_plan(&unsupported).is_err());
    }

    #[test]
    fn digest_binds_every_governed_input() {
        let base = digest_fixture();
        let digest = compile_plan(&base).unwrap().digest;
        let mut variants = Vec::new();
        let mut p = base.clone();
        p.version += 1;
        variants.push(("version", p));
        let mut p = base.clone();
        p.steps[0].kind = StepKind::Api;
        variants.push(("kind", p));
        let mut p = base.clone();
        p.steps[0].effect_class = EffectClass::Financial;
        variants.push(("effect", p));
        let mut p = base.clone();
        p.steps[0].require_confirmation = true;
        variants.push(("confirmation", p));
        let mut p = base.clone();
        p.steps[0].rollback_step = Some("undo-a".to_string());
        variants.push(("rollback", p));
        let mut p = base.clone();
        p.steps[0].command = "different operation".to_string();
        variants.push(("command", p));
        let mut p = base.clone();
        p.steps[0].step_id = "different-id".to_string();
        variants.push(("step identity", p));
        let mut p = base.clone();
        p.procedure_id = "another-procedure".to_string();
        variants.push(("procedure identity", p));
        for (meaning, changed) in variants {
            assert_ne!(
                digest,
                compile_plan(&changed).unwrap().digest,
                "{meaning} disappeared from plan identity"
            );
        }
        let mut ordered = base;
        ordered
            .steps
            .push(step("b", StepKind::Agent, EffectClass::None, None));
        let before = compile_plan(&ordered).unwrap().digest;
        ordered.steps.reverse();
        assert_ne!(before, compile_plan(&ordered).unwrap().digest);
    }

    #[test]
    fn digest_does_not_alias_unframed_step_strings() {
        let mut left = digest_fixture();
        left.steps[0].step_id = "a".to_string();
        left.steps[0].command = "bc".to_string();
        let mut right = left.clone();
        right.steps[0].step_id = "ab".to_string();
        right.steps[0].command = "c".to_string();
        assert_ne!(
            compile_plan(&left).unwrap().digest,
            compile_plan(&right).unwrap().digest
        );
    }

    fn step(id: &str, kind: StepKind, effect: EffectClass, rollback: Option<&str>) -> Step {
        Step {
            step_id: id.to_string(),
            kind,
            effect_class: effect,
            require_confirmation: false,
            rollback_step: rollback.map(|r| r.to_string()),
            command: format!("run {id}"),
        }
    }

    #[test]
    fn security_and_destructive_steps_force_confirmation() {
        let procedure = Procedure {
            schema: PROCEDURE_SCHEMA.to_string(),
            procedure_id: "p1".to_string(),
            version: 1,
            steps: vec![
                step("s1", StepKind::Shell, EffectClass::None, None),
                step("s2", StepKind::Shell, EffectClass::Security, None),
                step("s3", StepKind::Api, EffectClass::Destructive, None),
                step("s4", StepKind::Shell, EffectClass::Financial, None),
            ],
        };
        let plan = compile_plan(&procedure).unwrap();
        assert!(!plan.plan_steps[0].confirmation_required);
        assert!(plan.plan_steps[1].confirmation_required);
        assert!(plan.plan_steps[2].confirmation_required);
        assert!(plan.plan_steps[3].confirmation_required);
    }

    #[test]
    fn rollback_order_is_exact_reverse() {
        let procedure = Procedure {
            schema: PROCEDURE_SCHEMA.to_string(),
            procedure_id: "p2".to_string(),
            version: 1,
            steps: vec![
                step("a", StepKind::Shell, EffectClass::Local, Some("ra")),
                step("b", StepKind::Shell, EffectClass::Local, Some("rb")),
                step("c", StepKind::Shell, EffectClass::Local, Some("rc")),
            ],
        };
        let plan = compile_plan(&procedure).unwrap();
        assert_eq!(plan.rollback_order, vec!["c", "b", "a"]);
    }

    #[test]
    fn handoff_and_approval_steps_block() {
        let procedure = Procedure {
            schema: PROCEDURE_SCHEMA.to_string(),
            procedure_id: "p3".to_string(),
            version: 1,
            steps: vec![
                step("h", StepKind::HumanHandoff, EffectClass::External, None),
                step("a", StepKind::Approval, EffectClass::Security, None),
            ],
        };
        let plan = compile_plan(&procedure).unwrap();
        assert!(plan.plan_steps.iter().all(|s| s.handoff_required));
    }

    #[test]
    fn duplicate_ids_and_empty_steps_are_rejected() {
        let mut procedure = Procedure {
            schema: PROCEDURE_SCHEMA.to_string(),
            procedure_id: "p4".to_string(),
            version: 1,
            steps: vec![
                step("a", StepKind::Shell, EffectClass::None, None),
                step("a", StepKind::Shell, EffectClass::None, None),
            ],
        };
        assert!(compile_plan(&procedure).is_err());
        procedure.steps.clear();
        assert!(compile_plan(&procedure).is_err());
    }

    #[test]
    fn digest_is_stable() {
        let procedure = Procedure {
            schema: PROCEDURE_SCHEMA.to_string(),
            procedure_id: "p5".to_string(),
            version: 1,
            steps: vec![step("a", StepKind::Shell, EffectClass::None, None)],
        };
        let plan1 = compile_plan(&procedure).unwrap();
        let plan2 = compile_plan(&procedure).unwrap();
        assert_eq!(plan1.digest, plan2.digest);
        assert!(plan1.digest.starts_with("sha256:"));
    }
}
