use crate::agent_runtime_constitution::*;
use crate::agent_runtime_constitution_authority::instruction_source_from_bytes;
use crate::agent_runtime_constitution_orchestrator::*;
use std::collections::{BTreeMap, BTreeSet};

fn input(confirmed: bool) -> CristRuntimeInput {
    CristRuntimeInput {
        project_ref: "project:focusa".into(), genesis_ref: "genesis:1".into(),
        approved_spec_ref: "docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md".into(),
        operator_confirmed: confirmed, agent_identity_ref: AgentIdentityReference("agent:pi".into()),
        kernel_ref: ConstitutionalKernelReference("kernel:1".into()), role_ref: RoleProfileReference("role:builder".into()),
        mission: "Implement the approved mission".into(), responsibilities: vec!["Follow authority".into()],
        non_responsibilities: vec!["Never invent intent".into()], authority_order: vec!["operator".into(), "project".into()],
        execution_boundaries: vec!["verified project".into()], output_contracts: vec!["evidence before completion".into()],
        awareness_ref: RuntimeAwarenessContractReference("awareness:1".into()),
        instruction_sources: vec![instruction_source_from_bytes("source", "AGENTS.md", b"rules", InstructionSourceAuthority::ProjectRoot, InstructionTrustClass::TrustedProject, "/project")],
        target_profile: TargetCapabilityProfile {
            profile_version: "1".into(), target: "pi".into(),
            supported_layers: [PromptLayer::HarnessSystem, PromptLayer::FocusaKernel, PromptLayer::ProjectConstitution].into_iter().collect::<BTreeSet<_>>(),
            supported_features: ["append".into()].into_iter().collect(), unsupported_features: BTreeMap::new(),
            max_prompt_bytes: 64 * 1024, supports_session_pinning: true,
        },
        changed_paths: vec!["crates/focusa-core/src/lib.rs".into()],
    }
}

#[test]
fn crist_composition_is_operator_gated_and_complete() {
    assert!(
        compose_runtime_constitution(input(false))
            .unwrap_err()
            .contains(&"approved_crist_or_operator_confirmation_required".into())
    );
    let result = compose_runtime_constitution(input(true)).unwrap();
    assert_eq!(
        result.constitution.status,
        RuntimeConstitutionLifecycleState::Draft
    );
    assert!(
        result
            .constitution
            .constitution_id
            .starts_with("runtime-constitution:project-focusa")
    );
    assert!(
        result
            .stable_obligation_refs
            .contains(STABLE_TEMPORAL_OBLIGATION)
    );
    assert!(
        result
            .stable_obligation_refs
            .contains(STABLE_PRESENCE_OBLIGATION)
    );
    assert_eq!(result.assembly_plan.target_profile.target, "pi");
    assert!(
        result
            .validation_matrix
            .rules
            .iter()
            .any(|rule| rule.rule_id == "rust-check")
    );
}

#[test]
fn incomplete_operating_contract_fails_closed() {
    let mut invalid = input(true);
    invalid.output_contracts.clear();
    invalid.instruction_sources.clear();
    let errors = compose_runtime_constitution(invalid).unwrap_err();
    assert!(errors.contains(&"operating_contract_incomplete".into()));
    assert!(errors.contains(&"instruction_source_inventory_required".into()));
}
