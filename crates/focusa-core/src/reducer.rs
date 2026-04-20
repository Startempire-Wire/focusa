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
use crate::types::*;

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
                clt::append_interaction(
                    &mut state.clt,
                    state.session.as_ref().map(|s| s.session_id),
                    "assistant",
                    assistant_output.as_deref(),
                    CltMetadata::default(),
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
        } => {
            state.work_loop.enabled = true;
            state.work_loop.status = WorkLoopStatus::Idle;
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
            state.work_loop.enabled_at = Some(Utc::now());
            state.work_loop.last_turn_requested_at = None;
            state.work_loop.turn_count = 0;
            state.work_loop.consecutive_failures_for_task_class = 0;
            state.work_loop.consecutive_low_productivity_turns = 0;
            state.work_loop.consecutive_same_work_item_retries = 0;
            state.work_loop.last_observed_work_item_id = None;
        }
        FocusaEvent::ContinuousWorkModeDisabled { reason } => {
            state.work_loop.enabled = false;
            state.work_loop.status = WorkLoopStatus::Idle;
            state.work_loop.current_task = None;
            state.work_loop.last_continue_reason = Some(reason);
            state.work_loop.enabled_at = None;
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
        } => {
            state.work_loop.transport_adapter = Some(adapter);
            state.work_loop.run.worker_session_id = Some(session_id.clone());
            state.work_loop.transport_session_state = Some("attached".to_string());
            state.work_loop.last_transport_event_kind = Some("session_attached".to_string());
            state.work_loop.last_transport_event_summary = Some(session_id);
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
        } => {
            state.work_loop.status = WorkLoopStatus::AdvancingTask;
            state.work_loop.run.task_run_id = task_run_id;
            state.work_loop.last_completed_task_id = work_item_id.clone();
            state.work_loop.last_recorded_bd_transition_id = work_item_id.clone();
            state.work_loop.last_continue_reason = continue_reason;
            state.work_loop.consecutive_failures_for_task_class = 0;
            state.work_loop.consecutive_low_productivity_turns = 0;
            state.work_loop.consecutive_same_work_item_retries = 0;
            state.work_loop.last_observed_work_item_id = work_item_id.clone();
            state.work_loop.run.worker_session_id = None;
            state.work_loop.current_task = None;
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
        FocusaEvent::ContinuousLoopBudgetExhausted { reason } => {
            state.work_loop.status = WorkLoopStatus::Paused;
            state.work_loop.last_blocker_reason = Some(reason);
        }
        FocusaEvent::ContinuousLoopTransportDegraded { reason } => {
            state.work_loop.status = WorkLoopStatus::TransportDegraded;
            state.work_loop.last_blocker_reason = Some(reason);
        }
        FocusaEvent::ContinuousLoopResumed { reason } => {
            state.work_loop.status = WorkLoopStatus::Idle;
            state.work_loop.pause_flags = WorkLoopPauseFlags::default();
            state.work_loop.last_safe_reentry_prompt_basis = Some(reason.clone());
            state.work_loop.last_continue_reason = Some(reason);
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

            // Pause current active frame.
            if let Some(active_id) = stack.active_id
                && let Some(frame) = stack.frames.iter_mut().find(|f| f.id == active_id)
            {
                frame.status = FrameStatus::Paused;
                frame.updated_at = now;
            }

            let parent_id = stack.active_id;

            stack.frames.push(FrameRecord {
                id: frame_id,
                parent_id,
                created_at: now,
                updated_at: now,
                status: FrameStatus::Active,
                title,
                goal,
                beads_issue_id,
                tags,
                priority_hint: None,
                ascc_checkpoint_id: None,
                stats: FrameStats::default(),
                constraints,
                focus_state: FocusState::default(),
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
            if state.focus_stack.active_id != Some(frame_id) {
                return Err(ReducerError::InvalidEvent(format!(
                    "FocusStateUpdated for {} but active is {:?}",
                    frame_id, state.focus_stack.active_id
                )));
            }

            let frame = state
                .focus_stack
                .frames
                .iter_mut()
                .find(|f| f.id == frame_id)
                .ok_or_else(|| ReducerError::FrameNotFound(frame_id.to_string()))?;

            apply_delta(&mut frame.focus_state, &delta);
            frame.updated_at = Utc::now();
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
            proposal_id,
            object_type,
            object_id,
            source,
        } => {
            let now = Utc::now();
            let record = OntologyProposalRecord {
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
                .find(|p| p.proposal_id == proposal_id)
            {
                *existing = record;
            } else {
                state.ontology.proposals.push(record);
            }
            if let Some(id) = object_id.clone() {
                let exists = state
                    .ontology
                    .objects
                    .iter()
                    .any(|o| o.get("id").and_then(|v| v.as_str()) == Some(id.as_str()));
                if !exists {
                    state.ontology.objects.push(serde_json::json!({
                        "id": id,
                        "object_type": object_type,
                        "status": "proposed",
                        "provenance_class": "model_inferred",
                        "source": source,
                    }));
                }
            }
            state.ontology.delta_log.push(OntologyDeltaRecord {
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
            proposal_id,
            link_type,
            source_id,
            target_id,
            source,
        } => {
            let now = Utc::now();
            let record = OntologyProposalRecord {
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
                .find(|p| p.proposal_id == proposal_id)
            {
                *existing = record;
            } else {
                state.ontology.proposals.push(record);
            }
            state.ontology.links.push(serde_json::json!({
                "type": link_type,
                "source_id": source_id,
                "target_id": target_id,
                "status": "proposed",
                "evidence": "proposal_submitted",
                "proposal_id": proposal_id,
                "source": source,
            }));
            state.ontology.delta_log.push(OntologyDeltaRecord {
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
            proposal_id,
            subject,
            from_status,
            to_status,
            source,
        } => {
            let now = Utc::now();
            let record = OntologyProposalRecord {
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
                .find(|p| p.proposal_id == proposal_id)
            {
                *existing = record;
            } else {
                state.ontology.proposals.push(record);
            }
            if let Some(object) = state
                .ontology
                .objects
                .iter_mut()
                .find(|o| o.get("id").and_then(|v| v.as_str()) == Some(subject.as_str()))
            {
                object["status"] = serde_json::Value::String(to_status.clone());
            }
            state.ontology.delta_log.push(OntologyDeltaRecord {
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
            proposal_id,
            subject,
            operation,
            source,
        } => {
            let now = Utc::now();
            let record = OntologyProposalRecord {
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
                .find(|p| p.proposal_id == proposal_id)
            {
                *existing = record;
            } else {
                state.ontology.proposals.push(record);
            }
            if let Some(object) = state
                .ontology
                .objects
                .iter_mut()
                .find(|o| o.get("id").and_then(|v| v.as_str()) == Some(subject.as_str()))
            {
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
            proposal_id,
            target_class,
            applied_kind,
        } => {
            let now = Utc::now();
            if let Some(proposal_idx) = state
                .ontology
                .proposals
                .iter()
                .position(|p| p.proposal_id == proposal_id)
            {
                let proposal = state.ontology.proposals[proposal_idx].clone();
                state.ontology.proposals[proposal_idx].status = "promoted".to_string();
                state.ontology.proposals[proposal_idx].updated_at = Some(now);

                match proposal.proposal_kind.as_str() {
                    "object_upsert" => {
                        if let Some(object_id) = proposal.object_id.as_ref() {
                            if let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                                })
                            {
                                object["status"] = serde_json::Value::String("promoted".to_string());
                                object["provenance_class"] =
                                    serde_json::Value::String("reducer_promoted".to_string());
                                object["promoted_by"] =
                                    serde_json::Value::String(proposal_id.to_string());
                            } else {
                                state.ontology.objects.push(serde_json::json!({
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
                                l.get("type").and_then(|v| v.as_str()) == Some(link_type.as_str())
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
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
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
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
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
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
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
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
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
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
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
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
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
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
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
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
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
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("candidate".to_string());
                            object["risk_state"] =
                                serde_json::Value::String("identified".to_string());
                        }
                    }
                    "mark_blocked" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] = serde_json::Value::String("blocked".to_string());
                        }
                    }
                    "restore_progress" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
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
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("verified".to_string());
                            object["progress_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    "refresh_working_set" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
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
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("completed".to_string());
                            object["completion_state"] =
                                serde_json::Value::String("closed".to_string());
                        }
                    }
                    "complete_task" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("completed".to_string());
                            object["completion_state"] =
                                serde_json::Value::String("closed".to_string());
                        }
                    }
                    "detect_affordances" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("candidate".to_string());
                            object["affordance_state"] =
                                serde_json::Value::String("detected".to_string());
                        }
                    }
                    "verify_permissions" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("verified".to_string());
                            object["permission_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    "verify_preconditions" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("verified".to_string());
                            object["precondition_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    "evaluate_dependencies" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("active".to_string());
                            object["dependency_state"] =
                                serde_json::Value::String("evaluated".to_string());
                        }
                    }
                    "estimate_cost" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("active".to_string());
                            object["estimation_state"] =
                                serde_json::Value::String("estimated".to_string());
                        }
                    }
                    "estimate_latency" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("active".to_string());
                            object["estimation_state"] =
                                serde_json::Value::String("estimated".to_string());
                        }
                    }
                    "estimate_reliability" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("active".to_string());
                            object["estimation_state"] =
                                serde_json::Value::String("estimated".to_string());
                        }
                    }
                    "estimate_reversibility" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("active".to_string());
                            object["estimation_state"] =
                                serde_json::Value::String("estimated".to_string());
                        }
                    }
                    "choose_execution_path" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("active".to_string());
                            object["execution_path_state"] =
                                serde_json::Value::String("selected".to_string());
                        }
                    }
                    "escalate_authority" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("active".to_string());
                            object["authority_state"] =
                                serde_json::Value::String("escalated".to_string());
                        }
                    }
                    "mark_unavailable" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("blocked".to_string());
                            object["availability_state"] =
                                serde_json::Value::String("unavailable".to_string());
                        }
                    }
                    "determine_current_ask" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("active".to_string());
                            object["ask_state"] =
                                serde_json::Value::String("determined".to_string());
                        }
                    }
                    "build_query_scope" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("active".to_string());
                            object["scope_state"] =
                                serde_json::Value::String("built".to_string());
                        }
                    }
                    "select_relevant_context" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("active".to_string());
                            object["selection_state"] =
                                serde_json::Value::String("selected".to_string());
                        }
                    }
                    "exclude_irrelevant_context" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("active".to_string());
                            object["selection_state"] =
                                serde_json::Value::String("pruned".to_string());
                        }
                    }
                    "verify_answer_scope" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("verified".to_string());
                            object["scope_state"] =
                                serde_json::Value::String("verified".to_string());
                        }
                    }
                    "record_scope_failure" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state.ontology.objects.iter_mut().find(|o| {
                                o.get("id").and_then(|v| v.as_str()) == Some(object_id.as_str())
                            })
                        {
                            object["status"] =
                                serde_json::Value::String("failed".to_string());
                            object["scope_state"] =
                                serde_json::Value::String("failed".to_string());
                        }
                    }
                    "create_version" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                                })
                        {
                            object["status"] = serde_json::Value::String("active".to_string());
                            object["version_state"] =
                                serde_json::Value::String("created".to_string());
                        }
                    }
                    "declare_compatibility" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
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
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
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
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                                })
                        {
                            object["status"] = serde_json::Value::String("deprecated".to_string());
                            object["lifecycle"] =
                                serde_json::Value::String("deprecated".to_string());
                        }
                    }
                    "review_governance_change" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
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
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
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
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
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
                            l.get("type").and_then(|v| v.as_str()) == Some(link_type.as_str())
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
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                                })
                        {
                            object["status"] = serde_json::Value::String("rejected".to_string());
                            object["rejection_reason"] = serde_json::Value::String(reason.clone());
                        }
                    }
                    "working_set_membership" => {
                        if let Some(object_id) = proposal.object_id.as_ref()
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
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
            proposal_id,
            verification,
            outcome,
        } => {
            let now = Utc::now();
            state
                .ontology
                .verifications
                .push(OntologyVerificationRecord {
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
                    .find(|p| p.proposal_id == pid)
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
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                                })
                        {
                            object["status"] = serde_json::Value::String(verified_status.to_string());
                            object["verification"] = serde_json::Value::String(verification.clone());
                        }
                    }
                    "link_upsert" => {
                        if let (Some(link_type), Some(source_id), Some(target_id)) = (
                            proposal.link_type.as_ref(),
                            proposal.source_id.as_ref(),
                            proposal.target_id.as_ref(),
                        ) && let Some(link) = state.ontology.links.iter_mut().find(|l| {
                            l.get("type").and_then(|v| v.as_str()) == Some(link_type.as_str())
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
                            && let Some(object) = state
                                .ontology
                                .objects
                                .iter_mut()
                                .find(|o| {
                                    o.get("id").and_then(|v| v.as_str())
                                        == Some(object_id.as_str())
                                })
                        {
                            object["status"] = serde_json::Value::String(verified_status.to_string());
                            object["verification"] = serde_json::Value::String(verification.clone());
                        }
                    }
                    _ => {}
                }
            }

            state.ontology.delta_log.push(OntologyDeltaRecord {
                delta_kind: "ontology_verification_applied".to_string(),
                payload: serde_json::json!({
                    "proposal_id": proposal_id,
                    "verification": verification,
                    "outcome": outcome,
                }),
                timestamp: Some(now),
            });
        }
        FocusaEvent::OntologyWorkingSetRefreshed { scope, reason } => {
            let now = Utc::now();
            state
                .ontology
                .working_set_refreshes
                .push(OntologyWorkingSetRefreshRecord {
                    scope: scope.clone(),
                    reason: reason.clone(),
                    timestamp: Some(now),
                });
            let context_set_id = format!("relevant_context_set:{}:{}", scope, reason);
            if let Some(existing) = state
                .ontology
                .objects
                .iter_mut()
                .find(|o| o.get("id").and_then(|v| v.as_str()) == Some(context_set_id.as_str()))
            {
                existing["status"] = serde_json::Value::String("active".to_string());
                existing["scope_kind"] = serde_json::Value::String(scope.clone());
                existing["reason"] = serde_json::Value::String(reason.clone());
                existing["provenance_class"] =
                    serde_json::Value::String("reducer_promoted".to_string());
            } else {
                state.ontology.objects.push(serde_json::json!({
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
                delta_kind: "ontology_working_set_refreshed".to_string(),
                payload: serde_json::json!({
                    "scope": scope,
                    "reason": reason,
                }),
                timestamp: Some(now),
            });
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
            proposal_id,
            status,
        } => {
            let proposal = state
                .pre
                .proposals
                .iter_mut()
                .find(|p| p.id == proposal_id)
                .ok_or_else(|| {
                    ReducerError::InvalidEvent(format!(
                        "Proposal {} not found for status update",
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
    // INVARIANT 1: At most one active Focus Frame exists,
    // and active_id must point to it (or both must be None).
    let active_count = state
        .focus_stack
        .frames
        .iter()
        .filter(|f| f.status == FrameStatus::Active)
        .count();
    if active_count > 1 {
        return Err(ReducerError::InvariantViolation(format!(
            "Multiple active Focus Frames: {} found",
            active_count
        )));
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
            if active_count != 0 {
                return Err(ReducerError::InvariantViolation(format!(
                    "active_id is None but {} frame(s) have Active status",
                    active_count
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
        };
        reduce(state, event).unwrap().new_state
    }

    fn push_frame(state: FocusaState, title: &str) -> (FocusaState, FrameId) {
        let frame_id = Uuid::now_v7();
        let event = FocusaEvent::FocusFramePushed {
            frame_id,
            beads_issue_id: "BEAD-001".into(),
            title: title.into(),
            goal: format!("Goal for {}", title),
            constraints: vec![],
            tags: vec![],
        };
        let state = reduce(state, event).unwrap().new_state;
        (state, frame_id)
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
    fn test_push_empty_beads_id_rejected() {
        let event = FocusaEvent::FocusFramePushed {
            frame_id: Uuid::now_v7(),
            beads_issue_id: "".into(),
            title: "Bad frame".into(),
            goal: "No beads".into(),
            constraints: vec![],
            tags: vec![],
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
            constraints: vec![],
            tags: vec![],
        };
        let result = reduce(state, event);
        assert!(result.is_err());
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
                pinned: false,
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
                pinned: false,
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
                pinned: false,
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
                    pinned: false,
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
                    pinned: false,
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
            tags: vec![],
            priority_hint: None,
            ascc_checkpoint_id: None,
            stats: FrameStats::default(),
            constraints: vec![],
            focus_state: FocusState::default(),
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
}
