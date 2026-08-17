//! Core Reducer — single-writer state machine.
//!
//! Source: core-reducer.md
//!
//! Contract:
//!   reduce(state: FocusaState, event: FocusaEvent) -> ReductionResult
//!
//! Guarantees:
//!   - Deterministic
//!   - Replayable from event log
//!   - Crash-safe
//!   - Testable in isolation
//!   - Free of side effects
//!
//! Global Invariants (checked pre/post):
//!   1. At most one active Focus Frame exists
//!   2. Every Focus Frame maps to a Beads issue
//!   3. Focus State sections always exist (FocusState Default is valid)
//!   4. Intuition Engine cannot mutate focus (structural — gate events don't touch stack)
//!   5. Focus Gate is advisory only (structural — gate events don't touch stack)
//!   6. Artifacts are immutable once registered
//!   7. Conversation never mutates cognition (structural — no conversation in state)

use crate::focus::stack::rebuild_stack_path;
use crate::focus::state::apply_delta;
use crate::scoped_state::WorkstreamKey;
use crate::types::*;

fn ontology_value_matches_workstream(
    value: &serde_json::Value,
    expected: &Option<WorkstreamKey>,
) -> bool {
    match expected {
        Some(expected) => value.get("workstream") == Some(&serde_json::json!(expected)),
        None => value
            .get("workstream")
            .is_none_or(serde_json::Value::is_null),
    }
}

fn apply_ontology_scope_migration_selection(
    state: &mut FocusaState,
    target: &WorkstreamKey,
    selection: &OntologyScopeMigrationSelection,
) -> Result<OntologyScopeMigrationEntry, ReducerError> {
    if selection.evidence_refs.is_empty()
        || selection
            .evidence_refs
            .iter()
            .any(|value| value.trim().is_empty())
    {
        return Err(ReducerError::InvalidEvent(
            "ontology scope migration requires non-empty evidence per record".to_string(),
        ));
    }

    macro_rules! clone_typed_record {
        ($records:expr) => {{
            let matches = $records
                .iter()
                .filter(|record| {
                    record.workstream.is_none()
                        && ontology_scope_record_hash(*record) == selection.source_hash
                })
                .cloned()
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(ReducerError::InvalidEvent(format!(
                    "ontology migration source hash must identify exactly one unowned record: {} matches={}",
                    selection.source_hash,
                    matches.len()
                )));
            }
            let mut cloned = matches.into_iter().next().expect("one migration match");
            cloned.workstream = Some(target.clone());
            let clone_hash = ontology_scope_record_hash(&cloned);
            $records.push(cloned);
            clone_hash
        }};
    }

    let clone_hash = match selection.record_kind {
        OntologyScopeMigrationRecordKind::Object | OntologyScopeMigrationRecordKind::Link => {
            let records = if matches!(
                selection.record_kind,
                OntologyScopeMigrationRecordKind::Object
            ) {
                &mut state.ontology.objects
            } else {
                &mut state.ontology.links
            };
            let matches = records
                .iter()
                .filter(|record| {
                    record
                        .get("workstream")
                        .is_none_or(serde_json::Value::is_null)
                        && ontology_scope_record_hash(*record) == selection.source_hash
                })
                .cloned()
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(ReducerError::InvalidEvent(format!(
                    "ontology migration source hash must identify exactly one unowned JSON record: {} matches={}",
                    selection.source_hash,
                    matches.len()
                )));
            }
            let mut cloned = matches.into_iter().next().expect("one migration match");
            cloned["workstream"] = serde_json::json!(target);
            let clone_hash = ontology_scope_record_hash(&cloned);
            records.push(cloned);
            clone_hash
        }
        OntologyScopeMigrationRecordKind::Proposal => {
            clone_typed_record!(&mut state.ontology.proposals)
        }
        OntologyScopeMigrationRecordKind::Verification => {
            clone_typed_record!(&mut state.ontology.verifications)
        }
        OntologyScopeMigrationRecordKind::WorkingSetRefresh => {
            clone_typed_record!(&mut state.ontology.working_set_refreshes)
        }
        OntologyScopeMigrationRecordKind::Delta => {
            clone_typed_record!(&mut state.ontology.delta_log)
        }
        OntologyScopeMigrationRecordKind::PreProposal => {
            clone_typed_record!(&mut state.pre.proposals)
        }
    };

    Ok(OntologyScopeMigrationEntry {
        record_kind: selection.record_kind,
        source_hash: selection.source_hash.clone(),
        clone_hash,
        evidence_refs: selection.evidence_refs.clone(),
    })
}

fn rollback_ontology_scope_migration_entry(
    state: &mut FocusaState,
    target: &WorkstreamKey,
    entry: &OntologyScopeMigrationEntry,
) -> Result<(), ReducerError> {
    macro_rules! remove_typed_clone {
        ($records:expr) => {{
            let matches = $records
                .iter()
                .filter(|record| {
                    record.workstream.as_ref() == Some(target)
                        && ontology_scope_record_hash(*record) == entry.clone_hash
                })
                .count();
            if matches != 1 {
                return Err(ReducerError::InvalidEvent(format!(
                    "ontology migration rollback requires one unchanged clone: {} matches={}",
                    entry.clone_hash, matches
                )));
            }
            $records.retain(|record| ontology_scope_record_hash(record) != entry.clone_hash);
        }};
    }

    match entry.record_kind {
        OntologyScopeMigrationRecordKind::Object | OntologyScopeMigrationRecordKind::Link => {
            let records = if matches!(entry.record_kind, OntologyScopeMigrationRecordKind::Object) {
                &mut state.ontology.objects
            } else {
                &mut state.ontology.links
            };
            let matches = records
                .iter()
                .filter(|record| {
                    ontology_value_matches_workstream(record, &Some(target.clone()))
                        && ontology_scope_record_hash(*record) == entry.clone_hash
                })
                .count();
            if matches != 1 {
                return Err(ReducerError::InvalidEvent(format!(
                    "ontology migration rollback requires one unchanged JSON clone: {} matches={}",
                    entry.clone_hash, matches
                )));
            }
            records.retain(|record| ontology_scope_record_hash(record) != entry.clone_hash);
        }
        OntologyScopeMigrationRecordKind::Proposal => {
            remove_typed_clone!(&mut state.ontology.proposals)
        }
        OntologyScopeMigrationRecordKind::Verification => {
            remove_typed_clone!(&mut state.ontology.verifications)
        }
        OntologyScopeMigrationRecordKind::WorkingSetRefresh => {
            remove_typed_clone!(&mut state.ontology.working_set_refreshes)
        }
        OntologyScopeMigrationRecordKind::Delta => {
            remove_typed_clone!(&mut state.ontology.delta_log)
        }
        OntologyScopeMigrationRecordKind::PreProposal => {
            remove_typed_clone!(&mut state.pre.proposals)
        }
    }
    Ok(())
}

fn upsert_context_claim(
    state: &mut FocusaState,
    claim: ContextClaimRecord,
    require_existing: bool,
) -> Result<(), ReducerError> {
    if let Some(index) = state
        .context_claims
        .iter()
        .position(|existing| existing.claim_id == claim.claim_id)
    {
        let expected_revision = state.context_claims[index].revision + 1;
        if claim.revision != expected_revision {
            return Err(ReducerError::InvalidEvent(format!(
                "Context claim revision mismatch: claim={} expected={} actual={}",
                claim.claim_id, expected_revision, claim.revision
            )));
        }
        state.context_claims[index] = claim;
    } else {
        if require_existing || claim.revision != 1 {
            return Err(ReducerError::InvalidEvent(format!(
                "Context claim must exist or start at revision 1: {}",
                claim.claim_id
            )));
        }
        state.context_claims.push(claim);
    }
    Ok(())
}

fn refresh_reactive_context(
    state: &mut FocusaState,
    project_root: &str,
    continuity_id: &str,
    attachment_id: &str,
    updated_at: chrono::DateTime<chrono::Utc>,
) {
    let mut unresolved_contradiction_refs: Vec<String> = state
        .context_contradictions
        .iter()
        .filter(|edge| {
            edge.project_root == project_root
                && edge.continuity_id == continuity_id
                && edge.attachment_id == attachment_id
                && edge.status == "open"
        })
        .map(|edge| edge.contradiction_id.clone())
        .collect();
    unresolved_contradiction_refs.sort();
    unresolved_contradiction_refs.dedup();

    let blocked: std::collections::BTreeSet<String> = state
        .context_contradictions
        .iter()
        .filter(|edge| unresolved_contradiction_refs.contains(&edge.contradiction_id))
        .flat_map(|edge| [edge.left_claim_id.clone(), edge.right_claim_id.clone()])
        .collect();
    let scoped_claims = state.context_claims.iter().filter(|claim| {
        claim.project_root == project_root
            && claim.continuity_id == continuity_id
            && claim.attachment_id == attachment_id
    });
    let mut accepted_claim_refs = Vec::new();
    let mut candidate_claim_refs = Vec::new();
    for claim in scoped_claims {
        if claim.status == "accepted" && !blocked.contains(&claim.claim_id) {
            accepted_claim_refs.push(claim.claim_id.clone());
        } else if !matches!(claim.status.as_str(), "rejected" | "superseded") {
            candidate_claim_refs.push(claim.claim_id.clone());
        }
    }
    accepted_claim_refs.sort();
    candidate_claim_refs.sort();
    let blocked_claim_refs = blocked.into_iter().collect();
    let projection = ReactiveContextProjection {
        project_root: project_root.to_string(),
        continuity_id: continuity_id.to_string(),
        attachment_id: attachment_id.to_string(),
        accepted_claim_refs,
        candidate_claim_refs,
        blocked_claim_refs,
        unresolved_contradiction_refs,
        revision: state.version + 1,
        updated_at: Some(updated_at),
    };
    if let Some(index) = state.reactive_context.iter().position(|existing| {
        existing.project_root == project_root
            && existing.continuity_id == continuity_id
            && existing.attachment_id == attachment_id
    }) {
        state.reactive_context[index] = projection;
    } else {
        state.reactive_context.push(projection);
    }
}

fn outcome_is_positive(outcome: &str) -> bool {
    let lowered = outcome.to_ascii_lowercase();
    lowered.contains("pass")
        || lowered.contains("success")
        || lowered.contains("verified")
        || lowered.contains("approve")
        || lowered.contains("accept")
}

fn recommended_worker_for_task(
    task_class: TaskClass,
    degraded: bool,
    repeated_failures: u32,
) -> WorkerCapabilityProfile {
    let fallback = repeated_failures >= 2;
    let (worker_id, edit_reliable, structured_output_reliable, code_generation_strong) =
        match (task_class, fallback) {
            (TaskClass::DocSpec | TaskClass::Architecture, false) => {
                ("fidelity-spec-worker", true, true, false)
            }
            (TaskClass::DocSpec | TaskClass::Architecture, true) => {
                ("fidelity-spec-fallback-worker", true, true, false)
            }
            (TaskClass::Code | TaskClass::Integration | TaskClass::Refactor, false) => {
                ("fidelity-code-worker", true, true, true)
            }
            (TaskClass::Code | TaskClass::Integration | TaskClass::Refactor, true) => {
                ("fidelity-code-fallback-worker", true, true, true)
            }
            (TaskClass::Unknown, false) => ("balanced-worker", true, true, true),
            (TaskClass::Unknown, true) => ("balanced-fallback-worker", true, true, true),
        };

    WorkerCapabilityProfile {
        worker_id: worker_id.to_string(),
        tool_use_supported: true,
        edit_reliable,
        structured_output_reliable,
        code_generation_strong,
        context_window_class: Some(
            if degraded {
                "degraded-bounded"
            } else {
                "standard"
            }
            .to_string(),
        ),
        latency_class: Some(
            if degraded || fallback {
                "slower-safer"
            } else {
                "balanced"
            }
            .to_string(),
        ),
        cost_tier: Some("standard".to_string()),
        fallback_available: !fallback,
    }
}

fn truncate_front<T>(items: &mut Vec<T>, cap: usize) {
    if items.len() > cap {
        let excess = items.len() - cap;
        items.drain(0..excess);
    }
}

fn bound_workpoint_record(record: &mut WorkpointRecord) {
    truncate_front(&mut record.active_object_refs, workpoint_caps::OBJECT_REFS);
    truncate_front(
        &mut record.verification_records,
        workpoint_caps::VERIFICATIONS,
    );
    truncate_front(&mut record.blockers, workpoint_caps::BLOCKERS);
    if let Some(intent) = &mut record.action_intent {
        truncate_front(
            &mut intent.verification_hooks,
            workpoint_caps::VERIFICATIONS,
        );
    }
}

fn find_workpoint_mut(
    state: &mut FocusaState,
    workpoint_id: WorkpointId,
) -> Result<&mut WorkpointRecord, ReducerError> {
    state
        .workpoint
        .records
        .iter_mut()
        .find(|w| w.workpoint_id == workpoint_id)
        .ok_or_else(|| ReducerError::InvalidEvent(format!("Workpoint {} not found", workpoint_id)))
}

fn upsert_workpoint_record(
    state: &mut FocusaState,
    mut record: WorkpointRecord,
    now: chrono::DateTime<Utc>,
) {
    bound_workpoint_record(&mut record);
    if record.created_at.is_none() {
        record.created_at = Some(now);
    }
    record.updated_at = Some(now);
    if let Some(existing) = state
        .workpoint
        .records
        .iter_mut()
        .find(|w| w.workpoint_id == record.workpoint_id)
    {
        *existing = record;
    } else {
        state.workpoint.records.push(record);
        truncate_front(&mut state.workpoint.records, workpoint_caps::RECORDS);
    }
}

fn bound_trajectory_record(record: &mut TrajectoryProjectionRecord) {
    truncate_front(&mut record.waypoints, trajectory_caps::WAYPOINTS);
    for waypoint in &mut record.waypoints {
        truncate_front(
            &mut waypoint.current_state_evidence_refs,
            trajectory_caps::EVIDENCE_REFS,
        );
        truncate_front(
            &mut waypoint.completion_evidence_refs,
            trajectory_caps::EVIDENCE_REFS,
        );
    }
    truncate_front(&mut record.goal_provenance, trajectory_caps::PROVENANCE);
    truncate_front(&mut record.blockers, trajectory_caps::WAYPOINTS);
    truncate_front(&mut record.open_questions, trajectory_caps::WAYPOINTS);
    if let Some(dod) = &mut record.definition_of_done {
        truncate_front(&mut dod.criteria, trajectory_caps::WAYPOINTS);
        truncate_front(&mut dod.evidence_required, trajectory_caps::EVIDENCE_REFS);
        truncate_front(
            &mut dod.verified_evidence_refs,
            trajectory_caps::EVIDENCE_REFS,
        );
        truncate_front(
            &mut dod.required_evidence_refs,
            trajectory_caps::EVIDENCE_REFS,
        );
        truncate_front(&mut dod.required_checks, trajectory_caps::WAYPOINTS);
        truncate_front(&mut dod.acceptance_risks, trajectory_caps::WAYPOINTS);
        truncate_front(&mut dod.not_done_if, trajectory_caps::WAYPOINTS);
    }
}

fn same_trajectory_authority_scope(
    a: &TrajectoryProjectionRecord,
    b: &TrajectoryProjectionRecord,
) -> bool {
    a.project_root.is_some()
        && a.continuity_id.is_some()
        && a.project_root == b.project_root
        && a.continuity_id == b.continuity_id
}

fn upsert_trajectory_record(
    state: &mut FocusaState,
    mut record: TrajectoryProjectionRecord,
    now: chrono::DateTime<Utc>,
) {
    bound_trajectory_record(&mut record);
    if record.created_at.is_none() {
        record.created_at = Some(now);
    }
    record.updated_at = Some(now);
    if let Some(existing) = state
        .trajectory
        .records
        .iter_mut()
        .find(|item| item.trajectory_id == record.trajectory_id)
    {
        *existing = record;
    } else {
        state.trajectory.records.push(record);
        truncate_front(&mut state.trajectory.records, trajectory_caps::RECORDS);
    }
}

use chrono::Utc;
use uuid::Uuid;

/// Core reducer: apply an event to state, producing new state + emitted events.
///
/// Flow: pre-check invariants → apply event → post-check invariants → bump version.
///
/// The input event is included in emitted_events on success (for event log persistence).
pub fn reduce(state: FocusaState, event: FocusaEvent) -> Result<ReductionResult, ReducerError> {
    // Default: no ownership enforcement (local events)
    reduce_with_meta(state, event, None, None, false)
}

