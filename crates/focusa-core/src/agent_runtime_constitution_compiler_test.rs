use crate::agent_runtime_constitution::*;
use crate::agent_runtime_constitution_authority::instruction_source_from_bytes;
use crate::agent_runtime_constitution_compiler::*;
use std::collections::{BTreeMap, BTreeSet};

fn input(mode: PromptCompilationMode) -> PromptCompileInput {
    let source = instruction_source_from_bytes(
        "source-1",
        "AGENTS.md",
        b"rules",
        InstructionSourceAuthority::ProjectRoot,
        InstructionTrustClass::TrustedProject,
        "/project",
    );
    let contract = AgentOperatingContract {
        purpose: "Deliver the approved project mission".into(),
        responsibilities: vec!["Follow explicit authority".into()],
        non_responsibilities: vec!["Never invent operator intent".into()],
        authority_order: vec!["Operator steering".into(), "Project constitution".into()],
        execution_boundaries: vec!["Execute in the verified project root".into()],
        output_contracts: vec!["Attach tests and evidence before completion".into()],
    };
    let constitution = ProjectAgentRuntimeConstitution {
        schema: "focusa.project_agent_runtime_constitution.v1".into(),
        constitution_id: "constitution-1".into(),
        project_ref: "project:focusa".into(),
        genesis_ref: "genesis-1".into(),
        approved_spec_ref: "docs/140.md".into(),
        agent_identity_ref: AgentIdentityReference("agent:pi".into()),
        base_agent_constitution_ref: ConstitutionalKernelReference("kernel:1".into()),
        role_profile_ref: RoleProfileReference("role:builder".into()),
        revision: 1,
        status: RuntimeConstitutionLifecycleState::Approved,
        operating_contract: contract,
        instruction_sources: vec![source],
        claim_refs: vec![],
        resolution_refs: vec![],
        awareness_contract_ref: RuntimeAwarenessContractReference("awareness:1".into()),
        extensions: BTreeMap::new(),
    };
    let claims = [
        ("coordination", "Coordinate through Beads"),
        ("tool", "Use typed tools before shell"),
        ("skill", "Load matching skills"),
        ("lifecycle", "Close only after proof"),
        ("temporal", "Do not invent deadlines"),
        ("presence", "Respect operator interruption state"),
        ("epistemic", "Separate observation from inference"),
        ("communication", "Report concise evidence"),
        ("recovery", "Resume from canonical checkpoints"),
    ]
    .into_iter()
    .map(|(key, value)| (key.into(), value.into()))
    .collect();
    PromptCompileInput {
        plan: SystemPromptAssemblyPlan {
            plan_id: "plan-1".into(),
            ordered_layers: vec![PromptLayer::FocusaKernel, PromptLayer::ProjectConstitution],
            source_refs: vec!["AGENTS.md".into()],
            excluded_claims: BTreeMap::new(),
            target_profile: TargetCapabilityProfile {
                profile_version: "1".into(),
                target: "pi".into(),
                supported_layers: [
                    PromptLayer::HarnessSystem,
                    PromptLayer::FocusaKernel,
                    PromptLayer::ProjectConstitution,
                ]
                .into_iter()
                .collect::<BTreeSet<_>>(),
                supported_features: ["append".into(), "replace".into(), "runtime_compiled".into()]
                    .into_iter()
                    .collect(),
                unsupported_features: BTreeMap::new(),
                max_prompt_bytes: 64 * 1024,
                supports_session_pinning: true,
            },
        },
        constitution,
        mode,
        replacement_approved: mode == PromptCompilationMode::Replace,
        baseline_evaluation_ref: (mode == PromptCompilationMode::Replace)
            .then(|| "evaluation:baseline".into()),
        harness_default_prompt: Some("Harness default".into()),
        approved_claims: claims,
        session_binding: SessionEnvironmentBinding {
            binding_id: "binding-1".into(),
            project_root: "/project".into(),
            continuity_id: "workstream-1".into(),
            target: "pi".into(),
            environment_sha256: "e".repeat(64),
        },
        dynamic_context: [("current_action".into(), "test".into())]
            .into_iter()
            .collect(),
    }
}

