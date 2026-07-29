//! Spec 140 deterministic prompt and cross-harness artifact compiler.

use crate::agent_runtime_constitution::{
    PiPromptVariant, ProjectAgentRuntimeConstitution, PromptGroundingManifest,
    PromptIntegrityManifest, PromptLayer, RuntimeArtifactProjection, SessionEnvironmentBinding,
    SystemPromptAssemblyPlan,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const REQUIRED_PROMPT_SECTIONS: &[&str] = &[
    "harness_and_agent_identity",
    "project_mission",
    "authority_model",
    "operating_doctrine",
    "environment_and_execution",
    "multi_agent_coordination",
    "tool_protocol",
    "skill_protocol",
    "work_lifecycle",
    "validation_and_proof",
    "temporal_awareness",
    "presence_awareness",
    "epistemic_behavior",
    "communication_contract",
    "recovery",
    "hard_non_negotiables",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptCompilationMode {
    Append,
    Replace,
    RuntimeCompiled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCompileInput {
    pub plan: SystemPromptAssemblyPlan,
    pub constitution: ProjectAgentRuntimeConstitution,
    pub mode: PromptCompilationMode,
    pub replacement_approved: bool,
    pub baseline_evaluation_ref: Option<String>,
    pub harness_default_prompt: Option<String>,
    pub approved_claims: BTreeMap<String, String>,
    pub session_binding: SessionEnvironmentBinding,
    pub dynamic_context: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPromptLayers {
    pub stable_constitutional_prompt: String,
    pub session_environment_block: String,
    pub turn_dynamic_context: BTreeMap<String, String>,
    pub variant: PiPromptVariant,
    pub grounding: PromptGroundingManifest,
    pub integrity: PromptIntegrityManifest,
}

pub fn compile_prompt(input: PromptCompileInput) -> Result<CompiledPromptLayers, Vec<String>> {
    input.constitution.validate()?;
    let mut errors = Vec::new();
    for required in REQUIRED_PROMPT_SECTIONS {
        if !section_body(required, &input).is_some_and(|body| !body.trim().is_empty()) {
            errors.push(format!("missing_required_prompt_section:{required}"));
        }
    }
    if input.mode == PromptCompilationMode::Replace {
        if !input.replacement_approved
            || input
                .baseline_evaluation_ref
                .as_deref()
                .is_none_or(str::is_empty)
        {
            errors.push("replace_mode_requires_approval_and_baseline_evaluation".into());
        }
        if !input
            .plan
            .target_profile
            .supported_layers
            .contains(&PromptLayer::HarnessSystem)
        {
            errors.push("replace_mode_not_supported_by_target".into());
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let stable = REQUIRED_PROMPT_SECTIONS
        .iter()
        .map(|section| {
            format!(
                "## {}\n{}",
                title(section),
                section_body(section, &input).unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    if stable.len() > input.plan.target_profile.max_prompt_bytes {
        return Err(vec!["stable_prompt_exceeds_target_budget".into()]);
    }
    let session = format!(
        "## Session Environment\nproject_root: {}\ncontinuity_id: {}\ntarget: {}\nenvironment_sha256: {}",
        input.session_binding.project_root,
        input.session_binding.continuity_id,
        input.session_binding.target,
        input.session_binding.environment_sha256
    );
    let managed = format!(
        "<focusa-managed constitution=\"{}\">\n{}\n\n{}\n</focusa-managed>",
        input.constitution.constitution_id, stable, session
    );
    let body = match input.mode {
        PromptCompilationMode::Append => format!(
            "{}\n\n{}",
            input.harness_default_prompt.as_deref().unwrap_or(""),
            managed
        )
        .trim()
        .to_string(),
        PromptCompilationMode::Replace | PromptCompilationMode::RuntimeCompiled => managed,
    };
    let prompt_sha256 = sha256(&body);
    let mut source_hashes = BTreeMap::new();
    for source in &input.constitution.instruction_sources {
        source_hashes.insert(source.source_ref.clone(), source.content_sha256.clone());
    }
    let grounding = PromptGroundingManifest {
        prompt_sha256: prompt_sha256.clone(),
        source_hashes,
        resolution_refs: input.constitution.resolution_refs.clone(),
        generated_at: Utc::now(),
    };
    let assembly_plan_sha256 = sha256(
        &serde_json::to_string(&input.plan)
            .map_err(|_| vec!["assembly_plan_serialization_failed".into()])?,
    );
    Ok(CompiledPromptLayers {
        stable_constitutional_prompt: stable,
        session_environment_block: session,
        turn_dynamic_context: input.dynamic_context,
        variant: PiPromptVariant {
            variant_id: format!("{}:{}", input.plan.plan_id, &prompt_sha256[..16]),
            target: input.plan.target_profile.target.clone(),
            prompt_sha256: prompt_sha256.clone(),
            body,
        },
        grounding,
        integrity: PromptIntegrityManifest {
            assembly_plan_sha256,
            prompt_sha256,
            signature_ref: None,
            verified: true,
        },
    })
}

pub fn compile_prompt_with_safe_fallback(input: PromptCompileInput) -> CompiledPromptLayers {
    match compile_prompt(input.clone()) {
        Ok(compiled) => compiled,
        Err(errors) => {
            let fallback = format!(
                "{}\n\n<focusa-managed-fallback>\nProject constitution compilation unavailable; preserve harness defaults.\nreason_codes: {}\n</focusa-managed-fallback>",
                input.harness_default_prompt.as_deref().unwrap_or(""),
                errors.join(",")
            )
            .trim()
            .to_string();
            let prompt_sha256 = sha256(&fallback);
            let source_hashes = input
                .constitution
                .instruction_sources
                .iter()
                .map(|source| (source.source_ref.clone(), source.content_sha256.clone()))
                .collect();
            CompiledPromptLayers {
                stable_constitutional_prompt:
                    "Project constitution unavailable; harness defaults preserved.".into(),
                session_environment_block: format!(
                    "project_root: {}",
                    input.session_binding.project_root
                ),
                turn_dynamic_context: input.dynamic_context,
                variant: PiPromptVariant {
                    variant_id: format!("fallback:{}", &prompt_sha256[..16]),
                    target: input.plan.target_profile.target,
                    prompt_sha256: prompt_sha256.clone(),
                    body: fallback,
                },
                grounding: PromptGroundingManifest {
                    prompt_sha256: prompt_sha256.clone(),
                    source_hashes,
                    resolution_refs: input.constitution.resolution_refs,
                    generated_at: Utc::now(),
                },
                integrity: PromptIntegrityManifest {
                    assembly_plan_sha256: sha256("fallback"),
                    prompt_sha256,
                    signature_ref: None,
                    verified: false,
                },
            }
        }
    }
}

fn section_body(section: &str, input: &PromptCompileInput) -> Option<String> {
    let contract = &input.constitution.operating_contract;
    let values = match section {
        "harness_and_agent_identity" => vec![
            input.constitution.agent_identity_ref.0.clone(),
            input.plan.target_profile.target.clone(),
        ],
        "project_mission" => vec![contract.purpose.clone()],
        "authority_model" => contract.authority_order.clone(),
        "operating_doctrine" => contract.responsibilities.clone(),
        "environment_and_execution" => contract.execution_boundaries.clone(),
        "multi_agent_coordination" => claim_values("coordination", &input.approved_claims),
        "tool_protocol" => claim_values("tool", &input.approved_claims),
        "skill_protocol" => claim_values("skill", &input.approved_claims),
        "work_lifecycle" => claim_values("lifecycle", &input.approved_claims),
        "validation_and_proof" => contract.output_contracts.clone(),
        "temporal_awareness" => claim_values("temporal", &input.approved_claims),
        "presence_awareness" => claim_values("presence", &input.approved_claims),
        "epistemic_behavior" => claim_values("epistemic", &input.approved_claims),
        "communication_contract" => claim_values("communication", &input.approved_claims),
        "recovery" => claim_values("recovery", &input.approved_claims),
        "hard_non_negotiables" => contract.non_responsibilities.clone(),
        _ => Vec::new(),
    };
    (!values.is_empty()).then(|| {
        values
            .into_iter()
            .map(|value| format!("- {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn claim_values(prefix: &str, claims: &BTreeMap<String, String>) -> Vec<String> {
    claims
        .iter()
        .filter(|(key, _)| key.starts_with(prefix))
        .map(|(_, value)| value.clone())
        .collect()
}

fn title(section: &str) -> String {
    section
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn sha256(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledRuntimeArtifact {
    pub projection: RuntimeArtifactProjection,
    pub body: String,
}

pub fn compile_cross_harness_artifact(
    target: &str,
    layers: &CompiledPromptLayers,
) -> Result<CompiledRuntimeArtifact, String> {
    let artifact_ref = match target {
        "pi" => ".pi/APPEND_SYSTEM.md",
        "claude" => "CLAUDE.md",
        "gemini" => "GEMINI.md",
        "copilot" => ".github/copilot-instructions.md",
        "generic" => "AGENTS.md",
        _ => return Err("unsupported_harness_target".into()),
    };
    let content = &layers.variant.body;
    Ok(CompiledRuntimeArtifact {
        projection: RuntimeArtifactProjection {
            target: target.into(),
            artifact_ref: artifact_ref.into(),
            content_sha256: sha256(content),
            verified: true,
        },
        body: content.clone(),
    })
}

pub fn compile_agents_artifacts(
    layers: &CompiledPromptLayers,
    nested_deltas: &BTreeMap<String, Vec<String>>,
) -> Result<Vec<CompiledRuntimeArtifact>, String> {
    let mut artifacts = vec![compile_cross_harness_artifact("generic", layers)?];
    for (path, deltas) in nested_deltas {
        let relative = std::path::Path::new(path);
        if relative.is_absolute()
            || relative
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
            || relative.file_name().and_then(|name| name.to_str()) != Some("AGENTS.md")
            || deltas.is_empty()
        {
            return Err("invalid_nested_agents_projection".into());
        }
        let body = format!(
            "<!-- focusa-managed delta-only -->\n{}",
            deltas
                .iter()
                .map(|delta| format!("- {delta}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        artifacts.push(CompiledRuntimeArtifact {
            projection: RuntimeArtifactProjection {
                target: "generic_nested".into(),
                artifact_ref: path.clone(),
                content_sha256: sha256(&body),
                verified: true,
            },
            body,
        });
    }
    Ok(artifacts)
}

pub fn supported_harness_targets() -> BTreeSet<&'static str> {
    ["pi", "claude", "gemini", "copilot", "generic"]
        .into_iter()
        .collect()
}