/// Reduce with ownership metadata (docs/43 Policy #5).
///
/// If `is_observation` is true, the event is recorded but does not mutate canonical state.
/// If `machine_id` and `thread_id` are provided, enforces that only the thread owner
/// can mutate canonical Focus Stack / Focus State.
pub fn reduce_with_meta(
    state: FocusaState,
    event: FocusaEvent,
    machine_id: Option<&str>,
    thread_id: Option<Uuid>,
    is_observation: bool,
) -> Result<ReductionResult, ReducerError> {
    check_invariants(&state)?;

    // Policy #2: Observations don't mutate canonical state
    if is_observation {
        return Ok(ReductionResult {
            new_state: state,
            emitted_events: vec![event],
        });
    }

    // Policy #5: Per-thread ownership enforcement
    if let Some(tid) = thread_id {
        let thread = state.threads.iter().find(|t| t.id == tid);
        if let Some(owner) = thread.and_then(|t| t.owner_machine_id.as_ref()) {
            // Thread has an owner — verify the machine_id matches
            if machine_id != Some(owner.as_str()) {
                // Non-owner attempting to mutate canonical state — reject
                return Err(ReducerError::OwnershipViolation {
                    thread_id: tid,
                    owner: owner.clone(),
                    attempted_by: machine_id.map(|s| s.to_string()),
                });
            }
        }
        // If thread exists but has no owner, mutation is allowed (unowned threads)
        // If thread doesn't exist in state, reject (can't verify ownership)
        if thread.is_none() {
            return Err(ReducerError::InvalidEvent(format!(
                "Thread {} not found in state — cannot verify ownership for mutation",
                tid
            )));
        }
    }

    let mut state = state;
    let emitted_event = event.clone();

    match event {
        FocusaEvent::CallGraphFrameDispatched { .. } | FocusaEvent::CallGraphFrameSettled { .. } => {}
        // ─── Context corpus ─────────────────────────────────────────────
        FocusaEvent::ContextSourceCommitted { source } => {
            if source.receipt.before_state_version != state.version
                || source.receipt.after_state_version != state.version + 1
            {
                return Err(ReducerError::InvalidEvent(format!(
                    "Context receipt version mismatch: state={} before={} after={}",
                    state.version,
                    source.receipt.before_state_version,
                    source.receipt.after_state_version
                )));
            }
            if state.context_sources.iter().any(|existing| {
                existing.source_id == source.source_id
                    || (existing.project_root == source.project_root
                        && existing.continuity_id == source.continuity_id
                        && existing.attachment_id == source.attachment_id
                        && existing.idempotency_key == source.idempotency_key)
            }) {
                return Err(ReducerError::InvalidEvent(format!(
                    "duplicate Context source commit: {}",
                    source.source_id
                )));
            }
            state.context_sources.push(source);
        }
        FocusaEvent::ContextSourceIngested { source } => {
            if source.receipt.before_state_version != state.version
                || source.receipt.after_state_version != state.version + 1
            {
                return Err(ReducerError::InvalidEvent(format!(
                    "Context ingestion receipt version mismatch: state={} before={} after={}",
                    state.version,
                    source.receipt.before_state_version,
                    source.receipt.after_state_version
                )));
            }
            if state.context_sources.iter().any(|existing| {
                existing.project_root == source.project_root
                    && existing.continuity_id == source.continuity_id
                    && existing.attachment_id == source.attachment_id
                    && existing.idempotency_key == source.idempotency_key
            }) {
                return Err(ReducerError::InvalidEvent(format!(
                    "duplicate Context source ingestion: {}",
                    source.idempotency_key
                )));
            }
            if let Some(index) = state
                .context_sources
                .iter()
                .position(|existing| existing.source_id == source.source_id)
            {
                let expected_revision = state.context_sources[index].revision + 1;
                if source.revision != expected_revision {
                    return Err(ReducerError::InvalidEvent(format!(
                        "Context source revision mismatch: source={} expected={} actual={}",
                        source.source_id, expected_revision, source.revision
                    )));
                }
                state.context_sources[index] = source;
            } else {
                if source.revision != 1 {
                    return Err(ReducerError::InvalidEvent(format!(
                        "new Context source must start at revision 1: {}",
                        source.source_id
                    )));
                }
                state.context_sources.push(source);
            }
        }
        FocusaEvent::WorkspaceArtifactLinked { artifact, .. } => {
            if state.workspace_artifacts.iter().any(|existing| {
                existing.scope.project_root == artifact.scope.project_root
                    && existing.scope.continuity_id == artifact.scope.continuity_id
                    && existing.origin.attachment_id == artifact.origin.attachment_id
                    && existing.idempotency_key == artifact.idempotency_key
            }) {
                return Err(ReducerError::InvalidEvent(format!(
                    "duplicate Workspace Artifact link idempotency key: {}",
                    artifact.idempotency_key
                )));
            }
            if let Some(index) = state
                .workspace_artifacts
                .iter()
                .position(|existing| existing.artifact_id == artifact.artifact_id)
            {
                let existing = &state.workspace_artifacts[index];
                if artifact.revision != existing.revision + 1
                    || artifact.linked_at != existing.linked_at
                    || artifact.scope != existing.scope
                    || artifact.source.system != existing.source.system
                    || artifact.source.source_ref != existing.source.source_ref
                    || artifact.content.sha256 != existing.content.sha256
                {
                    return Err(ReducerError::InvalidEvent(format!(
                        "invalid Workspace Artifact projection revision: {}",
                        artifact.artifact_id
                    )));
                }
                state.workspace_artifacts[index] = artifact;
            } else {
                if artifact.revision != 1 {
                    return Err(ReducerError::InvalidEvent(format!(
                        "new Workspace Artifact must start at revision 1: {}",
                        artifact.artifact_id
                    )));
                }
                state.workspace_artifacts.push(artifact);
                state
                    .workspace_artifacts
                    .sort_by(|left, right| left.artifact_id.cmp(&right.artifact_id));
            }
        }
        FocusaEvent::ProjectRoleProfileRevised { profile } => {
            if profile.grants_permissions {
                return Err(ReducerError::InvalidEvent(
                    "project role profiles cannot grant permission".to_string(),
                ));
            }
            if profile.grounding.operator_seed_ref.trim().is_empty()
                || (profile.grounding.context_artifact_refs.is_empty()
                    && profile.grounding.context_claim_refs.is_empty()
                    && profile.grounding.interview_answer_refs.is_empty())
            {
                return Err(ReducerError::InvalidEvent(
                    "project role profile requires an operator seed and Context grounding"
                        .to_string(),
                ));
            }
            if state.project_role_profiles.iter().any(|existing| {
                existing.project_root == profile.project_root
                    && existing.continuity_id == profile.continuity_id
                    && existing.attachment_id == profile.attachment_id
                    && existing.idempotency_key == profile.idempotency_key
            }) {
                return Err(ReducerError::InvalidEvent(format!(
                    "duplicate project role profile idempotency key: {}",
                    profile.idempotency_key
                )));
            }
            let revisions: Vec<&ProjectAgentRoleProfile> = state
                .project_role_profiles
                .iter()
                .filter(|existing| existing.role_profile_id == profile.role_profile_id)
                .collect();
            if let Some(latest) = revisions.iter().max_by_key(|existing| existing.revision) {
                if profile.revision != latest.revision + 1
                    || profile.created_at != latest.created_at
                    || profile.project_root != latest.project_root
                    || profile.continuity_id != latest.continuity_id
                    || profile.attachment_id != latest.attachment_id
                {
                    return Err(ReducerError::InvalidEvent(format!(
                        "invalid project role profile revision: {}",
                        profile.role_profile_id
                    )));
                }
            } else if profile.revision != 1 {
                return Err(ReducerError::InvalidEvent(format!(
                    "new project role profile must start at revision 1: {}",
                    profile.role_profile_id
                )));
            }
            let review_matches_status = match (&profile.status, profile.review.as_ref()) {
                (RoleProfileStatus::Approved, Some(review)) => {
                    matches!(review.decision, RoleReviewDecision::Approve)
                }
                (RoleProfileStatus::Superseded, Some(review)) => {
                    matches!(review.decision, RoleReviewDecision::Reject)
                }
                (RoleProfileStatus::PendingOperator, Some(review)) => {
                    matches!(review.decision, RoleReviewDecision::Defer)
                }
                (RoleProfileStatus::Draft | RoleProfileStatus::PendingOperator, None) => true,
                _ => false,
            };
            if !review_matches_status {
                return Err(ReducerError::InvalidEvent(
                    "project role profile status does not match its explicit review".to_string(),
                ));
            }
            state.project_role_profiles.push(profile);
            state.project_role_profiles.sort_by(|left, right| {
                left.role_profile_id
                    .cmp(&right.role_profile_id)
                    .then(left.revision.cmp(&right.revision))
            });
        }
        FocusaEvent::ProjectInterviewSessionRevised { session } => {
            if session.interview_session_id.trim().is_empty()
                || session.project_root.trim().is_empty()
                || session.continuity_id.trim().is_empty()
                || session.attachment_id.trim().is_empty()
                || session.idempotency_key.trim().is_empty()
                || session.state_revision == 0
            {
                return Err(ReducerError::InvalidEvent(
                    "project Interview session requires identity, exact scope, idempotency key, and positive revision".to_string(),
                ));
            }
            let role_is_approved = state.project_role_profiles.iter().any(|profile| {
                profile.role_profile_id == session.approved_role_profile_ref
                    && profile.project_root == session.project_root
                    && profile.continuity_id == session.continuity_id
                    && profile.attachment_id == session.attachment_id
                    && matches!(profile.status, RoleProfileStatus::Approved)
            });
            if !role_is_approved {
                return Err(ReducerError::InvalidEvent(
                    "project Interview session requires an approved Role Profile in exact scope"
                        .to_string(),
                ));
            }
            let revisions: Vec<&ProjectInterviewSessionRecord> = state
                .project_interview_sessions
                .iter()
                .filter(|existing| existing.interview_session_id == session.interview_session_id)
                .collect();
            let expected_revision = revisions
                .iter()
                .map(|existing| existing.state_revision)
                .max()
                .unwrap_or(0)
                + 1;
            if session.state_revision != expected_revision {
                return Err(ReducerError::InvalidEvent(format!(
                    "project Interview revision must be {expected_revision}"
                )));
            }
            let branch_ids: std::collections::BTreeSet<_> = session
                .branches
                .iter()
                .map(|branch| branch.decision_branch_id.as_str())
                .collect();
            if branch_ids.len() != session.branches.len()
                || session.branches.iter().any(|branch| {
                    branch.decision_branch_id.trim().is_empty()
                        || branch.tranche.trim().is_empty()
                        || branch.label.trim().is_empty()
                        || branch
                            .parent_branch_id
                            .as_deref()
                            .is_some_and(|parent| !branch_ids.contains(parent))
                })
            {
                return Err(ReducerError::InvalidEvent(
                    "project Interview branches require unique IDs, tranche, label, and valid parents".to_string(),
                ));
            }
            let question_ids: std::collections::BTreeSet<_> = session
                .questions
                .iter()
                .map(|question| question.question_id.as_str())
                .collect();
            if question_ids.len() != session.questions.len()
                || session.questions.iter().any(|question| {
                    question.session_id != session.interview_session_id
                        || !branch_ids.contains(question.decision_branch_id.as_str())
                        || question.question.trim().is_empty()
                        || question.stop_condition.trim().is_empty()
                        || question.environment_facts_checked.is_empty()
                        || question.linked_context_refs.is_empty()
                        || (question.decision_required
                            && (question.recommendation.trim().is_empty()
                                || question.recommendation_basis_refs.is_empty()))
                })
            {
                return Err(ReducerError::InvalidEvent(
                    "project Interview questions require unique IDs, valid branch, fact refs, stop condition, and cited recommendation".to_string(),
                ));
            }
            if session.answers.iter().any(|answer| {
                !question_ids.contains(answer.question_id.as_str())
                    || answer.operator_id.trim().is_empty()
            }) {
                return Err(ReducerError::InvalidEvent(
                    "project Interview answers require a valid question and operator".to_string(),
                ));
            }
            if session
                .active_branch_id
                .as_deref()
                .is_some_and(|branch| !branch_ids.contains(branch))
                || session
                    .current_question_id
                    .as_deref()
                    .is_some_and(|question| !question_ids.contains(question))
            {
                return Err(ReducerError::InvalidEvent(
                    "project Interview resume pointers must reference retained branch and question state".to_string(),
                ));
            }
            let closed_at_matches = match session.status {
                ProjectInterviewSessionStatus::Closed => session.closed_at.is_some(),
                _ => session.closed_at.is_none(),
            };
            if !closed_at_matches {
                return Err(ReducerError::InvalidEvent(
                    "project Interview closed_at must match closed status".to_string(),
                ));
            }
            state.project_interview_sessions.push(session);
            state.project_interview_sessions.sort_by(|left, right| {
                left.interview_session_id
                    .cmp(&right.interview_session_id)
                    .then(left.state_revision.cmp(&right.state_revision))
            });
        }
        FocusaEvent::SpecWorkbenchSessionRevised { session } => {
            if session.workbench_session_id.trim().is_empty()
                || session.project_root.trim().is_empty()
                || session.continuity_id.trim().is_empty()
                || session.attachment_id.trim().is_empty()
                || session.current_ask.trim().is_empty()
                || session.idempotency_key.trim().is_empty()
                || session.state_revision == 0
                || !session.canonical
                || !session.advisory_agents
                || !session.operator_required
            {
                return Err(ReducerError::InvalidEvent("Spec Workbench requires identity, exact scope, ask, canonical operator authority, idempotency, and positive revision".to_string()));
            }
            let expected_revision = state
                .spec_workbench_sessions
                .iter()
                .filter(|existing| existing.workbench_session_id == session.workbench_session_id)
                .map(|existing| existing.state_revision)
                .max()
                .unwrap_or(0)
                + 1;
            if session.state_revision != expected_revision {
                return Err(ReducerError::InvalidEvent(format!(
                    "Spec Workbench revision must be {expected_revision}"
                )));
            }
            let section_ids: std::collections::BTreeSet<_> = session
                .sections
                .iter()
                .map(|section| section.section_id.as_str())
                .collect();
            let round_ids: std::collections::BTreeSet<_> = session
                .rounds
                .iter()
                .map(|round| round.round_id.as_str())
                .collect();
            let objection_ids: std::collections::BTreeSet<_> = session
                .objections
                .iter()
                .map(|item| item.objection_id.as_str())
                .collect();
            let gate_ids: std::collections::BTreeSet<_> = session
                .gates
                .iter()
                .map(|gate| gate.gate_id.as_str())
                .collect();
            let amendment_ids: std::collections::BTreeSet<_> = session
                .amendments
                .iter()
                .map(|item| item.amendment_id.as_str())
                .collect();
            if section_ids.len() != session.sections.len()
                || round_ids.len() != session.rounds.len()
                || objection_ids.len() != session.objections.len()
                || gate_ids.len() != session.gates.len()
                || amendment_ids.len() != session.amendments.len()
            {
                return Err(ReducerError::InvalidEvent(
                    "Spec Workbench records require unique IDs".to_string(),
                ));
            }
            if session
                .current_section_id
                .as_deref()
                .is_some_and(|id| !section_ids.contains(id))
                || session.rounds.iter().any(|round| {
                    !section_ids.contains(round.section_id.as_str())
                        || round.transcript_ref.trim().is_empty()
                })
                || session.objections.iter().any(|item| {
                    !section_ids.contains(item.section_id.as_str())
                        || !round_ids.contains(item.round_id.as_str())
                        || item.evidence_refs.is_empty()
                })
                || session.gates.iter().any(|gate| {
                    !section_ids.contains(gate.section_id.as_str())
                        || gate.decided_by.trim().is_empty()
                        || gate.evidence_refs.is_empty()
                })
                || session.amendments.iter().any(|item| {
                    !section_ids.contains(item.section_id.as_str())
                        || item.after_revision != item.before_revision + 1
                        || item.evidence_refs.is_empty()
                })
            {
                return Err(ReducerError::InvalidEvent("Spec Workbench references, grounding evidence, gates, and amendments must remain linked".to_string()));
            }
            for section in &session.sections {
                if section.title.trim().is_empty()
                    || section.content.trim().is_empty()
                    || section
                        .objection_ids
                        .iter()
                        .any(|id| !objection_ids.contains(id.as_str()))
                    || section
                        .amendment_ids
                        .iter()
                        .any(|id| !amendment_ids.contains(id.as_str()))
                {
                    return Err(ReducerError::InvalidEvent(
                        "Spec section requires content and valid objection/amendment links"
                            .to_string(),
                    ));
                }
                if matches!(section.status, SpecSectionStatus::Approved) {
                    let grounded = !section.grounding.context_refs.is_empty()
                        && !section.grounding.evidence_refs.is_empty();
                    let unresolved = session.objections.iter().any(|item| {
                        item.section_id == section.section_id
                            && matches!(item.status, SpecObjectionStatus::Open)
                    });
                    let approved_gate = section.operator_gate_id.as_deref().is_some_and(|id| {
                        gate_ids.contains(id)
                            && session.gates.iter().any(|gate| {
                                gate.gate_id == id
                                    && matches!(gate.decision, SpecGateDecision::Approve)
                            })
                    });
                    if !grounded
                        || unresolved
                        || !approved_gate
                        || section.approved_revision != Some(section.revision)
                    {
                        return Err(ReducerError::InvalidEvent("approved Spec section requires grounding, resolved objections, matching revision, and explicit operator gate".to_string()));
                    }
                }
            }
            if matches!(session.status, SpecWorkbenchStatus::FinalApproved)
                && (session.sections.is_empty()
                    || session
                        .sections
                        .iter()
                        .any(|section| !matches!(section.status, SpecSectionStatus::Approved))
                    || session.final_spec_id.is_none())
            {
                return Err(ReducerError::InvalidEvent(
                    "final Spec approval requires all sections approved and final_spec_id"
                        .to_string(),
                ));
            }
            if matches!(session.status, SpecWorkbenchStatus::Closed) != session.closed_at.is_some()
            {
                return Err(ReducerError::InvalidEvent(
                    "Spec Workbench closed_at must match closed status".to_string(),
                ));
            }
            state.spec_workbench_sessions.push(session);
            state.spec_workbench_sessions.sort_by(|left, right| {
                left.workbench_session_id
                    .cmp(&right.workbench_session_id)
                    .then(left.state_revision.cmp(&right.state_revision))
            });
        }
        FocusaEvent::ProviderNeutralTaskPlanRevised { task_plan } => {
            if task_plan.task_plan_id.trim().is_empty()
                || task_plan.project_root.trim().is_empty()
                || task_plan.continuity_id.trim().is_empty()
                || task_plan.attachment_id.trim().is_empty()
                || task_plan.workbench_session_id.trim().is_empty()
                || task_plan.final_spec_id.trim().is_empty()
                || task_plan.idempotency_key.trim().is_empty()
                || task_plan.state_revision == 0
                || task_plan.materialized
            {
                return Err(ReducerError::InvalidEvent("provider-neutral task plan requires identity, exact scope, approved Spec refs, idempotency, positive revision, and unmaterialized state".to_string()));
            }
            let source = state
                .spec_workbench_sessions
                .iter()
                .find(|source| {
                    source.workbench_session_id == task_plan.workbench_session_id
                        && source.project_root == task_plan.project_root
                        && source.continuity_id == task_plan.continuity_id
                        && source.attachment_id == task_plan.attachment_id
                        && source.final_spec_id.as_deref() == Some(task_plan.final_spec_id.as_str())
                        && matches!(source.status, SpecWorkbenchStatus::FinalApproved)
                })
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(
                        "task plan source must be an exact-scoped final-approved Spec Workbench"
                            .to_string(),
                    )
                })?;
            let expected_revision = state
                .provider_neutral_task_plans
                .iter()
                .filter(|existing| existing.task_plan_id == task_plan.task_plan_id)
                .map(|existing| existing.state_revision)
                .max()
                .unwrap_or(0)
                + 1;
            if task_plan.state_revision != expected_revision {
                return Err(ReducerError::InvalidEvent(format!(
                    "task plan revision must be {expected_revision}"
                )));
            }
            let task_ids: std::collections::BTreeSet<_> = task_plan
                .tasks
                .iter()
                .map(|task| task.provider_neutral_id.as_str())
                .collect();
            let section_ids: std::collections::BTreeSet<_> = source
                .sections
                .iter()
                .map(|section| section.section_id.as_str())
                .collect();
            if task_ids.len() != task_plan.tasks.len() {
                return Err(ReducerError::InvalidEvent(
                    "task DAG requires unique provider-neutral IDs".to_string(),
                ));
            }
            if task_plan.tasks.iter().any(|task| {
                task.provider_neutral_id.trim().is_empty()
                    || task.title.trim().is_empty()
                    || task.description.trim().is_empty()
                    || task.linked_spec_sections.is_empty()
                    || task
                        .linked_spec_sections
                        .iter()
                        .any(|id| !section_ids.contains(id.as_str()))
                    || task.requirement_refs.is_empty()
                    || task.acceptance_criteria.is_empty()
                    || task.evidence_requirements.is_empty()
                    || task.verification_policy_ref.trim().is_empty()
                    || task.allowed_scope.is_empty()
                    || task.task_class.trim().is_empty()
                    || task.closure_kind.trim().is_empty()
                    || task.closure_policy_ref.trim().is_empty()
                    || task.dependencies.iter().any(|id| {
                        id == &task.provider_neutral_id || !task_ids.contains(id.as_str())
                    })
            }) {
                return Err(ReducerError::InvalidEvent("every task requires valid Spec/requirement/proof links, policy, scope, and in-graph dependencies".to_string()));
            }
            let mut resolved = std::collections::BTreeSet::new();
            loop {
                let before = resolved.len();
                for task in &task_plan.tasks {
                    if task
                        .dependencies
                        .iter()
                        .all(|dependency| resolved.contains(dependency.as_str()))
                    {
                        resolved.insert(task.provider_neutral_id.as_str());
                    }
                }
                if resolved.len() == before {
                    break;
                }
            }
            if resolved.len() != task_plan.tasks.len() {
                return Err(ReducerError::InvalidEvent(
                    "task dependency graph must be acyclic".to_string(),
                ));
            }
            match task_plan.status {
                TaskPlanStatus::Draft => {
                    if task_plan.approved_revision.is_some() || task_plan.approved_by.is_some() {
                        return Err(ReducerError::InvalidEvent(
                            "draft task plan cannot contain approval authority".to_string(),
                        ));
                    }
                }
                TaskPlanStatus::PendingOperator => {
                    if task_plan.tasks.is_empty()
                        || task_plan.preview_token.as_deref().is_none_or(str::is_empty)
                        || task_plan.previewed_revision != Some(task_plan.state_revision)
                    {
                        return Err(ReducerError::InvalidEvent("operator preview requires a non-empty valid DAG and revision-bound preview token".to_string()));
                    }
                }
                TaskPlanStatus::Approved => {
                    if task_plan.tasks.is_empty()
                        || task_plan.preview_token.as_deref().is_none_or(str::is_empty)
                        || task_plan.previewed_revision != Some(task_plan.state_revision - 1)
                        || task_plan.approved_revision != Some(task_plan.state_revision)
                        || task_plan.approved_by.as_deref().is_none_or(str::is_empty)
                        || task_plan.receipt_refs.is_empty()
                    {
                        return Err(ReducerError::InvalidEvent("task plan approval requires prior revision-bound preview, explicit operator, matching revision, and Receipt".to_string()));
                    }
                }
            }
            state.provider_neutral_task_plans.push(task_plan);
            state.provider_neutral_task_plans.sort_by(|left, right| {
                left.task_plan_id
                    .cmp(&right.task_plan_id)
                    .then(left.state_revision.cmp(&right.state_revision))
            });
        }
        FocusaEvent::TaskPlanMaterialized { materialization } => {
            if materialization.materialization_id.trim().is_empty()
                || materialization.project_root.trim().is_empty()
                || materialization.continuity_id.trim().is_empty()
                || materialization.attachment_id.trim().is_empty()
                || materialization.provider != "work_item.bd"
                || materialization.worktree_prefix.trim().is_empty()
                || materialization.permission_grant_ref.trim().is_empty()
                || materialization.idempotency_key.trim().is_empty()
                || materialization.evidence_ref.trim().is_empty()
                || materialization.receipt_ref.trim().is_empty()
            {
                return Err(ReducerError::InvalidEvent("task materialization requires identity, exact scope, Beads provider, prefix, permission, idempotency, Evidence, and Receipt".to_string()));
            }
            if state.task_materializations.iter().any(|existing| {
                existing.materialization_id == materialization.materialization_id
                    || (existing.project_root == materialization.project_root
                        && existing.continuity_id == materialization.continuity_id
                        && existing.attachment_id == materialization.attachment_id
                        && existing.idempotency_key == materialization.idempotency_key)
            }) {
                return Err(ReducerError::InvalidEvent(
                    "duplicate task materialization".to_string(),
                ));
            }
            let plan = state
                .provider_neutral_task_plans
                .iter()
                .find(|plan| {
                    plan.task_plan_id == materialization.task_plan_id
                        && plan.state_revision == materialization.task_plan_revision
                        && plan.project_root == materialization.project_root
                        && plan.continuity_id == materialization.continuity_id
                        && plan.attachment_id == materialization.attachment_id
                        && matches!(plan.status, TaskPlanStatus::Approved)
                })
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(
                        "task materialization requires exact-scoped approved task plan revision"
                            .to_string(),
                    )
                })?;
            let expected_ledger = std::path::Path::new(&materialization.project_root)
                .join(".beads/issues.jsonl")
                .to_string_lossy()
                .to_string();
            if materialization.target_ledger_ref != expected_ledger {
                return Err(ReducerError::InvalidEvent("task materialization target must be canonical project_root/.beads/issues.jsonl".to_string()));
            }
            let refs: std::collections::BTreeMap<_, _> = materialization
                .tasks
                .iter()
                .map(|task| (task.provider_neutral_id.as_str(), task))
                .collect();
            if refs.len() != plan.tasks.len() || materialization.tasks.len() != plan.tasks.len() {
                return Err(ReducerError::InvalidEvent(
                    "materialization must map every approved task exactly once".to_string(),
                ));
            }
            for task in &plan.tasks {
                let mapped = refs.get(task.provider_neutral_id.as_str()).ok_or_else(|| {
                    ReducerError::InvalidEvent("approved task missing provider mapping".to_string())
                })?;
                let expected_dependencies: std::collections::BTreeSet<_> = task
                    .dependencies
                    .iter()
                    .filter_map(|dependency| {
                        refs.get(dependency.as_str())
                            .map(|item| item.provider_id.as_str())
                    })
                    .collect();
                let actual_dependencies: std::collections::BTreeSet<_> = mapped
                    .provider_dependency_ids
                    .iter()
                    .map(String::as_str)
                    .collect();
                if !mapped
                    .provider_id
                    .starts_with(&format!("{}-", materialization.worktree_prefix))
                    || mapped.external_ref
                        != format!(
                            "focusa-task-plan:{}:{}",
                            plan.task_plan_id, task.provider_neutral_id
                        )
                    || expected_dependencies != actual_dependencies
                {
                    return Err(ReducerError::InvalidEvent("materialized IDs, external refs, and dependency links must remain stable and exact".to_string()));
                }
            }
            state.task_materializations.push(materialization);
            state
                .task_materializations
                .sort_by(|left, right| left.materialization_id.cmp(&right.materialization_id));
        }
        FocusaEvent::WorkRailRevised { record } => {
            if record.work_rail_id.trim().is_empty()
                || record.state_revision == 0
                || record.provider != "work_item.bd"
                || record.provider_item_id.trim().is_empty()
                || record.title.trim().is_empty()
                || record.project_root.trim().is_empty()
                || record.working_subpath_id.trim().is_empty()
                || record.continuity_id.trim().is_empty()
                || record.attachment_id.trim().is_empty()
                || record.idempotency_key.trim().is_empty()
            {
                return Err(ReducerError::InvalidEvent("Work Rail requires identity, exact project/working-subpath/continuity/attachment scope, Beads provider, and idempotency".to_string()));
            }
            let expected_revision = state
                .work_rail_records
                .iter()
                .filter(|existing| existing.work_rail_id == record.work_rail_id)
                .map(|existing| existing.state_revision)
                .max()
                .unwrap_or(0)
                + 1;
            if record.state_revision != expected_revision {
                return Err(ReducerError::InvalidEvent(format!(
                    "Work Rail revision must be {expected_revision}"
                )));
            }
            let workpoint = state.workpoint.records.iter().find(|workpoint| {
                workpoint.workpoint_id == record.workpoint_id
                    && workpoint.canonical
                    && workpoint.project_root.as_deref() == Some(record.project_root.as_str())
                    && workpoint.continuity_id.as_deref() == Some(record.continuity_id.as_str())
                    && workpoint.work_item_id.as_deref() == Some(record.provider_item_id.as_str())
                    && workpoint.session_identity.as_ref().and_then(|identity| identity.working_subpath_id.as_deref()) == Some(record.working_subpath_id.as_str())
            }).ok_or_else(|| ReducerError::InvalidEvent("Work Rail authority requires one canonical Workpoint matching project, working sub-path, continuity, and Bead".to_string()))?;
            if matches!(record.focusa_status, WorkRailStatus::VerifiedComplete) {
                let linked: std::collections::BTreeSet<_> = workpoint
                    .verification_records
                    .iter()
                    .filter_map(|verification| verification.evidence_ref.as_deref())
                    .collect();
                if record.provider_status != "closed"
                    || record.evidence_refs.is_empty()
                    || record
                        .evidence_refs
                        .iter()
                        .any(|evidence| !linked.contains(evidence.as_str()))
                    || record.receipt_ref.as_deref().is_none_or(str::is_empty)
                    || record
                        .closure_claim_ref
                        .as_deref()
                        .is_none_or(str::is_empty)
                    || !record.blockers.is_empty()
                {
                    return Err(ReducerError::InvalidEvent("verified Work Rail closure requires provider closed, Workpoint-linked proof, no blockers, closure claim, and Receipt".to_string()));
                }
            }
            state.work_rail_records.push(record);
            state.work_rail_records.sort_by(|left, right| {
                left.work_rail_id
                    .cmp(&right.work_rail_id)
                    .then(left.state_revision.cmp(&right.state_revision))
            });
        }
        FocusaEvent::MissionCanvasSurfaceRevised { surface } => {
            if surface.work_surface_id.trim().is_empty()
                || surface.state_revision == 0
                || surface.project_root.trim().is_empty()
                || surface.continuity_id.trim().is_empty()
                || surface.attachment_id.trim().is_empty()
                || surface.instance_id.trim().is_empty()
                || surface.mission_ref.trim().is_empty()
                || surface.title.trim().is_empty()
                || surface.surface_kind.trim().is_empty()
                || surface.pane_id.trim().is_empty()
                || surface.canonical_state_refs.is_empty()
                || surface.idempotency_key.trim().is_empty()
            {
                return Err(ReducerError::InvalidEvent("Mission Canvas Work Surface requires identity, exact scope, mission, pane/tab placement, bounded canonical refs, and idempotency".to_string()));
            }
            let expected_revision = state
                .mission_canvas_surfaces
                .iter()
                .filter(|existing| existing.work_surface_id == surface.work_surface_id)
                .map(|existing| existing.state_revision)
                .max()
                .unwrap_or(0)
                + 1;
            if surface.state_revision != expected_revision {
                return Err(ReducerError::InvalidEvent(format!(
                    "Mission Canvas Work Surface revision must be {expected_revision}"
                )));
            }
            if surface
                .canonical_state_refs
                .iter()
                .any(|reference| reference.trim().is_empty())
            {
                return Err(ReducerError::InvalidEvent("Work Surface canonical refs must be bounded handles, never duplicated state payloads".to_string()));
            }
            state.mission_canvas_surfaces.push(surface);
            state.mission_canvas_surfaces.sort_by(|left, right| {
                left.work_surface_id
                    .cmp(&right.work_surface_id)
                    .then(left.state_revision.cmp(&right.state_revision))
            });
        }
        FocusaEvent::MissionCanvasSurfaceBindingRevised { binding } => {
            if binding.binding_id.trim().is_empty()
                || binding.state_revision == 0
                || binding.project_root.trim().is_empty()
                || binding.continuity_id.trim().is_empty()
                || binding.attachment_id.trim().is_empty()
                || binding.work_surface_id.trim().is_empty()
                || binding.target_ref.trim().is_empty()
                || !matches!(binding.access_mode.as_str(), "read" | "write" | "invoke")
                || binding.idempotency_key.trim().is_empty()
            {
                return Err(ReducerError::InvalidEvent("Mission Canvas binding requires identity, exact attachment scope, Work Surface, target, access mode, and idempotency".to_string()));
            }
            if binding.binding_kind == MissionCanvasBindingKind::BrowserContext {
                let owns_session = state
                    .mission_canvas_surface_bindings
                    .iter()
                    .any(|existing| {
                        existing.active
                            && existing.binding_kind == MissionCanvasBindingKind::Session
                            && existing.project_root == binding.project_root
                            && existing.continuity_id == binding.continuity_id
                            && existing.attachment_id == binding.attachment_id
                            && existing.work_surface_id == binding.work_surface_id
                    });
                if binding.active && !owns_session {
                    return Err(ReducerError::InvalidEvent(
                        "Browser context requires an active UIAI session in exact attachment scope"
                            .to_string(),
                    ));
                }
                let isolation = binding.browser_isolation_class.ok_or_else(|| {
                    ReducerError::InvalidEvent(
                        "Browser context binding requires an isolation class".to_string(),
                    )
                })?;
                let expected_sharing =
                    if isolation == MissionCanvasBrowserIsolationClass::SharedAuthenticated {
                        "shared_explicit"
                    } else {
                        "isolated"
                    };
                if binding.authentication_sharing.as_deref() != Some(expected_sharing)
                    || !matches!(
                        binding.retention_policy.as_deref(),
                        Some("persistent" | "dispose_on_close" | "manual")
                    )
                {
                    return Err(ReducerError::InvalidEvent(
                        "Browser context binding requires matching authentication sharing and retention policy"
                            .to_string(),
                    ));
                }
                if binding.active {
                    for existing in state
                        .mission_canvas_surface_bindings
                        .iter()
                        .filter(|existing| existing.active)
                        .filter(|existing| {
                            existing.binding_kind == MissionCanvasBindingKind::BrowserContext
                                && existing.target_ref == binding.target_ref
                                && existing.binding_id != binding.binding_id
                        })
                    {
                        let cross_project = existing.project_root != binding.project_root
                            || existing.continuity_id != binding.continuity_id;
                        let explicitly_shared = !cross_project
                            && isolation == MissionCanvasBrowserIsolationClass::SharedAuthenticated
                            && existing.browser_isolation_class
                                == Some(MissionCanvasBrowserIsolationClass::SharedAuthenticated)
                            && existing.authentication_sharing.as_deref()
                                == Some("shared_explicit");
                        if !explicitly_shared {
                            return Err(ReducerError::InvalidEvent(
                                "Browser context reuse requires exact ownership or explicit same-project sharing"
                                    .to_string(),
                            ));
                        }
                    }
                }
            } else if binding.binding_kind == MissionCanvasBindingKind::BrowserTarget {
                let owns_context = state
                    .mission_canvas_surface_bindings
                    .iter()
                    .any(|existing| {
                        existing.active
                            && existing.binding_kind == MissionCanvasBindingKind::BrowserContext
                            && existing.project_root == binding.project_root
                            && existing.continuity_id == binding.continuity_id
                            && existing.attachment_id == binding.attachment_id
                            && existing.work_surface_id == binding.work_surface_id
                    });
                if binding.active && !owns_context {
                    return Err(ReducerError::InvalidEvent(
                        "Browser target requires an active browser context in exact attachment scope"
                            .to_string(),
                    ));
                }
            } else if binding.browser_isolation_class.is_some()
                || binding.authentication_sharing.is_some()
                || binding.retention_policy.is_some()
            {
                return Err(ReducerError::InvalidEvent(
                    "Browser isolation metadata is valid only for browser context bindings"
                        .to_string(),
                ));
            }
            let surface_exists = state.mission_canvas_surfaces.iter().any(|surface| {
                surface.work_surface_id == binding.work_surface_id
                    && surface.project_root == binding.project_root
                    && surface.continuity_id == binding.continuity_id
                    && surface.attachment_id == binding.attachment_id
            });
            if !surface_exists {
                return Err(ReducerError::InvalidEvent(
                    "Mission Canvas binding cannot cross surface or attachment scope".to_string(),
                ));
            }
            let expected_revision = state
                .mission_canvas_surface_bindings
                .iter()
                .filter(|existing| existing.binding_id == binding.binding_id)
                .map(|existing| existing.state_revision)
                .max()
                .unwrap_or(0)
                + 1;
            if binding.state_revision != expected_revision {
                return Err(ReducerError::InvalidEvent(format!(
                    "Mission Canvas binding revision must be {expected_revision}"
                )));
            }
            state.mission_canvas_surface_bindings.push(binding);
            state
                .mission_canvas_surface_bindings
                .sort_by(|left, right| {
                    left.binding_id
                        .cmp(&right.binding_id)
                        .then(left.state_revision.cmp(&right.state_revision))
                });
        }
        FocusaEvent::MissionCanvasStateRevised { canvas } => {
            if canvas.canvas_id.trim().is_empty()
                || canvas.state_revision == 0
                || canvas.project_root.trim().is_empty()
                || canvas.continuity_id.trim().is_empty()
                || canvas.client_instance_id.trim().is_empty()
                || canvas.user_id.trim().is_empty()
                || canvas.device_id.trim().is_empty()
                || canvas.idempotency_key.trim().is_empty()
            {
                return Err(ReducerError::InvalidEvent(
                    "Mission Canvas state requires identity, exact project/continuity scope, client, user, device, and idempotency"
                        .to_string(),
                ));
            }
            let bounded = [
                &canvas.open_work_surface_ids,
                &canvas.group_order,
                &canvas.aggregate_project_roots,
                &canvas.aggregate_continuity_ids,
                &canvas.aggregate_surface_kinds,
                &canvas.aggregate_surface_states,
                &canvas.selected_context_refs,
            ]
            .iter()
            .all(|values| {
                values.len() <= 64
                    && values.iter().all(|value| !value.trim().is_empty())
                    && values
                        .iter()
                        .enumerate()
                        .all(|(index, value)| !values[..index].contains(value))
            });
            if !bounded {
                return Err(ReducerError::InvalidEvent(
                    "Mission Canvas state collections must be bounded, non-empty, and unique"
                        .to_string(),
                ));
            }
            let surfaces_are_exact = canvas.open_work_surface_ids.iter().all(|surface_id| {
                state.mission_canvas_surfaces.iter().any(|surface| {
                    surface.work_surface_id == *surface_id
                        && surface.project_root == canvas.project_root
                        && surface.continuity_id == canvas.continuity_id
                })
            });
            if !surfaces_are_exact {
                return Err(ReducerError::InvalidEvent(
                    "Mission Canvas state cannot adopt a Work Surface outside its exact project and continuity scope"
                        .to_string(),
                ));
            }
            for focused in [
                canvas.focused_work_surface_id.as_ref(),
                canvas.secondary_focused_surface_id.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                if !canvas.open_work_surface_ids.contains(focused) {
                    return Err(ReducerError::InvalidEvent(
                        "Focused Mission Canvas surfaces must remain in the open topology"
                            .to_string(),
                    ));
                }
            }
            let expected_revision = state
                .mission_canvas_states
                .iter()
                .filter(|existing| existing.canvas_id == canvas.canvas_id)
                .map(|existing| existing.state_revision)
                .max()
                .unwrap_or(0)
                + 1;
            if canvas.state_revision != expected_revision {
                return Err(ReducerError::InvalidEvent(format!(
                    "Mission Canvas state revision must be {expected_revision}"
                )));
            }
            state.mission_canvas_states.push(canvas);
            state.mission_canvas_states.sort_by(|left, right| {
                left.canvas_id
                    .cmp(&right.canvas_id)
                    .then(left.state_revision.cmp(&right.state_revision))
            });
        }
        FocusaEvent::ContextClaimProposed { claim } => {
            if state.context_claims.iter().any(|existing| {
                existing.claim_id == claim.claim_id
                    || (existing.project_root == claim.project_root
                        && existing.continuity_id == claim.continuity_id
                        && existing.attachment_id == claim.attachment_id
                        && existing.idempotency_key == claim.idempotency_key)
            }) {
                return Err(ReducerError::InvalidEvent(format!(
                    "duplicate Context claim proposal: {}",
                    claim.claim_id
                )));
            }
            let scope = (
                claim.project_root.clone(),
                claim.continuity_id.clone(),
                claim.attachment_id.clone(),
                claim.committed_at,
            );
            upsert_context_claim(&mut state, claim, false)?;
            refresh_reactive_context(&mut state, &scope.0, &scope.1, &scope.2, scope.3);
        }
        FocusaEvent::ContextClaimReviewed { claim, decision } => {
            let scope = (
                claim.project_root.clone(),
                claim.continuity_id.clone(),
                claim.attachment_id.clone(),
                decision.decided_at,
            );
            upsert_context_claim(&mut state, claim, true)?;
            if state
                .context_decisions
                .iter()
                .any(|existing| existing.decision_id == decision.decision_id)
            {
                return Err(ReducerError::InvalidEvent(format!(
                    "duplicate Context decision: {}",
                    decision.decision_id
                )));
            }
            state.context_decisions.push(decision);
            refresh_reactive_context(&mut state, &scope.0, &scope.1, &scope.2, scope.3);
        }
        FocusaEvent::ContextContradictionOpened {
            contradiction,
            claims,
        } => {
            if contradiction.revision != 1
                || state.context_contradictions.iter().any(|existing| {
                    existing.contradiction_id == contradiction.contradiction_id
                        || existing.idempotency_key == contradiction.idempotency_key
                })
            {
                return Err(ReducerError::InvalidEvent(format!(
                    "duplicate or invalid Context contradiction: {}",
                    contradiction.contradiction_id
                )));
            }
            if claims.len() != 2 {
                return Err(ReducerError::InvalidEvent(
                    "Context contradiction must update exactly two claims".to_string(),
                ));
            }
            for claim in claims {
                upsert_context_claim(&mut state, claim, true)?;
            }
            let scope = (
                contradiction.project_root.clone(),
                contradiction.continuity_id.clone(),
                contradiction.attachment_id.clone(),
                contradiction.committed_at,
            );
            state.context_contradictions.push(contradiction);
            refresh_reactive_context(&mut state, &scope.0, &scope.1, &scope.2, scope.3);
        }
        FocusaEvent::ContextContradictionResolved {
            contradiction,
            claims,
            decision,
        } => {
            let Some(index) = state
                .context_contradictions
                .iter()
                .position(|existing| existing.contradiction_id == contradiction.contradiction_id)
            else {
                return Err(ReducerError::InvalidEvent(format!(
                    "Context contradiction not found: {}",
                    contradiction.contradiction_id
                )));
            };
            let expected_revision = state.context_contradictions[index].revision + 1;
            if contradiction.revision != expected_revision || contradiction.status != "resolved" {
                return Err(ReducerError::InvalidEvent(format!(
                    "Context contradiction resolution revision/status mismatch: {}",
                    contradiction.contradiction_id
                )));
            }
            for claim in claims {
                upsert_context_claim(&mut state, claim, true)?;
            }
            let scope = (
                contradiction.project_root.clone(),
                contradiction.continuity_id.clone(),
                contradiction.attachment_id.clone(),
                decision.decided_at,
            );
            state.context_contradictions[index] = contradiction;
            if state
                .context_decisions
                .iter()
                .any(|existing| existing.decision_id == decision.decision_id)
            {
                return Err(ReducerError::InvalidEvent(format!(
                    "duplicate Context decision: {}",
                    decision.decision_id
                )));
            }
            state.context_decisions.push(decision);
            refresh_reactive_context(&mut state, &scope.0, &scope.1, &scope.2, scope.3);
        }

        // ─── Instance Lifecycle ─────────────────────────────────────────
        FocusaEvent::InstanceConnected { instance_id, kind } => {
            if !state.instances.iter().any(|i| i.id == instance_id) {
                state.instances.push(Instance {
                    id: instance_id,
                    kind,
                    created_at: Utc::now(),
                    thread_id: None,
                });
            }
        }

        FocusaEvent::InstanceDisconnected {
            instance_id,
            reason: _,
        } => {
            // Keep instances for auditability; mark offline later when schema supports it.
            // For now, remove to avoid stale UI.
            state.instances.retain(|i| i.id != instance_id);
            // NOTE: attachments are keyed by session_id, not instance_id.
            // Removal on disconnect will happen once session<->instance mapping is stored.
        }

        // ─── Thread Attachments (docs/40) ───────────────────────────────
        FocusaEvent::ThreadAttached {
            instance_id: _,
            session_id,
            thread_id,
            role,
        } => {
            // One attachment per (session_id, thread_id) pair.
            if !state
                .attachments
                .iter()
                .any(|a| a.session_id == session_id && a.thread_id == thread_id)
            {
                state.attachments.push(Attachment {
                    session_id,
                    thread_id,
                    role,
                    attached_at: Utc::now(),
                });
            }
        }

        FocusaEvent::ThreadDetached {
            instance_id: _,
            session_id,
            thread_id,
            reason: _,
        } => {
            state
                .attachments
                .retain(|a| !(a.session_id == session_id && a.thread_id == thread_id));
        }

        // ─── Session Lifecycle ───────────────────────────────────────────
        FocusaEvent::SessionStarted {
            session_id,
            adapter_id,
            workspace_id,
            project_root,
            continuity_id,
        } => {
            if let Some(existing) = &state.session
                && existing.status == SessionStatus::Active
            {
                return Err(ReducerError::InvalidEvent(
                    "SessionStarted but an active session already exists".into(),
                ));
            }
            state.session = Some(SessionState {
                session_id,
                created_at: Utc::now(),
                adapter_id,
                workspace_id,
                project_root,
                continuity_id,
                status: SessionStatus::Active,
            });
        }

        FocusaEvent::SessionRestored { session_id } => {
            // The daemon pre-loads state from disk before emitting this event.
            // Validate the loaded session matches the requested ID.
            match &state.session {
                Some(s) if s.session_id == session_id => {
                    // Already loaded — nothing to change.
                }
                Some(s) => {
                    return Err(ReducerError::SessionError(format!(
                        "SessionRestored for {} but loaded session is {}",
                        session_id, s.session_id
                    )));
                }
                None => {
                    return Err(ReducerError::SessionError(format!(
                        "SessionRestored for {} but no session in state — daemon must pre-load",
                        session_id
                    )));
                }
            }
        }

        FocusaEvent::SessionClosed { reason: _ } => {
            let session = state.session.as_mut().ok_or_else(|| {
                ReducerError::SessionError("SessionClosed but no session exists".into())
            })?;
            if session.status != SessionStatus::Active {
                return Err(ReducerError::SessionError(
                    "SessionClosed but session is already Closed".into(),
                ));
            }
            session.status = SessionStatus::Closed;
        }

        // ─── Turn Lifecycle ───────────────────────────────────────────────
        FocusaEvent::TurnStarted {
            turn_id,
            harness_name,
            adapter_id,
            raw_user_input,
        } => {
            // Store turn in active_turn for correlation.
            state.active_turn = Some(ActiveTurn {
                turn_id,
                adapter_id,
                harness_name,
                started_at: Utc::now(),
                raw_user_input,
                assembled_prompt: None,
            });
        }

        FocusaEvent::TurnCompleted {
            turn_id,
            harness_name: _,
            raw_user_input: _,
            assistant_output,
            artifacts_used: _,
            errors,
            prompt_tokens,
            completion_tokens,
        } => {
            // Validate turn_id matches before clearing.
            // Note: active_turn might already be None if turn_complete API cleared it.
            if let Some(ref turn) = state.active_turn
                && turn.turn_id != turn_id
            {
                tracing::warn!(
                    expected = %turn.turn_id,
                    got = %turn_id,
                    "TurnCompleted with mismatched turn_id"
                );
            }

            // Clear active turn only if IDs match and turn exists.
            if state
                .active_turn
                .as_ref()
                .is_some_and(|t| t.turn_id == turn_id)
            {
                state.active_turn.take();
            }

            // Record turn completion in CLT (conversation depth tracking).
            {
                use crate::clt;
                let metadata = CltMetadata {
                    trajectory: state.trajectory_ladder_context(),
                    ..CltMetadata::default()
                };
                clt::append_interaction(
                    &mut state.clt,
                    state.session.as_ref().map(|s| s.session_id),
                    "assistant",
                    assistant_output.as_deref(),
                    metadata,
                );
            }

            if let Some(tokens) = prompt_tokens {
                state.telemetry.total_prompt_tokens += tokens as u64;
            }
            if let Some(tokens) = completion_tokens {
                state.telemetry.total_completion_tokens += tokens as u64;
            }

            // Update FrameStats on active frame (G1-detail-05 §FrameStats).
            if let Some(active_id) = state.focus_stack.active_id
                && let Some(frame) = state
                    .focus_stack
                    .frames
                    .iter_mut()
                    .find(|f| f.id == active_id)
            {
                frame.stats.turn_count += 1;
                frame.stats.last_turn_id = Some(turn_id.clone());
                frame.stats.last_token_estimate = prompt_tokens;
            }

            // Emit errors as intuition signals.
            for err in errors {
                let signal_id = Uuid::now_v7();
                state.focus_gate.signals.push(Signal {
                    id: signal_id,
                    ts: Utc::now(),
                    origin: SignalOrigin::Daemon,
                    kind: SignalKind::Error,
                    frame_context: state.focus_stack.active_id,
                    summary: err,
                    payload_ref: None,
                    tags: vec![],
                });
            }
        }

        // ─── Continuous Work Loop ───────────────────────────────────────
        FocusaEvent::ContinuousWorkModeEnabled {
            project_run_id,
            policy,
            scope,
            work_item_id,
            workpoint_id,
        } => {
            state.work_loop.execution_scope = scope;
            state.work_loop.execution_work_item_id = work_item_id;
            state.work_loop.execution_workpoint_id = workpoint_id;
            state.work_loop.enabled = true;
            state.work_loop.status = WorkLoopStatus::Idle;
            state.work_loop.current_task = None;
            state.work_loop.deferred_items.clear();
            state.work_loop.run.task_run_id = None;
            state.work_loop.run.tranche_run_id = None;
            state.work_loop.run.worker_session_id = None;
            state.work_loop.policy = policy;
            state.work_loop.run.project_run_id = project_run_id;
            state.work_loop.last_blocker_class = None;
            state.work_loop.last_blocker_reason = None;
            state.work_loop.last_continue_reason = None;
            state.work_loop.last_observed_summary = None;
            state.work_loop.last_safe_reentry_prompt_basis =
                Some("resume from enabled continuous work".to_string());
            state.work_loop.restored_context_summary = Some(
                "project mission active; constraints and verification posture inherited from current state"
                    .to_string(),
            );
            let now = Utc::now();
            state.work_loop.enabled_at = Some(now);
            state.work_loop.budget_epoch_id = Some(Uuid::now_v7());
            state.work_loop.budget_epoch_started_at = Some(now);
            state.work_loop.budget_renewal_count = 0;
            state.work_loop.budget_exhaustion = None;
            state.work_loop.last_turn_requested_at = None;
            state.work_loop.turn_count = 0;
            state.work_loop.consecutive_failures_for_task_class = 0;
            state.work_loop.consecutive_low_productivity_turns = 0;
            state.work_loop.consecutive_same_work_item_retries = 0;
            state.work_loop.last_observed_work_item_id = None;
        }
        FocusaEvent::ContinuousWorkModeDisabled { reason } => {
            state.work_loop.execution_scope = None;
            state.work_loop.execution_work_item_id = None;
            state.work_loop.execution_workpoint_id = None;
            state.work_loop.transport_session_id = None;
            state.work_loop.transport_scope = None;
            state.work_loop.transport_work_item_id = None;
            state.work_loop.transport_workpoint_id = None;
            state.work_loop.enabled = false;
            state.work_loop.status = WorkLoopStatus::Idle;
            state.work_loop.current_task = None;
            state.work_loop.deferred_items.clear();
            state.work_loop.run.task_run_id = None;
            state.work_loop.run.tranche_run_id = None;
            state.work_loop.run.worker_session_id = None;
            state.work_loop.last_continue_reason = Some(reason);
            state.work_loop.enabled_at = None;
            state.work_loop.budget_epoch_id = None;
            state.work_loop.budget_epoch_started_at = None;
            state.work_loop.budget_exhaustion = None;
            state.work_loop.last_turn_requested_at = None;
        }
        FocusaEvent::ContinuousPauseFlagsUpdated {
            destructive_confirmation_required,
            governance_decision_pending,
            operator_override_active,
            reason,
        } => {
            state.work_loop.pause_flags = WorkLoopPauseFlags {
                destructive_confirmation_required,
                governance_decision_pending,
                operator_override_active,
                reason: reason.clone(),
            };
            if destructive_confirmation_required
                || governance_decision_pending
                || operator_override_active
            {
                state.work_loop.status = WorkLoopStatus::Paused;
                state.work_loop.last_blocker_reason = reason;
            }
        }
        FocusaEvent::ContinuousDecisionContextUpdated {
            current_ask,
            ask_kind,
            scope_kind,
            carryover_policy,
            excluded_context_reason,
            excluded_context_labels,
            source_turn_id,
            operator_steering_detected,
        } => {
            if current_ask.is_some() {
                state.work_loop.decision_context.current_ask = current_ask;
            }
            if ask_kind.is_some() {
                state.work_loop.decision_context.ask_kind = ask_kind;
            }
            if scope_kind.is_some() {
                state.work_loop.decision_context.scope_kind = scope_kind;
            }
            if carryover_policy.is_some() {
                state.work_loop.decision_context.carryover_policy = carryover_policy;
            }
            if excluded_context_reason.is_some() {
                state.work_loop.decision_context.excluded_context_reason = excluded_context_reason;
            }
            if let Some(labels) = excluded_context_labels {
                state.work_loop.decision_context.excluded_context_labels = labels;
            }
            if source_turn_id.is_some() {
                state.work_loop.decision_context.source_turn_id = source_turn_id;
            }
            if let Some(steering) = operator_steering_detected {
                state.work_loop.decision_context.operator_steering_detected = steering;
            }
            if operator_steering_detected == Some(true) {
                state.work_loop.last_continue_reason =
                    Some("operator steering detected".to_string());
                // Steering redirects active work; it must not imply stop/pause.
                if state.work_loop.pause_flags.reason.as_deref()
                    == Some("operator steering detected")
                {
                    state.work_loop.pause_flags = WorkLoopPauseFlags::default();
                    if state.work_loop.enabled && state.work_loop.status == WorkLoopStatus::Paused {
                        state.work_loop.status = WorkLoopStatus::Idle;
                    }
                }
            }
        }
        FocusaEvent::ContinuousTransportSessionAttached {
            adapter,
            session_id,
            scope,
            work_item_id,
            workpoint_id,
        } => {
            state.work_loop.transport_adapter = Some(adapter);
            state.work_loop.transport_session_id = Some(session_id.clone());
            state.work_loop.transport_scope = Some(scope);
            state.work_loop.transport_work_item_id = Some(work_item_id);
            state.work_loop.transport_workpoint_id = Some(workpoint_id);
            state.work_loop.run.worker_session_id = Some(session_id.clone());
            state.work_loop.transport_session_state = Some("attached".to_string());
            state.work_loop.last_transport_event_kind = Some("session_attached".to_string());
            state.work_loop.last_transport_event_summary = Some(session_id);
            if state.work_loop.enabled
                && state.work_loop.status == WorkLoopStatus::TransportDegraded
            {
                state.work_loop.status = if state.work_loop.current_task.is_some() {
                    WorkLoopStatus::SelectingReadyWork
                } else {
                    WorkLoopStatus::Idle
                };
                state.work_loop.last_blocker_class = None;
                state.work_loop.last_blocker_reason = None;
            }
        }
        FocusaEvent::ContinuousTransportAbortForwarded { reason } => {
            state.work_loop.transport_abort_reason = Some(reason.clone());
            state.work_loop.transport_session_state = Some("abort_requested".to_string());
            state.work_loop.last_transport_event_kind = Some("abort_requested".to_string());
            state.work_loop.last_transport_event_summary = Some(reason);
        }
        FocusaEvent::ContinuousTransportEventIngested {
            sequence,
            kind,
            session_id,
            turn_id,
            summary,
        } => {
            state.work_loop.last_transport_event_sequence = sequence;
            state.work_loop.last_transport_event_kind = Some(kind.clone());
            state.work_loop.last_transport_event_summary =
                Some(match (session_id, turn_id, summary) {
                    (Some(session_id), Some(turn_id), Some(summary)) => {
                        format!("session={session_id} turn={turn_id} {summary}")
                    }
                    (Some(session_id), _, Some(summary)) => {
                        format!("session={session_id} {summary}")
                    }
                    (_, Some(turn_id), Some(summary)) => format!("turn={turn_id} {summary}"),
                    (_, _, Some(summary)) => summary,
                    (_, Some(turn_id), None) => format!("turn={turn_id}"),
                    (Some(session_id), _, None) => format!("session={session_id}"),
                    _ => kind.clone(),
                });
            state.work_loop.transport_session_state = Some(match kind.as_str() {
                "agent_start" => "running".to_string(),
                "turn_start" => "turn_active".to_string(),
                "message_update" => "streaming".to_string(),
                "turn_end" => "turn_completed".to_string(),
                "agent_end" => "agent_completed".to_string(),
                "response" | "extension_ui_request" => "attached".to_string(),
                "stream_closed" => "detached".to_string(),
                "stderr_line" => "degraded".to_string(),
                _ => "observed".to_string(),
            });
        }
        FocusaEvent::ContinuousAuthorshipDelegated {
            delegate_id,
            scope,
            amendment_summary,
        } => {
            let requires_replan = amendment_summary.is_some();
            state.work_loop.authorship_mode = AuthorshipMode::Delegated;
            state.work_loop.delegated_authorship = Some(DelegatedAuthorshipState {
                delegate_id,
                scope,
                amendment_summary,
            });
            if requires_replan {
                state.work_loop.status = WorkLoopStatus::Paused;
                state.work_loop.current_task = None;
                state.work_loop.last_blocker_class = Some(BlockerClass::SpecGap);
                state.work_loop.last_blocker_reason = Some(
                    "authoritative spec amendment requires replan of current/queued work"
                        .to_string(),
                );
            }
        }
        FocusaEvent::ContinuousAuthorshipDelegationCleared { reason } => {
            state.work_loop.authorship_mode = AuthorshipMode::OperatorOnly;
            state.work_loop.delegated_authorship = None;
            state.work_loop.last_continue_reason = Some(reason);
        }
        FocusaEvent::ContinuousWorkItemSelected {
            task_run_id,
            packet,
        } => {
            let degraded = state.work_loop.status == WorkLoopStatus::TransportDegraded;
            let worker = recommended_worker_for_task(
                packet.task_class,
                degraded,
                state.work_loop.consecutive_failures_for_task_class,
            );
            let task_run_id = task_run_id.or_else(|| Some(Uuid::now_v7()));
            state.work_loop.status = WorkLoopStatus::SelectingReadyWork;
            state.work_loop.run.task_run_id = task_run_id;
            state.work_loop.run.tranche_run_id = packet.tranche_id.as_ref().map(|_| Uuid::now_v7());
            state.work_loop.run.worker_session_id = state
                .work_loop
                .run
                .task_run_id
                .map(|task_id| format!("{}:{}", worker.worker_id, task_id));
            state.work_loop.active_worker = Some(worker);
            state.work_loop.last_safe_reentry_prompt_basis = Some(format!(
                "resume selected work item {}: {}",
                packet.work_item_id, packet.title
            ));
            state.work_loop.restored_context_summary = Some(format!(
                "allowed_scope={:?}; linked_spec_refs={:?}; verification_tier={:?}",
                packet.allowed_scope, packet.linked_spec_refs, packet.required_verification_tier
            ));
            state.work_loop.current_task = Some(packet);
        }
        FocusaEvent::ContinuousWorkItemDeferred {
            work_item_id,
            reason,
        } => {
            state
                .work_loop
                .deferred_items
                .retain(|item| item.work_item_id != work_item_id);
            state.work_loop.deferred_items.push(WorkLoopDeferredItem {
                work_item_id: work_item_id.clone(),
                reason: reason.clone(),
                deferred_at: Utc::now(),
            });
            state.work_loop.last_blocker_reason = Some(reason);
            if state
                .work_loop
                .current_task
                .as_ref()
                .map(|task| task.work_item_id.as_str())
                == Some(work_item_id.as_str())
            {
                state.work_loop.current_task = None;
            }
            state.work_loop.status = WorkLoopStatus::SelectingReadyWork;
        }
        FocusaEvent::ContinuousTurnRequested {
            task_run_id,
            work_item_id: _,
            reason,
        } => {
            state.work_loop.status = WorkLoopStatus::PreparingTurn;
            state.work_loop.run.task_run_id = task_run_id;
            state.work_loop.last_continue_reason = Some(reason);
            state.work_loop.last_turn_requested_at = Some(Utc::now());
            state.work_loop.turn_count += 1;
        }
        FocusaEvent::ContinuousTurnStarted {
            task_run_id,
            work_item_id: _,
        } => {
            state.work_loop.status = WorkLoopStatus::AwaitingHarnessTurn;
            state.work_loop.run.task_run_id = task_run_id;
        }
        FocusaEvent::ContinuousTurnObserved {
            task_run_id,
            summary,
        } => {
            state.work_loop.status = WorkLoopStatus::EvaluatingOutcome;
            state.work_loop.run.task_run_id = task_run_id;
            state.work_loop.last_observed_summary = Some(summary.clone());
            state.work_loop.last_safe_reentry_prompt_basis = Some(summary.clone());
            state.work_loop.last_continue_reason = Some(summary);
        }
        FocusaEvent::ContinuousTurnCompleted {
            task_run_id,
            work_item_id,
            continue_reason,
            verification_satisfied: _,
            spec_conformant: _,
            outcome_status,
            ..
        } => {
            state.work_loop.run.task_run_id = task_run_id;
            state.work_loop.last_continue_reason = continue_reason;
            state.work_loop.last_observed_work_item_id = work_item_id.clone();
            match outcome_status {
                WorkLoopOutcomeStatus::Completed => {
                    state.work_loop.status = WorkLoopStatus::AdvancingTask;
                    state.work_loop.last_completed_task_id = work_item_id.clone();
                    state.work_loop.last_recorded_bd_transition_id = work_item_id;
                    state.work_loop.consecutive_failures_for_task_class = 0;
                    state.work_loop.consecutive_low_productivity_turns = 0;
                    state.work_loop.consecutive_same_work_item_retries = 0;
                    state.work_loop.deferred_items.clear();
                    state.work_loop.run.worker_session_id = None;
                    state.work_loop.current_task = None;
                }
                WorkLoopOutcomeStatus::Blocked => {
                    state.work_loop.status = WorkLoopStatus::Blocked;
                }
                WorkLoopOutcomeStatus::Continue => {
                    state.work_loop.status = WorkLoopStatus::Idle;
                }
            }
        }
        FocusaEvent::ContinuousSecondaryLoopOutcomeRecorded { .. } => {
            // Runtime updates secondary-loop telemetry eagerly in daemon state.
            // Keep reducer no-op for this observability event so replay remains
            // backward-compatible while still retaining auditable event-log data.
        }
        FocusaEvent::ContinuousTurnPaused { reason } => {
            state.work_loop.status = WorkLoopStatus::Paused;
            state.work_loop.last_safe_reentry_prompt_basis = Some(reason.clone());
            state.work_loop.last_continue_reason = Some(reason);
        }
        FocusaEvent::ContinuousTurnBlocked {
            blocker_class,
            reason,
            work_item_id: _,
        } => {
            state.work_loop.status = WorkLoopStatus::Blocked;
            state.work_loop.last_blocker_class = Some(blocker_class);
            state.work_loop.last_blocker_reason = Some(reason);
            state.work_loop.consecutive_failures_for_task_class += 1;
            if let Some(current_task) = state.work_loop.current_task.as_ref() {
                let degraded = state.work_loop.status == WorkLoopStatus::TransportDegraded;
                state.work_loop.active_worker = Some(recommended_worker_for_task(
                    current_task.task_class,
                    degraded,
                    state.work_loop.consecutive_failures_for_task_class,
                ));
            }
        }
        FocusaEvent::ContinuousTurnEscalated {
            reason,
            work_item_id: _,
        } => {
            state.work_loop.status = WorkLoopStatus::Paused;
            state.work_loop.last_blocker_reason = Some(reason);
        }
        FocusaEvent::ContinuousTrancheCompleted {
            tranche_id: _,
            reason,
        } => {
            state.work_loop.status = WorkLoopStatus::AdvancingTask;
            state.work_loop.last_continue_reason = Some(reason);
            state.work_loop.run.tranche_run_id = None;
        }
        FocusaEvent::ContinuousLoopBudgetExhausted { dimension, reason } => {
            state.work_loop.status = WorkLoopStatus::Paused;
            state.work_loop.last_blocker_reason = Some(reason.clone());
            state.work_loop.budget_exhaustion = Some(WorkLoopBudgetExhaustion {
                dimension,
                reason,
                exhausted_at: Utc::now(),
                epoch_id: state.work_loop.budget_epoch_id.unwrap_or_else(Uuid::now_v7),
            });
        }
        FocusaEvent::ContinuousLoopTransportDegraded { reason } => {
            state.work_loop.status = WorkLoopStatus::TransportDegraded;
            state.work_loop.last_blocker_reason = Some(reason);
        }
        FocusaEvent::ContinuousLoopResumed {
            reason,
            budget_renewed,
            policy,
        } => {
            state.work_loop.status = WorkLoopStatus::Idle;
            state.work_loop.pause_flags = WorkLoopPauseFlags::default();
            state.work_loop.last_safe_reentry_prompt_basis = Some(reason.clone());
            state.work_loop.last_continue_reason = Some(reason);
            if let Some(policy) = policy {
                state.work_loop.policy = policy;
            }
            if budget_renewed {
                state.work_loop.budget_epoch_id = Some(Uuid::now_v7());
                state.work_loop.budget_epoch_started_at = Some(Utc::now());
                state.work_loop.budget_renewal_count =
                    state.work_loop.budget_renewal_count.saturating_add(1);
                state.work_loop.budget_exhaustion = None;
                state.work_loop.turn_count = 0;
                state.work_loop.consecutive_failures_for_task_class = 0;
                state.work_loop.consecutive_low_productivity_turns = 0;
                state.work_loop.consecutive_same_work_item_retries = 0;
            }
        }
        FocusaEvent::ContinuousLoopRecoveryCheckpointed {
            checkpoint_id,
            summary,
        } => {
            state.work_loop.run.last_checkpoint_id = Some(checkpoint_id);
            state.work_loop.last_safe_reentry_prompt_basis = Some(summary.clone());
            state.work_loop.last_continue_reason = Some(summary);
        }

        // ─── Focus Stack ─────────────────────────────────────────────────
        FocusaEvent::FocusFramePushed {
            frame_id,
            beads_issue_id,
            title,
            goal,
            project_root,
            continuity_id,
            constraints,
            tags,
        } => {
            if beads_issue_id.is_empty() {
                return Err(ReducerError::InvariantViolation(
                    "FocusFramePushed with empty beads_issue_id".into(),
                ));
            }

            let now = Utc::now();
            let stack = &mut state.focus_stack;

            if stack.frames.iter().any(|f| f.id == frame_id) {
                return Err(ReducerError::InvalidEvent(format!(
                    "FocusFramePushed with duplicate frame_id {}",
                    frame_id
                )));
            }

            // Pause only the current active frame in the same logical continuity scope.
            // Other same-root sessions keep their own active frame; scoped reads use continuity_id.
            let parent_id = stack
                .frames
                .iter()
                .rev()
                .find(|frame| {
                    frame.status == FrameStatus::Active
                        && frame.project_root == project_root
                        && frame.continuity_id == continuity_id
                })
                .map(|frame| frame.id);
            if let Some(parent_id) = parent_id
                && let Some(frame) = stack.frames.iter_mut().find(|f| f.id == parent_id)
            {
                frame.status = FrameStatus::Paused;
                frame.updated_at = now;
            }

            stack.frames.push(FrameRecord {
                id: frame_id,
                parent_id,
                created_at: now,
                updated_at: now,
                status: FrameStatus::Active,
                title,
                goal,
                beads_issue_id,
                project_root,
                continuity_id,
                tags,
                priority_hint: None,
                ascc_checkpoint_id: None,
                stats: FrameStats::default(),
                constraints,
                focus_state: FocusState::default(),
                temporal_context: None,
                completed_at: None,
                completion_reason: None,
            });

            stack.active_id = Some(frame_id);
            if stack.root_id.is_none() {
                stack.root_id = Some(frame_id);
            }
            rebuild_stack_path(stack);
            stack.version += 1;
        }

        FocusaEvent::FocusFrameCompleted {
            frame_id,
            completion_reason,
        } => {
            let stack = &mut state.focus_stack;

            // Must be completing the active frame.
            if stack.active_id != Some(frame_id) {
                return Err(ReducerError::InvalidEvent(format!(
                    "FocusFrameCompleted for {} but active is {:?}",
                    frame_id, stack.active_id
                )));
            }

            let active_idx = stack
                .frames
                .iter()
                .position(|f| f.id == frame_id)
                .ok_or_else(|| ReducerError::FrameNotFound(frame_id.to_string()))?;

            let parent_id = stack.frames[active_idx].parent_id;

            // Validate parent is Paused (if it exists).
            if let Some(pid) = parent_id {
                let parent = stack
                    .frames
                    .iter()
                    .find(|f| f.id == pid)
                    .ok_or_else(|| ReducerError::FrameNotFound(format!("parent {}", pid)))?;
                if parent.status != FrameStatus::Paused {
                    return Err(ReducerError::InvariantViolation(format!(
                        "Parent frame {} has status {:?}, expected Paused",
                        pid, parent.status
                    )));
                }
            }

            if parent_id.is_none() {
                return Err(ReducerError::InvariantViolation(format!(
                    "FocusFrameCompleted cannot complete root frame {} without parent handoff",
                    frame_id
                )));
            }

            // All checks passed — mutate.
            let now = Utc::now();
            stack.frames[active_idx].status = FrameStatus::Completed;
            stack.frames[active_idx].updated_at = now;
            // G1-detail-05 UPDATE: store completed_at + completion_reason on FrameRecord.
            stack.frames[active_idx].completed_at = Some(now);
            stack.frames[active_idx].completion_reason = Some(completion_reason);

            // G1-detail-05 UPDATE §Focus Gate Integration:
            // "blocked → raises surface pressure on related candidates"
            // "abandoned → suppress related candidates"
            match completion_reason {
                CompletionReason::Blocked => {
                    // Raise pressure on candidates related to this frame.
                    for candidate in &mut state.focus_gate.candidates {
                        if candidate.related_frame_id == Some(frame_id)
                            && candidate.state != CandidateState::Resolved
                        {
                            candidate.pressure += 1.0;
                            candidate.updated_at = now;
                        }
                    }
                }
                CompletionReason::Abandoned => {
                    // Suppress candidates related to this frame.
                    for candidate in &mut state.focus_gate.candidates {
                        if candidate.related_frame_id == Some(frame_id)
                            && candidate.state != CandidateState::Resolved
                        {
                            candidate.state = CandidateState::Suppressed;
                            candidate.pressure = 0.0;
                            candidate.updated_at = now;
                        }
                    }
                }
                _ => {}
            }

            if let Some(pid) = parent_id {
                if let Some(parent) = stack.frames.iter_mut().find(|f| f.id == pid) {
                    parent.status = FrameStatus::Active;
                    parent.updated_at = now;
                }
                stack.active_id = Some(pid);
            } else {
                stack.active_id = None;
                stack.root_id = None;
            }

            rebuild_stack_path(stack);
            stack.version += 1;
        }

        FocusaEvent::FocusFrameSuspended {
            frame_id,
            reason: _,
        } => {
            let stack = &mut state.focus_stack;

            if stack.active_id != Some(frame_id) {
                return Err(ReducerError::InvalidEvent(format!(
                    "FocusFrameSuspended for {} but active is {:?}",
                    frame_id, stack.active_id
                )));
            }

            let now = Utc::now();
            if let Some(frame) = stack.frames.iter_mut().find(|f| f.id == frame_id) {
                frame.status = FrameStatus::Paused;
                frame.updated_at = now;
            }

            // Suspension clears active — user must explicitly resume or push.
            stack.active_id = None;
            rebuild_stack_path(stack);
            stack.version += 1;
        }

        FocusaEvent::FocusFrameResumed { frame_id } => {
            let stack = &mut state.focus_stack;
            let now = Utc::now();

            // Target frame must exist and be Paused or Suspended.
            let target = stack.frames.iter().find(|f| f.id == frame_id);
            match target {
                None => {
                    return Err(ReducerError::InvalidEvent(format!(
                        "FocusFrameResumed: frame {} not found",
                        frame_id
                    )));
                }
                Some(f) if f.status != FrameStatus::Paused => {
                    return Err(ReducerError::InvalidEvent(format!(
                        "FocusFrameResumed: frame {} is {:?}, not Paused",
                        frame_id, f.status
                    )));
                }
                _ => {}
            }

            // Suspend current active frame (if any).
            if let Some(active_id) = stack.active_id
                && let Some(active) = stack.frames.iter_mut().find(|f| f.id == active_id)
            {
                active.status = FrameStatus::Paused;
                active.updated_at = now;
            }

            // Activate target.
            if let Some(frame) = stack.frames.iter_mut().find(|f| f.id == frame_id) {
                frame.status = FrameStatus::Active;
                frame.updated_at = now;
            }

            stack.active_id = Some(frame_id);
            rebuild_stack_path(stack);
            stack.version += 1;
        }

        // ─── Focus State ─────────────────────────────────────────────────
        FocusaEvent::FocusStateUpdated { frame_id, delta } => {
            let frame = state
                .focus_stack
                .frames
                .iter_mut()
                .find(|f| f.id == frame_id)
                .ok_or_else(|| ReducerError::FrameNotFound(frame_id.to_string()))?;
            if frame.status == FrameStatus::Completed {
                return Err(ReducerError::InvalidEvent(format!(
                    "FocusStateUpdated for completed frame {}",
                    frame_id
                )));
            }

            apply_delta(&mut frame.focus_state, &delta);
            frame.updated_at = Utc::now();
        }
        FocusaEvent::TemporalFrameContextProjected { frame_id, context } => {
            let frame = state
                .focus_stack
                .frames
                .iter_mut()
                .find(|candidate| candidate.id == frame_id)
                .ok_or_else(|| ReducerError::FrameNotFound(frame_id.to_string()))?;
            if frame.status == FrameStatus::Completed {
                return Err(ReducerError::InvalidEvent(format!(
                    "TemporalFrameContextProjected for completed frame {}",
                    frame_id
                )));
            }
            let frame_root = frame.project_root.as_deref().unwrap_or_default();
            let frame_continuity = frame.continuity_id.as_deref().unwrap_or_default();
            if frame_root != context.projection.scope.project_root
                || frame_continuity != context.projection.scope.continuity_id
            {
                return Err(ReducerError::InvalidEvent(format!(
                    "temporal context scope mismatch for frame {}",
                    frame_id
                )));
            }
            if context.projected_at < context.projection.as_of {
                return Err(ReducerError::InvalidEvent(format!(
                    "temporal context projection time precedes source projection for frame {}",
                    frame_id
                )));
            }
            frame.updated_at = context.projected_at;
            frame.temporal_context = Some(context);
            state.focus_stack.version += 1;
        }

        // ─── Intuition → Gate ────────────────────────────────────────────
        FocusaEvent::IntuitionSignalObserved {
            signal_id,
            signal_type,
            severity: _,
            summary,
            related_frame_id,
        } => {
            let now = Utc::now();
            state.focus_gate.signals.push(Signal {
                id: signal_id,
                ts: now,
                origin: SignalOrigin::Daemon,
                kind: signal_type,
                frame_context: related_frame_id,
                summary,
                payload_ref: None,
                tags: vec![],
            });
        }

        FocusaEvent::CandidateSurfaced {
            candidate_id,
            kind,
            description,
            pressure,
            related_frame_id,
        } => {
            let now = Utc::now();
            // Upsert: update if exists, create if new.
            if let Some(existing) = state
                .focus_gate
                .candidates
                .iter_mut()
                .find(|c| c.id == candidate_id)
            {
                existing.pressure = pressure;
                existing.label = description;
                existing.last_seen_at = now;
                existing.times_seen += 1;
                existing.updated_at = now;
                // Re-surface if was latent.
                if existing.state == CandidateState::Latent {
                    existing.state = CandidateState::Surfaced;
                }
            } else {
                state.focus_gate.candidates.push(Candidate {
                    id: candidate_id,
                    created_at: now,
                    updated_at: now,
                    kind,
                    label: description,
                    origin_signal_ids: vec![],
                    related_frame_id,
                    state: CandidateState::Surfaced,
                    pressure,
                    last_seen_at: now,
                    times_seen: 1,
                    suppressed_until: None,
                    resolution: None,
                    pinned: false,
                });
            }
        }

        FocusaEvent::CandidatePinned { candidate_id } => {
            let candidate = state
                .focus_gate
                .candidates
                .iter_mut()
                .find(|c| c.id == candidate_id)
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!("Candidate {} not found", candidate_id))
                })?;
            candidate.pinned = true;
            candidate.updated_at = Utc::now();
        }

        FocusaEvent::CandidateSuppressed {
            candidate_id,
            scope: _,
            suppressed_until,
        } => {
            let candidate = state
                .focus_gate
                .candidates
                .iter_mut()
                .find(|c| c.id == candidate_id)
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!("Candidate {} not found", candidate_id))
                })?;
            candidate.state = CandidateState::Suppressed;
            candidate.pressure = 0.0;
            candidate.suppressed_until = suppressed_until;
            candidate.updated_at = Utc::now();
        }

        // ─── Reference Store ─────────────────────────────────────────────
        FocusaEvent::ArtifactRegistered {
            handle,
            storage_uri: _,
        } => {
            // Check immutability: if this artifact_id already exists, reject.
            if state
                .reference_index
                .handles
                .iter()
                .any(|h| h.id == handle.id)
            {
                return Err(ReducerError::InvariantViolation(format!(
                    "Artifact {} already registered — artifacts are immutable",
                    handle.id
                )));
            }

            state.reference_index.handles.push(handle);
        }

        FocusaEvent::ArtifactPinned { artifact_id } => {
            let handle = state
                .reference_index
                .handles
                .iter_mut()
                .find(|h| h.id == artifact_id)
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!("Artifact {} not found", artifact_id))
                })?;
            handle.pinned = true;
        }

        FocusaEvent::ArtifactGarbageCollected { artifact_id } => {
            let idx = state
                .reference_index
                .handles
                .iter()
                .position(|h| h.id == artifact_id)
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!("Artifact {} not found for GC", artifact_id))
                })?;
            // Pinned artifacts cannot be garbage collected.
            if state.reference_index.handles[idx].pinned {
                return Err(ReducerError::InvariantViolation(format!(
                    "Artifact {} is pinned — cannot garbage collect",
                    artifact_id
                )));
            }
            state.reference_index.handles.remove(idx);
        }

        // ─── Workers ─────────────────────────────────────────────────────
        FocusaEvent::WorkerJobEnqueued { .. }
        | FocusaEvent::WorkerJobStarted { .. }
        | FocusaEvent::WorkerJobCompleted { .. }
        | FocusaEvent::WorkerJobFailed { .. } => {
            // Worker events are advisory/telemetry only.
        }

        // ─── Prompt Assembly ─────────────────────────────────────────────
        FocusaEvent::PromptAssembled { .. } => {
            // Prompt assembly events are telemetry only.
        }

        FocusaEvent::AutonomyAdjusted {
            level,
            scope,
            ttl,
            reason,
        } => {
            crate::autonomy::grant_level(&mut state.autonomy, level, scope, ttl, &reason);
        }

        // ─── Memory ──────────────────────────────────────────────────────
        FocusaEvent::SemanticMemoryUpserted { key, value, source } => {
            let memory_source = match source.as_str() {
                "worker" => crate::types::MemorySource::Worker,
                "manual" => crate::types::MemorySource::Manual,
                "operator" => crate::types::MemorySource::Operator,
                "constitution" => crate::types::MemorySource::Constitution,
                "focus_state" => crate::types::MemorySource::FocusState,
                "context_core" => crate::types::MemorySource::ContextCore,
                "mem0" => crate::types::MemorySource::Mem0,
                _ => crate::types::MemorySource::User,
            };
            let _ = crate::memory::semantic::upsert(&mut state.memory, key, value, memory_source);
        }
        FocusaEvent::SemanticMemoryContradictionsResolved { reason: _ } => {
            crate::memory::semantic::resolve_contradictions(&mut state.memory);
        }
        FocusaEvent::RuleReinforced { .. } | FocusaEvent::MemoryDecayTick { .. } => {
            // Memory maintenance events remain advisory here.
        }

        // ─── RFM ─────────────────────────────────────────────────────────
        FocusaEvent::RfmRegenerationTriggered { .. } => {
            // RFM regeneration events are telemetry only.
            // Actual regeneration is handled by the daemon/proxy layer.
        }

        // ─── Ontology Classification / Reducer ──────────────────────────
        FocusaEvent::OntologyObjectUpsertProposed {
            workstream,
            proposal_id,
            object_type,
            object_id,
            source,
        } => {
            let now = Utc::now();
            let record = OntologyProposalRecord {
                workstream: workstream.clone(),
                proposal_id,
                proposal_kind: "object_upsert".to_string(),
                target_class: object_type.clone(),
                status: "proposed".to_string(),
                source: Some(source.clone()),
                object_type: Some(object_type.clone()),
                object_id: object_id.clone(),
                link_type: None,
                source_id: None,
                target_id: None,
                notes: None,
                updated_at: Some(now),
            };
            if let Some(existing) = state
                .ontology
                .proposals
                .iter_mut()
                .find(|p| p.proposal_id == proposal_id && p.workstream == workstream)
            {
                *existing = record;
            } else {
                state.ontology.proposals.push(record);
            }
            if let Some(id) = object_id.clone() {
                let exists = state.ontology.objects.iter().any(|o| {
                    o.get("id").and_then(|v| v.as_str()) == Some(id.as_str())
                        && ontology_value_matches_workstream(o, &workstream)
                });
                if !exists {
                    state.ontology.objects.push(serde_json::json!({
                        "workstream": workstream,
                        "id": id,
                        "object_type": object_type,
                        "status": "proposed",
                        "provenance_class": "model_inferred",
                        "source": source,
                    }));
                }
            }
            state.ontology.delta_log.push(OntologyDeltaRecord {
                workstream: workstream.clone(),
                delta_kind: "ontology_object_upsert_proposed".to_string(),
                payload: serde_json::json!({
                    "proposal_id": proposal_id,
                    "object_type": object_type,
                    "object_id": object_id,
                    "source": source,
                }),
                timestamp: Some(now),
            });
        }
        FocusaEvent::OntologyLinkUpsertProposed {
            workstream,
            proposal_id,
            link_type,
            source_id,
            target_id,
            source,
        } => {
            let now = Utc::now();
            let record = OntologyProposalRecord {
                workstream: workstream.clone(),
                proposal_id,
                proposal_kind: "link_upsert".to_string(),
                target_class: link_type.clone(),
                status: "proposed".to_string(),
                source: Some(source.clone()),
                object_type: None,
                object_id: None,
                link_type: Some(link_type.clone()),
                source_id: Some(source_id.clone()),
                target_id: Some(target_id.clone()),
                notes: None,
                updated_at: Some(now),
            };
            if let Some(existing) = state
                .ontology
                .proposals
                .iter_mut()
                .find(|p| p.proposal_id == proposal_id && p.workstream == workstream)
            {
                *existing = record;
            } else {
                state.ontology.proposals.push(record);
            }
            state.ontology.links.push(serde_json::json!({
                "workstream": workstream,
                "type": link_type,
                "source_id": source_id,
                "target_id": target_id,
                "status": "proposed",
                "evidence": "proposal_submitted",
                "proposal_id": proposal_id,
                "source": source,
            }));
            state.ontology.delta_log.push(OntologyDeltaRecord {
                workstream: workstream.clone(),
                delta_kind: "ontology_link_upsert_proposed".to_string(),
                payload: serde_json::json!({
                    "proposal_id": proposal_id,
                    "link_type": link_type,
                    "source_id": source_id,
                    "target_id": target_id,
                    "source": source,
                }),
                timestamp: Some(now),
            });
        }
        FocusaEvent::OntologyStatusChangeProposed {
            workstream,
            proposal_id,
            subject,
            from_status,
            to_status,
            source,
        } => {
            let now = Utc::now();
            let record = OntologyProposalRecord {
                workstream: workstream.clone(),
                proposal_id,
                proposal_kind: "status_change".to_string(),
                target_class: "status".to_string(),
                status: "proposed".to_string(),
                source: Some(source.clone()),
                object_type: None,
                object_id: Some(subject.clone()),
                link_type: None,
                source_id: None,
                target_id: None,
                notes: Some(format!(
                    "{} -> {}",
                    from_status.clone().unwrap_or_else(|| "unknown".to_string()),
                    to_status
                )),
                updated_at: Some(now),
            };
            if let Some(existing) = state
                .ontology
                .proposals
                .iter_mut()
                .find(|p| p.proposal_id == proposal_id && p.workstream == workstream)
            {
                *existing = record;
            } else {
                state.ontology.proposals.push(record);
            }
            if let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                ontology_value_matches_workstream(o, &workstream)
                    && o.get("id").and_then(|v| v.as_str()) == Some(subject.as_str())
                    && ontology_value_matches_workstream(o, &workstream)
            }) {
                object["status"] = serde_json::Value::String(to_status.clone());
            }
            state.ontology.delta_log.push(OntologyDeltaRecord {
                workstream: workstream.clone(),
                delta_kind: "ontology_status_change_proposed".to_string(),
                payload: serde_json::json!({
                    "proposal_id": proposal_id,
                    "subject": subject,
                    "from_status": from_status,
                    "to_status": to_status,
                    "source": source,
                }),
                timestamp: Some(now),
            });
        }
        FocusaEvent::OntologyWorkingSetMembershipProposed {
            workstream,
            proposal_id,
            subject,
            operation,
            source,
        } => {
            let now = Utc::now();
            let record = OntologyProposalRecord {
                workstream: workstream.clone(),
                proposal_id,
                proposal_kind: "working_set_membership".to_string(),
                target_class: "working_set".to_string(),
                status: "proposed".to_string(),
                source: Some(source.clone()),
                object_type: Some("object_set".to_string()),
                object_id: Some(subject.clone()),
                link_type: None,
                source_id: None,
                target_id: None,
                notes: Some(operation.clone()),
                updated_at: Some(now),
            };
            if let Some(existing) = state
                .ontology
                .proposals
                .iter_mut()
                .find(|p| p.proposal_id == proposal_id && p.workstream == workstream)
            {
                *existing = record;
            } else {
                state.ontology.proposals.push(record);
            }
            if let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                ontology_value_matches_workstream(o, &workstream)
                    && o.get("id").and_then(|v| v.as_str()) == Some(subject.as_str())
                    && ontology_value_matches_workstream(o, &workstream)
            }) {
                let mut memberships = object
                    .get("working_set_memberships")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                if operation.eq_ignore_ascii_case("add") {
                    if !memberships
                        .iter()
                        .any(|v| v.as_str() == Some(source.as_str()))
                    {
                        memberships.push(serde_json::Value::String(source.clone()));
                    }
                } else {
                    memberships.retain(|v| v.as_str() != Some(source.as_str()));
                }
                object["working_set_memberships"] = serde_json::Value::Array(memberships);
                object["membership_class"] = if operation.eq_ignore_ascii_case("add") {
                    serde_json::Value::String("deterministic".to_string())
                } else {
                    serde_json::Value::String("provisional".to_string())
                };
                object["status"] = serde_json::Value::String("candidate".to_string());
            }
            state.ontology.delta_log.push(OntologyDeltaRecord {
                workstream: workstream.clone(),
                delta_kind: "ontology_working_set_membership_proposed".to_string(),
                payload: serde_json::json!({
                    "proposal_id": proposal_id,
                    "subject": subject,
                    "operation": operation,
                    "source": source,
                }),
                timestamp: Some(now),
            });
        }
        FocusaEvent::OntologyProposalPromoted {
            workstream,
            proposal_id,
            target_class,
            applied_kind,
        } => {
            let now = Utc::now();
            if let Some(proposal_idx) = state
                .ontology
                .proposals
                .iter()
                .position(|p| p.proposal_id == proposal_id && p.workstream == workstream)
            {
                let proposal = state.ontology.proposals[proposal_idx].clone();
                state.ontology.proposals[proposal_idx].status = "promoted".to_string();
                state.ontology.proposals[proposal_idx].updated_at = Some(now);

                match proposal.proposal_kind.as_str() {
                    "object_upsert" => {
                        if let Some(object_id) = proposal.object_id.as_ref() {
                            if let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            }) {
                                object["status"] =
                                    serde_json::Value::String("promoted".to_string());
                                object["provenance_class"] =
                                    serde_json::Value::String("reducer_promoted".to_string());
                                object["promoted_by"] =
                                    serde_json::Value::String(proposal_id.to_string());
                            } else {
                                state.ontology.objects.push(serde_json::json!({
                                    "workstream": workstream,
                                    "id": object_id,
                                    "object_type": proposal
                                        .object_type
                                        .clone()
                                        .unwrap_or_else(|| target_class.clone()),
                                    "status": "promoted",
                                    "provenance_class": "reducer_promoted",
                                    "promoted_by": proposal_id,
                                }));
                            }
                        }
                    }
                    "link_upsert" => {
                        if let (Some(link_type), Some(source_id), Some(target_id)) = (
                            proposal.link_type.as_ref(),
                            proposal.source_id.as_ref(),
                            proposal.target_id.as_ref(),
                        ) {
                            if let Some(link) = state.ontology.links.iter_mut().find(|l| {
                                ontology_value_matches_workstream(l, &workstream)
                                    && l.get("type").and_then(|v| v.as_str())
                                        == Some(link_type.as_str())
                                    && l.get("source_id").and_then(|v| v.as_str())
                                        == Some(source_id.as_str())
                                    && l.get("target_id").and_then(|v| v.as_str())
                                        == Some(target_id.as_str())
                            }) {
                                link["status"] = serde_json::Value::String("promoted".to_string());
                                link["proposal_id"] =
                                    serde_json::Value::String(proposal_id.to_string());
                            } else {
                                state.ontology.links.push(serde_json::json!({
                                    "workstream": workstream,
                                    "type": link_type,
                                    "source_id": source_id,
                                    "target_id": target_id,
                                    "status": "promoted",
                                    "proposal_id": proposal_id,
                                    "evidence": "proposal_promoted",
                                }));
                            }
                        }
                    }
                    "status_change" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["provenance_class"] =
                                serde_json::Value::String("reducer_promoted".to_string());
                        }
                    }
                    "working_set_membership" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["membership_class"] =
                                serde_json::Value::String("deterministic".to_string());
                        }
                    }
                    _ => {}
                }

                match applied_kind.as_str() {
                    "execute_migration" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("migrated".to_string());
                            object["migration_state"] =
                                serde_json::Value::String("applied".to_string());
                            object["applied_at"] = serde_json::Value::String(now.to_rfc3339());
                        }
                    }
                    "resolve_identity" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("canonical".to_string());
                            object["entity_class"] =
                                serde_json::Value::String("canonical".to_string());
                        }
                        let proposal_id_str = proposal_id.to_string();
                        for link in state.ontology.links.iter_mut().filter(|l| {
                            l.get("proposal_id").and_then(|v| v.as_str())
                                == Some(proposal_id_str.as_str())
                                && l.get("type").and_then(|v| v.as_str()) == Some("canonicalizes")
                        }) {
                            link["status"] = serde_json::Value::String("promoted".to_string());
                            link["evidence"] =
                                serde_json::Value::String("identity_resolved".to_string());
                        }
                    }
                    "switch_view_profile" => {
                        if let Some(active_id) = proposal.object_id.as_ref() {
                            for object in state.ontology.objects.iter_mut().filter(|o| {
                                o.get("object_type").and_then(|v| v.as_str())
                                    == Some("view_profile")
                            }) {
                                let is_active = object.get("id").and_then(|v| v.as_str())
                                    == Some(active_id.as_str());
                                object["status"] = serde_json::Value::String(
                                    if is_active { "active" } else { "inactive" }.to_string(),
                                );
                            }
                        }
                    }
                    "decompose_goal" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["decomposition_state"] =
                                serde_json::Value::String("decomposed".to_string());
                        }
                    }
                    "prioritize_work" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["priority_state"] =
                                serde_json::Value::String("prioritized".to_string());
                        }
                    }
                    "record_decision" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["decision_state"] =
                                serde_json::Value::String("recorded".to_string());
                        }
                    }
                    "register_constraint" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["constraint_state"] =
                                serde_json::Value::String("registered".to_string());
                        }
                    }
                    "identify_risk" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("candidate".to_string());
                            object["risk_state"] =
                                serde_json::Value::String("identified".to_string());
                        }
                    }
                    "mark_blocked" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("blocked".to_string());
                        }
                    }
                    "restore_progress" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["progress_state"] =
                                serde_json::Value::String("restored".to_string());
                        }
                    }
                    "verify_progress" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("verified".to_string());
                            object["progress_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    "refresh_working_set" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["membership_class"] =
                                serde_json::Value::String("deterministic".to_string());
                        }
                    }
                    "close_loop" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("completed".to_string());
                            object["completion_state"] =
                                serde_json::Value::String("closed".to_string());
                        }
                    }
                    "complete_task" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("completed".to_string());
                            object["completion_state"] =
                                serde_json::Value::String("closed".to_string());
                        }
                    }
                    "detect_affordances" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("candidate".to_string());
                            object["affordance_state"] =
                                serde_json::Value::String("detected".to_string());
                        }
                    }
                    "verify_permissions" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("verified".to_string());
                            object["permission_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    "verify_preconditions" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("verified".to_string());
                            object["precondition_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    "evaluate_dependencies" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["dependency_state"] =
                                serde_json::Value::String("evaluated".to_string());
                        }
                    }
                    "estimate_cost" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["estimation_state"] =
                                serde_json::Value::String("estimated".to_string());
                        }
                    }
                    "estimate_latency" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["estimation_state"] =
                                serde_json::Value::String("estimated".to_string());
                        }
                    }
                    "estimate_reliability" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["estimation_state"] =
                                serde_json::Value::String("estimated".to_string());
                        }
                    }
                    "estimate_reversibility" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["estimation_state"] =
                                serde_json::Value::String("estimated".to_string());
                        }
                    }
                    "choose_execution_path" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["execution_path_state"] =
                                serde_json::Value::String("selected".to_string());
                        }
                    }
                    "escalate_authority" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["authority_state"] =
                                serde_json::Value::String("escalated".to_string());
                        }
                    }
                    "mark_unavailable" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("blocked".to_string());
                            object["availability_state"] =
                                serde_json::Value::String("unavailable".to_string());
                        }
                    }
                    "determine_current_ask" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["ask_state"] =
                                serde_json::Value::String("determined".to_string());
                        }
                    }
                    "build_query_scope" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["scope_state"] = serde_json::Value::String("built".to_string());
                        }
                    }
                    "select_relevant_context" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["selection_state"] =
                                serde_json::Value::String("selected".to_string());
                        }
                    }
                    "exclude_irrelevant_context" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("stale".to_string());
                            object["selection_state"] =
                                serde_json::Value::String("pruned".to_string());
                        }
                    }
                    "verify_answer_scope" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("verified".to_string());
                            object["scope_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    "record_scope_failure" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("failed".to_string());
                            object["scope_state"] = serde_json::Value::String("failed".to_string());
                        }
                    }
                    "establish_identity" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["identity_state"] =
                                serde_json::Value::String("established".to_string());
                        }
                    }
                    "load_role_profile" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["role_state"] = serde_json::Value::String("loaded".to_string());
                        }
                    }
                    "verify_capability_profile" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("verified".to_string());
                            object["capability_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    "verify_permission_profile" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("verified".to_string());
                            object["permission_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    "assign_responsibility" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["responsibility_state"] =
                                serde_json::Value::String("assigned".to_string());
                        }
                    }
                    "determine_handoff_boundary" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["handoff_state"] =
                                serde_json::Value::String("bounded".to_string());
                        }
                    }
                    "restore_identity_continuity" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["continuity_state"] =
                                serde_json::Value::String("restored".to_string());
                        }
                    }
                    "form_intention" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["intention_state"] =
                                serde_json::Value::String("formed".to_string());
                        }
                    }
                    "promote_commitment" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["commitment_state"] =
                                serde_json::Value::String("promoted".to_string());
                        }
                    }
                    "apply_inhibition" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("blocked".to_string());
                            object["inhibition_state"] =
                                serde_json::Value::String("applied".to_string());
                        }
                    }
                    "evaluate_switch" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["switch_state"] =
                                serde_json::Value::String("evaluated".to_string());
                        }
                    }
                    "maintain_commitment" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["commitment_state"] =
                                serde_json::Value::String("maintained".to_string());
                        }
                    }
                    "authorize_abandonment" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("retired".to_string());
                            object["abandonment_state"] =
                                serde_json::Value::String("authorized".to_string());
                        }
                    }
                    "push_to_completion" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("completed".to_string());
                            object["completion_state"] =
                                serde_json::Value::String("pushed".to_string());
                        }
                    }
                    "record_goal_conflict" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("blocked".to_string());
                            object["conflict_state"] =
                                serde_json::Value::String("recorded".to_string());
                        }
                    }
                    "detect_aliases" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("candidate".to_string());
                            object["alias_state"] =
                                serde_json::Value::String("detected".to_string());
                        }
                    }
                    "build_resolution_candidates" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("candidate".to_string());
                            object["resolution_state"] =
                                serde_json::Value::String("candidates_built".to_string());
                        }
                    }
                    "verify_resolution" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("verified".to_string());
                            object["resolution_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    "build_projection" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["projection_state"] =
                                serde_json::Value::String("built".to_string());
                        }
                    }
                    "compress_projection" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["projection_state"] =
                                serde_json::Value::String("compressed".to_string());
                        }
                    }
                    "verify_projection_fidelity" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("verified".to_string());
                            object["projection_state"] =
                                serde_json::Value::String("fidelity_verified".to_string());
                        }
                    }
                    "evaluate_retention" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["retention_state"] =
                                serde_json::Value::String("evaluated".to_string());
                        }
                    }
                    "apply_decay" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("stale".to_string());
                            object["retention_state"] =
                                serde_json::Value::String("decayed".to_string());
                        }
                    }
                    "archive_object" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("retired".to_string());
                            object["archive_state"] =
                                serde_json::Value::String("archived".to_string());
                        }
                    }
                    "prune_active_context" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("stale".to_string());
                            object["context_state"] =
                                serde_json::Value::String("pruned".to_string());
                        }
                    }
                    "restore_from_archive" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["archive_state"] =
                                serde_json::Value::String("restored".to_string());
                        }
                    }
                    "record_supersession" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("superseded".to_string());
                            object["supersession_state"] =
                                serde_json::Value::String("recorded".to_string());
                        }
                    }
                    "create_version" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("experimental".to_string());
                            object["version_state"] =
                                serde_json::Value::String("created".to_string());
                        }
                    }
                    "declare_compatibility" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("declared".to_string());
                            object["compatibility_state"] =
                                serde_json::Value::String("declared".to_string());
                        }
                    }
                    "build_migration_plan" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("planned".to_string());
                            object["migration_state"] =
                                serde_json::Value::String("planned".to_string());
                        }
                    }
                    "deprecate_schema_element" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("retired".to_string());
                            object["lifecycle"] =
                                serde_json::Value::String("deprecated".to_string());
                        }
                    }
                    "review_governance_change" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("approved".to_string());
                            object["governance_state"] =
                                serde_json::Value::String("reviewed".to_string());
                        }
                    }
                    "verify_post_migration_conformance" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("verified".to_string());
                            object["conformance_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    _ => {}
                }
            }
            state.ontology.delta_log.push(OntologyDeltaRecord {
                workstream: workstream.clone(),
                delta_kind: "ontology_proposal_promoted".to_string(),
                payload: serde_json::json!({
                    "proposal_id": proposal_id,
                    "target_class": target_class,
                    "applied_kind": applied_kind,
                }),
                timestamp: Some(now),
            });
        }
        FocusaEvent::OntologyProposalRejected {
            workstream,
            proposal_id,
            target_class,
            reason,
        } => {
            let now = Utc::now();
            if let Some(proposal_idx) = state
                .ontology
                .proposals
                .iter()
                .position(|p| p.proposal_id == proposal_id)
            {
                let proposal = state.ontology.proposals[proposal_idx].clone();
                state.ontology.proposals[proposal_idx].status = "rejected".to_string();
                state.ontology.proposals[proposal_idx].notes = Some(reason.clone());
                state.ontology.proposals[proposal_idx].updated_at = Some(now);

                match proposal.proposal_kind.as_str() {
                    "object_upsert" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("rejected".to_string());
                            object["rejection_reason"] = serde_json::Value::String(reason.clone());
                        }
                    }
                    "link_upsert" => {
                        if let (Some(link_type), Some(source_id), Some(target_id)) = (
                            proposal.link_type.as_ref(),
                            proposal.source_id.as_ref(),
                            proposal.target_id.as_ref(),
                        ) && let Some(link) = state.ontology.links.iter_mut().find(|l| {
                            ontology_value_matches_workstream(l, &workstream)
                                && l.get("type").and_then(|v| v.as_str())
                                    == Some(link_type.as_str())
                                && l.get("source_id").and_then(|v| v.as_str())
                                    == Some(source_id.as_str())
                                && l.get("target_id").and_then(|v| v.as_str())
                                    == Some(target_id.as_str())
                        }) {
                            link["status"] = serde_json::Value::String("rejected".to_string());
                            link["rejection_reason"] = serde_json::Value::String(reason.clone());
                        }
                    }
                    "status_change" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("rejected".to_string());
                            object["rejection_reason"] = serde_json::Value::String(reason.clone());
                        }
                    }
                    "working_set_membership" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("rejected".to_string());
                            object["rejection_reason"] = serde_json::Value::String(reason.clone());
                        }
                    }
                    _ => {}
                }
            }
            state.ontology.delta_log.push(OntologyDeltaRecord {
                workstream: workstream.clone(),
                delta_kind: "ontology_proposal_rejected".to_string(),
                payload: serde_json::json!({
                    "proposal_id": proposal_id,
                    "target_class": target_class,
                    "reason": reason,
                }),
                timestamp: Some(now),
            });
        }
        FocusaEvent::OntologyVerificationApplied {
            workstream,
            proposal_id,
            verification,
            outcome,
        } => {
            let now = Utc::now();
            state
                .ontology
                .verifications
                .push(OntologyVerificationRecord {
                    workstream: workstream.clone(),
                    proposal_id,
                    verification: verification.clone(),
                    outcome: outcome.clone(),
                    timestamp: Some(now),
                });

            if let Some(pid) = proposal_id
                && let Some(proposal) = state
                    .ontology
                    .proposals
                    .iter()
                    .find(|p| p.proposal_id == pid && p.workstream == workstream)
                    .cloned()
            {
                let verified_status = if outcome_is_positive(&outcome) {
                    "verified"
                } else {
                    "failed"
                };

                match proposal.proposal_kind.as_str() {
                    "object_upsert" | "status_change" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String(verified_status.to_string());
                            object["verification"] =
                                serde_json::Value::String(verification.clone());
                        }
                    }
                    "link_upsert" => {
                        if let (Some(link_type), Some(source_id), Some(target_id)) = (
                            proposal.link_type.as_ref(),
                            proposal.source_id.as_ref(),
                            proposal.target_id.as_ref(),
                        ) && let Some(link) = state.ontology.links.iter_mut().find(|l| {
                            ontology_value_matches_workstream(l, &workstream)
                                && l.get("type").and_then(|v| v.as_str())
                                    == Some(link_type.as_str())
                                && l.get("source_id").and_then(|v| v.as_str())
                                    == Some(source_id.as_str())
                                && l.get("target_id").and_then(|v| v.as_str())
                                    == Some(target_id.as_str())
                        }) {
                            link["status"] = serde_json::Value::String(verified_status.to_string());
                            link["verification"] = serde_json::Value::String(verification.clone());
                        }
                    }
                    "working_set_membership" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                ontology_value_matches_workstream(o, &workstream)
                                    && o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String(verified_status.to_string());
                            object["verification"] =
                                serde_json::Value::String(verification.clone());
                        }
                    }
                    _ => {}
                }
            }

            state.ontology.delta_log.push(OntologyDeltaRecord {
                workstream: workstream.clone(),
                delta_kind: "ontology_verification_applied".to_string(),
                payload: serde_json::json!({
                    "proposal_id": proposal_id,
                    "verification": verification,
                    "outcome": outcome,
                }),
                timestamp: Some(now),
            });
        }
        FocusaEvent::OntologyWorkingSetRefreshed {
            workstream,
            scope,
            reason,
        } => {
            let now = Utc::now();
            state
                .ontology
                .working_set_refreshes
                .push(OntologyWorkingSetRefreshRecord {
                    workstream: workstream.clone(),
                    scope: scope.clone(),
                    reason: reason.clone(),
                    timestamp: Some(now),
                });
            let context_set_id = format!("relevant_context_set:{}:{}", scope, reason);
            if let Some(existing) = state.ontology.objects.iter_mut().find(|o| {
                o.get("id").and_then(|v| v.as_str()) == Some(context_set_id.as_str())
                    && ontology_value_matches_workstream(o, &workstream)
            }) {
                existing["status"] = serde_json::Value::String("active".to_string());
                existing["scope_kind"] = serde_json::Value::String(scope.clone());
                existing["reason"] = serde_json::Value::String(reason.clone());
                existing["provenance_class"] =
                    serde_json::Value::String("reducer_promoted".to_string());
            } else {
                state.ontology.objects.push(serde_json::json!({
                    "workstream": workstream,
                    "id": context_set_id,
                    "object_type": "relevant_context_set",
                    "selection_kind": scope.clone(),
                    "reason": reason.clone(),
                    "status": "active",
                    "membership_class": "deterministic",
                    "provenance_class": "reducer_promoted",
                }));
            }
            state.ontology.delta_log.push(OntologyDeltaRecord {
                workstream: workstream.clone(),
                delta_kind: "ontology_working_set_refreshed".to_string(),
                payload: serde_json::json!({
                    "scope": scope,
                    "reason": reason,
                }),
                timestamp: Some(now),
            });
        }

        // ─── Workpoint Continuity (Spec88) ──────────────────────────────
        FocusaEvent::OntologyScopeMigrationApplied {
            migration_id,
            target_workstream,
            selections,
            evidence_refs,
        } => {
            target_workstream.validate().map_err(|error| {
                ReducerError::InvalidEvent(format!("invalid migration target workstream: {error}"))
            })?;
            if selections.is_empty() || evidence_refs.is_empty() {
                return Err(ReducerError::InvalidEvent(
                    "ontology scope migration requires selections and migration evidence"
                        .to_string(),
                ));
            }
            if state
                .ontology
                .scope_migration_receipts
                .iter()
                .any(|receipt| receipt.migration_id == migration_id)
            {
                return Ok(ReductionResult {
                    new_state: state,
                    emitted_events: vec![emitted_event],
                });
            }
            let unique = selections
                .iter()
                .map(|selection| (selection.record_kind, selection.source_hash.as_str()))
                .collect::<std::collections::BTreeSet<_>>();
            if unique.len() != selections.len() {
                return Err(ReducerError::InvalidEvent(
                    "ontology scope migration contains duplicate selections".to_string(),
                ));
            }
            let mut entries = Vec::with_capacity(selections.len());
            for selection in &selections {
                entries.push(apply_ontology_scope_migration_selection(
                    &mut state,
                    &target_workstream,
                    selection,
                )?);
            }
            state
                .ontology
                .scope_migration_receipts
                .push(OntologyScopeMigrationReceipt {
                    receipt_id: migration_id,
                    migration_id,
                    operation: "apply".to_string(),
                    target_workstream,
                    entries,
                    evidence_refs,
                    recorded_at: Utc::now(),
                });
        }
        FocusaEvent::OntologyScopeMigrationRolledBack {
            rollback_id,
            migration_id,
            evidence_refs,
        } => {
            if evidence_refs.is_empty() {
                return Err(ReducerError::InvalidEvent(
                    "ontology scope migration rollback requires evidence".to_string(),
                ));
            }
            if state
                .ontology
                .scope_migration_receipts
                .iter()
                .any(|receipt| receipt.receipt_id == rollback_id)
            {
                return Ok(ReductionResult {
                    new_state: state,
                    emitted_events: vec![emitted_event],
                });
            }
            if state
                .ontology
                .scope_migration_receipts
                .iter()
                .any(|receipt| {
                    receipt.migration_id == migration_id && receipt.operation == "rollback"
                })
            {
                return Err(ReducerError::InvalidEvent(format!(
                    "ontology scope migration already rolled back: {migration_id}"
                )));
            }
            let applied = state
                .ontology
                .scope_migration_receipts
                .iter()
                .find(|receipt| {
                    receipt.migration_id == migration_id && receipt.operation == "apply"
                })
                .cloned()
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!(
                        "ontology scope migration apply receipt not found: {migration_id}"
                    ))
                })?;
            for entry in &applied.entries {
                rollback_ontology_scope_migration_entry(
                    &mut state,
                    &applied.target_workstream,
                    entry,
                )?;
            }
            state
                .ontology
                .scope_migration_receipts
                .push(OntologyScopeMigrationReceipt {
                    receipt_id: rollback_id,
                    migration_id,
                    operation: "rollback".to_string(),
                    target_workstream: applied.target_workstream,
                    entries: applied.entries,
                    evidence_refs,
                    recorded_at: Utc::now(),
                });
        }

        FocusaEvent::WorkpointCheckpointProposed { workpoint } => {
            let now = Utc::now();
            upsert_workpoint_record(&mut state, workpoint, now);
        }
        FocusaEvent::WorkpointCheckpointPromoted {
            workpoint_id,
            confidence,
            reason: _,
        } => {
            let now = Utc::now();
            let superseded_ids: Vec<_> = {
                let promoted = find_workpoint_mut(&mut state, workpoint_id)?;
                if promoted.status == WorkpointStatus::Rejected {
                    return Err(ReducerError::InvalidEvent(format!(
                        "Rejected workpoint {} cannot be promoted",
                        workpoint_id
                    )));
                }
                if promoted.status == WorkpointStatus::DegradedFallback || !promoted.canonical {
                    return Err(ReducerError::InvalidEvent(format!(
                        "Non-canonical degraded workpoint {} cannot be promoted silently",
                        workpoint_id
                    )));
                }
                promoted.status = WorkpointStatus::Active;
                promoted.confidence = confidence;
                promoted.updated_at = Some(now);
                let work_item_id = promoted.work_item_id.clone();
                let project_root = promoted.project_root.clone();
                let continuity_id = promoted.continuity_id.clone();
                let previous_active_id = state.workpoint.active_workpoint_id;
                state
                    .workpoint
                    .records
                    .iter()
                    .filter(|w| {
                        if w.workpoint_id == workpoint_id || w.status != WorkpointStatus::Active {
                            return false;
                        }
                        if let Some(ref continuity_id) = continuity_id {
                            return w.continuity_id.as_ref() == Some(continuity_id)
                                && (project_root.is_none() || w.project_root == project_root);
                        }
                        if let Some(ref work_item_id) = work_item_id {
                            return w.work_item_id.as_ref() == Some(work_item_id)
                                && (project_root.is_none() || w.project_root == project_root);
                        }
                        Some(w.workpoint_id) == previous_active_id
                    })
                    .map(|w| w.workpoint_id)
                    .collect()
            };
            for old_id in superseded_ids {
                if let Some(old) = state
                    .workpoint
                    .records
                    .iter_mut()
                    .find(|w| w.workpoint_id == old_id)
                {
                    old.status = WorkpointStatus::Superseded;
                    old.updated_at = Some(now);
                }
            }
            state.workpoint.active_workpoint_id = Some(workpoint_id);
            let promoted_scope = state
                .workpoint
                .records
                .iter()
                .find(|record| record.workpoint_id == workpoint_id)
                .map(|record| (record.project_root.clone(), record.continuity_id.clone()));
            if let Some((Some(project_root), Some(continuity_id))) = promoted_scope {
                let active_trajectory_id = state.trajectory.active_trajectory_id.as_deref();
                let exact_scope = |trajectory: &TrajectoryProjectionRecord| {
                    trajectory.project_root.as_deref() == Some(project_root.as_str())
                        && trajectory.continuity_id.as_deref() == Some(continuity_id.as_str())
                };
                let trajectory_index = active_trajectory_id
                    .and_then(|active_id| {
                        state.trajectory.records.iter().position(|trajectory| {
                            trajectory.trajectory_id == active_id && exact_scope(trajectory)
                        })
                    })
                    .or_else(|| state.trajectory.records.iter().rposition(exact_scope));
                if let Some(index) = trajectory_index {
                    state.trajectory.records[index].active_workpoint_id = Some(workpoint_id);
                    state.trajectory.records[index].updated_at = Some(now);
                }
            }
        }
        FocusaEvent::WorkpointCheckpointRejected {
            workpoint_id,
            reason,
        } => {
            let now = Utc::now();
            let record = find_workpoint_mut(&mut state, workpoint_id)?;
            record.status = WorkpointStatus::Rejected;
            record.rejection_reason = Some(reason);
            record.updated_at = Some(now);
            if state.workpoint.active_workpoint_id == Some(workpoint_id) {
                state.workpoint.active_workpoint_id = None;
            }
        }
        FocusaEvent::WorkpointSuperseded {
            old_workpoint_id,
            new_workpoint_id,
            reason: _,
        } => {
            let now = Utc::now();
            if !state
                .workpoint
                .records
                .iter()
                .any(|w| w.workpoint_id == new_workpoint_id)
            {
                return Err(ReducerError::InvalidEvent(format!(
                    "New workpoint {} not found",
                    new_workpoint_id
                )));
            }
            let old = find_workpoint_mut(&mut state, old_workpoint_id)?;
            old.status = WorkpointStatus::Superseded;
            old.updated_at = Some(now);
            state.workpoint.active_workpoint_id = Some(new_workpoint_id);
        }
        FocusaEvent::WorkpointResumeRendered {
            workpoint_id,
            mode,
            rendered_summary,
        } => {
            state
                .workpoint
                .resume_events
                .push(WorkpointResumeRenderRecord {
                    workpoint_id,
                    mode,
                    rendered_summary,
                    rendered_at: Some(Utc::now()),
                });
            truncate_front(
                &mut state.workpoint.resume_events,
                workpoint_caps::RESUME_EVENTS,
            );
        }
        FocusaEvent::WorkpointDriftDetected {
            workpoint_id,
            severity,
            reason,
            recovery_hint,
        } => {
            state.workpoint.drift_events.push(WorkpointDriftRecord {
                workpoint_id,
                severity,
                reason,
                recovery_hint,
                detected_at: Some(Utc::now()),
            });
            truncate_front(
                &mut state.workpoint.drift_events,
                workpoint_caps::DRIFT_EVENTS,
            );
        }
        FocusaEvent::WorkpointEvidenceLinked {
            workpoint_id,
            mut verification,
        } => {
            let now = Utc::now();
            let record = find_workpoint_mut(&mut state, workpoint_id)?;
            if !record.canonical || record.status == WorkpointStatus::DegradedFallback {
                return Err(ReducerError::InvalidEvent(format!(
                    "Cannot link canonical evidence to non-canonical workpoint {}",
                    workpoint_id
                )));
            }
            if verification.verified_at.is_none() {
                verification.verified_at = Some(now);
            }
            record.verification_records.push(verification);
            truncate_front(
                &mut record.verification_records,
                workpoint_caps::VERIFICATIONS,
            );
            record.updated_at = Some(now);
        }
        FocusaEvent::WorkpointDegradedFallbackRecorded {
            workpoint_id,
            reason,
            packet,
        } => {
            let now = Utc::now();
            let record = WorkpointRecord {
                workpoint_id,
                status: WorkpointStatus::DegradedFallback,
                checkpoint_reason: WorkpointCheckpointReason::ContextOverflow,
                canonical: false,
                created_at: Some(now),
                updated_at: Some(now),
                ..WorkpointRecord::default()
            };
            upsert_workpoint_record(&mut state, record, now);
            state
                .workpoint
                .degraded_fallbacks
                .push(WorkpointDegradedFallbackRecord {
                    workpoint_id,
                    reason,
                    packet,
                    recorded_at: Some(now),
                });
            truncate_front(
                &mut state.workpoint.degraded_fallbacks,
                workpoint_caps::DEGRADED_FALLBACKS,
            );
        }
        FocusaEvent::OntologyActionIntentBound {
            workpoint_id,
            mut action_intent,
        } => {
            truncate_front(
                &mut action_intent.verification_hooks,
                workpoint_caps::VERIFICATIONS,
            );
            let record = find_workpoint_mut(&mut state, workpoint_id)?;
            record.action_intent = Some(action_intent);
            record.updated_at = Some(Utc::now());
        }
        FocusaEvent::OntologyVerificationLinked {
            workpoint_id,
            verification,
        } => {
            let record = find_workpoint_mut(&mut state, workpoint_id)?;
            record.verification_records.push(verification);
            truncate_front(
                &mut record.verification_records,
                workpoint_caps::VERIFICATIONS,
            );
            record.updated_at = Some(Utc::now());
        }

        // ─── Trajectory Projection (Spec96) ─────────────────────────────
        FocusaEvent::TrajectoryGoalDefined { trajectory } => {
            if trajectory.canonical
                && (trajectory.trajectory_id.trim().is_empty()
                    || trajectory.long_term_goal.trim().is_empty()
                    || trajectory.desired_end_state.trim().is_empty())
            {
                return Err(ReducerError::InvalidEvent(
                    "Canonical trajectory requires id, long_term_goal, and desired_end_state"
                        .to_string(),
                ));
            }
            let now = Utc::now();
            let new_id = trajectory.trajectory_id.clone();
            let supersedes = trajectory.supersedes_trajectory_id.clone();
            let superseded_ids = state
                .trajectory
                .records
                .iter()
                .filter(|existing| {
                    existing.trajectory_id != new_id
                        && existing.canonical
                        && (supersedes.as_deref() == Some(existing.trajectory_id.as_str())
                            || same_trajectory_authority_scope(existing, &trajectory))
                })
                .map(|existing| existing.trajectory_id.clone())
                .collect::<Vec<_>>();
            for old_id in superseded_ids {
                if let Some(existing) = state
                    .trajectory
                    .records
                    .iter_mut()
                    .find(|item| item.trajectory_id == old_id)
                {
                    existing.canonical = false;
                    existing.root_goal_stability = TrajectoryRootGoalStability::Superseded;
                    existing.updated_at = Some(now);
                }
            }
            let canonical = trajectory.canonical;
            upsert_trajectory_record(&mut state, trajectory, now);
            if canonical {
                state.trajectory.active_trajectory_id = Some(new_id);
            }
        }
        FocusaEvent::TrajectoryCheckpointPersisted {
            trajectory_id,
            checkpoint,
            summary,
        } => {
            let now = Utc::now();
            state
                .trajectory
                .checkpoints
                .push(TrajectoryCheckpointRecord {
                    trajectory_id,
                    summary,
                    packet: checkpoint,
                    persisted_at: Some(now),
                });
            truncate_front(
                &mut state.trajectory.checkpoints,
                trajectory_caps::CHECKPOINTS,
            );
        }
        FocusaEvent::TrajectoryStateDeltaRecorded {
            trajectory_id,
            current_state,
            mut evidence_refs,
            reason,
        } => {
            truncate_front(&mut evidence_refs, trajectory_caps::EVIDENCE_REFS);
            let now = Utc::now();
            if let Some(record) = state
                .trajectory
                .records
                .iter_mut()
                .find(|item| item.trajectory_id == trajectory_id)
            {
                if let Some(current_state) = current_state.clone() {
                    record.current_state = Some(current_state);
                }
                if let Some(dod) = &mut record.definition_of_done {
                    dod.verified_evidence_refs.extend(evidence_refs.clone());
                    truncate_front(
                        &mut dod.verified_evidence_refs,
                        trajectory_caps::EVIDENCE_REFS,
                    );
                }
                record.updated_at = Some(now);
            }
            state
                .trajectory
                .state_deltas
                .push(TrajectoryStateDeltaRecord {
                    trajectory_id,
                    current_state,
                    evidence_refs,
                    reason,
                    recorded_at: Some(now),
                });
            truncate_front(
                &mut state.trajectory.state_deltas,
                trajectory_caps::STATE_DELTAS,
            );
        }

        // ─── Errors ──────────────────────────────────────────────────────
        FocusaEvent::InvariantViolation {
            invariant: _,
            details: _,
        } => {
            // Log-only event — no state mutation.
            // The event itself is recorded in the event log via emitted_events.
        }

        // ─── Thread Ownership ────────────────────────────────────────────
        FocusaEvent::ThreadOwnershipTransferred {
            thread_id,
            from_machine_id,
            to_machine_id,
            reason: _,
        } => {
            // Validate that from_machine_id matches current owner (if specified).
            // This prevents unauthorized ownership transfers.
            let thread = state.threads.iter().find(|t| t.id == thread_id);

            // If thread doesn't exist, reject the transfer.
            // Ownership transfers require the thread to exist so we can verify ownership
            // and apply the ownership change atomically.
            let thread = match thread {
                Some(t) => t,
                None => {
                    return Err(ReducerError::InvalidEvent(format!(
                        "Thread {} not found — cannot transfer ownership of non-existent thread",
                        thread_id
                    )));
                }
            };

            if let Some(from_id) = &from_machine_id {
                if let Some(current_owner) = &thread.owner_machine_id {
                    if current_owner != from_id {
                        return Err(ReducerError::OwnershipViolation {
                            thread_id,
                            owner: current_owner.clone(),
                            attempted_by: Some(from_id.clone()),
                        });
                    }
                } else {
                    // Thread has no owner but from_machine_id is specified — reject.
                    // This prevents claiming a thread's ownership when you never owned it.
                    return Err(ReducerError::InvalidEvent(format!(
                        "Thread {} has no owner but transfer specifies from_machine_id '{}'",
                        thread_id, from_id
                    )));
                }
            }

            // Update owner_machine_id on the thread.
            if let Some(thread) = state.threads.iter_mut().find(|t| t.id == thread_id) {
                thread.owner_machine_id = Some(to_machine_id);
                thread.updated_at = Utc::now();
            }
        }

        FocusaEvent::ThreadCreated {
            thread_id,
            name,
            primary_intent,
            owner_machine_id,
        } => {
            // Reject duplicate thread IDs.
            if state.threads.iter().any(|t| t.id == thread_id) {
                return Err(ReducerError::InvalidEvent(format!(
                    "Thread {} already exists",
                    thread_id
                )));
            }
            // Create thread record using the same structure as threads::create_thread.
            use crate::types::{Thread, ThreadStatus, ThreadThesis};
            state.threads.push(Thread {
                id: thread_id,
                name,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                status: ThreadStatus::Active,
                thesis: ThreadThesis {
                    primary_intent,
                    updated_at: Some(Utc::now()),
                    ..Default::default()
                },
                clt_head: None,
                autonomy_history: vec![],
                owner_machine_id,
            });
        }

        FocusaEvent::ThreadForked {
            source_thread_id,
            thread_id,
            name,
            owner_machine_id,
        } => {
            if state.threads.iter().any(|t| t.id == thread_id) {
                return Err(ReducerError::InvalidEvent(format!(
                    "Thread {} already exists",
                    thread_id
                )));
            }
            let source = state
                .threads
                .iter()
                .find(|t| t.id == source_thread_id)
                .cloned()
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!(
                        "Source thread {} not found for fork",
                        source_thread_id
                    ))
                })?;
            let mut forked =
                crate::threads::fork_thread(&source, &name, owner_machine_id.as_deref());
            forked.id = thread_id;
            let branch_marker = crate::clt::insert_branch_marker(
                &mut state.clt,
                "thread_fork",
                vec![source.id.to_string(), forked.id.to_string()],
            );
            forked.clt_head = Some(branch_marker);
            state.threads.push(forked);
        }

        FocusaEvent::ThreadThesisUpdated { thread_id, thesis } => {
            let thread = state
                .threads
                .iter_mut()
                .find(|t| t.id == thread_id)
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!(
                        "Thread {} not found for thesis update",
                        thread_id
                    ))
                })?;
            thread.thesis = thesis;
            thread.updated_at = Utc::now();
        }

        FocusaEvent::ProposalSubmitted {
            workstream,
            proposal_id,
            kind,
            source,
            payload,
            deadline_ms,
            score,
        } => {
            let now = Utc::now();
            let deadline = now + chrono::Duration::milliseconds(deadline_ms as i64);
            state.pre.proposals.push(crate::types::Proposal {
                workstream,
                id: proposal_id,
                kind,
                source,
                created_at: now,
                deadline,
                payload,
                score: score.unwrap_or(0.0).clamp(0.0, 1.0),
                status: crate::types::ProposalStatus::Pending,
            });
        }

        FocusaEvent::ProposalStatusChanged {
            workstream,
            proposal_id,
            status,
        } => {
            let proposal = state
                .pre
                .proposals
                .iter_mut()
                .find(|p| p.id == proposal_id && p.workstream == workstream)
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!(
                        "Proposal {} not found in workstream for status update",
                        proposal_id
                    ))
                })?;
            proposal.status = status;
        }

        FocusaEvent::ConstitutionLoaded {
            version,
            agent_id,
            principles,
            safety_rules,
            expression_rules,
        } => {
            crate::constitution::create_version(
                &mut state.constitution,
                &agent_id,
                &version,
                principles,
                safety_rules,
                expression_rules,
            );
            crate::constitution::activate_version(&mut state.constitution, &version)
                .map_err(ReducerError::InvalidEvent)?;
        }
    }

    state.version += 1;

    check_invariants(&state)?;

    Ok(ReductionResult {
        new_state: state,
        emitted_events: vec![emitted_event],
    })
}

