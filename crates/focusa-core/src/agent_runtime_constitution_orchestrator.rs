//! Spec 140 C.R.I.S.T./Project Genesis to Runtime Constitution composition.

use crate::agent_runtime_constitution::*;
use crate::agent_runtime_constitution_enforcement::compile_validation_matrix;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const STABLE_TEMPORAL_OBLIGATION: &str =
    "docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md";
pub const STABLE_PRESENCE_OBLIGATION: &str = "docs/139-focusa-presence-awareness-system-spec.md";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CristRuntimeInput {
    pub project_ref: String,
    pub genesis_ref: String,
    pub approved_spec_ref: String,
    pub operator_confirmed: bool,
    pub agent_identity_ref: AgentIdentityReference,
    pub kernel_ref: ConstitutionalKernelReference,
    pub role_ref: RoleProfileReference,
    pub mission: String,
    pub responsibilities: Vec<String>,
    pub non_responsibilities: Vec<String>,
    pub authority_order: Vec<String>,
    pub execution_boundaries: Vec<String>,
    pub output_contracts: Vec<String>,
    pub awareness_ref: RuntimeAwarenessContractReference,
    pub instruction_sources: Vec<InstructionSource>,
    pub target_profile: TargetCapabilityProfile,
    pub changed_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConstitutionComposition {
    pub constitution: ProjectAgentRuntimeConstitution,
    pub assembly_plan: SystemPromptAssemblyPlan,
    pub validation_matrix: ValidationMatrix,
    pub stable_obligation_refs: BTreeSet<String>,
}

pub fn compose_runtime_constitution(
    input: CristRuntimeInput,
) -> Result<RuntimeConstitutionComposition, Vec<String>> {
    let mut errors = Vec::new();
    if !input.operator_confirmed {
        errors.push("approved_crist_or_operator_confirmation_required".into());
    }
    for (field, value) in [
        ("project_ref", input.project_ref.as_str()),
        ("genesis_ref", input.genesis_ref.as_str()),
        ("approved_spec_ref", input.approved_spec_ref.as_str()),
        ("mission", input.mission.as_str()),
    ] {
        if value.trim().is_empty() {
            errors.push(format!("missing_{field}"));
        }
    }
    if input.responsibilities.is_empty()
        || input.authority_order.is_empty()
        || input.execution_boundaries.is_empty()
        || input.output_contracts.is_empty()
    {
        errors.push("operating_contract_incomplete".into());
    }
    if input.instruction_sources.is_empty() {
        errors.push("instruction_source_inventory_required".into());
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let constitution_id = format!(
        "runtime-constitution:{}:{}",
        sanitize_id(&input.project_ref),
        sanitize_id(&input.role_ref.0)
    );
    let source_refs = input
        .instruction_sources
        .iter()
        .map(|source| source.source_ref.clone())
        .collect();
    let constitution = ProjectAgentRuntimeConstitution {
        schema: "focusa.project_agent_runtime_constitution.v1".into(),
        constitution_id: constitution_id.clone(),
        project_ref: input.project_ref,
        genesis_ref: input.genesis_ref,
        approved_spec_ref: input.approved_spec_ref,
        agent_identity_ref: input.agent_identity_ref,
        base_agent_constitution_ref: input.kernel_ref,
        role_profile_ref: input.role_ref,
        revision: 1,
        status: RuntimeConstitutionLifecycleState::Draft,
        operating_contract: AgentOperatingContract {
            purpose: input.mission,
            responsibilities: input.responsibilities,
            non_responsibilities: input.non_responsibilities,
            authority_order: input.authority_order,
            execution_boundaries: input.execution_boundaries,
            output_contracts: input.output_contracts,
        },
        instruction_sources: input.instruction_sources,
        claim_refs: Vec::new(),
        resolution_refs: Vec::new(),
        awareness_contract_ref: input.awareness_ref,
        extensions: BTreeMap::from([
            (
                "temporal_obligation_ref".into(),
                serde_json::Value::String(STABLE_TEMPORAL_OBLIGATION.into()),
            ),
            (
                "presence_obligation_ref".into(),
                serde_json::Value::String(STABLE_PRESENCE_OBLIGATION.into()),
            ),
        ]),
    };
    constitution.validate()?;
    let assembly_plan = SystemPromptAssemblyPlan {
        plan_id: format!("assembly:{constitution_id}:1"),
        ordered_layers: vec![
            PromptLayer::FocusaKernel,
            PromptLayer::ProjectConstitution,
            PromptLayer::Role,
            PromptLayer::PathOverlay,
        ],
        source_refs,
        excluded_claims: BTreeMap::new(),
        target_profile: input.target_profile,
    };
    let validation_matrix = compile_validation_matrix(
        format!("validation:{constitution_id}:1"),
        &input.changed_paths,
    );
    Ok(RuntimeConstitutionComposition {
        constitution,
        assembly_plan,
        validation_matrix,
        stable_obligation_refs: [
            STABLE_TEMPORAL_OBLIGATION.into(),
            STABLE_PRESENCE_OBLIGATION.into(),
        ]
        .into_iter()
        .collect(),
    })
}

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
