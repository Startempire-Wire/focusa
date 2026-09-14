//! One authority envelope for Ask, ProjectIdentity, Continuity, Trajectory, and Workpoint.

use crate::silent_session::{
    OperatorAskBinding, SilentSession, SilentSessionId, SilentSessionRun, SilentSessionRunId,
    WorkpointBinding,
};
use crate::silent_session_bootstrap::{AgentBootstrapPacket, SilentSessionOntologyBindings};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

pub const SILENT_SESSION_AUTHORITY_SCHEMA: &str = "focusa.silent_session_authority.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorSteeringBinding {
    pub steering_ref: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub workpoint_ref: WorkpointBinding,
    pub exact_text: String,
    pub text_sha256: String,
    pub revision: u64,
    pub occurred_at: DateTime<Utc>,
}

impl OperatorSteeringBinding {
    #[allow(clippy::too_many_arguments)]
    pub fn capture(
        steering_ref: impl Into<String>,
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        generation: u64,
        project_identity_ref: impl Into<String>,
        continuity_id: impl Into<String>,
        workpoint_ref: WorkpointBinding,
        exact_text: impl Into<String>,
        revision: u64,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        let exact_text = exact_text.into();
        Self {
            steering_ref: steering_ref.into(),
            session_id,
            run_id,
            generation,
            project_identity_ref: project_identity_ref.into(),
            continuity_id: continuity_id.into(),
            workpoint_ref,
            text_sha256: hex::encode(Sha256::digest(exact_text.as_bytes())),
            exact_text,
            revision,
            occurred_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectionSource {
    OperatorAsk,
    OperatorSteering,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentDirection {
    pub source: DirectionSource,
    pub source_ref: String,
    pub exact_text: String,
    pub text_sha256: String,
    pub revision: u64,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionAuthorityEnvelope {
    pub schema: String,
    pub envelope_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: u64,
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub trajectory_ref: String,
    pub trajectory_is_advisory: bool,
    pub trajectory_waypoints: Vec<String>,
    pub active_gap: String,
    pub workpoint_ref: WorkpointBinding,
    pub workpoint_is_action_authority: bool,
    pub exact_operator_ask: OperatorAskBinding,
    pub ontology: SilentSessionOntologyBindings,
    pub context_risk_refs: Vec<String>,
    pub valid_next_tools: Vec<String>,
    pub current_direction: CurrentDirection,
    pub exact_next_action: String,
    pub active_object_refs: Vec<String>,
    pub hook_refs: Vec<String>,
    pub blockers: Vec<String>,
    pub do_not_drift: Vec<String>,
    pub applied_steering_refs: Vec<String>,
    pub ignored_stale_steering_refs: Vec<String>,
}

pub fn compose_authority_envelope(
    session: &SilentSession,
    run: &SilentSessionRun,
    bootstrap: &AgentBootstrapPacket,
    steering: &[OperatorSteeringBinding],
    now: DateTime<Utc>,
) -> Result<SilentSessionAuthorityEnvelope, SilentSessionAuthorityError> {
    session
        .validate()
        .map_err(|_| SilentSessionAuthorityError::InvalidSession)?;
    run.validate(session)
        .map_err(|_| SilentSessionAuthorityError::InvalidRun)?;
    bootstrap
        .validate(now)
        .map_err(|_| SilentSessionAuthorityError::InvalidBootstrap)?;

    if run.session_id != session.session_id
        || bootstrap.session_id != session.session_id
        || bootstrap.run_id != run.run_id
        || bootstrap.generation != run.generation
    {
        return Err(SilentSessionAuthorityError::RuntimeScopeMismatch);
    }
    if bootstrap.project_identity.project_identity_ref != session.project_identity_ref
        || bootstrap.project_identity.project_root != session.project_root
        || bootstrap.continuity_id != session.continuity_id
        || bootstrap.trajectory.project_identity_ref != session.project_identity_ref
        || bootstrap.trajectory.continuity_id != session.continuity_id
        || bootstrap.workpoint.project_identity_ref != session.project_identity_ref
        || bootstrap.workpoint.continuity_id != session.continuity_id
        || bootstrap.operator_ask != session.operator_ask
    {
        return Err(SilentSessionAuthorityError::FocusaScopeMismatch);
    }
    let trajectory_ref = session
        .trajectory_ref
        .as_ref()
        .filter(|value| **value == bootstrap.trajectory.trajectory_ref)
        .cloned()
        .ok_or(SilentSessionAuthorityError::TrajectoryMismatch)?;
    let workpoint_ref = session
        .workpoint_ref
        .as_ref()
        .filter(|value| **value == bootstrap.workpoint.workpoint_ref)
        .cloned()
        .ok_or(SilentSessionAuthorityError::WorkpointMismatch)?;

    let (current_direction, applied_steering_refs, ignored_stale_steering_refs) =
        select_current_direction(session, run, &workpoint_ref, steering)?;

    Ok(SilentSessionAuthorityEnvelope {
        schema: SILENT_SESSION_AUTHORITY_SCHEMA.into(),
        envelope_id: Uuid::now_v7(),
        generated_at: now,
        session_id: session.session_id,
        run_id: run.run_id,
        generation: run.generation,
        project_root: session.project_root.clone(),
        project_identity_ref: session.project_identity_ref.clone(),
        continuity_id: session.continuity_id.clone(),
        trajectory_ref,
        trajectory_is_advisory: true,
        trajectory_waypoints: bootstrap.trajectory.waypoints.clone(),
        active_gap: bootstrap.trajectory.active_gap.clone(),
        workpoint_ref,
        workpoint_is_action_authority: true,
        exact_operator_ask: session.operator_ask.clone(),
        ontology: bootstrap.ontology.clone(),
        context_risk_refs: bootstrap.context.risk_refs.clone(),
        valid_next_tools: bootstrap.context.valid_next_tools.clone(),
        current_direction,
        exact_next_action: bootstrap.exact_next_action.clone(),
        active_object_refs: bootstrap.active_object_refs.clone(),
        hook_refs: bootstrap.hook_refs.clone(),
        blockers: bootstrap.blockers.clone(),
        do_not_drift: bootstrap.do_not_drift.clone(),
        applied_steering_refs,
        ignored_stale_steering_refs,
    })
}

fn select_current_direction(
    session: &SilentSession,
    run: &SilentSessionRun,
    workpoint_ref: &WorkpointBinding,
    steering: &[OperatorSteeringBinding],
) -> Result<(CurrentDirection, Vec<String>, Vec<String>), SilentSessionAuthorityError> {
    let ask = &session.operator_ask;
    let mut current = CurrentDirection {
        source: DirectionSource::OperatorAsk,
        source_ref: ask.ask_ref.clone(),
        exact_text: ask.exact_text.clone(),
        text_sha256: ask.text_sha256.clone(),
        revision: ask.revision,
        observed_at: ask.captured_at,
    };
    let mut applied = Vec::new();
    let mut stale = Vec::new();
    let mut revisions = BTreeSet::from([ask.revision]);
    let mut ordered = steering.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|event| {
        (
            event.revision,
            event.occurred_at,
            event.steering_ref.as_str(),
        )
    });

    for event in ordered {
        validate_steering(event, session, run, workpoint_ref)?;
        if !revisions.insert(event.revision) {
            return Err(SilentSessionAuthorityError::DuplicateDirectionRevision);
        }
        if event.revision <= current.revision {
            stale.push(event.steering_ref.clone());
            continue;
        }
        if current.source == DirectionSource::OperatorSteering {
            stale.push(current.source_ref.clone());
        }
        current = CurrentDirection {
            source: DirectionSource::OperatorSteering,
            source_ref: event.steering_ref.clone(),
            exact_text: event.exact_text.clone(),
            text_sha256: event.text_sha256.clone(),
            revision: event.revision,
            observed_at: event.occurred_at,
        };
        applied.push(event.steering_ref.clone());
    }
    Ok((current, applied, stale))
}

fn validate_steering(
    event: &OperatorSteeringBinding,
    session: &SilentSession,
    run: &SilentSessionRun,
    workpoint_ref: &WorkpointBinding,
) -> Result<(), SilentSessionAuthorityError> {
    if event.steering_ref.trim().is_empty()
        || event.exact_text.trim().is_empty()
        || event.revision == 0
        || event.generation == 0
        || event.text_sha256 != hex::encode(Sha256::digest(event.exact_text.as_bytes()))
    {
        return Err(SilentSessionAuthorityError::InvalidSteering);
    }
    if event.session_id != session.session_id
        || event.run_id != run.run_id
        || event.generation != run.generation
        || event.project_identity_ref != session.project_identity_ref
        || event.continuity_id != session.continuity_id
        || &event.workpoint_ref != workpoint_ref
    {
        return Err(SilentSessionAuthorityError::SteeringScopeMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SilentSessionAuthorityError {
    #[error("silent session authority requires a valid session")]
    InvalidSession,
    #[error("silent session authority requires a valid run")]
    InvalidRun,
    #[error("silent session authority requires a fresh valid bootstrap")]
    InvalidBootstrap,
    #[error("session, run, and bootstrap scope differ")]
    RuntimeScopeMismatch,
    #[error("ProjectIdentity or Continuity differs across authority inputs")]
    FocusaScopeMismatch,
    #[error("Trajectory binding differs from the session")]
    TrajectoryMismatch,
    #[error("Workpoint binding differs from the session")]
    WorkpointMismatch,
    #[error("operator steering binding is invalid")]
    InvalidSteering,
    #[error("operator steering is bound to another authority scope")]
    SteeringScopeMismatch,
    #[error("operator direction revisions must be unique")]
    DuplicateDirectionRevision,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::silent_session::{
        ModelBinding, SilentSessionConfigRevisionId, SilentSessionHealth,
        SilentSessionLifecycleState, SilentSessionVersions, WorkspaceBinding, WorkspaceStrategy,
    };
    use crate::silent_session_bootstrap::{
        AGENT_BOOTSTRAP_PACKET_SCHEMA, BootstrapWorkspaceBinding, ContextBootstrapBinding,
        ProjectIdentityBootstrapBinding, TrajectoryBootstrapBinding, TrajectoryBootstrapStatus,
        WorkpointBootstrapBinding,
    };
    use chrono::Duration;

    fn versions() -> SilentSessionVersions {
        SilentSessionVersions {
            silent_session_schema_version: 1,
            config_schema_version: 1,
            event_schema_version: 1,
            daemon_runner_protocol_version: 1,
            harness_adapter_protocol_version: 1,
            process_backend_protocol_version: 1,
            stream_chunk_format_version: 1,
            receipt_mapping_version: 1,
        }
    }

    fn session() -> SilentSession {
        let now = Utc::now();
        SilentSession {
            schema: crate::silent_session::SILENT_SESSION_SCHEMA.into(),
            versions: versions(),
            session_id: SilentSessionId::new(),
            display_name: "authority".into(),
            created_at: now,
            created_by_actor_ref: "actor:test".into(),
            operator_principal_ref: "operator:test".into(),
            os_execution_user: "runner".into(),
            project_root: crate::test_support::absolute_path("silent-authority-project"),
            project_identity_ref: "project:focusa".into(),
            continuity_id: "continuity:test".into(),
            trajectory_ref: Some("trajectory:test".into()),
            workpoint_ref: Some(WorkpointBinding {
                workpoint_id: "workpoint:test".into(),
                revision: Some(3),
            }),
            work_item_ref: Some("focusa-a6yq6.7.1".into()),
            operator_ask: OperatorAskBinding::capture(
                "ask:1",
                "Implement the governed slice",
                1,
                now,
            ),
            mission: "Implement the governed slice".into(),
            lifecycle_state: SilentSessionLifecycleState::Running,
            health: SilentSessionHealth::Healthy,
            semantic_observation: None,
            active_run_id: None,
            config_revision_id: SilentSessionConfigRevisionId::new(),
            writer_lease_ref: None,
            retention_policy_ref: "retention:test".into(),
            receipt_refs: vec![],
        }
    }

    fn run(session: &SilentSession) -> SilentSessionRun {
        SilentSessionRun {
            schema: crate::silent_session::SILENT_SESSION_RUN_SCHEMA.into(),
            versions: versions(),
            run_id: SilentSessionRunId::new(),
            session_id: session.session_id,
            generation: 1,
            runner_id: "runner:test".into(),
            adapter_id: "adapter:test".into(),
            process_backend_id: "process:test".into(),
            requested_model_binding: ModelBinding {
                provider: "provider".into(),
                model: "model".into(),
                thinking: None,
            },
            effective_model_binding: None,
            observed_model_binding: None,
            workspace_binding: WorkspaceBinding {
                workspace_id: "workspace:test".into(),
                root: crate::test_support::absolute_path("silent-authority-worktree"),
                strategy: WorkspaceStrategy::IsolatedWorktree,
                branch_ref: Some("aaaaaaaa".into()),
            },
            process_identity: None,
            harness_native_session_ref: None,
            started_at: Some(Utc::now()),
            ended_at: None,
            exit_status: None,
            current_event_seq: 0,
            output_stream_refs: vec![],
            runtime_checkpoint_refs: vec![],
            workpoint_checkpoint_refs: vec![],
        }
    }

    fn bootstrap(session: &SilentSession, run: &SilentSessionRun) -> AgentBootstrapPacket {
        let now = Utc::now();
        let fresh_until = now + Duration::minutes(5);
        let workpoint_ref = session.workpoint_ref.clone().unwrap();
        AgentBootstrapPacket {
            schema: AGENT_BOOTSTRAP_PACKET_SCHEMA.into(),
            packet_id: Uuid::now_v7(),
            session_id: session.session_id,
            run_id: run.run_id,
            generation: run.generation,
            generated_at: now,
            fresh_until,
            project_identity: ProjectIdentityBootstrapBinding {
                project_identity_ref: session.project_identity_ref.clone(),
                project_root: session.project_root.clone(),
                fingerprint: "focusa-authority-fingerprint".into(),
                snapshot_ref: "project-snapshot:authority".into(),
                snapshot_sha256: "1".repeat(64),
                verified_at: now,
                fresh_until,
            },
            continuity_id: session.continuity_id.clone(),
            trajectory: TrajectoryBootstrapBinding {
                trajectory_ref: session.trajectory_ref.clone().unwrap(),
                project_identity_ref: session.project_identity_ref.clone(),
                continuity_id: session.continuity_id.clone(),
                snapshot_ref: "trajectory-snapshot:authority".into(),
                snapshot_sha256: "2".repeat(64),
                generated_at: now,
                fresh_until,
                status: TrajectoryBootstrapStatus::CanonicalAdvisory,
                waypoints: vec!["close authority gap".into()],
                active_gap: "authority envelope is not verified".into(),
            },
            workpoint: WorkpointBootstrapBinding {
                workpoint_ref: workpoint_ref.clone(),
                project_identity_ref: session.project_identity_ref.clone(),
                continuity_id: session.continuity_id.clone(),
                snapshot_ref: "workpoint-snapshot:authority".into(),
                snapshot_sha256: "3".repeat(64),
                generated_at: now,
                fresh_until,
            },
            operator_ask: session.operator_ask.clone(),
            context: ContextBootstrapBinding {
                context_packet_ref: "context-packet:authority".into(),
                project_identity_ref: session.project_identity_ref.clone(),
                continuity_id: session.continuity_id.clone(),
                trajectory_ref: session.trajectory_ref.clone().unwrap(),
                workpoint_ref,
                source_snapshot_ref: "context-snapshot:authority".into(),
                packet_sha256: "4".repeat(64),
                generated_at: now,
                fresh_until,
                advisory: true,
                canonical: false,
                canonical_mutation_allowed: false,
                selected_context: vec!["spec:133".into()],
                excluded_context: vec![],
                risk_refs: vec!["risk:authority-drift".into()],
                valid_next_tools: vec!["tool:workpoint-resume".into()],
            },
            ontology: SilentSessionOntologyBindings {
                agent_identity_ref: "agent:authority-test".into(),
                actor_instance_ref: "actor-instance:authority-test".into(),
                role_profile_ref: "role:authority-test".into(),
                capability_profile_ref: "capability:authority-compose".into(),
                permission_profile_ref: "permission:read-authority".into(),
                responsibility_ref: "responsibility:authority-test".into(),
                handoff_boundary_ref: "handoff:operator".into(),
                execution_context_ref: "execution-context:authority-test".into(),
                tool_surface_ref: "tool-surface:core-test".into(),
                affordance_ref: "affordance:compose".into(),
                resource_ref: "resource:test".into(),
                cost_model_ref: "cost-model:test".into(),
                reliability_profile_ref: "reliability:strict".into(),
                reversibility_profile_ref: "reversibility:pure".into(),
                work_item_ref: "focusa-a6yq6.7.1".into(),
                action_intent_ref: "action-intent:compose".into(),
                blocker_ref: "blocker:server-proof".into(),
                verification_record_ref: "verification:authority".into(),
                evidence_artifact_ref: "evidence:authority".into(),
            },
            work_item_ref: session.work_item_ref.clone(),
            workspace: BootstrapWorkspaceBinding {
                workspace_ref: "workspace:authority".into(),
                workspace_root: run.workspace_binding.root.clone(),
            },
            model: run.requested_model_binding.clone(),
            role_ref: "role:authority-test".into(),
            mission: session.mission.clone(),
            exact_next_action: "compose authority envelope".into(),
            active_object_refs: vec!["spec:133".into()],
            hook_refs: vec!["hook:before-run".into()],
            blockers: vec!["server proof pending".into()],
            do_not_drift: vec!["do not use generic trajectory".into()],
            evidence_refs: vec!["test:authority-compose".into()],
            proof_gaps: vec!["server build".into()],
            completion_expectations: vec!["authority envelope verified".into()],
        }
    }

    fn steering(
        session: &SilentSession,
        run: &SilentSessionRun,
        revision: u64,
        text: &str,
    ) -> OperatorSteeringBinding {
        OperatorSteeringBinding::capture(
            format!("steering:{revision}"),
            session.session_id,
            run.run_id,
            run.generation,
            session.project_identity_ref.clone(),
            session.continuity_id.clone(),
            session.workpoint_ref.clone().unwrap(),
            text,
            revision,
            Utc::now() + Duration::seconds(revision as i64),
        )
    }

    #[test]
    fn newer_operator_steering_outranks_exact_ask_and_older_steering() {
        let session = session();
        let run = run(&session);
        let current = select_current_direction(
            &session,
            &run,
            session.workpoint_ref.as_ref().unwrap(),
            &[
                steering(&session, &run, 3, "new direction"),
                steering(&session, &run, 2, "older direction"),
            ],
        )
        .unwrap();
        assert_eq!(current.0.source, DirectionSource::OperatorSteering);
        assert_eq!(current.0.exact_text, "new direction");
        assert_eq!(current.0.revision, 3);
        assert!(current.2.contains(&"steering:2".into()));
    }

    #[test]
    fn compose_binds_every_focusa_authority_and_rejects_project_mismatch() {
        let mut session = session();
        let run = run(&session);
        session.active_run_id = Some(run.run_id);
        let packet = bootstrap(&session, &run);
        let envelope = compose_authority_envelope(
            &session,
            &run,
            &packet,
            &[steering(&session, &run, 2, "follow corrected direction")],
            Utc::now(),
        )
        .unwrap();
        assert_eq!(envelope.exact_operator_ask, session.operator_ask);
        assert_eq!(
            envelope.current_direction.exact_text,
            "follow corrected direction"
        );
        assert_eq!(envelope.trajectory_waypoints, vec!["close authority gap"]);
        assert_eq!(envelope.active_gap, "authority envelope is not verified");
        assert_eq!(envelope.exact_next_action, "compose authority envelope");
        assert_eq!(envelope.hook_refs, vec!["hook:before-run"]);
        assert_eq!(envelope.ontology.agent_identity_ref, "agent:authority-test");
        assert_eq!(envelope.context_risk_refs, vec!["risk:authority-drift"]);
        assert_eq!(envelope.valid_next_tools, vec!["tool:workpoint-resume"]);
        assert!(envelope.trajectory_is_advisory);
        assert!(envelope.workpoint_is_action_authority);

        let mut wrong_project = session;
        wrong_project.project_identity_ref = "project:other".into();
        assert_eq!(
            compose_authority_envelope(&wrong_project, &run, &packet, &[], Utc::now()),
            Err(SilentSessionAuthorityError::FocusaScopeMismatch)
        );
    }

    #[test]
    fn stale_or_cross_scope_prompts_cannot_outrank_current_authority() {
        let session = session();
        let run = run(&session);
        let stale = OperatorSteeringBinding::capture(
            "steering:stale",
            session.session_id,
            run.run_id,
            run.generation,
            session.project_identity_ref.clone(),
            session.continuity_id.clone(),
            session.workpoint_ref.clone().unwrap(),
            "stale",
            0,
            Utc::now(),
        );
        assert_eq!(
            select_current_direction(
                &session,
                &run,
                session.workpoint_ref.as_ref().unwrap(),
                &[stale],
            ),
            Err(SilentSessionAuthorityError::InvalidSteering)
        );

        let mut cross_scope = steering(&session, &run, 2, "wrong scope");
        cross_scope.continuity_id = "continuity:other".into();
        assert_eq!(
            select_current_direction(
                &session,
                &run,
                session.workpoint_ref.as_ref().unwrap(),
                &[cross_scope],
            ),
            Err(SilentSessionAuthorityError::SteeringScopeMismatch)
        );
    }
}
