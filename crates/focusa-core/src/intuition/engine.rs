//! Intuition Engine core — observe patterns, emit signals.
//!
//! Source: 05-intuition-engine.md
//!
//! Signal sources (MVP):
//!   - Temporal: frame duration exceeds bounds, prolonged inactivity
//!   - Repetition: repeated errors, repeated tool invocations
//!   - Structural: deep stack nesting (>3 frames)
//!
//! Architecture:
//!   - The engine observes state snapshots (no direct mutation)
//!   - Signals emitted via mpsc channel → daemon ingests as Action::IngestSignal
//!   - Bounded observation window (last N turns)
//!   - O(1) per signal detection pass
//!
//! Performance: Zero blocking, bounded memory, O(1) per signal target.
//!
//! Forbidden: writing memory, altering focus, triggering actions, injecting prompt content.

use crate::intuition::signals::create_signal;
use crate::types::*;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Maximum observation window — only the last N turns are tracked.
const MAX_OBSERVATION_WINDOW: usize = 50;

/// Frame duration threshold — signal if active frame exceeds this.
const FRAME_DURATION_THRESHOLD_SECS: i64 = 1800; // 30 minutes

/// Inactivity threshold — signal if no turn observed for this long.
const INACTIVITY_THRESHOLD_SECS: i64 = 300; // 5 minutes

/// Error repetition threshold — signal after N errors in window.
const ERROR_REPEAT_THRESHOLD: usize = 3;

/// Stack depth threshold — signal if stack deeper than this.
const STACK_DEPTH_THRESHOLD: usize = 3;

/// The intuition engine — a pure signal source.
///
/// Holds bounded observation state and emits signals via channel.
/// Never mutates FocusaState directly.
pub struct IntuitionEngine {
    /// Channel to emit signals to the daemon.
    signal_tx: mpsc::Sender<Signal>,
    /// Rolling window of recent turn timestamps.
    observations: Vec<TurnTimestamp>,
    /// Error counts per frame (bounded by active frames).
    error_counts: HashMap<FrameId, usize>,
    /// Last time a turn was observed (for inactivity detection).
    last_activity: Option<DateTime<Utc>>,
}

/// Timestamp of a turn observation — tracks activity for inactivity detection.
/// Additional fields (frame_id, content_hash) can be added when detectors
/// need them; currently only the observation count and timing matter.
type TurnTimestamp = DateTime<Utc>;

impl IntuitionEngine {
    pub fn new(signal_tx: mpsc::Sender<Signal>) -> Self {
        Self {
            signal_tx,
            observations: Vec::new(),
            error_counts: HashMap::new(),
            last_activity: None,
        }
    }

    /// Observe a turn and potentially emit signals.
    ///
    /// Called by the daemon after each action is processed.
    /// Runs async but NEVER blocks the hot path — uses try_send.
    pub async fn observe_turn(
        &mut self,
        frame_id: Option<FrameId>,
        content: &str,
    ) {
        let now = Utc::now();

        // Record observation timestamp.
        self.observations.push(now);
        if self.observations.len() > MAX_OBSERVATION_WINDOW {
            self.observations.remove(0);
        }

        // Classify content for error detection.
        let has_error = content.contains("error") || content.contains("Error")
            || content.contains("ERROR") || content.contains("panic")
            || content.contains("failed");

        if has_error {
            if let Some(fid) = frame_id {
                let count = self.error_counts.entry(fid).or_insert(0);
                *count += 1;
            }
        }

        // ── Run detectors ────────────────────────────────────────────

        self.detect_error_repetition(frame_id).await;
        self.detect_inactivity(now, frame_id).await;

        self.last_activity = Some(now);
    }

    /// Observe the current focus stack state for structural signals.
    ///
    /// Called periodically or after stack mutations.
    pub async fn observe_stack(&self, stack: &FocusStackState) {
        let now = Utc::now();

        // Deep nesting detection.
        if stack.stack_path_cache.len() > STACK_DEPTH_THRESHOLD {
            let signal = create_signal(
                SignalOrigin::Daemon,
                SignalKind::Warning,
                stack.active_id,
                format!(
                    "Deep stack nesting: {} frames deep (threshold: {})",
                    stack.stack_path_cache.len(),
                    STACK_DEPTH_THRESHOLD,
                ),
                None,
                vec!["structural".into(), "deep_nesting".into()],
            );
            let _ = self.signal_tx.try_send(signal);
        }

        // Long-running frame detection.
        if let Some(active_id) = stack.active_id {
            if let Some(frame) = stack.frames.iter().find(|f| f.id == active_id) {
                let duration = now - frame.created_at;
                if duration > Duration::seconds(FRAME_DURATION_THRESHOLD_SECS) {
                    let signal = create_signal(
                        SignalOrigin::Daemon,
                        SignalKind::DeadlineTick,
                        Some(active_id),
                        format!(
                            "Frame '{}' active for {} minutes (threshold: {} min)",
                            frame.title,
                            duration.num_minutes(),
                            FRAME_DURATION_THRESHOLD_SECS / 60,
                        ),
                        None,
                        vec!["temporal".into(), "long_running".into()],
                    );
                    let _ = self.signal_tx.try_send(signal);
                }
            }
        }
    }

    /// Detect repeated errors within the observation window for a frame.
    async fn detect_error_repetition(&self, frame_id: Option<FrameId>) {
        if let Some(fid) = frame_id {
            if let Some(&count) = self.error_counts.get(&fid) {
                if count >= ERROR_REPEAT_THRESHOLD && count % ERROR_REPEAT_THRESHOLD == 0 {
                    let signal = create_signal(
                        SignalOrigin::Daemon,
                        SignalKind::Error,
                        Some(fid),
                        format!(
                            "Repeated errors detected: {} errors in current frame",
                            count,
                        ),
                        None,
                        vec!["repetition".into(), "error_loop".into()],
                    );
                    let _ = self.signal_tx.try_send(signal);
                }
            }
        }
    }

    /// Detect prolonged inactivity.
    async fn detect_inactivity(&self, now: DateTime<Utc>, frame_id: Option<FrameId>) {
        if let Some(last) = self.last_activity {
            let gap = now - last;
            if gap > Duration::seconds(INACTIVITY_THRESHOLD_SECS) {
                let signal = create_signal(
                    SignalOrigin::Daemon,
                    SignalKind::Warning,
                    frame_id,
                    format!(
                        "Prolonged inactivity: {} minutes since last turn",
                        gap.num_minutes(),
                    ),
                    None,
                    vec!["temporal".into(), "inactivity".into()],
                );
                let _ = self.signal_tx.try_send(signal);
            }
        }
    }

    /// Reset error counts for a completed frame (call on frame pop).
    pub fn clear_frame(&mut self, frame_id: FrameId) {
        self.error_counts.remove(&frame_id);
    }
}


