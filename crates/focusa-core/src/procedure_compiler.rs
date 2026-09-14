//! Governed procedure compiler — #298 slice 1: typed procedures, the
//! deterministic execution plan, confirmation gates, and exact reverse
//! rollback ordering. Pure compiler — execution dispatch reuses the
//! CallGraph commit boundary (254) in a later slice.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PROCEDURE_SCHEMA: &str = "focusa.procedure.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Shell,
    Api,
    Agent,
    HumanHandoff,
    Approval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectClass {
    None,
    Local,
    External,
    Destructive,
    Financial,
    Security,
}

impl EffectClass {
    pub fn requires_confirmation(&self) -> bool {
        matches!(
            self,
            EffectClass::Destructive | EffectClass::Financial | EffectClass::Security
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub step_id: String,
    pub kind: StepKind,
    pub effect_class: EffectClass,
    pub require_confirmation: bool,
    pub rollback_step: Option<String>,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Procedure {
    pub schema: String,
    pub procedure_id: String,
    pub version: u64,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep {
    pub step_id: String,
    pub effect_class: EffectClass,
    pub confirmation_required: bool,
    pub handoff_required: bool,
    pub rollback_of: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub procedure_id: String,
    pub version: u64,
    pub plan_steps: Vec<PlanStep>,
    pub rollback_order: Vec<String>,
    pub digest: String,
}

/// Deterministic plan compile: confirmation flags (explicit OR
/// effect-class-mandated), handoff markers, digest, and reverse
/// rollback ordering over steps that declare a rollback step.
pub fn compile_plan(procedure: &Procedure) -> Result<ExecutionPlan, String> {
    if procedure.schema != PROCEDURE_SCHEMA {
        return Err(format!("unexpected schema {}", procedure.schema));
    }
    if procedure.steps.is_empty() {
        return Err("at least one step required".to_string());
    }
    let mut ids = std::collections::HashSet::new();
    for step in &procedure.steps {
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
            rollback_order.insert(0, step.step_id.clone());
        }
        plan_steps.push(PlanStep {
            step_id: step.step_id.clone(),
            effect_class: step.effect_class,
            confirmation_required,
            handoff_required,
            rollback_of: step.rollback_step.clone(),
        });
    }
    let mut hasher = Sha256::new();
    hasher.update(procedure.procedure_id.as_bytes());
    for step in &procedure.steps {
        hasher.update(step.step_id.as_bytes());
        hasher.update(step.command.as_bytes());
    }
    let digest = format!("sha256:{}", hex(&hasher.finalize()));
    Ok(ExecutionPlan {
        procedure_id: procedure.procedure_id.clone(),
        version: procedure.version,
        plan_steps,
        rollback_order,
        digest,
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
