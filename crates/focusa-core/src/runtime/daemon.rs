//! Daemon runtime — single-writer event loop.
//!
//! Source: G1-detail-03-runtime-daemon.md
//!
//! Startup sequence:
//!   1. Load config
//!   2. Ensure directories (persistence)
//!   3. Load state snapshot (or create fresh)
//!   4. Open event log
//!   5. Enter event loop
//!
//! Event loop (per action):
//!   1. Receive Action from mpsc channel
//!   2. Translate Action → FocusaEvent(s)
//!   3. For each event: call reducer(state, event)
//!   4. Persist: save state snapshot + append event log
//!   5. Broadcast emitted events (for subscribers)
//!
//! Shutdown:
//!   - Flush persistence
//!   - Close event log cleanly

use crate::reducer::{self, ReducerError};
use crate::reference::store::ReferenceStore;
use crate::runtime::events::create_entry;
use crate::runtime::persistence::Persistence;
use crate::types::*;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// The main daemon handle.
pub struct Daemon {
    config: FocusaConfig,
    state: FocusaState,
    /// Shared state handle — written after every successful reduction so the API
    /// server (and any other reader) always sees current state.
    shared_state: Arc<RwLock<FocusaState>>,
    persistence: Persistence,
    ecs: ReferenceStore,
    command_tx: mpsc::Sender<Action>,
    command_rx: mpsc::Receiver<Action>,
}

impl Daemon {
    /// Create a new daemon, initializing persistence and loading saved state.
    ///
    /// `shared_state` is the read handle that the API server uses. The daemon
    /// writes to it after every successful reduction.
    pub fn new(
        config: FocusaConfig,
        shared_state: Arc<RwLock<FocusaState>>,
    ) -> anyhow::Result<Self> {
        let persistence = Persistence::new(&config)?;
        let ecs_root = persistence.data_dir.join("ecs");
        let ecs = ReferenceStore::new(ecs_root)?;

        // Load existing state or create fresh.
        let state = persistence.load_state()?.unwrap_or_default();

        // Sync loaded state immediately so the API sees it before run() is called.
        // No contention at construction time, so try_write always succeeds.
        {
            let mut shared = shared_state.try_write()
                .expect("no contention at daemon construction");
            *shared = state.clone();
        }

        let (command_tx, command_rx) = mpsc::channel(256);
        Ok(Self {
            config,
            state,
            shared_state,
            persistence,
            ecs,
            command_tx,
            command_rx,
        })
    }

    /// Get a clone of the command sender (for API server, CLI, etc.).
    pub fn command_sender(&self) -> mpsc::Sender<Action> {
        self.command_tx.clone()
    }

    /// Run the main event loop. Blocks until the channel is closed.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        tracing::info!("Focusa daemon starting (version {})", self.state.version);

        while let Some(action) = self.command_rx.recv().await {
            if let Err(e) = self.process_action(action).await {
                tracing::error!("Action processing failed: {}", e);
            }
        }

