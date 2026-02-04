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
use tokio::sync::mpsc;
use uuid::Uuid;

/// The main daemon handle.
pub struct Daemon {
    config: FocusaConfig,
    state: FocusaState,
    persistence: Persistence,
    ecs: ReferenceStore,
    pub command_tx: mpsc::Sender<Action>,
    command_rx: mpsc::Receiver<Action>,
}

impl Daemon {
    /// Create a new daemon, initializing persistence and loading saved state.
    pub fn new(config: FocusaConfig) -> anyhow::Result<Self> {
        let persistence = Persistence::new(&config)?;
        let ecs_root = persistence.data_dir.join("ecs");
        let ecs = ReferenceStore::new(ecs_root)?;

        // Load existing state or create fresh.
        let state = persistence.load_state()?.unwrap_or_default();

        let (command_tx, command_rx) = mpsc::channel(256);
        Ok(Self {
            config,
            state,
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

    /// Read-only access to current state.
    pub fn state(&self) -> &FocusaState {
        &self.state
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
        let events = self.translate_action(action)?;

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
    fn translate_action(&mut self, action: Action) -> anyhow::Result<Vec<FocusaEvent>> {
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

            Action::SetActiveFrame { frame_id } => {
                // Setting active frame = suspend current + directly activate target.
                // For MVP: suspend current, then the API layer must push/resume explicitly.
                let mut events = Vec::new();
                if let Some(current_id) = self.state.focus_stack.active_id {
                    if current_id != frame_id {
                        events.push(FocusaEvent::FocusFrameSuspended {
                            frame_id: current_id,
                            reason: format!("Switching to frame {}", frame_id),
                        });
                    }
                }
                // TODO: Resume target frame (needs a FocusFrameResumed event or similar).
                // For now, SetActiveFrame only suspends the current frame.
                Ok(events)
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

            Action::SuppressCandidate {
                candidate_id,
                scope,
            } => Ok(vec![FocusaEvent::CandidateSuppressed {
                candidate_id,
                scope,
            }]),

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
                Ok(vec![FocusaEvent::ArtifactRegistered {
                    artifact_id: handle.id,
                    artifact_type: format!("{:?}", handle.kind).to_lowercase(),
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
                // Memory upsert doesn't have a dedicated event type.
                // Apply directly via a synthetic FocusStateUpdated or handle in-loop.
                // For MVP: mutate state directly (bypass reducer for memory ops).
                crate::memory::semantic::upsert(&mut self.state.memory, key, value, source);
                self.persistence.save_state(&self.state)?;
                Ok(vec![])
            }

            Action::ReinforceRule { rule_id } => {
                crate::memory::procedural::reinforce(&mut self.state.memory, &rule_id);
                self.persistence.save_state(&self.state)?;
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