/// Verify all 7 global invariants hold on the given state.
pub fn check_invariants(state: &FocusaState) -> Result<(), ReducerError> {
    // INVARIANT 1: At most one active Focus Frame exists per logical scope.
    // `active_id` is the most recently touched active frame, not the only active frame globally.
    let active_frames: Vec<&FrameRecord> = state
        .focus_stack
        .frames
        .iter()
        .filter(|f| f.status == FrameStatus::Active)
        .collect();
    let mut active_scopes: std::collections::HashSet<(Option<String>, Option<String>)> =
        std::collections::HashSet::new();
    for frame in &active_frames {
        let key = (frame.project_root.clone(), frame.continuity_id.clone());
        if !active_scopes.insert(key.clone()) {
            return Err(ReducerError::InvariantViolation(format!(
                "Multiple active Focus Frames for scope {:?}",
                key
            )));
        }
    }
    match state.focus_stack.active_id {
        Some(aid) => match state.focus_stack.frames.iter().find(|f| f.id == aid) {
            None => {
                return Err(ReducerError::InvariantViolation(format!(
                    "active_id {} points to nonexistent frame",
                    aid
                )));
            }
            Some(f) if f.status != FrameStatus::Active => {
                return Err(ReducerError::InvariantViolation(format!(
                    "active_id {} points to frame with status {:?}, expected Active",
                    aid, f.status
                )));
            }
            _ => {}
        },
        None => {
            if !active_frames.is_empty() {
                return Err(ReducerError::InvariantViolation(format!(
                    "active_id is None but {} frame(s) have Active status",
                    active_frames.len()
                )));
            }
        }
    }

    // INVARIANT 2: Every Focus Frame maps to a Beads issue.
    for frame in &state.focus_stack.frames {
        if frame.beads_issue_id.is_empty() {
            return Err(ReducerError::InvariantViolation(format!(
                "Frame {} has no Beads issue linkage",
                frame.id
            )));
        }
    }

    // INVARIANT 3: Focus State sections always exist.
    // Structurally guaranteed — FocusState derives Default and all fields have defaults.
    // No runtime check needed.

    // INVARIANT 4: Intuition Engine cannot mutate focus.
    // Structurally guaranteed — IntuitionSignalObserved only touches focus_gate,
    // never focus_stack. Enforced by the match arms above.

    // INVARIANT 5: Focus Gate is advisory only.
    // Structurally guaranteed — CandidateSurfaced/Pinned/Suppressed only touch
    // focus_gate.candidates, never focus_stack.

    // INVARIANT 6: Artifacts are immutable once registered.
    // Enforced at registration time: ArtifactRegistered rejects duplicate IDs.
    // No handles in reference_index share the same ID.
    let handle_count = state.reference_index.handles.len();
    let unique_count = {
        let mut ids: Vec<_> = state.reference_index.handles.iter().map(|h| h.id).collect();
        ids.sort();
        ids.dedup();
        ids.len()
    };
    if handle_count != unique_count {
        return Err(ReducerError::InvariantViolation(format!(
            "Duplicate artifact IDs in reference_index: {} handles but {} unique",
            handle_count, unique_count
        )));
    }

    // INVARIANT 7: Conversation never mutates cognition.
    // Structurally guaranteed — FocusaState has no conversation/chat history field.
    // No event type carries raw conversation data.

    Ok(())
}

