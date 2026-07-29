use crate::agent_runtime_constitution::*;
use chrono::Utc;
use std::collections::BTreeMap;

fn source(trust: InstructionTrustClass) -> InstructionSource {
    InstructionSource {
        source_id: "source-1".into(),
        source_ref: "AGENTS.md".into(),
        content_sha256: "a".repeat(64),
        authority: InstructionSourceAuthority::ProjectRoot,
        trust,
        freshness: InstructionFreshness::Current,
        scope_ref: "/project".into(),
        discovered_at: Utc::now(),
    }
}

fn constitution() -> ProjectAgentRuntimeConstitution {
    ProjectAgentRuntimeConstitution {
        schema: "focusa.project_agent_runtime_constitution.v1".into(),
        constitution_id: "constitution-1".into(),
        project_ref: "project:focusa".into(),
        genesis_ref: "genesis:1".into(),
        approved_spec_ref: "docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md".into(),
        agent_identity_ref: AgentIdentityReference("agent:pi".into()),
        base_agent_constitution_ref: ConstitutionalKernelReference("kernel:v1".into()),
        role_profile_ref: RoleProfileReference("role:builder".into()),
        revision: 1,
        status: RuntimeConstitutionLifecycleState::Approved,
        operating_contract: AgentOperatingContract {
            purpose: "Implement approved project work".into(),
            responsibilities: vec!["preserve authority".into()],
            non_responsibilities: vec!["invent operator intent".into()],
            authority_order: vec!["operator".into(), "project".into()],
            execution_boundaries: vec!["project root".into()],
            output_contracts: vec!["typed receipt".into()],
        },
        instruction_sources: vec![source(InstructionTrustClass::TrustedProject)],
        claim_refs: vec!["claim:1".into()],
        resolution_refs: vec![],
        awareness_contract_ref: RuntimeAwarenessContractReference("awareness:v1".into()),
        extensions: BTreeMap::new(),
    }
}

#[test]
fn valid_constitution_round_trips() {
    let item = constitution();
    assert_eq!(item.validate(), Ok(()));
    let encoded = serde_json::to_string(&item).unwrap();
    let decoded: ProjectAgentRuntimeConstitution = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.constitution_id, item.constitution_id);
    assert_eq!(decoded.instruction_sources.len(), 1);
}

#[test]
fn active_constitution_rejects_untrusted_or_duplicate_sources() {
    let mut item = constitution();
    item.status = RuntimeConstitutionLifecycleState::Active;
    item.instruction_sources = vec![
        source(InstructionTrustClass::Untrusted),
        source(InstructionTrustClass::TrustedProject),
    ];
    let errors = item.validate().unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.starts_with("active_untrusted_source"))
    );
    assert!(
        errors
            .iter()
            .any(|error| error == "invalid_or_duplicate_instruction_source")
    );
}

#[test]
fn all_reducer_event_names_are_stable() {
    let version = RuntimeConstitutionVersion {
        version: "1".into(),
        parent_version: None,
        content_sha256: "b".repeat(64),
        lifecycle: RuntimeConstitutionLifecycleState::Draft,
        created_at: Utc::now(),
    };
    let events = vec![
        RuntimeConstitutionEvent::InstructionSourceScanStarted(serde_json::json!({})),
        RuntimeConstitutionEvent::RuntimeConstitutionDrafted(version.clone()),
        RuntimeConstitutionEvent::RuntimeConstitutionApproved(version.clone()),
        RuntimeConstitutionEvent::RuntimeConstitutionActivated(version.clone()),
        RuntimeConstitutionEvent::ContractRollbackActivated(version),
    ];
    assert_eq!(events[0].event_name(), "instruction.source_scan_started");
    assert_eq!(events[1].event_name(), "runtime_constitution.drafted");
    assert_eq!(events[2].event_name(), "runtime_constitution.approved");
    assert_eq!(events[3].event_name(), "runtime_constitution.activated");
    assert_eq!(events[4].event_name(), "contract.rollback_activated");
}

#[test]
fn prompt_integrity_and_tool_routing_are_typed() {
    let manifest = PromptIntegrityManifest {
        assembly_plan_sha256: "c".repeat(64),
        prompt_sha256: "d".repeat(64),
        signature_ref: None,
        verified: true,
    };
    let routing = ToolRoutingPlan {
        plan_id: "routing:1".into(),
        allowed_tools: ["read".into()].into_iter().collect(),
        denied_tools: [("shell".into(), "not granted".into())]
            .into_iter()
            .collect(),
        confirmation_required: ["release".into()].into_iter().collect(),
    };
    assert!(manifest.verified);
    assert!(routing.allowed_tools.contains("read"));
    assert_eq!(routing.denied_tools.get("shell").unwrap(), "not granted");
}
