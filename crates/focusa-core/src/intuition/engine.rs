//! Intuition Engine core — observe patterns, emit signals.
//!
//! Signal sources (MVP):
//!   - Temporal: frame duration, prolonged inactivity
//!   - Repetition: repeated errors, edits, tool invocations
//!   - Consistency: contradictory decisions, intent drift
//!   - Structural: deep stack nesting, frequent frame switching
//!
//! Performance: Zero blocking, bounded memory, O(1) per signal target.

use crate::types::*;
use tokio::sync::mpsc;

/// The intuition engine — a pure signal source.
pub struct IntuitionEngine {
    /// Channel to emit signals to the Focus Gate.
    signal_tx: mpsc::Sender<Signal>,
}

impl IntuitionEngine {
    pub fn new(signal_tx: mpsc::Sender<Signal>) -> Self {
        Self { signal_tx }
    }

    /// Observe a turn and potentially emit signals.
    /// This runs async and NEVER blocks the hot path.
    pub async fn observe_turn(
        &self,
        _turn_id: &str,
        _frame_id: Option<FrameId>,
        _content: &str,
    ) -> anyhow::Result<()> {
        // TODO: Implement signal detection per 05-intuition-engine.md
        // - Check for repeated errors
        // - Check frame duration
        // - Detect contradictions
        Ok(())
    }
}