/// Errors from the reducer.
#[derive(Debug, thiserror::Error)]
pub enum ReducerError {
    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Invalid event for current state: {0}")]
    InvalidEvent(String),

    #[error("Frame not found: {0}")]
    FrameNotFound(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error(
        "Ownership violation: thread {thread_id} owned by {owner}, attempted by {attempted_by:?}"
    )]
    OwnershipViolation {
        thread_id: Uuid,
        owner: String,
        attempted_by: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn fresh_state() -> FocusaState {
        FocusaState::default()
    }

    fn start_session(state: FocusaState) -> FocusaState {
        let event = FocusaEvent::SessionStarted {
            session_id: Uuid::now_v7(),
            adapter_id: None,
            workspace_id: None,
            project_root: Some("/repo/test".to_string()),
            continuity_id: Some("cont-test".to_string()),
        };
        reduce(state, event).unwrap().new_state
    }

    #[test]
    fn context_source_commit_is_canonical_scoped_and_idempotency_guarded() {
        let committed_at = Utc::now();
        let source = ContextSourceRecord {
            source_id: "context-source:test".to_string(),
            project_root: "/repo/test".to_string(),
            continuity_id: "cont-test".to_string(),
            attachment_id: "attachment:test".to_string(),
            source_kind: "markdown".to_string(),
            title: "Project context".to_string(),
            content: "# Context".to_string(),
            content_hash: "abc123".to_string(),
            idempotency_key: "idem-test".to_string(),
            revision: 1,
            committed_at,
            evidence: ContextSourceEvidence {
                evidence_ref: "evidence:context-source:test".to_string(),
                target_ref: "context-source:test".to_string(),
                result: "committed".to_string(),
                content_hash: "abc123".to_string(),
                captured_at: committed_at,
            },
            receipt: ContextSourceReceipt {
                receipt_ref: "receipt:context-source:test".to_string(),
                operation_id: "focusa.context.source.commit".to_string(),
                idempotency_key: "idem-test".to_string(),
                before_state_version: 0,
                after_state_version: 1,
                reversible: true,
                committed_at,
            },
            source_locator: String::new(),
            source_revision: String::new(),
            mime_type: String::new(),
            adapter_id: "focusa.context.commit".to_string(),
            ingestion_status: "committed".to_string(),
            extraction_diagnostics: Vec::new(),
            health: Default::default(),
        };
        let event = FocusaEvent::ContextSourceCommitted {
            source: source.clone(),
        };
        let reduced = reduce(fresh_state(), event.clone()).expect("Context commit reduces");
        assert_eq!(reduced.new_state.version, 1);
        assert_eq!(reduced.new_state.context_sources, vec![source]);
        assert!(matches!(
            reduce(reduced.new_state, event),
            Err(ReducerError::InvalidEvent(message)) if message.contains("version mismatch")
        ));
    }

    fn push_frame(state: FocusaState, title: &str) -> (FocusaState, FrameId) {
        let frame_id = Uuid::now_v7();
        let event = FocusaEvent::FocusFramePushed {
            frame_id,
            beads_issue_id: "BEAD-001".into(),
            title: title.into(),
            goal: format!("Goal for {}", title),
            project_root: Some("/repo/test".to_string()),
            continuity_id: Some("cont-test".to_string()),
            constraints: vec![],
            tags: vec!["continuity_id:cont-test".to_string()],
        };
        let state = reduce(state, event).unwrap().new_state;
        (state, frame_id)
    }

    fn temporal_context(
        project_root: &str,
        continuity_id: &str,
        projected_at: chrono::DateTime<Utc>,
    ) -> TemporalFrameContext {
        TemporalFrameContext {
            projection: crate::temporal::TemporalProjection {
                scope: crate::temporal::TemporalScope {
                    project_root: project_root.to_string(),
                    continuity_id: continuity_id.to_string(),
                    host_id: None,
                    operator_id: None,
                    workpoint_id: None,
                    task_id: None,
                    item_id: None,
                },
                as_of: projected_at,
                deadline_status: crate::temporal::DeadlineStatus::None,
                approaching_deadlines: vec![],
                deadline_conflict_state: "none".to_string(),
                human_calendar_context: None,
                temporal_priority_frame: None,
                temporal_execution_guard: None,
                authorized_forecast_range: None,
                latest_forecast_evaluation: None,
                active_commitment: None,
                active_forecast: None,
                observed_duration_count: 0,
                critical_path_ms: None,
                slack_ms: None,
                urgency: None,
                warnings: vec![],
            },
            source_event_count: 0,
            projected_at,
        }
    }

    fn workpoint_record(work_item_id: &str) -> WorkpointRecord {
        WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            work_item_id: Some(work_item_id.to_string()),
            project_root: Some("/repo/test".to_string()),
            continuity_id: Some("cont-test".to_string()),
            session_id: Some("pi-session".to_string()),
            status: WorkpointStatus::Proposed,
            checkpoint_reason: WorkpointCheckpointReason::Manual,
            confidence: WorkpointConfidence::High,
            canonical: true,
            mission: Some("Preserve typed continuation".to_string()),
            next_slice: Some("Implement reducer-owned workpoint state".to_string()),
            action_intent: Some(WorkpointActionIntentRecord {
                action_type: "checkpoint_workpoint".to_string(),
                target_ref: Some(work_item_id.to_string()),
                verification_hooks: vec![
                    "cargo test -p focusa-core reducer::tests::test_workpoint".to_string(),
                ],
                status: Some("ready".to_string()),
            }),
            ..WorkpointRecord::default()
        }
    }

