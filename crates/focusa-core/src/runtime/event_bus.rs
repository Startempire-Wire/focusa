use tokio::sync::broadcast;

/// In-process event bus for streaming events to API/UI.
///
/// The daemon is the producer; API server clones the sender into AppState.
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<String>,
}

impl EventBus {
    pub fn new(tx: broadcast::Sender<String>) -> Self {
        Self { tx }
    }

    pub fn publish(&self, json: String) {
        // Best-effort: ignore if no receivers.
        let _ = self.tx.send(json);
    }
}
