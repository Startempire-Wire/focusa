//! Daemon runtime — single-writer event loop.
//!
//! Startup sequence:
//!   1. Load config
//!   2. Ensure directories exist
//!   3. Load state snapshots
//!   4. Open event log
//!   5. Start API server
//!   6. Start worker scheduler
//!
//! Shutdown:
//!   - Flush persistence
//!   - Stop API
//!   - Close event log cleanly

use crate::types::{Action, FocusaConfig, FocusaState};
use tokio::sync::mpsc;

/// The main daemon handle.
pub struct Daemon {
    pub config: FocusaConfig,
    pub state: FocusaState,
    pub command_tx: mpsc::Sender<Action>,
    command_rx: mpsc::Receiver<Action>,
}

impl Daemon {
    /// Create a new daemon with default config.
    pub fn new(config: FocusaConfig) -> Self {
        let (command_tx, command_rx) = mpsc::channel(256);
        Self {
            config,
            state: FocusaState::new(),
            command_tx,
            command_rx,
        }
    }

    /// Run the main event loop.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        // TODO: Implement event loop per G1-detail-03-runtime-daemon.md
        // 1. Receive Action from channel
        // 2. Translate to FocusaEvent
        // 3. Call reducer
        // 4. Persist state + append event log
        // 5. Dispatch emitted events
        tracing::info!("Focusa daemon starting");
        while let Some(_action) = self.command_rx.recv().await {
            // Process action through reducer
            todo!("Process action through reducer")
        }
        Ok(())
    }
}