    // ─── Workpoint continuity (Spec88) ────────────────────────────────

    #[test]
    fn test_workpoint_defaults_are_empty_and_noncanonical() {
        let state = fresh_state();
        assert!(state.workpoint.active_workpoint_id.is_none());
        assert!(state.workpoint.records.is_empty());
        let record = WorkpointRecord::default();
        assert_eq!(record.status, WorkpointStatus::Proposed);
        assert!(!record.canonical);
    }

    #[test]
    fn test_workpoint_promote_sets_active_pointer() {
        let mut state = fresh_state();
        let trajectory = TrajectoryProjectionRecord {
            trajectory_id: "trajectory-test".to_string(),
            project_root: Some("/repo/test".to_string()),
            continuity_id: Some("cont-test".to_string()),
            ..TrajectoryProjectionRecord::default()
        };
        state.trajectory.active_trajectory_id = Some("stale-trajectory-id".to_string());
        state.trajectory.records.push(trajectory);
        let record = workpoint_record("focusa-a2w2.2");
        let workpoint_id = record.workpoint_id;
        let state = reduce(
            state,
            FocusaEvent::WorkpointCheckpointProposed { workpoint: record },
        )
        .unwrap()
        .new_state;
        assert!(state.workpoint.active_workpoint_id.is_none());

        let state = reduce(
            state,
            FocusaEvent::WorkpointCheckpointPromoted {
                workpoint_id,
                confidence: WorkpointConfidence::Verified,
                reason: "phase1 accepted".to_string(),
            },
        )
        .unwrap()
        .new_state;

        assert_eq!(state.workpoint.active_workpoint_id, Some(workpoint_id));
        let active = state
            .workpoint
            .records
            .iter()
            .find(|w| w.workpoint_id == workpoint_id)
            .unwrap();
        assert_eq!(active.status, WorkpointStatus::Active);
        assert_eq!(active.confidence, WorkpointConfidence::Verified);
        assert_eq!(state.trajectory.records[0].active_workpoint_id, Some(workpoint_id));
    }

