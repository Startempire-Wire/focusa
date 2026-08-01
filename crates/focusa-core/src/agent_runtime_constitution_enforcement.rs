//! Spec 140 skill, tool, validation, and deterministic enforcement compilation.

use crate::agent_runtime_constitution::{
    AgentEnforcementPlan, EnforcementControl, InstructionClaim, PermissionProfileReference,
    SkillActivationPlan, SkillBinding, ToolRoutingPlan, ValidationMatrix, ValidationRule,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct ToolCapability {
    pub tool_name: String,
    pub operation_classes: BTreeSet<String>,
    pub typed: bool,
    pub mutation: bool,
    pub authority_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnforcementDecision {
    Allow,
    RequireConfirmation { control_id: String },
    Deny { reason: String },
}

pub fn compile_skill_activation_plan(
    plan_id: impl Into<String>,
    candidates: Vec<SkillBinding>,
    applicable_skill_ids: &BTreeSet<String>,
) -> SkillActivationPlan {
    let mut bindings = Vec::new();
    let mut excluded = BTreeMap::new();
    for binding in candidates {
        if applicable_skill_ids.contains(&binding.skill_id) {
            bindings.push(binding);
        } else {
            excluded.insert(
                binding.skill_id,
                "not_applicable_to_current_role_or_work".into(),
            );
        }
    }
    bindings.sort_by(|left, right| left.skill_id.cmp(&right.skill_id));
    SkillActivationPlan {
        plan_id: plan_id.into(),
        bindings,
        excluded,
    }
}

pub fn compile_tool_routing_plan(
    plan_id: impl Into<String>,
    required_operations: &BTreeSet<String>,
    capabilities: &[ToolCapability],
) -> Result<ToolRoutingPlan, Vec<String>> {
    let mut allowed_tools = BTreeSet::new();
    let mut denied_tools = BTreeMap::new();
    let mut confirmation_required = BTreeSet::new();
    let mut missing = Vec::new();
    for operation in required_operations {
        let mut matches: Vec<_> = capabilities
            .iter()
            .filter(|capability| capability.operation_classes.contains(operation))
            .collect();
        matches.sort_by_key(|capability| (!capability.typed, capability.tool_name.clone()));
        match matches.first() {
            Some(capability) => {
                allowed_tools.insert(capability.tool_name.clone());
                if capability.mutation {
                    confirmation_required.insert(capability.tool_name.clone());
                }
                for fallback in matches.iter().skip(1) {
                    denied_tools.insert(
                        fallback.tool_name.clone(),
                        format!("superseded_by_typed_route:{}", capability.tool_name),
                    );
                }
            }
            None => missing.push(format!("missing_tool_route:{operation}")),
        }
    }
    if missing.is_empty() {
        Ok(ToolRoutingPlan {
            plan_id: plan_id.into(),
            allowed_tools,
            denied_tools,
            confirmation_required,
        })
    } else {
        Err(missing)
    }
}

pub fn compile_validation_matrix(
    matrix_id: impl Into<String>,
    changed_paths: &[String],
) -> ValidationMatrix {
    let mut rules = BTreeMap::<String, ValidationRule>::new();
    let mut insert = |rule_id: &str, requirement: &str, kind: &str| {
        rules.entry(rule_id.into()).or_insert(ValidationRule {
            rule_id: rule_id.into(),
            requirement_ref: requirement.into(),
            check_kind: kind.into(),
            required: true,
        });
    };
    insert("format", "quality:format", "command");
    insert("unit", "quality:unit", "test");
    for path in changed_paths {
        if path.ends_with(".rs") {
            insert("rust-check", "quality:rust", "cargo_check");
        }
        if path.ends_with(".ts") || path.ends_with(".mjs") {
            insert("typecheck", "quality:typescript", "typecheck");
            insert("lint", "quality:lint", "lint");
        }
        if path.starts_with("docs/") {
            insert("docs-drift", "quality:docs", "generated_drift");
        }
        if path.contains("auth") || path.contains("secret") || path.contains("permission") {
            insert(
                "security-negative",
                "quality:security",
                "negative_security_test",
            );
        }
        if path.contains("migration") || path.contains("persistence") {
            insert(
                "restart-recovery",
                "quality:recovery",
                "restart_recovery_test",
            );
        }
    }
    ValidationMatrix {
        matrix_id: matrix_id.into(),
        rules: rules.into_values().collect(),
        evidence_refs: Vec::new(),
    }
}

pub fn compile_enforcement_plan(
    plan_id: impl Into<String>,
    hard_claims: &[InstructionClaim],
    permission_profile_refs: Vec<PermissionProfileReference>,
) -> AgentEnforcementPlan {
    let mut controls = Vec::new();
    for claim in hard_claims {
        let boundary = match claim.claim_class.as_str() {
            "release_authority" => "release_publication",
            "security_boundary" => "security_mutation",
            "file_mutation" => "filesystem_mutation",
            _ => "runtime_operation",
        };
        controls.push(EnforcementControl {
            control_id: format!("control:{}", claim.claim_id),
            boundary: boundary.into(),
            enforcement_point: "daemon_preflight_and_receipt".into(),
            failure_posture: "fail_closed".into(),
        });
    }
    controls.sort_by(|left, right| left.control_id.cmp(&right.control_id));
    AgentEnforcementPlan {
        plan_id: plan_id.into(),
        controls,
        permission_profile_refs,
    }
}

pub fn enforce_tool_operation(
    routing: &ToolRoutingPlan,
    enforcement: &AgentEnforcementPlan,
    tool_name: &str,
    operation_boundary: &str,
    confirmed: bool,
) -> EnforcementDecision {
    if let Some(reason) = routing.denied_tools.get(tool_name) {
        return EnforcementDecision::Deny {
            reason: reason.clone(),
        };
    }
    if !routing.allowed_tools.contains(tool_name) {
        return EnforcementDecision::Deny {
            reason: "tool_not_in_runtime_constitution".into(),
        };
    }
    if let Some(control) = enforcement
        .controls
        .iter()
        .find(|control| control.boundary == operation_boundary)
    {
        if control.failure_posture == "fail_closed"
            && (routing.confirmation_required.contains(tool_name)
                || operation_boundary == "release_publication")
            && !confirmed
        {
            return EnforcementDecision::RequireConfirmation {
                control_id: control.control_id.clone(),
            };
        }
    }
    EnforcementDecision::Allow
}