#[test]
fn compiler_is_deterministic_and_keeps_dynamic_context_out_of_stable_hash() {
    let first = compile_prompt(input(PromptCompilationMode::Append)).unwrap();
    let mut changed = input(PromptCompilationMode::Append);
    changed
        .dynamic_context
        .insert("current_action".into(), "release".into());
    let second = compile_prompt(changed).unwrap();
    assert_eq!(first.variant.prompt_sha256, second.variant.prompt_sha256);
    assert_eq!(
        first.stable_constitutional_prompt,
        second.stable_constitutional_prompt
    );
    assert_ne!(first.turn_dynamic_context, second.turn_dynamic_context);
    for section in REQUIRED_PROMPT_SECTIONS {
        assert!(
            first.stable_constitutional_prompt.contains(
                &section
                    .split('_')
                    .map(|part| {
                        let mut chars = part.chars();
                        chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        );
    }
}

#[test]
fn append_preserves_harness_default_and_replace_requires_capability() {
    let appended = compile_prompt(input(PromptCompilationMode::Append)).unwrap();
    assert!(appended.variant.body.starts_with("Harness default"));
    let mut replacement = input(PromptCompilationMode::Replace);
    replacement
        .plan
        .target_profile
        .supported_layers
        .remove(&PromptLayer::HarnessSystem);
    replacement.replacement_approved = false;
    let errors = compile_prompt(replacement).unwrap_err();
    assert!(errors.contains(&"replace_mode_requires_approval_and_baseline_evaluation".into()));
    assert!(errors.contains(&"replace_mode_not_supported_by_target".into()));
}

#[test]
fn cross_harness_outputs_are_explicit_and_unknown_targets_fail_closed() {
    let compiled = compile_prompt(input(PromptCompilationMode::RuntimeCompiled)).unwrap();
    let expected = [
        ("pi", ".pi/APPEND_SYSTEM.md"),
        ("claude", "CLAUDE.md"),
        ("gemini", "GEMINI.md"),
        ("copilot", ".github/copilot-instructions.md"),
        ("generic", "AGENTS.md"),
    ];
    for (target, path) in expected {
        let artifact = compile_cross_harness_artifact(target, &compiled).unwrap();
        assert_eq!(artifact.projection.artifact_ref, path);
        assert!(artifact.projection.verified);
        assert!(!artifact.body.is_empty());
    }
    assert_eq!(
        compile_cross_harness_artifact("unknown", &compiled).unwrap_err(),
        "unsupported_harness_target"
    );
}

#[test]
fn safe_fallback_preserves_harness_default_without_claiming_integrity() {
    let mut invalid = input(PromptCompilationMode::Append);
    invalid.approved_claims.clear();
    let fallback = compile_prompt_with_safe_fallback(invalid);
    assert!(fallback.variant.body.starts_with("Harness default"));
    assert!(fallback.variant.body.contains("focusa-managed-fallback"));
    assert!(!fallback.integrity.verified);
}

#[test]
fn agents_compiler_emits_root_and_delta_only_nested_artifacts() {
    let compiled = compile_prompt(input(PromptCompilationMode::RuntimeCompiled)).unwrap();
    let nested = [(
        "crates/core/AGENTS.md".into(),
        vec!["Run core-specific tests".into()],
    )]
    .into_iter()
    .collect();
    let artifacts = compile_agents_artifacts(&compiled, &nested).unwrap();
    assert_eq!(artifacts[0].projection.artifact_ref, "AGENTS.md");
    assert_eq!(
        artifacts[1].projection.artifact_ref,
        "crates/core/AGENTS.md"
    );
    assert!(artifacts[1].body.contains("delta-only"));
    assert!(!artifacts[1].body.contains("Harness default"));
    let invalid = [("../AGENTS.md".into(), vec!["escape".into()])]
        .into_iter()
        .collect();
    assert_eq!(
        compile_agents_artifacts(&compiled, &invalid).unwrap_err(),
        "invalid_nested_agents_projection"
    );
}

#[test]
fn missing_required_section_blocks_compilation() {
    let mut invalid = input(PromptCompilationMode::Append);
    invalid.approved_claims.remove("recovery");
    let errors = compile_prompt(invalid).unwrap_err();
    assert!(errors.contains(&"missing_required_prompt_section:recovery".into()));
}
