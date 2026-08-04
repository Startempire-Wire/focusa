use agent_stateful_cognitive_runtime::{
    CognitiveLoopOwner, RuntimeBinding, RuntimeEpochIdentity, RuntimeMode,
};
use letta_adapter::{
    LettaResumeDecision, LettaScopeBinding, LettaTurnReceipt,
    checkpoint::{AgentCheckpoint, AgentLifecycleStatus},
    operations::{LettaRuntimeEvent, LettaRuntimeProjection},
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

fn fixtures() -> (RuntimeBinding, LettaScopeBinding) {
    let epoch_id = Uuid::now_v7();
    let runtime = RuntimeBinding {
        schema: RuntimeBinding::SCHEMA.into(),
        mode: RuntimeMode::LettaManaged,
        owner: CognitiveLoopOwner::Letta,
        epoch: RuntimeEpochIdentity {
            epoch_id,
            project_root: "/project".into(),
            continuity_id: "continuity".into(),
            agent_instance_id: "agent-instance".into(),
            native_session_id: None,
        },
        provider_agent_id: Some("letta-agent".into()),
        admitted_client_tools: BTreeSet::new(),
    };
    let scope = LettaScopeBinding {
        schema: "focusa.letta_scope_binding.v1".into(),
        project_root: "/project".into(),
        continuity_id: "continuity".into(),
        workpoint_id: "workpoint-1".into(),
        provider_agent_id: "letta-agent".into(),
        provider_thread_id: "thread-1".into(),
        epoch_id,
        replay_key_prefix: "continuity/workpoint-1".into(),
    };
    (runtime, scope)
}

#[test]
fn create_read_send_resume_checkpoint_replay_and_unknown_calls_are_typed() {
    let (runtime, scope) = fixtures();
    let (mut projection, create) =
        LettaRuntimeProjection::create("op-create", runtime.clone(), scope.clone()).unwrap();
    assert_eq!(create.operation, "create");
    assert_eq!(projection.read(&scope).unwrap(), projection);
    let mut foreign = scope.clone();
    foreign.provider_thread_id = "foreign".into();
    assert!(projection.read(&foreign).is_err());

    let turn = LettaTurnReceipt {
        schema: "focusa.letta_turn_receipt.v1".into(),
        request_id: Uuid::now_v7(),
        event_id: "event-1".into(),
        provider_agent_id: "letta-agent".into(),
        epoch_id: runtime.epoch.epoch_id,
        response_digest: "sha256:response".into(),
        evidence_refs: vec!["evidence:turn".into()],
        tool_continuations: 0,
    };
    let send_event = LettaRuntimeEvent::SendCommitted {
        operation_id: "op-send".into(),
        receipt: turn,
    };
    let send = projection.apply(send_event.clone()).unwrap();
    assert_eq!(send.operation, "send");
    assert_eq!(projection.apply(send_event).unwrap(), send);

    let resume = projection
        .apply(LettaRuntimeEvent::ResumeEvaluated {
            operation_id: "op-resume".into(),
            decision: LettaResumeDecision {
                schema: "focusa.letta_resume_decision.v1".into(),
                status: "quarantined".into(),
                binding: None,
                failure_class: Some("foreign_scope_or_thread".into()),
                quarantined_candidate_digest: Some("sha256:foreign".into()),
            },
        })
        .unwrap();
    assert!(resume.recovery_required);

    let checkpoint = AgentCheckpoint::create(
        runtime,
        AgentLifecycleStatus::Alive,
        projection.state_revision,
        BTreeMap::from([("working_memory".into(), "sha256:memory".into())]),
        vec!["receipt:turn".into()],
        None,
    )
    .unwrap();
    projection
        .apply(LettaRuntimeEvent::CheckpointCommitted {
            operation_id: "op-checkpoint".into(),
            checkpoint,
        })
        .unwrap();
    let restarted: LettaRuntimeProjection =
        serde_json::from_slice(&serde_json::to_vec(&projection).unwrap()).unwrap();
    assert_eq!(restarted, projection);
    assert!(
        serde_json::from_str::<LettaRuntimeEvent>(
            r#"{"operation":"invented_sdk_call","operation_id":"bad"}"#
        )
        .is_err()
    );
}
