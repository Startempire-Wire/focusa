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
//!   5. Run intuition engine observers
//!   6. Drain intuition signals → IngestSignal actions
//!
//! Shutdown:
//!   - Flush persistence
//!   - Close event log cleanly

use crate::intuition::engine::IntuitionEngine;

const ACTIVE_TURN_TTL_SECS: i64 = 1800;
use crate::reducer::{self, ReducerError};
use crate::reference::store::ReferenceStore;
use crate::runtime::events::create_entry;
use crate::runtime::persistence_sqlite::SqlitePersistence as Persistence;
use crate::types::*;
use crate::workers::{executor, priority_queue};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

/// The main daemon handle.
pub struct Daemon {
    config: FocusaConfig,
    state: FocusaState,
    /// This daemon's machine ID (for ownership enforcement).
    machine_id: String,
    current_instance_id: Option<Uuid>,
    current_thread_id: Option<Uuid>,
    /// Shared state handle — written after every successful reduction so the API
    /// server (and any other reader) always sees current state.
    shared_state: Arc<RwLock<FocusaState>>,
    persistence: Persistence,
    ecs: ReferenceStore,
    intuition: IntuitionEngine,
    /// ASCC checkpoints per frame (G1-07-ascc).
    /// Persisted via state snapshot; keyed by frame_id.
    checkpoints: std::collections::HashMap<Uuid, crate::types::CheckpointRecord>,
    /// Cache store (docs/18-19).
    cache: crate::cache::CacheStore,
    /// Receive signals from the intuition engine.
    signal_rx: mpsc::Receiver<Signal>,
    command_tx: mpsc::Sender<Action>,
    command_rx: mpsc::Receiver<Action>,
    worker_tx: priority_queue::PrioritySender,
    worker_rx: priority_queue::PriorityReceiver,
    event_bus: Option<crate::runtime::event_bus::EventBus>,
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
            let mut shared = shared_state
                .try_write()
                .expect("no contention at daemon construction");
            *shared = state.clone();
        }

        // Intuition engine signal channel (bounded, non-blocking sender).
        let (signal_tx, signal_rx) = mpsc::channel(64);
        let intuition = IntuitionEngine::new(signal_tx);

        let (command_tx, command_rx) = mpsc::channel(256);
        let (worker_tx, worker_rx) = priority_queue::priority_channel(config.worker_queue_size);

        // Get this daemon's machine ID for ownership enforcement.
        let machine_id = persistence.machine_id()?;

        Ok(Self {
            config,
            state,
            machine_id,
            current_instance_id: None,
            current_thread_id: None,
            shared_state,
            persistence,
            ecs,
            intuition,
            signal_rx,
            command_tx,
            command_rx,
            worker_tx,
            worker_rx,
            checkpoints: std::collections::HashMap::new(),
            cache: crate::cache::CacheStore::new(),
            event_bus: None,
        })
    }

    /// Get a clone of the command sender (for API server, CLI, etc.).
    pub fn command_sender(&self) -> mpsc::Sender<Action> {
        self.command_tx.clone()
    }

    /// Get a clone of the persistence handle (for API server sync routes).
    pub fn persistence(&self) -> Persistence {
        self.persistence.clone()
    }

    /// Attach an in-process event bus (used for SSE).
    pub fn attach_event_bus(&mut self, bus: crate::runtime::event_bus::EventBus) {
        self.event_bus = Some(bus);
    }

    /// Run the main event loop. Blocks until the channel is closed.
    ///
    /// Processes actions from the command channel and runs a periodic
    /// decay tick every 30 seconds (pressure decay + rule weight decay).
    pub async fn run(&mut self) -> anyhow::Result<()> {
        tracing::info!("Focusa daemon starting (version {})", self.state.version);

        // Seed default constitution on first start (docs/16 §2-§6).
        crate::constitution::seed_default(&mut self.state.constitution);

        let mut decay_interval = tokio::time::interval(std::time::Duration::from_secs(30));
        // Don't fire immediately on startup — first tick is a no-op.
        decay_interval.tick().await;

        loop {
            tokio::select! {
                action = self.command_rx.recv() => {
                    match action {
                        Some(action) => {
                            if let Err(e) = self.process_action(action).await {
                                tracing::error!("Action processing failed: {}", e);
                            }
                            self.drain_intuition_signals().await;
                            self.run_gate_pipeline();
                        }
                        None => break, // Channel closed.
                    }
                }
                job = self.worker_rx.recv() => {
                    if let Some(job) = job {
                        if let Err(e) = self.handle_worker_job(job).await {
                            tracing::error!("Worker job failed: {}", e);
                        }
                        self.drain_intuition_signals().await;
                        self.run_gate_pipeline();
                    }
                }
                _ = decay_interval.tick() => {
                    // Periodic decay tick — reduces candidate pressure and rule weights.
                    if let Err(e) = self.process_action(Action::DecayTick).await {
                        tracing::debug!("Decay tick failed: {}", e);
                    }
                    // Run gate pipeline after decay to re-check surfacing thresholds.
                    self.run_gate_pipeline();
                }
            }
        }

        // Channel closed — flush final state.
        tracing::info!("Focusa daemon shutting down");
        self.persistence.save_state(&self.state)?;
        Ok(())
    }

    /// Translate an Action to event(s), reduce, persist, observe.
    async fn process_action(&mut self, action: Action) -> anyhow::Result<()> {
        // Track whether this action touches the focus stack (for observe_stack).
        let is_stack_action = matches!(
            action,
            Action::PushFrame { .. } | Action::PopFrame { .. } | Action::SetActiveFrame { .. }
        );

        let events = self.translate_action(action).await?;

        for event in events {
            // Determine thread_id for ownership enforcement.
            // current_thread_id is set during ThreadAttach actions.
            let thread_id = self.current_thread_id;

            // Use reduce_with_meta for ownership enforcement (Policy #5).
            match reducer::reduce_with_meta(
                self.state.clone(),
                event.clone(),
                Some(&self.machine_id),
                thread_id,
                false, // Daemon events are never observations
            ) {
                Ok(result) => {
                    self.state = result.new_state;

                    // Persist each emitted event to the log.
                    for emitted in &result.emitted_events {
                        let mut entry = create_entry(emitted.clone(), SignalOrigin::Daemon, None);
                        entry.instance_id = self.current_instance_id;
                        entry.thread_id = self.current_thread_id;
                        entry.session_id = self.state.session.as_ref().map(|s| s.session_id);

                        if let Err(e) = self.persistence.append_event(&entry) {
                            tracing::error!("Failed to persist event: {}", e);
                        } else if let Ok(json) = serde_json::to_string(&entry)
                            && let Some(bus) = &self.event_bus
                        {
                            bus.publish(json);
                        }
                    }

                    // Intuition: observe turn completions for signals.
                    if let FocusaEvent::TurnCompleted {
                        ref turn_id,
                        ref assistant_output,
                        ref raw_user_input,
                        ref errors,
                        ref prompt_tokens,
                        ref completion_tokens,
                        ..
                    } = event
                    {
                        let frame_id = self.state.focus_stack.active_id;
                        if let Some(output) = assistant_output.as_deref() {
                            if !output.is_empty() {
                                self.intuition.observe_turn(frame_id, output);
                            }
                        }
                        for err in errors {
                            self.intuition.observe_turn(frame_id, err);
                        }

                        // Background workers: enqueue jobs per G1-10-workers.
                        // Workers are advisory — results flow through
                        // apply_worker_result → FocusStateDelta / gate signals.
                        self.enqueue_turn_workers(
                            turn_id,
                            frame_id,
                            assistant_output.as_deref(),
                            raw_user_input.as_deref(),
                            errors,
                        ).await;

                        // Autonomy: record observation from turn (docs/12, docs/37).
                        let had_errors = !errors.is_empty();
                        let focus_populated = frame_id
                            .and_then(|fid| {
                                self.state.focus_stack.frames.iter().find(|f| f.id == fid)
                            })
                            .map(|f| !f.focus_state.intent.is_empty() || !f.focus_state.decisions.is_empty())
                            .unwrap_or(false);
                        let stack_depth = self.state.focus_stack.stack_path_cache.len();
                        let pt = prompt_tokens.unwrap_or(0);
                        let ct = completion_tokens.unwrap_or(0);
                        crate::autonomy::observe_turn(
                            &mut self.state.autonomy,
                            had_errors,
                            focus_populated,
                            stack_depth,
                            pt,
                            ct,
                        );

                        // RFM: run validators on assistant output (docs/36 §6).
                        if let Some(output) = assistant_output.as_deref() {
                            if !output.is_empty() {
                                let frame_constraints: Vec<String> = frame_id
                                    .and_then(|fid| {
                                        self.state.focus_stack.frames.iter().find(|f| f.id == fid)
                                    })
                                    .map(|f| f.constraints.clone())
                                    .unwrap_or_default();
                                let results =
                                    crate::rfm::validate(output, &frame_constraints);
                                let level_changed =
                                    crate::rfm::update_rfm(&mut self.state.rfm, results);
                                if level_changed {
                                    tracing::info!(
                                        level = ?self.state.rfm.level,
                                        ais = self.state.rfm.ais_score,
                                        "RFM level changed"
                                    );
                                }
                            }
                        }

                        // UFI/UXP: detect friction signals from user input (docs/14).
                        if let Some(input) = raw_user_input.as_deref() {
                            if !input.is_empty() {
                                let ufi_signals =
                                    crate::workers::executor::detect_ufi_signals(input);
                                let session_id =
                                    self.state.session.as_ref().map(|s| s.session_id);
                                for sig_type in &ufi_signals {
                                    crate::uxp::record_ufi_signal(
                                        &mut self.state.ufi,
                                        *sig_type,
                                        session_id,
                                    );
                                }
                                // Bridge UFI → UXP if signals detected.
                                if !ufi_signals.is_empty() {
                                    let ufi_agg = self.state.ufi.aggregate;
                                    // Update interruption_sensitivity dimension
                                    // (most directly affected by user friction).
                                    crate::uxp::bridge_ufi_to_uxp(
                                        &mut self.state.uxp.interruption_sensitivity,
                                        ufi_agg,
                                    );
                                }
                            }
                        }
                    }

                    // ECS: auto-externalize large turn content (G1-detail-08 §Threshold Policy).
                    // "ecs.externalize_bytes_threshold default 8KB,
                    //  ecs.externalize_token_estimate_threshold default 800 tokens.
                    //  If either exceeded, externalize."
                    if let FocusaEvent::TurnCompleted {
                        ref turn_id,
                        ref assistant_output,
                        ..
                    } = event
                    {
                        if let Some(output) = assistant_output.as_deref() {
                            let bytes = output.len() as u64;
                            let est_tokens = (bytes / 4) as u32;
                            if bytes > self.config.ecs_externalize_bytes_threshold
                                || est_tokens > self.config.ecs_externalize_token_threshold
                            {
                                let label = format!("turn-output-{}", turn_id);
                                match self.ecs.store(
                                    HandleKind::Text,
                                    label.clone(),
                                    output.as_bytes(),
                                    self.state.session.as_ref().map(|s| s.session_id),
                                ) {
                                    Ok(handle) => {
                                        tracing::info!(
                                            turn_id = %turn_id,
                                            bytes,
                                            handle_id = %handle.id,
                                            "Auto-externalized large turn output to ECS"
                                        );
                                        // Register handle in state via reducer.
                                        let kind_str = crate::reference::artifact::handle_kind_str(handle.kind);
                                        let reg_event = FocusaEvent::ArtifactRegistered {
                                            artifact_id: handle.id,
                                            artifact_type: kind_str.to_string(),
                                            summary: label,
                                            storage_uri: format!("ecs://{}", handle.sha256),
                                        };
                                        if let Ok(result) = reducer::reduce(
                                            self.state.clone(),
                                            reg_event.clone(),
                                        ) {
                                            self.state = result.new_state;
                                            let entry = create_entry(reg_event, SignalOrigin::Daemon, None);
                                            let _ = self.persistence.append_event(&entry);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("ECS auto-externalize failed: {}", e);
                                    }
                                }
                            }
                        }
                    }

                    // ASCC: update checkpoint after FocusState changes (G1-07).
                    if let FocusaEvent::FocusStateUpdated { frame_id, .. } = &event {
                        self.update_checkpoint(*frame_id);
                        // docs/18 §6: Focus State revision changed → bust C1/C2.
                        self.cache.bust(CacheBustCategory::FreshEvidence);
                    }

                    // CLT: track interaction nodes for each event.
                    self.track_clt_event(&event);

                    // Telemetry: record each event.
                    self.state.telemetry.total_events += 1;

                    // Save state snapshot (after all mutations so CLT + telemetry are captured).
                    if let Err(e) = self.persistence.save_state(&self.state) {
                        tracing::error!("Failed to save state snapshot: {}", e);
                    }

                    // Sync to shared handle so the API sees all updates.
                    self.sync_shared_state().await;
                }
                Err(e) => {
                    tracing::error!("Reducer error: {}", e);
                    self.handle_reducer_error(e)?;
                }
            }
        }

        // Post-action: run intuition observers.
        if is_stack_action {
            self.intuition.observe_stack(&self.state.focus_stack);
            // docs/18 §6: Focus Stack push/pop → bust C1/C2 caches.
            self.cache.bust(CacheBustCategory::AuthorityChange);
        }

        Ok(())
    }

    /// Drain signals from the intuition engine and ingest them as events.
    ///
    /// Uses try_recv to avoid blocking — drains all available signals in one pass.
    async fn drain_intuition_signals(&mut self) {
        while let Ok(signal) = self.signal_rx.try_recv() {
            let event = FocusaEvent::IntuitionSignalObserved {
                signal_id: signal.id,
                signal_type: signal.kind,
                severity: "info".into(),
                summary: signal.summary.clone(),
                related_frame_id: signal.frame_context,
            };
            // Use reduce_with_meta for ownership enforcement.
            match reducer::reduce_with_meta(
                self.state.clone(),
                event.clone(),
                Some(&self.machine_id),
                self.current_thread_id,
                false,
            ) {
                Ok(result) => {
                    self.state = result.new_state;
                    for emitted in &result.emitted_events {
                        let mut entry = create_entry(emitted.clone(), SignalOrigin::Daemon, None);
                        entry.instance_id = self.current_instance_id;
                        entry.thread_id = self.current_thread_id;
                        entry.session_id = self.state.session.as_ref().map(|s| s.session_id);

                        if let Err(e) = self.persistence.append_event(&entry) {
                            tracing::error!("Failed to persist intuition signal: {}", e);
                        } else if let Ok(json) = serde_json::to_string(&entry)
                            && let Some(bus) = &self.event_bus
                        {
                            bus.publish(json);
                        }
                    }

                    // Same post-reduction bookkeeping as process_action.
                    self.track_clt_event(&event);
                    self.state.telemetry.total_events += 1;

                    if let Err(e) = self.persistence.save_state(&self.state) {
                        tracing::error!("Failed to save state after intuition signal: {}", e);
                    }
                    self.sync_shared_state().await;
                }
                Err(e) => {
                    tracing::warn!("Intuition signal rejected by reducer: {}", e);
                }
            }
        }
    }

    /// Run the Focus Gate 5-step pipeline (G1-detail-06).
    ///
    /// Aggregates signals into candidates, applies pressure modifiers,
    /// surfaces candidates above threshold. Called after signal ingestion
    /// and on periodic decay tick.
    fn run_gate_pipeline(&mut self) {
        let active_id = self.state.focus_stack.active_id;
        let stack_path = self.state.focus_stack.stack_path_cache.clone();
        let threshold = self.config.gate_surface_threshold;

        let newly_surfaced = crate::gate::focus_gate::run_gate_pipeline(
            &mut self.state.focus_gate,
            active_id,
            &stack_path,
            threshold,
        );

        if newly_surfaced > 0 {
            tracing::info!(
                newly_surfaced,
                total_candidates = self.state.focus_gate.candidates.len(),
                "Focus Gate: candidates surfaced"
            );
        }
    }

    /// Create or update the ASCC checkpoint for a frame after FocusState changes.
    ///
    /// Per G1-07: "revision += 1, anchor_turn_id = turn_id, updated_at = now.
    /// Emit event: ascc.delta_applied."
    ///
    /// Checkpoint is derived from the frame's live FocusState.
    fn update_checkpoint(&mut self, frame_id: FrameId) {
        let frame = match self.state.focus_stack.frames.iter().find(|f| f.id == frame_id) {
            Some(f) => f,
            None => return,
        };

        // Skip if FocusState is completely empty (no content to checkpoint).
        let sections = AsccSections::from(&frame.focus_state);
        if sections.is_empty() {
            return;
        }

        let turn_id = self
            .state
            .active_turn
            .as_ref()
            .map(|t| t.turn_id.clone())
            .unwrap_or_else(|| format!("daemon-{}", self.state.version));

        match self.checkpoints.get_mut(&frame_id) {
            Some(cp) => {
                cp.update_from_frame(frame, &turn_id);
                tracing::debug!(
                    frame_id = %frame_id,
                    revision = cp.revision,
                    anchor = %cp.anchor_turn_id,
                    "ASCC checkpoint updated"
                );
                // Persist to file (G1-07 §Persistence).
                if let Err(e) = crate::ascc::save_checkpoint(&self.config.data_dir, cp) {
                    tracing::warn!("Failed to persist ASCC checkpoint: {}", e);
                }
            }
            None => {
                let cp = CheckpointRecord::from_frame(frame, &turn_id);
                tracing::info!(
                    frame_id = %frame_id,
                    revision = cp.revision,
                    "ASCC checkpoint created"
                );
                // Persist to file (G1-07 §Persistence).
                if let Err(e) = crate::ascc::save_checkpoint(&self.config.data_dir, &cp) {
                    tracing::warn!("Failed to persist ASCC checkpoint: {}", e);
                }
                self.checkpoints.insert(frame_id, cp);
            }
        }
    }

    /// Get the ASCC checkpoint for a frame (if it exists).
    pub fn checkpoint_for_frame(&self, frame_id: FrameId) -> Option<&CheckpointRecord> {
        self.checkpoints.get(&frame_id)
    }

    /// Enqueue background worker jobs after a turn completes.
    ///
    /// Per G1-10-workers: workers are advisory, async, non-blocking.
    /// Job kinds per spec: extract_ascc_delta, scan_for_errors,
    /// detect_repetition, classify_turn, suggest_memory.
    ///
    /// Content is passed via correlation_id (MVP inline transport).
    /// Worker results flow through handle_worker_job → apply_worker_result.
    ///
    /// Jobs are queued by priority (High > Normal > Low) per G1-10 §Scheduling.
    async fn enqueue_turn_workers(
        &self,
        turn_id: &str,
        frame_id: Option<FrameId>,
        assistant_output: Option<&str>,
        raw_user_input: Option<&str>,
        _errors: &[String],
    ) {
        let now = Utc::now();
        let timeout = self.config.worker_job_timeout_ms;

        // extract_ascc_delta: extract decisions/constraints/failures/next_steps
        // from assistant output into structured FocusStateDelta.
        if let Some(output) = assistant_output {
            if !output.is_empty() {
                let job = WorkerJob {
                    id: Uuid::now_v7(),
                    kind: WorkerJobKind::ExtractAsccDelta,
                    created_at: now,
                    priority: JobPriority::High,
                    payload_ref: None,
                    frame_context: frame_id,
                    correlation_id: Some(output.chars().take(4000).collect()),
                    timeout_ms: timeout,
                };
                if !self.worker_tx.try_send(job).await {
                    tracing::warn!(turn_id, "Worker queue full: dropped ExtractAsccDelta job");
                }
            }
        }

        // scan_for_errors: detect error patterns in assistant output.
        if let Some(output) = assistant_output {
            if !output.is_empty() {
                let job = WorkerJob {
                    id: Uuid::now_v7(),
                    kind: WorkerJobKind::ScanForErrors,
                    created_at: now,
                    priority: JobPriority::Normal,
                    payload_ref: None,
                    frame_context: frame_id,
                    correlation_id: Some(output.chars().take(4000).collect()),
                    timeout_ms: timeout,
                };
                if !self.worker_tx.try_send(job).await {
                    tracing::warn!(turn_id, "Worker queue full: dropped ScanForErrors job");
                }
            }
        }

        // detect_repetition: check for repeated content patterns.
        if let Some(output) = assistant_output {
            if !output.is_empty() {
                let job = WorkerJob {
                    id: Uuid::now_v7(),
                    kind: WorkerJobKind::DetectRepetition,
                    created_at: now,
                    priority: JobPriority::Low,
                    payload_ref: None,
                    frame_context: frame_id,
                    correlation_id: Some(output.chars().take(4000).collect()),
                    timeout_ms: timeout,
                };
                if !self.worker_tx.try_send(job).await {
                    tracing::debug!(turn_id, "Worker queue full: dropped DetectRepetition job");
                }
            }
        }

        // classify_turn: classify user input as task/question/correction/meta.
        if let Some(input) = raw_user_input {
            if !input.is_empty() {
                let job = WorkerJob {
                    id: Uuid::now_v7(),
                    kind: WorkerJobKind::ClassifyTurn,
                    created_at: now,
                    priority: JobPriority::Low,
                    payload_ref: None,
                    frame_context: frame_id,
                    correlation_id: Some(input.chars().take(2000).collect()),
                    timeout_ms: timeout,
                };
                if !self.worker_tx.try_send(job).await {
                    tracing::debug!(turn_id, "Worker queue full: dropped ClassifyTurn job");
                }
            }
        }

        // suggest_memory: look for stable patterns worth remembering.
        // Only run if there's substantial output.
        if let Some(output) = assistant_output {
            if output.len() > 200 {
                let job = WorkerJob {
                    id: Uuid::now_v7(),
                    kind: WorkerJobKind::SuggestMemory,
                    created_at: now,
                    priority: JobPriority::Low,
                    payload_ref: None,
                    frame_context: frame_id,
                    correlation_id: Some(output.chars().take(4000).collect()),
                    timeout_ms: timeout,
                };
                if !self.worker_tx.try_send(job).await {
                    tracing::debug!(turn_id, "Worker queue full: dropped SuggestMemory job");
                }
            }
        }
    }

    /// Translate Action → FocusaEvent(s).
    ///
    /// This is where IDs are generated and command parameters become event data.
    /// The resulting events are deterministic inputs to the reducer.
    async fn translate_action(&mut self, action: Action) -> anyhow::Result<Vec<FocusaEvent>> {
        match action {
            // ─── Session ─────────────────────────────────────────────────
            Action::InstanceConnect { kind } => {
                let instance_id = Uuid::now_v7();
                self.current_instance_id = Some(instance_id);
                Ok(vec![FocusaEvent::InstanceConnected { instance_id, kind }])
            }

            Action::InstanceDisconnect {
                instance_id,
                reason,
            } => {
                if self.current_instance_id == Some(instance_id) {
                    self.current_instance_id = None;
                }
                Ok(vec![FocusaEvent::InstanceDisconnected {
                    instance_id,
                    reason,
                }])
            }

            Action::StartSession {
                adapter_id,
                workspace_id,
                instance_id,
            } => {
                if instance_id.is_some() {
                    self.current_instance_id = instance_id;
                }
                Ok(vec![FocusaEvent::SessionStarted {
                    session_id: Uuid::now_v7(),
                    adapter_id,
                    workspace_id,
                }])
            }

            Action::CloseSession {
                reason,
                instance_id: _,
            } => Ok(vec![FocusaEvent::SessionClosed { reason }]),

            Action::ThreadAttach {
                instance_id,
                session_id,
                thread_id,
                role,
            } => {
                self.current_instance_id = Some(instance_id);
                self.current_thread_id = Some(thread_id);
                Ok(vec![FocusaEvent::ThreadAttached {
                    instance_id,
                    session_id,
                    thread_id,
                    role,
                }])
            }

            Action::ThreadDetach {
                instance_id,
                session_id,
                thread_id,
                reason,
            } => Ok(vec![FocusaEvent::ThreadDetached {
                instance_id,
                session_id,
                thread_id,
                reason,
            }]),

            Action::SubmitProposal {
                kind,
                source,
                payload,
                deadline_ms,
            } => {
                crate::pre::submit(&mut self.state.pre, kind, &source, payload, deadline_ms);
                // Proposals don't produce reducer events — they live in PRE state.
                // Persist so proposals survive a daemon restart.
                self.persistence.save_state(&self.state)?;
                self.sync_shared_state().await;
                Ok(vec![])
            }

            Action::EmitEvent { event } => {
                // Direct event emission from API routes.
                Ok(vec![event])
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
                let frame_id = self
                    .state
                    .focus_stack
                    .active_id
                    .ok_or_else(|| anyhow::anyhow!("PopFrame but no active frame"))?;
                // Clean up intuition engine state for this frame.
                self.intuition.clear_frame(frame_id);
                Ok(vec![FocusaEvent::FocusFrameCompleted {
                    frame_id,
                    completion_reason,
                }])
            }

            Action::SetActiveFrame { frame_id } => {
                Ok(vec![FocusaEvent::FocusFrameResumed { frame_id }])
            }

            // ─── Gate ────────────────────────────────────────────────────
            Action::IngestSignal { signal } => Ok(vec![FocusaEvent::IntuitionSignalObserved {
                signal_id: signal.id,
                signal_type: signal.kind,
                severity: "info".into(),
                summary: signal.summary,
                related_frame_id: signal.frame_context,
            }]),

            Action::SurfaceCandidate {
                candidate_id,
                boost,
            } => {
                // Find and boost candidate pressure.
                let candidate = self
                    .state
                    .focus_gate
                    .candidates
                    .iter_mut()
                    .find(|c| c.id == candidate_id)
                    .ok_or_else(|| anyhow::anyhow!("Candidate {} not found", candidate_id))?;
                candidate.pressure += boost;
                let event = FocusaEvent::CandidateSurfaced {
                    candidate_id,
                    kind: candidate.kind,
                    description: candidate.label.clone(),
                    pressure: candidate.pressure,
                    related_frame_id: candidate.related_frame_id,
                };
                Ok(vec![event])
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
            // Memory ops mutate state directly (outside reducer) but emit audit
            // events for observability per G1-detail-15 §memory.semantic_upserted,
            // memory.rule_reinforced, memory.decay_tick.
            Action::UpsertSemantic { key, value, source } => {
                crate::memory::semantic::upsert(
                    &mut self.state.memory,
                    key.clone(),
                    value.clone(),
                    source,
                );
                self.persistence.save_state(&self.state)?;
                self.sync_shared_state().await;
                // Emit audit event to event log (not through reducer).
                self.emit_memory_event(&format!("memory.semantic_upserted: {}={}", key, value));
                Ok(vec![])
            }

            Action::ReinforceRule { rule_id } => {
                crate::memory::procedural::reinforce(&mut self.state.memory, &rule_id);
                self.persistence.save_state(&self.state)?;
                self.sync_shared_state().await;
                self.emit_memory_event(&format!("memory.rule_reinforced: {}", rule_id));
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
                self.expire_stale_turn();
                Ok(vec![])
            }

            // ─── Workers ─────────────────────────────────────────────────
            Action::WorkerEnqueue { job } => {
                let enqueued = self.worker_tx.try_send(job.clone()).await;
                if enqueued {
                    Ok(vec![FocusaEvent::WorkerJobEnqueued {
                        job_id: job.id,
                        kind: job.kind,
                        correlation_id: job.correlation_id.clone(),
                    }])
                } else {
                    Ok(vec![FocusaEvent::WorkerJobFailed {
                        job_id: job.id,
                        kind: job.kind,
                        duration_ms: 0,
                        error: "worker queue full".to_string(),
                    }])
                }
            }

            Action::WorkerComplete { job_id: _ } => {
                // Completion is handled by the worker runner; no reducer mutation.
                Ok(vec![])
            }
        }
    }

    /// Emit a memory audit event to the event log.
    ///
    /// Per G1-detail-15: memory.semantic_upserted, memory.rule_reinforced,
    /// memory.decay_tick must appear in the event log for replay observability.
    /// These bypass the reducer but are persisted for auditability.
    fn emit_memory_event(&self, details: &str) {
        let event = FocusaEvent::InvariantViolation {
            invariant: "memory_audit".into(),
            details: details.to_string(),
        };
        let mut entry = create_entry(event, SignalOrigin::Daemon, None);
        entry.instance_id = self.current_instance_id;
        entry.thread_id = self.current_thread_id;
        entry.session_id = self.state.session.as_ref().map(|s| s.session_id);
        if let Err(e) = self.persistence.append_event(&entry) {
            tracing::warn!("Failed to persist memory audit event: {}", e);
        }
    }

    fn expire_stale_turn(&mut self) {
        if let Some(turn) = &self.state.active_turn {
            let age = Utc::now() - turn.started_at;
            if age.num_seconds() > ACTIVE_TURN_TTL_SECS {
                tracing::warn!(turn_id = %turn.turn_id, age_secs = age.num_seconds(), "Expiring stale active_turn");
                self.state.active_turn = None;
                if let Err(e) = self.persistence.save_state(&self.state) {
                    tracing::error!("Failed to save state after expiring active_turn: {}", e);
                }
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
        if let Ok(json) = serde_json::to_string(&entry)
            && let Some(bus) = &self.event_bus
        {
            bus.publish(json);
        }
        Ok(())
    }

    async fn handle_worker_job(&mut self, job: WorkerJob) -> anyhow::Result<()> {
        let started = Instant::now();
        let job_id = job.id;
        let job_kind = job.kind;
        let _ = self
            .process_action(Action::EmitEvent {
                event: FocusaEvent::WorkerJobStarted {
                    job_id,
                    kind: job_kind,
                },
            })
            .await;

        // Enforce timeout per G1-10 §Job Execution Rules.
        // Default 200ms timeout if not specified.
        let timeout_ms = if job.timeout_ms > 0 {
            job.timeout_ms
        } else {
            200
        };
        let timeout_duration = std::time::Duration::from_millis(timeout_ms);

        // Clone job for execution (to avoid move issues).
        let exec_job = job.clone();
        let result = match tokio::time::timeout(timeout_duration, async move {
            executor::execute_job(&exec_job)
        }).await {
            Ok(result) => result,
            Err(_) => {
                // Timeout occurred.
                let duration_ms = started.elapsed().as_millis() as u64;
                let _ = self
                    .process_action(Action::EmitEvent {
                        event: FocusaEvent::WorkerJobFailed {
                            job_id,
                            kind: job_kind,
                            duration_ms,
                            error: format!("timeout after {}ms", timeout_ms),
                        },
                    })
                    .await;
                return Err(anyhow::anyhow!("Worker job timed out after {}ms", timeout_ms));
            }
        };

        let duration_ms = started.elapsed().as_millis() as u64;

        if result.success {
            let _ = self
                .process_action(Action::EmitEvent {
                    event: FocusaEvent::WorkerJobCompleted {
                        job_id,
                        kind: job_kind,
                        duration_ms,
                    },
                })
                .await;
            self.apply_worker_result(&job, &result).await?;
        } else {
            let _ = self
                .process_action(Action::EmitEvent {
                    event: FocusaEvent::WorkerJobFailed {
                        job_id,
                        kind: job_kind,
                        duration_ms,
                        error: result.payload.get("error").and_then(|v| v.as_str()).unwrap_or("unknown error").to_string(),
                    },
                })
                .await;
        }

        Ok(())
    }

    async fn apply_worker_result(
        &mut self,
        job: &WorkerJob,
        result: &executor::JobResult,
    ) -> anyhow::Result<()> {
        match job.kind {
            WorkerJobKind::ExtractAsccDelta => {
                if let Some(frame_id) = job.frame_context {
                    // Helper: extract string array from JSON payload.
                    let extract_strings = |key: &str| -> Option<Vec<String>> {
                        result
                            .payload
                            .get(key)
                            .and_then(|v| v.as_array())
                            .map(|v| {
                                v.iter()
                                    .filter_map(|d| d.as_str().map(String::from))
                                    .filter(|s| !s.is_empty())
                                    .collect()
                            })
                            .filter(|v: &Vec<String>| !v.is_empty())
                    };

                    // Extract current_state as a string.
                    let current_state = result
                        .payload
                        .get("current_state")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(String::from);

                    let delta = FocusStateDelta {
                        current_state,
                        decisions: extract_strings("decisions"),
                        next_steps: extract_strings("next_steps"),
                        constraints: extract_strings("constraints"),
                        failures: extract_strings("failures"),
                        open_questions: extract_strings("open_questions"),
                        recent_results: extract_strings("recent_results"),
                        notes: extract_strings("notes"),
                        ..Default::default()
                    };

                    let _ = self
                        .process_action(Action::UpdateCheckpointDelta {
                            frame_id,
                            turn_id: job.id.to_string(),
                            delta,
                        })
                        .await;
                }
            }
            WorkerJobKind::DetectRepetition => {
                if result
                    .payload
                    .get("is_repetitive")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let summary = format!(
                        "repetition detected (ratio={})",
                        result
                            .payload
                            .get("repetition_ratio")
                            .unwrap_or(&serde_json::Value::Null)
                    );
                    let signal = Signal {
                        id: Uuid::now_v7(),
                        ts: Utc::now(),
                        origin: SignalOrigin::Worker,
                        kind: SignalKind::RepeatedPattern,
                        frame_context: job.frame_context,
                        summary,
                        payload_ref: None,
                        tags: vec!["worker".into()],
                    };
                    let _ = self.process_action(Action::IngestSignal { signal }).await;
                }
            }
            WorkerJobKind::ScanForErrors => {
                if result
                    .payload
                    .get("has_errors")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let summary = format!(
                        "worker scan_for_errors detected patterns: {}",
                        result
                            .payload
                            .get("error_patterns_found")
                            .unwrap_or(&serde_json::Value::Null)
                    );
                    let signal = Signal {
                        id: Uuid::now_v7(),
                        ts: Utc::now(),
                        origin: SignalOrigin::Worker,
                        kind: SignalKind::Error,
                        frame_context: job.frame_context,
                        summary,
                        payload_ref: None,
                        tags: vec!["worker".into()],
                    };
                    let _ = self.process_action(Action::IngestSignal { signal }).await;
                }
            }
            WorkerJobKind::SuggestMemory => {
                let count = result
                    .payload
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if count > 0 {
                    let summary = format!("worker suggest_memory: {} suggestion(s)", count);
                    let signal = Signal {
                        id: Uuid::now_v7(),
                        ts: Utc::now(),
                        origin: SignalOrigin::Worker,
                        kind: SignalKind::Warning,
                        frame_context: job.frame_context,
                        summary,
                        payload_ref: None,
                        tags: vec!["worker".into()],
                    };
                    let _ = self.process_action(Action::IngestSignal { signal }).await;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Track CLT interaction node for significant events.
    fn track_clt_event(&mut self, event: &FocusaEvent) {
        use crate::clt;

        // Only track state-changing events as CLT interactions.
        let role = match event {
            FocusaEvent::FocusFramePushed { .. } => "system",
            FocusaEvent::FocusStateUpdated { .. } => "assistant",
            FocusaEvent::IntuitionSignalObserved { .. } => "system",
            _ => return,
        };

        let session_id = self.state.session.as_ref().map(|s| s.session_id);
        clt::append_interaction(
            &mut self.state.clt,
            session_id,
            role,
            None,
            CltMetadata::default(),
        );
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
        tracing::warn!(
            "Unrecognized suppression scope '{}', treating as session",
            scope
        );
        return None;
    }
    let (num_str, unit) = scope.split_at(scope.len() - 1);
    let num: i64 = match num_str.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            tracing::warn!(
                "Unrecognized suppression scope '{}', treating as session",
                scope
            );
            return None;
        }
    };
    let seconds = match unit {
        "s" => num,
        "m" => num * 60,
        "h" => num * 3600,
        _ => {
            tracing::warn!(
                "Unrecognized suppression scope '{}', treating as session",
                scope
            );
            return None;
        }
    };
    Some(Utc::now() + chrono::Duration::seconds(seconds))
}