    #[test]
    fn test_workpoint_promote_supersedes_same_scope_active_record() {
        let state = fresh_state();
        let first = workpoint_record("focusa-a2w2.2");
        let first_id = first.workpoint_id;
        let second = workpoint_record("focusa-a2w2.2");
        let second_id = second.workpoint_id;

        let state = reduce(
            state,
            FocusaEvent::WorkpointCheckpointProposed { workpoint: first },
        )
        .unwrap()
        .new_state;
        let state = reduce(
            state,
            FocusaEvent::WorkpointCheckpointPromoted {
                workpoint_id: first_id,
                confidence: WorkpointConfidence::High,
                reason: "first active".to_string(),
            },
        )
        .unwrap()
        .new_state;
        let state = reduce(
            state,
            FocusaEvent::WorkpointCheckpointProposed { workpoint: second },
        )
        .unwrap()
        .new_state;
        let state = reduce(
            state,
            FocusaEvent::WorkpointCheckpointPromoted {
                workpoint_id: second_id,
                confidence: WorkpointConfidence::Verified,
                reason: "newer checkpoint".to_string(),
            },
        )
        .unwrap()
        .new_state;

        assert_eq!(state.workpoint.active_workpoint_id, Some(second_id));
        let first = state
            .workpoint
            .records
            .iter()
            .find(|w| w.workpoint_id == first_id)
            .unwrap();
        assert_eq!(first.status, WorkpointStatus::Superseded);
    }

    #[test]
    fn test_workpoint_reject_and_degraded_fallback_cannot_promote() {
        let state = fresh_state();
        let record = workpoint_record("focusa-a2w2.2");
        let workpoint_id = record.workpoint_id;
        let state = reduce(
            state,
            FocusaEvent::WorkpointCheckpointProposed { workpoint: record },
        )
        .unwrap()
        .new_state;
        let state = reduce(
            state,
            FocusaEvent::WorkpointCheckpointRejected {
                workpoint_id,
                reason: "missing ontology refs".to_string(),
            },
        )
        .unwrap()
        .new_state;
        assert!(
            reduce(
                state,
                FocusaEvent::WorkpointCheckpointPromoted {
                    workpoint_id,
                    confidence: WorkpointConfidence::High,
                    reason: "should fail".to_string(),
                },
            )
            .is_err()
        );

        let fallback_id = Uuid::now_v7();
        let state = reduce(
            fresh_state(),
            FocusaEvent::WorkpointDegradedFallbackRecorded {
                workpoint_id: fallback_id,
                reason: "Focusa unavailable".to_string(),
                packet: serde_json::json!({ "next": "resume from local packet" }),
            },
        )
        .unwrap()
        .new_state;
        assert!(
            reduce(
                state,
                FocusaEvent::WorkpointCheckpointPromoted {
                    workpoint_id: fallback_id,
                    confidence: WorkpointConfidence::High,
                    reason: "should fail".to_string(),
                },
            )
            .is_err()
        );
    }