        // Channel closed — flush final state.
        tracing::info!("Focusa daemon shutting down");
        self.persistence.save_state(&self.state)?;
        Ok(())
    }

    /// Translate an Action to event(s), reduce, persist.
    async fn process_action(&mut self, action: Action) -> anyhow::Result<()> {
        let events = self.translate_action(action).await?;

        for event in events {
            match reducer::reduce(self.state.clone(), event.clone()) {
                Ok(result) => {
                    self.state = result.new_state;

                    // Persist each emitted event to the log.
                    for emitted in &result.emitted_events {
                        let entry =
                            create_entry(emitted.clone(), SignalOrigin::Daemon, None);
                        if let Err(e) = self.persistence.append_event(&entry) {
                            tracing::error!("Failed to persist event: {}", e);
                        }
                    }

                    // Save state snapshot after each reduction.
                    if let Err(e) = self.persistence.save_state(&self.state) {
                        tracing::error!("Failed to save state snapshot: {}", e);
                    }

                    // Sync to shared handle so the API sees the update.
                    self.sync_shared_state().await;
                }
                Err(e) => {
                    tracing::error!("Reducer error: {}", e);
                    self.handle_reducer_error(e)?;
                }
            }
        }

        Ok(())
    }

    /// Translate Action → FocusaEvent(s).
    ///
    /// This is where IDs are generated and command parameters become event data.
    /// The resulting events are deterministic inputs to the reducer.
    async fn translate_action(&mut self, action: Action) -> anyhow::Result<Vec<FocusaEvent>> {
        match action {
            // ─── Session ─────────────────────────────────────────────────

            Action::StartSession {
                adapter_id,
                workspace_id,
            } => Ok(vec![FocusaEvent::SessionStarted {
                session_id: Uuid::now_v7(),
                adapter_id,
                workspace_id,
            }]),

            Action::CloseSession { reason } => {
                Ok(vec![FocusaEvent::SessionClosed { reason }])
            }

            // ─── Focus Stack ─────────────────────────────────────────────

            Action::PushFrame {
                title,
                goal,
                beads_issue_id,
                constraints,
                tags,
            } => Ok(vec![FocusaEvent::FocusFramePushed {
                frame_id: Uuid::now_v7(),
                beads_issue_id,
                title,
                goal,
                constraints,
                tags,
            }]),

            Action::PopFrame { completion_reason } => {
                let frame_id = self.state.focus_stack.active_id.ok_or_else(|| {
                    anyhow::anyhow!("PopFrame but no active frame")
                })?;
                Ok(vec![FocusaEvent::FocusFrameCompleted {
                    frame_id,
                    completion_reason,
                }])
            }

            Action::SetActiveFrame { frame_id: _ } => {
                // Not yet implemented — requires a FocusFrameResumed event type.
                // Previous code suspended the current frame without activating the
                // target, leaving zero active frames (degraded state). A no-op is
                // safer than a half-op. Use push/pop for frame management in MVP.
                tracing::warn!("SetActiveFrame not yet implemented — use push/pop instead");
                Ok(vec![])
            }

            // ─── Gate ────────────────────────────────────────────────────

            Action::IngestSignal { signal } => Ok(vec![
                FocusaEvent::IntuitionSignalObserved {
                    signal_id: signal.id,
                    signal_type: signal.kind,
                    severity: "info".into(),
                    summary: signal.summary,
                    related_frame_id: signal.frame_context,
                },
            ]),

            Action::SurfaceCandidate { candidate_id } => {
                // Find existing candidate to get its data.
                let candidate = self
                    .state
                    .focus_gate
                    .candidates
                    .iter()
                    .find(|c| c.id == candidate_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("Candidate {} not found", candidate_id)
                    })?;
                Ok(vec![FocusaEvent::CandidateSurfaced {
                    candidate_id,
                    kind: candidate.kind,
                    description: candidate.label.clone(),
                    pressure: candidate.pressure,
                    related_frame_id: candidate.related_frame_id,
                }])
            }

            Action::PinCandidate { candidate_id } => {
                Ok(vec![FocusaEvent::CandidatePinned { candidate_id }])
            }

            Action::SuppressCandidate {
                candidate_id,
                scope,
            } => {
                let suppressed_until = parse_suppression_scope(&scope);
                Ok(vec![FocusaEvent::CandidateSuppressed {
                    candidate_id,
                    scope,
                    suppressed_until,
                }])
            }

            // ─── ASCC / Focus State ──────────────────────────────────────

            Action::UpdateCheckpointDelta {
                frame_id,
                turn_id: _,
                delta,
            } => Ok(vec![FocusaEvent::FocusStateUpdated { frame_id, delta }]),

            // ─── ECS ─────────────────────────────────────────────────────

            Action::StoreArtifact {
                kind,
                label,
                content,
            } => {
                let session_id = self.state.session.as_ref().map(|s| s.session_id);
                let handle = self.ecs.store(kind, label.clone(), &content, session_id)?;
                let kind_str = crate::reference::artifact::handle_kind_str(handle.kind);
                Ok(vec![FocusaEvent::ArtifactRegistered {
                    artifact_id: handle.id,
                    artifact_type: kind_str.to_string(),
                    summary: label,
                    storage_uri: format!("ecs://{}", handle.sha256),
                }])
            }

            Action::ResolveHandle { handle_id: _ } => {
                // Resolve is a read operation — no state mutation needed.
                Ok(vec![])
            }

            // ─── Memory ──────────────────────────────────────────────────

            Action::UpsertSemantic {
                key,
                value,
                source,
            } => {
                // Memory ops bypass the reducer (no dedicated event type in MVP).
                crate::memory::semantic::upsert(&mut self.state.memory, key, value, source);
                self.persistence.save_state(&self.state)?;
                self.sync_shared_state().await;
                Ok(vec![])
            }

            Action::ReinforceRule { rule_id } => {
                crate::memory::procedural::reinforce(&mut self.state.memory, &rule_id);
                self.persistence.save_state(&self.state)?;
                self.sync_shared_state().await;
                Ok(vec![])
            }

            Action::DecayTick => {
                crate::memory::procedural::decay_tick(
                    &mut self.state.memory,
                    self.config.gate_decay_factor,
                );
                crate::gate::focus_gate::decay_candidates(
                    &mut self.state.focus_gate,
                    self.config.gate_decay_factor,
                );
                self.persistence.save_state(&self.state)?;
                self.sync_shared_state().await;
                Ok(vec![])
            }

            // ─── Workers ─────────────────────────────────────────────────

            Action::WorkerEnqueue { job: _ } => {
                // Worker scheduling is handled outside the reducer.
                // TODO: Forward to worker queue when implemented.
                Ok(vec![])
            }

            Action::WorkerComplete { job_id: _ } => {
                // Worker completion results are translated to other actions
                // (e.g., UpsertSemantic, UpdateCheckpointDelta) by the worker runner.
                Ok(vec![])
            }
        }
    }

    /// Sync internal state to the shared handle for API readers.
    async fn sync_shared_state(&self) {
        let mut shared = self.shared_state.write().await;
        *shared = self.state.clone();
    }

    /// Handle reducer errors — log an InvariantViolation event.
    fn handle_reducer_error(&mut self, error: ReducerError) -> anyhow::Result<()> {
        let violation_event = FocusaEvent::InvariantViolation {
            invariant: "reducer".into(),
            details: error.to_string(),
        };
        let entry = create_entry(violation_event, SignalOrigin::Daemon, None);
        self.persistence.append_event(&entry)?;
        Ok(())
    }
}

/// Parse a suppression scope into a concrete deadline.
///
/// Supported formats:
///   - `"session"` → None (permanent for this session)
///   - `"<n>s"` → now + n seconds
///   - `"<n>m"` → now + n minutes
///   - `"<n>h"` → now + n hours
///   - Unrecognized → None (treated as session scope, logged as warning)
///
/// The deadline is computed here (command-side) and stored in the event,
/// so replay produces the same result regardless of wall-clock time.
fn parse_suppression_scope(scope: &str) -> Option<DateTime<Utc>> {
    if scope == "session" {
        return None;
    }
    if scope.len() < 2 || !scope.is_ascii() {
        tracing::warn!("Unrecognized suppression scope '{}', treating as session", scope);
        return None;
    }
    let (num_str, unit) = scope.split_at(scope.len() - 1);
    let num: i64 = match num_str.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            tracing::warn!("Unrecognized suppression scope '{}', treating as session", scope);
            return None;
        }
    };
    let seconds = match unit {
        "s" => num,
        "m" => num * 60,
        "h" => num * 3600,
        _ => {
            tracing::warn!("Unrecognized suppression scope '{}', treating as session", scope);
            return None;
        }
    };
    Some(Utc::now() + chrono::Duration::seconds(seconds))
}