    #[test]
    fn test_workpoint_bounds_vectors_at_reducer_boundary() {
        let state = fresh_state();
        let mut record = workpoint_record("focusa-a2w2.2");
        let workpoint_id = record.workpoint_id;
        record.active_object_refs = (0..64).map(|i| format!("object-{i}")).collect();
        record.verification_records = (0..64)
            .map(|i| WorkpointVerificationRecord {
                target_ref: format!("target-{i}"),
                result: "passed".to_string(),
                evidence_ref: None,
                verified_at: None,
            })
            .collect();
        record.blockers = (0..64)
            .map(|i| WorkpointBlockerRecord {
                reason: format!("blocker-{i}"),
                severity: Some("low".to_string()),
                target_ref: None,
                status: Some("open".to_string()),
            })
            .collect();

        let state = reduce(
            state,
            FocusaEvent::WorkpointCheckpointProposed { workpoint: record },
        )
        .unwrap()
        .new_state;
        let stored = state
            .workpoint
            .records
            .iter()
            .find(|w| w.workpoint_id == workpoint_id)
            .unwrap();
        assert_eq!(stored.active_object_refs.len(), workpoint_caps::OBJECT_REFS);
        assert_eq!(
            stored.verification_records.len(),
            workpoint_caps::VERIFICATIONS
        );
        assert_eq!(stored.blockers.len(), workpoint_caps::BLOCKERS);
    }

    // ─── Session lifecycle ───────────────────────────────────────────

    #[test]
    fn test_session_start_fresh() {
        let state = fresh_state();
        let state = start_session(state);
        assert!(state.session.is_some());
        assert_eq!(
            state.session.as_ref().unwrap().status,
            SessionStatus::Active
        );
        assert_eq!(state.version, 1);
    }

    #[test]
    fn test_session_start_rejects_active() {
        let state = start_session(fresh_state());
        let event = FocusaEvent::SessionStarted {
            session_id: Uuid::now_v7(),
            adapter_id: None,
            workspace_id: None,
            project_root: Some("/repo/test".to_string()),
            continuity_id: Some("cont-test-2".to_string()),
        };
        let result = reduce(state, event);
        assert!(result.is_err());
    }

    #[test]
    fn test_session_close_and_restart() {
        let state = start_session(fresh_state());
        // Close
        let event = FocusaEvent::SessionClosed {
            reason: "done".into(),
        };
        let state = reduce(state, event).unwrap().new_state;
        assert_eq!(
            state.session.as_ref().unwrap().status,
            SessionStatus::Closed
        );
        // Restart — should succeed (not reject closed session)
        let state = start_session(state);
        assert_eq!(
            state.session.as_ref().unwrap().status,
            SessionStatus::Active
        );
    }

    #[test]
    fn test_session_close_without_session_errors() {
        let result = reduce(
            fresh_state(),
            FocusaEvent::SessionClosed {
                reason: "test".into(),
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_session_double_close_errors() {
        let state = start_session(fresh_state());
        let state = reduce(
            state,
            FocusaEvent::SessionClosed {
                reason: "first".into(),
            },
        )
        .unwrap()
        .new_state;
        let result = reduce(
            state,
            FocusaEvent::SessionClosed {
                reason: "second".into(),
            },
        );
        assert!(result.is_err());
    }

    // ─── Focus Stack ─────────────────────────────────────────────────

    #[test]
    fn test_push_frame() {
        let state = fresh_state();
        let (state, frame_id) = push_frame(state, "Task A");
        assert_eq!(state.focus_stack.active_id, Some(frame_id));
        assert_eq!(state.focus_stack.frames.len(), 1);
        assert_eq!(state.focus_stack.frames[0].status, FrameStatus::Active);
        assert_eq!(state.focus_stack.root_id, Some(frame_id));
    }

    #[test]
    fn test_push_child_pauses_parent() {
        let state = fresh_state();
        let (state, parent_id) = push_frame(state, "Parent");
        let (state, child_id) = push_frame(state, "Child");

        assert_eq!(state.focus_stack.active_id, Some(child_id));
        let parent = state
            .focus_stack
            .frames
            .iter()
            .find(|f| f.id == parent_id)
            .unwrap();
        assert_eq!(parent.status, FrameStatus::Paused);
        let child = state
            .focus_stack
            .frames
            .iter()
            .find(|f| f.id == child_id)
            .unwrap();
        assert_eq!(child.status, FrameStatus::Active);
    }

    #[test]
    fn test_pop_frame_restores_parent() {
        let state = fresh_state();
        let (state, parent_id) = push_frame(state, "Parent");
        let (state, child_id) = push_frame(state, "Child");

        let event = FocusaEvent::FocusFrameCompleted {
            frame_id: child_id,
            completion_reason: CompletionReason::GoalAchieved,
        };
        let state = reduce(state, event).unwrap().new_state;

        assert_eq!(state.focus_stack.active_id, Some(parent_id));
        let parent = state
            .focus_stack
            .frames
            .iter()
            .find(|f| f.id == parent_id)
            .unwrap();
        assert_eq!(parent.status, FrameStatus::Active);
        let child = state
            .focus_stack
            .frames
            .iter()
            .find(|f| f.id == child_id)
            .unwrap();
        assert_eq!(child.status, FrameStatus::Completed);
    }

    #[test]
    fn test_pop_root_clears_stack() {
        let state = fresh_state();
        let (state, root_id) = push_frame(state, "Root");

        let event = FocusaEvent::FocusFrameCompleted {
            frame_id: root_id,
            completion_reason: CompletionReason::GoalAchieved,
        };
        let result = reduce(state, event);

        assert!(result.is_err());
    }

    #[test]
    fn temporal_context_projection_is_scoped_and_replayable() {
        let (state, frame_id) = push_frame(fresh_state(), "Temporal frame");
        let projected_at = Utc::now();
        let event = FocusaEvent::TemporalFrameContextProjected {
            frame_id,
            context: temporal_context("/repo/test", "cont-test", projected_at),
        };
        let first = reduce(state.clone(), event.clone()).unwrap().new_state;
        let replay = reduce(state, event).unwrap().new_state;
        assert_eq!(
            serde_json::to_value(&first.focus_stack).unwrap(),
            serde_json::to_value(&replay.focus_stack).unwrap()
        );
        let frame = first
            .focus_stack
            .frames
            .iter()
            .find(|frame| frame.id == frame_id)
            .unwrap();
        assert_eq!(
            frame
                .temporal_context
                .as_ref()
                .unwrap()
                .projection
                .scope
                .continuity_id,
            "cont-test"
        );
    }

    #[test]
    fn temporal_context_projection_rejects_foreign_scope() {
        let (state, frame_id) = push_frame(fresh_state(), "Temporal frame");
        let event = FocusaEvent::TemporalFrameContextProjected {
            frame_id,
            context: temporal_context("/repo/foreign", "cont-foreign", Utc::now()),
        };
        assert!(matches!(
            reduce(state, event),
            Err(ReducerError::InvalidEvent(message))
                if message.contains("temporal context scope mismatch")
        ));
    }

    #[test]
    fn temporal_context_projection_rejects_completed_frame() {
        let (state, _) = push_frame(fresh_state(), "Temporal root");
        let (state, frame_id) = push_frame(state, "Temporal child");
        let state = reduce(
            state,
            FocusaEvent::FocusFrameCompleted {
                frame_id,
                completion_reason: CompletionReason::GoalAchieved,
            },
        )
        .unwrap()
        .new_state;
        let event = FocusaEvent::TemporalFrameContextProjected {
            frame_id,
            context: temporal_context("/repo/test", "cont-test", Utc::now()),
        };
        assert!(matches!(
            reduce(state, event),
            Err(ReducerError::InvalidEvent(message))
                if message.contains("completed frame")
        ));
    }

    #[test]
    fn test_push_empty_beads_id_rejected() {
        let event = FocusaEvent::FocusFramePushed {
            frame_id: Uuid::now_v7(),
            beads_issue_id: "".into(),
            title: "Bad frame".into(),
            goal: "No beads".into(),
            project_root: Some("/repo/test".into()),
            continuity_id: Some("cont-test".into()),
            constraints: vec![],
            tags: vec!["continuity_id:cont-test".into()],
        };
        let result = reduce(fresh_state(), event);
        assert!(result.is_err());
    }

    #[test]
    fn test_push_duplicate_frame_id_rejected() {
        let frame_id = Uuid::now_v7();
        let state = fresh_state();
        let event = FocusaEvent::FocusFramePushed {
            frame_id,
            beads_issue_id: "BEAD-001".into(),
            title: "First".into(),
            goal: "Goal".into(),
            project_root: None,
            continuity_id: None,
            constraints: vec![],
            tags: vec![],
        };
        let state = reduce(state, event).unwrap().new_state;

        // Push again with same frame_id
        let event = FocusaEvent::FocusFramePushed {
            frame_id,
            beads_issue_id: "BEAD-002".into(),
            title: "Duplicate".into(),
            goal: "Goal".into(),
            project_root: None,
            continuity_id: None,
            constraints: vec![],
            tags: vec![],
        };
        let result = reduce(state, event);
        assert!(result.is_err());
    }

    #[test]
    fn same_project_distinct_continuity_frames_remain_active_without_cross_pause() {
        let state = fresh_state();
        let frame_a = Uuid::now_v7();
        let state = reduce(
            state,
            FocusaEvent::FocusFramePushed {
                frame_id: frame_a,
                beads_issue_id: "BEAD-A".into(),
                title: "Session A".into(),
                goal: "Short goal A".into(),
                project_root: Some("/repo/focusa".into()),
                continuity_id: Some("cont-a".into()),
                constraints: vec![],
                tags: vec!["continuity_id:cont-a".into()],
            },
        )
        .unwrap()
        .new_state;
        let frame_b = Uuid::now_v7();
        let state = reduce(
            state,
            FocusaEvent::FocusFramePushed {
                frame_id: frame_b,
                beads_issue_id: "BEAD-B".into(),
                title: "Session B".into(),
                goal: "Short goal B".into(),
                project_root: Some("/repo/focusa".into()),
                continuity_id: Some("cont-b".into()),
                constraints: vec![],
                tags: vec!["continuity_id:cont-b".into()],
            },
        )
        .unwrap()
        .new_state;
        assert_eq!(
            state
                .focus_stack
                .frames
                .iter()
                .filter(|frame| frame.status == FrameStatus::Active)
                .count(),
            2
        );
        assert_eq!(
            state
                .focus_stack
                .frames
                .iter()
                .find(|frame| frame.id == frame_a)
                .map(|frame| frame.status),
            Some(FrameStatus::Active)
        );
        assert_eq!(
            state
                .focus_stack
                .frames
                .iter()
                .find(|frame| frame.id == frame_b)
                .map(|frame| frame.status),
            Some(FrameStatus::Active)
        );
        check_invariants(&state).expect("one active frame per continuity scope");
    }

    #[test]
    fn test_complete_wrong_frame_rejected() {
        let (state, _active_id) = push_frame(fresh_state(), "Active");
        let wrong_id = Uuid::now_v7();
        let event = FocusaEvent::FocusFrameCompleted {
            frame_id: wrong_id,
            completion_reason: CompletionReason::GoalAchieved,
        };
        let result = reduce(state, event);
        assert!(result.is_err());
    }

    #[test]
    fn test_stack_path_cache() {
        let state = fresh_state();
        let (state, root_id) = push_frame(state, "Root");
        let (state, child_id) = push_frame(state, "Child");
        assert_eq!(state.focus_stack.stack_path_cache, vec![root_id, child_id]);
    }

    #[test]
    fn test_suspend_clears_active() {
        let (state, frame_id) = push_frame(fresh_state(), "Task");
        let event = FocusaEvent::FocusFrameSuspended {
            frame_id,
            reason: "user paused".into(),
        };
        let state = reduce(state, event).unwrap().new_state;
        assert_eq!(state.focus_stack.active_id, None);
        let frame = state
            .focus_stack
            .frames
            .iter()
            .find(|f| f.id == frame_id)
            .unwrap();
        assert_eq!(frame.status, FrameStatus::Paused);
    }

    // ─── Focus Gate ──────────────────────────────────────────────────

    #[test]
    fn test_candidate_surfaced() {
        let cid = Uuid::now_v7();
        let event = FocusaEvent::CandidateSurfaced {
            candidate_id: cid,
            kind: CandidateKind::SuggestFixError,
            description: "Fix the bug".into(),
            pressure: 2.5,
            related_frame_id: None,
        };
        let state = reduce(fresh_state(), event).unwrap().new_state;
        assert_eq!(state.focus_gate.candidates.len(), 1);
        assert_eq!(
            state.focus_gate.candidates[0].state,
            CandidateState::Surfaced
        );
        assert_eq!(state.focus_gate.candidates[0].pressure, 2.5);
    }

    #[test]
    fn test_candidate_upsert() {
        let cid = Uuid::now_v7();
        let event1 = FocusaEvent::CandidateSurfaced {
            candidate_id: cid,
            kind: CandidateKind::SuggestFixError,
            description: "v1".into(),
            pressure: 1.0,
            related_frame_id: None,
        };
        let state = reduce(fresh_state(), event1).unwrap().new_state;

        let event2 = FocusaEvent::CandidateSurfaced {
            candidate_id: cid,
            kind: CandidateKind::SuggestFixError,
            description: "v2".into(),
            pressure: 3.0,
            related_frame_id: None,
        };
        let state = reduce(state, event2).unwrap().new_state;

        // Should still be 1 candidate, updated.
        assert_eq!(state.focus_gate.candidates.len(), 1);
        assert_eq!(state.focus_gate.candidates[0].pressure, 3.0);
        assert_eq!(state.focus_gate.candidates[0].label, "v2");
        assert_eq!(state.focus_gate.candidates[0].times_seen, 2);
    }

    #[test]
    fn test_candidate_pin() {
        let cid = Uuid::now_v7();
        let state = reduce(
            fresh_state(),
            FocusaEvent::CandidateSurfaced {
                candidate_id: cid,
                kind: CandidateKind::SuggestFixError,
                description: "test".into(),
                pressure: 1.0,
                related_frame_id: None,
            },
        )
        .unwrap()
        .new_state;

        let state = reduce(state, FocusaEvent::CandidatePinned { candidate_id: cid })
            .unwrap()
            .new_state;
        assert!(state.focus_gate.candidates[0].pinned);
    }

    #[test]
    fn test_candidate_suppress() {
        let cid = Uuid::now_v7();
        let state = reduce(
            fresh_state(),
            FocusaEvent::CandidateSurfaced {
                candidate_id: cid,
                kind: CandidateKind::SuggestFixError,
                description: "test".into(),
                pressure: 2.0,
                related_frame_id: None,
            },
        )
        .unwrap()
        .new_state;

        let state = reduce(
            state,
            FocusaEvent::CandidateSuppressed {
                candidate_id: cid,
                scope: "session".into(),
                suppressed_until: None,
            },
        )
        .unwrap()
        .new_state;

        assert_eq!(
            state.focus_gate.candidates[0].state,
            CandidateState::Suppressed
        );
        assert_eq!(state.focus_gate.candidates[0].pressure, 0.0);
    }

    #[test]
    fn test_nonexistent_candidate_pin_errors() {
        let result = reduce(
            fresh_state(),
            FocusaEvent::CandidatePinned {
                candidate_id: Uuid::now_v7(),
            },
        );
        assert!(result.is_err());
    }

    // ─── Artifacts ───────────────────────────────────────────────────

    #[test]
    fn test_artifact_register() {
        let aid = Uuid::now_v7();
        let event = FocusaEvent::ArtifactRegistered {
            handle: HandleRef {
                id: aid,
                kind: HandleKind::Log,
                label: "Build output".into(),
                size: 42,
                sha256: "abc".into(),
                created_at: Utc::now(),
                session_id: None,
                project_root: None,
                continuity_id: None,
                pinned: false,
                trajectory: None,
            },
            storage_uri: "ecs://abc".into(),
        };
        let state = reduce(fresh_state(), event).unwrap().new_state;
        assert_eq!(state.reference_index.handles.len(), 1);
        assert_eq!(state.reference_index.handles[0].kind, HandleKind::Log);
        assert_eq!(state.reference_index.handles[0].size, 42);
        assert_eq!(state.reference_index.handles[0].sha256, "abc");
    }

    #[test]
    fn test_artifact_immutability() {
        let aid = Uuid::now_v7();
        let event = FocusaEvent::ArtifactRegistered {
            handle: HandleRef {
                id: aid,
                kind: HandleKind::Log,
                label: "v1".into(),
                size: 1,
                sha256: "abc".into(),
                created_at: Utc::now(),
                session_id: None,
                project_root: None,
                continuity_id: None,
                pinned: false,
                trajectory: None,
            },
            storage_uri: "ecs://abc".into(),
        };
        let state = reduce(fresh_state(), event).unwrap().new_state;

        // Re-registering same artifact_id should fail (immutability invariant).
        let event2 = FocusaEvent::ArtifactRegistered {
            handle: HandleRef {
                id: aid,
                kind: HandleKind::Log,
                label: "v2".into(),
                size: 2,
                sha256: "def".into(),
                created_at: Utc::now(),
                session_id: None,
                project_root: None,
                continuity_id: None,
                pinned: false,
                trajectory: None,
            },
            storage_uri: "ecs://def".into(),
        };
        let result = reduce(state, event2);
        assert!(result.is_err());
    }

    #[test]
    fn test_artifact_gc_removes() {
        let aid = Uuid::now_v7();
        let state = reduce(
            fresh_state(),
            FocusaEvent::ArtifactRegistered {
                handle: HandleRef {
                    id: aid,
                    kind: HandleKind::Text,
                    label: "temp".into(),
                    size: 1,
                    sha256: "abc".into(),
                    created_at: Utc::now(),
                    session_id: None,
                    project_root: None,
                    continuity_id: None,
                    pinned: false,
                    trajectory: None,
                },
                storage_uri: "ecs://abc".into(),
            },
        )
        .unwrap()
        .new_state;

        let state = reduce(
            state,
            FocusaEvent::ArtifactGarbageCollected { artifact_id: aid },
        )
        .unwrap()
        .new_state;
        assert!(state.reference_index.handles.is_empty());
    }

    #[test]
    fn test_pinned_artifact_gc_blocked() {
        let aid = Uuid::now_v7();
        let state = reduce(
            fresh_state(),
            FocusaEvent::ArtifactRegistered {
                handle: HandleRef {
                    id: aid,
                    kind: HandleKind::Log,
                    label: "important".into(),
                    size: 1,
                    sha256: "abc".into(),
                    created_at: Utc::now(),
                    session_id: None,
                    project_root: None,
                    continuity_id: None,
                    pinned: false,
                    trajectory: None,
                },
                storage_uri: "ecs://abc".into(),
            },
        )
        .unwrap()
        .new_state;

        let state = reduce(state, FocusaEvent::ArtifactPinned { artifact_id: aid })
            .unwrap()
            .new_state;

        let result = reduce(
            state,
            FocusaEvent::ArtifactGarbageCollected { artifact_id: aid },
        );
        assert!(result.is_err()); // Pinned — cannot GC.
    }

    // ─── Invariant checker ───────────────────────────────────────────

    #[test]
    fn test_complete_root_frame_rejected() {
        let (state, root_id) = push_frame(fresh_state(), "Root");
        let result = reduce(
            state,
            FocusaEvent::FocusFrameCompleted {
                frame_id: root_id,
                completion_reason: CompletionReason::GoalAchieved,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invariant_bidirectional() {
        // Manually create invalid state: active_id = None but a frame is Active.
        let mut state = fresh_state();
        state.focus_stack.frames.push(FrameRecord {
            id: Uuid::now_v7(),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: FrameStatus::Active,
            title: "Rogue".into(),
            goal: "test".into(),
            beads_issue_id: "BEAD-001".into(),
            project_root: Some("/repo/test".into()),
            continuity_id: Some("cont-rogue".into()),
            tags: vec!["continuity_id:cont-rogue".into()],
            priority_hint: None,
            ascc_checkpoint_id: None,
            stats: FrameStats::default(),
            constraints: vec![],
            focus_state: FocusState::default(),
            temporal_context: None,
            completed_at: None,
            completion_reason: None,
        });
        // active_id is None but a frame has Active status.
        let result = check_invariants(&state);
        assert!(result.is_err());
    }

    // ─── Version monotonicity ────────────────────────────────────────

    #[test]
    fn test_version_increments() {
        let state = fresh_state();
        assert_eq!(state.version, 0);

        let (state, _) = push_frame(state, "A");
        assert_eq!(state.version, 1);

        let state = start_session(state);
        assert_eq!(state.version, 2);
    }

    // ─── Thread Creation ─────────────────────────────────────────────

    #[test]
    fn test_thread_created() {
        let thread_id = Uuid::now_v7();
        let event = FocusaEvent::ThreadCreated {
            thread_id,
            name: "Test Thread".into(),
            primary_intent: "Testing thread creation".into(),
            owner_machine_id: Some("machine-a".into()),
        };
        let state = reduce(fresh_state(), event).unwrap().new_state;

        assert_eq!(state.threads.len(), 1);
        let thread = &state.threads[0];
        assert_eq!(thread.id, thread_id);
        assert_eq!(thread.name, "Test Thread");
        assert_eq!(thread.owner_machine_id, Some("machine-a".into()));
        assert_eq!(thread.status, ThreadStatus::Active);
        assert_eq!(thread.thesis.primary_intent, "Testing thread creation");
    }

    #[test]
    fn test_thread_created_duplicate_rejected() {
        let thread_id = Uuid::now_v7();

        // Create thread
        let event = FocusaEvent::ThreadCreated {
            thread_id,
            name: "First".into(),
            primary_intent: "First intent".into(),
            owner_machine_id: None,
        };
        let state = reduce(fresh_state(), event).unwrap().new_state;
        assert_eq!(state.threads.len(), 1);

        // Try to create duplicate
        let event = FocusaEvent::ThreadCreated {
            thread_id,
            name: "Second".into(),
            primary_intent: "Second intent".into(),
            owner_machine_id: None,
        };
        let result = reduce(state, event);
        assert!(result.is_err());
    }

    // ─── Thread Ownership Transfer ───────────────────────────────────

    fn create_thread_with_owner(state: FocusaState, thread_id: Uuid, owner: &str) -> FocusaState {
        let event = FocusaEvent::ThreadCreated {
            thread_id,
            name: "Owned Thread".into(),
            primary_intent: "Testing ownership".into(),
            owner_machine_id: Some(owner.into()),
        };
        reduce(state, event).unwrap().new_state
    }

    #[test]
    fn test_ownership_transfer_by_owner() {
        let thread_id = Uuid::now_v7();
        let state = fresh_state();
        let state = create_thread_with_owner(state, thread_id, "machine-a");

        // Transfer ownership from machine-a to machine-b
        let event = FocusaEvent::ThreadOwnershipTransferred {
            thread_id,
            from_machine_id: Some("machine-a".into()),
            to_machine_id: "machine-b".into(),
            reason: "Testing transfer".into(),
        };
        let state = reduce(state, event).unwrap().new_state;

        let thread = state.threads.iter().find(|t| t.id == thread_id).unwrap();
        assert_eq!(thread.owner_machine_id, Some("machine-b".into()));
    }

    #[test]
    fn test_ownership_transfer_by_non_owner_rejected() {
        let thread_id = Uuid::now_v7();
        let state = fresh_state();
        let state = create_thread_with_owner(state, thread_id, "machine-a");

        // Try to transfer from machine-b (not the owner)
        let event = FocusaEvent::ThreadOwnershipTransferred {
            thread_id,
            from_machine_id: Some("machine-b".into()),
            to_machine_id: "machine-c".into(),
            reason: "Unauthorized transfer".into(),
        };
        let result = reduce(state, event);
        assert!(result.is_err());

        // Check it's an ownership violation
        match result {
            Err(ReducerError::OwnershipViolation { owner, .. }) => {
                assert_eq!(owner, "machine-a");
            }
            _ => panic!("Expected OwnershipViolation error"),
        }
    }

    #[test]
    fn test_ownership_transfer_no_from_id_allowed() {
        // Transfer with no from_machine_id should work for unowned threads
        let thread_id = Uuid::now_v7();
        let event = FocusaEvent::ThreadCreated {
            thread_id,
            name: "Unowned Thread".into(),
            primary_intent: "No owner".into(),
            owner_machine_id: None,
        };
        let state = reduce(fresh_state(), event).unwrap().new_state;

        // Transfer with no from_machine_id
        let event = FocusaEvent::ThreadOwnershipTransferred {
            thread_id,
            from_machine_id: None,
            to_machine_id: "machine-a".into(),
            reason: "Claiming thread".into(),
        };
        let state = reduce(state, event).unwrap().new_state;

        let thread = state.threads.iter().find(|t| t.id == thread_id).unwrap();
        assert_eq!(thread.owner_machine_id, Some("machine-a".into()));
    }

    #[test]
    fn test_ownership_transfer_from_id_on_unowned_thread_rejected() {
        // If thread has no owner, from_machine_id must be None
        let thread_id = Uuid::now_v7();
        let event = FocusaEvent::ThreadCreated {
            thread_id,
            name: "Unowned Thread".into(),
            primary_intent: "No owner".into(),
            owner_machine_id: None,
        };
        let state = reduce(fresh_state(), event).unwrap().new_state;

        // Try to transfer with from_machine_id specified on unowned thread
        let event = FocusaEvent::ThreadOwnershipTransferred {
            thread_id,
            from_machine_id: Some("machine-a".into()), // Can't claim with from_id
            to_machine_id: "machine-b".into(),
            reason: "Invalid claim".into(),
        };
        let result = reduce(state, event);
        assert!(result.is_err());
    }

    #[test]
    fn test_ownership_transfer_nonexistent_thread_rejected() {
        let thread_id = Uuid::now_v7(); // Thread doesn't exist

        let event = FocusaEvent::ThreadOwnershipTransferred {
            thread_id,
            from_machine_id: None,
            to_machine_id: "machine-a".into(),
            reason: "Transfer non-existent thread".into(),
        };
        let result = reduce(fresh_state(), event);
        assert!(result.is_err());
    }

    // ─── Ownership Enforcement in reduce_with_meta ─────────────────────

    #[test]
    fn test_reduce_with_meta_ownership_enforcement() {
        let thread_id = Uuid::now_v7();
        let state = fresh_state();
        let state = create_thread_with_owner(state, thread_id, "owner-machine");

        // Owner can mutate
        let event = FocusaEvent::ThreadCreated {
            thread_id: Uuid::now_v7(),
            name: "New Thread".into(),
            primary_intent: "Test".into(),
            owner_machine_id: None,
        };
        let result = reduce_with_meta(
            state.clone(),
            event,
            Some("owner-machine"),
            Some(thread_id),
            false,
        );
        assert!(result.is_ok());

        // Non-owner is rejected
        let event = FocusaEvent::ThreadCreated {
            thread_id: Uuid::now_v7(),
            name: "Another Thread".into(),
            primary_intent: "Test".into(),
            owner_machine_id: None,
        };
        let result = reduce_with_meta(
            state,
            event,
            Some("attacker-machine"),
            Some(thread_id),
            false,
        );
        assert!(result.is_err());
    }

    fn ontology_test_workstream(id: &str, root: &str) -> WorkstreamKey {
        let scope = crate::scoped_state::ScopeRef::project(
            format!("project:{id}"),
            root,
            id,
            format!("fingerprint:{id}"),
        )
        .expect("valid test scope");
        WorkstreamKey::new(scope, "shared-continuity").expect("valid test workstream")
    }

    #[test]
    fn ontology_duplicate_ids_remain_isolated_by_workstream() {
        let proposal_id = Uuid::now_v7();
        let object_id = "decision:shared-id";
        let workstream_a = ontology_test_workstream("a", "/tmp/focusa-a");
        let workstream_b = ontology_test_workstream("b", "/tmp/focusa-b");

        let mut state = fresh_state();
        for workstream in [&workstream_a, &workstream_b] {
            state = reduce(
                state,
                FocusaEvent::OntologyObjectUpsertProposed {
                    workstream: Some(workstream.clone()),
                    proposal_id,
                    object_type: "decision".into(),
                    object_id: Some(object_id.into()),
                    source: "scope-isolation-test".into(),
                },
            )
            .unwrap()
            .new_state;
        }
        assert_eq!(state.ontology.proposals.len(), 2);
        assert_eq!(state.ontology.objects.len(), 2);

        state = reduce(
            state,
            FocusaEvent::OntologyProposalPromoted {
                workstream: Some(workstream_a.clone()),
                proposal_id,
                target_class: "decision".into(),
                applied_kind: "object_upsert".into(),
            },
        )
        .unwrap()
        .new_state;
        state = reduce(
            state,
            FocusaEvent::OntologyVerificationApplied {
                workstream: Some(workstream_b.clone()),
                proposal_id: Some(proposal_id),
                verification: "scope-isolation".into(),
                outcome: "rejected".into(),
            },
        )
        .unwrap()
        .new_state;

        let status_for = |workstream: &WorkstreamKey| {
            state
                .ontology
                .objects
                .iter()
                .find(|object| ontology_value_matches_workstream(object, &Some(workstream.clone())))
                .and_then(|object| object.get("status"))
                .and_then(serde_json::Value::as_str)
        };
        assert_eq!(status_for(&workstream_a), Some("promoted"));
        assert_eq!(status_for(&workstream_b), Some("failed"));
    }

    #[test]
    fn ontology_legacy_records_deserialize_as_unowned() {
        let record: OntologyProposalRecord = serde_json::from_value(serde_json::json!({
            "proposal_id": Uuid::now_v7(),
            "proposal_kind": "object_upsert",
            "target_class": "decision",
            "status": "proposed",
            "source": null,
            "object_type": null,
            "object_id": null,
            "link_type": null,
            "source_id": null,
            "target_id": null,
            "notes": null,
            "updated_at": null
        }))
        .expect("legacy proposal remains replayable");
        assert!(record.workstream.is_none());
    }

    #[test]
    fn ontology_scope_migration_clones_and_rolls_back_with_append_only_receipts() {
        let target = ontology_test_workstream("migration", "/tmp/focusa-migration");
        let mut state = fresh_state();
        let source = OntologyProposalRecord {
            proposal_id: Uuid::now_v7(),
            proposal_kind: "object_upsert".into(),
            target_class: "decision".into(),
            status: "proposed".into(),
            ..OntologyProposalRecord::default()
        };
        let source_hash = ontology_scope_record_hash(&source);
        state.ontology.proposals.push(source);
        let migration_id = Uuid::now_v7();
        let selection = OntologyScopeMigrationSelection {
            record_kind: OntologyScopeMigrationRecordKind::Proposal,
            source_hash,
            evidence_refs: vec!["evidence:operator-confirmed-owner".into()],
        };

        let apply = || FocusaEvent::OntologyScopeMigrationApplied {
            migration_id,
            target_workstream: target.clone(),
            selections: vec![selection.clone()],
            evidence_refs: vec!["evidence:migration-plan".into()],
        };
        state = reduce(state, apply()).unwrap().new_state;
        assert_eq!(state.ontology.proposals.len(), 2);
        assert_eq!(
            state
                .ontology
                .proposals
                .iter()
                .filter(|record| record.workstream.as_ref() == Some(&target))
                .count(),
            1
        );
        assert!(
            state
                .ontology
                .proposals
                .iter()
                .any(|record| record.workstream.is_none())
        );
        assert_eq!(state.ontology.scope_migration_receipts.len(), 1);

        state = reduce(state, apply()).unwrap().new_state;
        assert_eq!(state.ontology.proposals.len(), 2);
        assert_eq!(state.ontology.scope_migration_receipts.len(), 1);

        let rollback_id = Uuid::now_v7();
        let rollback = || FocusaEvent::OntologyScopeMigrationRolledBack {
            rollback_id,
            migration_id,
            evidence_refs: vec!["evidence:rollback-request".into()],
        };
        state = reduce(state, rollback()).unwrap().new_state;
        assert_eq!(state.ontology.proposals.len(), 1);
        assert!(state.ontology.proposals[0].workstream.is_none());
        assert_eq!(state.ontology.scope_migration_receipts.len(), 2);
        assert_eq!(
            state.ontology.scope_migration_receipts[1].operation,
            "rollback"
        );

        state = reduce(state, rollback()).unwrap().new_state;
        assert_eq!(state.ontology.proposals.len(), 1);
        assert_eq!(state.ontology.scope_migration_receipts.len(), 2);
    }

    #[test]
    fn ontology_scope_migration_rejects_records_without_evidence() {
        let target = ontology_test_workstream("migration", "/tmp/focusa-migration");
        let mut state = fresh_state();
        let source = OntologyProposalRecord {
            proposal_id: Uuid::now_v7(),
            proposal_kind: "object_upsert".into(),
            target_class: "decision".into(),
            status: "proposed".into(),
            ..OntologyProposalRecord::default()
        };
        let source_hash = ontology_scope_record_hash(&source);
        state.ontology.proposals.push(source);
        let result = reduce(
            state,
            FocusaEvent::OntologyScopeMigrationApplied {
                migration_id: Uuid::now_v7(),
                target_workstream: target,
                selections: vec![OntologyScopeMigrationSelection {
                    record_kind: OntologyScopeMigrationRecordKind::Proposal,
                    source_hash,
                    evidence_refs: vec![],
                }],
                evidence_refs: vec!["evidence:migration-plan".into()],
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn ontology_object_upsert_proposal_sets_proposed_status() {
        let proposal_id = Uuid::now_v7();
        let state = reduce(
            fresh_state(),
            FocusaEvent::OntologyObjectUpsertProposed {
                workstream: None,
                proposal_id,
                object_type: "decision".into(),
                object_id: Some("decision:proposed-1".into()),
                source: "ontology_test".into(),
            },
        )
        .unwrap()
        .new_state;

        let proposal = state
            .ontology
            .proposals
            .iter()
            .find(|p| p.proposal_id == proposal_id)
            .expect("proposal should be recorded");
        assert_eq!(proposal.status, "proposed");

        let object = state
            .ontology
            .objects
            .iter()
            .find(|o| o.get("id").and_then(|v| v.as_str()) == Some("decision:proposed-1"))
            .expect("proposed object should be present");
        assert_eq!(
            object.get("status").and_then(|v| v.as_str()),
            Some("proposed")
        );
    }

    #[test]
    fn ontology_verification_negative_outcome_sets_failed_status() {
        let proposal_id = Uuid::now_v7();
        let state = reduce(
            fresh_state(),
            FocusaEvent::OntologyObjectUpsertProposed {
                workstream: None,
                proposal_id,
                object_type: "decision".into(),
                object_id: Some("decision:failed-1".into()),
                source: "ontology_test".into(),
            },
        )
        .unwrap()
        .new_state;

        let state = reduce(
            state,
            FocusaEvent::OntologyVerificationApplied {
                workstream: None,
                proposal_id: Some(proposal_id),
                verification: "verification:failed-path".into(),
                outcome: "rejected".into(),
            },
        )
        .unwrap()
        .new_state;

        let object = state
            .ontology
            .objects
            .iter()
            .find(|o| o.get("id").and_then(|v| v.as_str()) == Some("decision:failed-1"))
            .expect("verified object should remain present");
        assert_eq!(
            object.get("status").and_then(|v| v.as_str()),
            Some("failed")
        );
    }

    // ─── Trajectory Projection (Spec96) ───────────────────────────────

    fn trajectory_record(id: &str, long_term_goal: &str) -> TrajectoryProjectionRecord {
        TrajectoryProjectionRecord {
            trajectory_id: id.to_string(),
            project_root: Some("/repo/test".to_string()),
            continuity_id: Some("cont-test".to_string()),
            root_long_term_goal: long_term_goal.to_string(),
            long_term_goal: long_term_goal.to_string(),
            desired_end_state: "Reducer-backed trajectory metadata is queryable".to_string(),
            short_term_goal: Some("Implement trajectory reducer events".to_string()),
            current_state: Some("Types are defined".to_string()),
            root_goal_stability: TrajectoryRootGoalStability::Stable,
            session_clarity_status: TrajectoryDefinitionStatus::Clear,
            definition_status: TrajectoryDefinitionStatus::Clear,
            confidence: TrajectoryConfidence::High,
            goal_provenance: vec![TrajectoryGoalProvenanceRecord {
                field: "long_term_goal".to_string(),
                source: "operator".to_string(),
                source_ref: Some("test".to_string()),
                inferred: false,
                confidence: TrajectoryConfidence::High,
            }],
            definition_of_done: Some(TrajectoryDefinitionOfDoneRecord {
                criteria: vec!["trajectory persisted".to_string()],
                evidence_required: vec!["reducer test".to_string()],
                verified_evidence_refs: vec![],
                status: "defined".to_string(),
                desired_end_state: Some("trajectory persisted".to_string()),
                required_evidence_refs: vec!["reducer test".to_string()],
                required_checks: vec!["cargo test -p focusa-core".to_string()],
                acceptance_risks: vec!["unbounded trajectory proof".to_string()],
                not_done_if: vec!["trajectory record is not reducer-visible".to_string()],
            }),
            canonical: true,
            ..TrajectoryProjectionRecord::default()
        }
    }

    #[test]
    fn test_trajectory_goal_defined_sets_active_and_supersedes_same_scope() {
        let state = fresh_state();
        let first = trajectory_record("trajectory:first", "Ship Focusa trajectory");
        let second = trajectory_record("trajectory:second", "Ship Focusa trajectory v2");
        let state = reduce(
            state,
            FocusaEvent::TrajectoryGoalDefined { trajectory: first },
        )
        .unwrap()
        .new_state;
        assert_eq!(
            state.trajectory.active_trajectory_id.as_deref(),
            Some("trajectory:first")
        );
        let state = reduce(
            state,
            FocusaEvent::TrajectoryGoalDefined { trajectory: second },
        )
        .unwrap()
        .new_state;
        assert_eq!(
            state.trajectory.active_trajectory_id.as_deref(),
            Some("trajectory:second")
        );
        let first = state
            .trajectory
            .records
            .iter()
            .find(|record| record.trajectory_id == "trajectory:first")
            .unwrap();
        assert!(!first.canonical);
        assert_eq!(
            first.root_goal_stability,
            TrajectoryRootGoalStability::Superseded
        );
    }

    #[test]
    fn work_loop_execution_scope_is_reducer_owned_and_cleared_on_stop() {
        let project = crate::scoped_state::ScopeRef::project(
            "project:focusa",
            "/repo/focusa",
            "Focusa",
            "sha256:focusa",
        )
        .unwrap();
        let scope = crate::scoped_state::WorkstreamKey::new(project, "cont-focusa").unwrap();
        let workpoint_id = Uuid::now_v7();
        let enabled = reduce(
            fresh_state(),
            FocusaEvent::ContinuousWorkModeEnabled {
                project_run_id: Uuid::now_v7(),
                policy: WorkLoopPolicy::default(),
                scope: Some(scope.clone()),
                work_item_id: Some("focusa-workloop-completion.2".to_string()),
                workpoint_id: Some(workpoint_id),
            },
        )
        .unwrap()
        .new_state;
        assert_eq!(enabled.work_loop.execution_scope, Some(scope));
        assert_eq!(
            enabled.work_loop.execution_work_item_id.as_deref(),
            Some("focusa-workloop-completion.2")
        );
        assert_eq!(enabled.work_loop.execution_workpoint_id, Some(workpoint_id));

        let deferred = reduce(
            enabled,
            FocusaEvent::ContinuousWorkItemDeferred {
                work_item_id: "focusa-workloop-completion.2.1".to_string(),
                reason: "temporary external dependency".to_string(),
            },
        )
        .unwrap()
        .new_state;
        assert_eq!(deferred.work_loop.deferred_items.len(), 1);

        let stopped = reduce(
            deferred,
            FocusaEvent::ContinuousWorkModeDisabled {
                reason: "operator stop".to_string(),
            },
        )
        .unwrap()
        .new_state;
        assert_eq!(stopped.work_loop.execution_scope, None);
        assert_eq!(stopped.work_loop.execution_work_item_id, None);
        assert_eq!(stopped.work_loop.execution_workpoint_id, None);
        assert!(stopped.work_loop.deferred_items.is_empty());
        assert!(stopped.work_loop.run.task_run_id.is_none());
        assert!(stopped.work_loop.run.tranche_run_id.is_none());
        assert!(stopped.work_loop.run.worker_session_id.is_none());
    }

    #[test]
    fn reenable_clears_stale_deferred_frontier_and_prior_run_selection() {
        let project = crate::scoped_state::ScopeRef::project(
            "project:focusa",
            "/repo/focusa",
            "Focusa",
            "sha256:focusa",
        )
        .unwrap();
        let scope = crate::scoped_state::WorkstreamKey::new(project, "cont-focusa").unwrap();
        let mut state = fresh_state();
        state.work_loop.deferred_items.push(WorkLoopDeferredItem {
            work_item_id: "settled-atom".to_string(),
            reason: "prior blocked frontier".to_string(),
            deferred_at: Utc::now(),
        });
        state.work_loop.current_task = Some(SpecLinkedTaskPacket {
            work_item_id: "settled-atom".to_string(),
            ..SpecLinkedTaskPacket::default()
        });
        state.work_loop.run.task_run_id = Some(Uuid::now_v7());
        state.work_loop.run.tranche_run_id = Some(Uuid::now_v7());
        state.work_loop.run.worker_session_id = Some("stale-worker".to_string());

        let rebound = reduce(
            state,
            FocusaEvent::ContinuousWorkModeEnabled {
                project_run_id: Uuid::now_v7(),
                policy: WorkLoopPolicy::default(),
                scope: Some(scope),
                work_item_id: Some("focusa-vbcqu.20.15".to_string()),
                workpoint_id: Some(Uuid::now_v7()),
            },
        )
        .unwrap()
        .new_state;

        assert!(rebound.work_loop.deferred_items.is_empty());
        assert!(rebound.work_loop.current_task.is_none());
        assert!(rebound.work_loop.run.task_run_id.is_none());
        assert!(rebound.work_loop.run.tranche_run_id.is_none());
        assert!(rebound.work_loop.run.worker_session_id.is_none());
        assert_eq!(rebound.work_loop.status, WorkLoopStatus::Idle);
    }

    #[test]
    fn deferred_blocker_yields_and_continue_outcome_keeps_current_task() {
        let mut state = fresh_state();
        state.work_loop.current_task = Some(SpecLinkedTaskPacket {
            work_item_id: "blocked".to_string(),
            title: "blocked".to_string(),
            ..SpecLinkedTaskPacket::default()
        });
        let deferred = reduce(
            state,
            FocusaEvent::ContinuousWorkItemDeferred {
                work_item_id: "blocked".to_string(),
                reason: "dependency unavailable".to_string(),
            },
        )
        .unwrap()
        .new_state;
        assert!(deferred.work_loop.current_task.is_none());
        assert_eq!(deferred.work_loop.deferred_items.len(), 1);
        assert_eq!(
            deferred.work_loop.status,
            WorkLoopStatus::SelectingReadyWork
        );

        let mut continuing = deferred;
        continuing.work_loop.current_task = Some(SpecLinkedTaskPacket {
            work_item_id: "alternate".to_string(),
            title: "alternate".to_string(),
            ..SpecLinkedTaskPacket::default()
        });
        let continued = reduce(
            continuing,
            FocusaEvent::ContinuousTurnCompleted {
                task_run_id: None,
                work_item_id: Some("alternate".to_string()),
                continue_reason: Some("more work remains".to_string()),
                verification_satisfied: false,
                spec_conformant: true,
                outcome_status: WorkLoopOutcomeStatus::Continue,
                evidence_citations: vec![],
            },
        )
        .unwrap()
        .new_state;
        assert_eq!(continued.work_loop.status, WorkLoopStatus::Idle);
        assert_eq!(
            continued
                .work_loop
                .current_task
                .as_ref()
                .map(|task| task.work_item_id.as_str()),
            Some("alternate")
        );
        assert_eq!(continued.work_loop.deferred_items.len(), 1);
    }

    #[test]
    fn budget_exhaustion_and_explicit_renewal_create_new_epoch() {
        let mut state = fresh_state();
        let initial_epoch = Uuid::now_v7();
        state.work_loop.budget_epoch_id = Some(initial_epoch);
        state.work_loop.budget_epoch_started_at = Some(Utc::now() - chrono::Duration::minutes(5));
        state.work_loop.turn_count = 30;
        let exhausted = reduce(
            state,
            FocusaEvent::ContinuousLoopBudgetExhausted {
                dimension: WorkLoopBudgetDimension::WallClock,
                reason: "max_wall_clock_ms budget exhausted".to_string(),
            },
        )
        .unwrap()
        .new_state;
        assert_eq!(exhausted.work_loop.status, WorkLoopStatus::Paused);
        assert_eq!(
            exhausted
                .work_loop
                .budget_exhaustion
                .as_ref()
                .map(|entry| entry.dimension),
            Some(WorkLoopBudgetDimension::WallClock)
        );
        let renewed = reduce(
            exhausted,
            FocusaEvent::ContinuousLoopResumed {
                reason: "approved renewal".to_string(),
                budget_renewed: true,
                policy: None,
            },
        )
        .unwrap()
        .new_state;
        assert_eq!(renewed.work_loop.status, WorkLoopStatus::Idle);
        assert!(renewed.work_loop.budget_exhaustion.is_none());
        assert_ne!(renewed.work_loop.budget_epoch_id, Some(initial_epoch));
        assert_eq!(renewed.work_loop.budget_renewal_count, 1);
        assert_eq!(renewed.work_loop.turn_count, 0);
    }

    #[test]
    fn transport_attachment_materializes_exact_execution_partition() {
        let project = crate::scoped_state::ScopeRef::project(
            "project:focusa",
            "/repo/focusa",
            "Focusa",
            "sha256:focusa",
        )
        .unwrap();
        let scope = crate::scoped_state::WorkstreamKey::new(project, "cont-focusa").unwrap();
        let workpoint_id = Uuid::now_v7();
        let attached = reduce(
            fresh_state(),
            FocusaEvent::ContinuousTransportSessionAttached {
                adapter: "pi-rpc".to_string(),
                session_id: "session-1".to_string(),
                scope: scope.clone(),
                work_item_id: "focusa-root".to_string(),
                workpoint_id,
            },
        )
        .unwrap()
        .new_state;
        assert_eq!(
            attached.work_loop.transport_session_id.as_deref(),
            Some("session-1")
        );
        assert_eq!(attached.work_loop.transport_scope, Some(scope));
        assert_eq!(
            attached.work_loop.transport_work_item_id.as_deref(),
            Some("focusa-root")
        );
        assert_eq!(
            attached.work_loop.transport_workpoint_id,
            Some(workpoint_id)
        );
    }

    #[test]
    fn replacement_transport_recovers_degraded_selected_task() {
        let project = crate::scoped_state::ScopeRef::project(
            "project:focusa",
            "/repo/focusa",
            "Focusa",
            "sha256:focusa",
        )
        .unwrap();
        let scope = crate::scoped_state::WorkstreamKey::new(project, "cont-focusa").unwrap();
        let mut state = fresh_state();
        state.work_loop.enabled = true;
        state.work_loop.status = WorkLoopStatus::TransportDegraded;
        state.work_loop.last_blocker_class = Some(BlockerClass::Transport);
        state.work_loop.last_blocker_reason = Some("rpc stream closed".to_string());
        state.work_loop.current_task = Some(SpecLinkedTaskPacket {
            work_item_id: "focusa-vbcqu.20.15.7".to_string(),
            ..SpecLinkedTaskPacket::default()
        });

        let recovered = reduce(
            state,
            FocusaEvent::ContinuousTransportSessionAttached {
                adapter: "pi-rpc".to_string(),
                session_id: "replacement-session".to_string(),
                scope,
                work_item_id: "focusa-vbcqu.20.15".to_string(),
                workpoint_id: Uuid::now_v7(),
            },
        )
        .unwrap()
        .new_state;

        assert_eq!(
            recovered.work_loop.status,
            WorkLoopStatus::SelectingReadyWork
        );
        assert!(recovered.work_loop.last_blocker_class.is_none());
        assert!(recovered.work_loop.last_blocker_reason.is_none());
        assert_eq!(
            recovered.work_loop.transport_session_id.as_deref(),
            Some("replacement-session")
        );
    }

    #[test]
    fn decision_context_source_turn_never_repartitions_execution_scope() {
        let project = crate::scoped_state::ScopeRef::project(
            "project:focusa",
            "/repo/focusa",
            "Focusa",
            "sha256:focusa",
        )
        .unwrap();
        let scope =
            crate::scoped_state::WorkstreamKey::new(project, "workloop-completion").unwrap();
        let enabled = reduce(
            fresh_state(),
            FocusaEvent::ContinuousWorkModeEnabled {
                project_run_id: Uuid::now_v7(),
                policy: WorkLoopPolicy::default(),
                scope: Some(scope.clone()),
                work_item_id: Some("focusa-a6yq6.2.3".to_string()),
                workpoint_id: Some(Uuid::now_v7()),
            },
        )
        .unwrap()
        .new_state;

        let updated = reduce(
            enabled,
            FocusaEvent::ContinuousDecisionContextUpdated {
                current_ask: Some("verify the loop".to_string()),
                ask_kind: Some("instruction".to_string()),
                scope_kind: Some("mission_carryover".to_string()),
                carryover_policy: Some("allow_if_relevant".to_string()),
                excluded_context_reason: None,
                excluded_context_labels: None,
                source_turn_id: Some("pi-turn-686".to_string()),
                operator_steering_detected: Some(true),
            },
        )
        .unwrap()
        .new_state;

        assert_eq!(updated.work_loop.execution_scope, Some(scope));
        assert_eq!(
            updated.work_loop.execution_work_item_id.as_deref(),
            Some("focusa-a6yq6.2.3")
        );
        assert_eq!(
            updated.work_loop.decision_context.source_turn_id.as_deref(),
            Some("pi-turn-686")
        );
    }

    #[test]
    fn test_trajectory_checkpoint_and_state_delta_are_queryable() {
        let state = reduce(
            fresh_state(),
            FocusaEvent::TrajectoryGoalDefined {
                trajectory: trajectory_record("trajectory:active", "Ship durable trajectory"),
            },
        )
        .unwrap()
        .new_state;
        let state = reduce(
            state,
            FocusaEvent::TrajectoryCheckpointPersisted {
                trajectory_id: "trajectory:active".to_string(),
                checkpoint: serde_json::json!({"summary":"checkpoint"}),
                summary: Some("checkpoint".to_string()),
            },
        )
        .unwrap()
        .new_state;
        let state = reduce(
            state,
            FocusaEvent::TrajectoryStateDeltaRecorded {
                trajectory_id: "trajectory:active".to_string(),
                current_state: Some("Reducer persisted trajectory delta".to_string()),
                evidence_refs: vec!["tests:trajectory_reducer".to_string()],
                reason: "verification".to_string(),
            },
        )
        .unwrap()
        .new_state;
        assert_eq!(state.trajectory.checkpoints.len(), 1);
        assert_eq!(state.trajectory.state_deltas.len(), 1);
        let active = state
            .trajectory
            .records
            .iter()
            .find(|record| record.trajectory_id == "trajectory:active")
            .unwrap();
        assert_eq!(
            active.current_state.as_deref(),
            Some("Reducer persisted trajectory delta")
        );
        assert!(
            active
                .definition_of_done
                .as_ref()
                .unwrap()
                .verified_evidence_refs
                .contains(&"tests:trajectory_reducer".to_string())
        );
    }
}
