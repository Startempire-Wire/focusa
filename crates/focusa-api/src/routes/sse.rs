//! Server-Sent Events (SSE) endpoint for real-time TUI updates.
//!
//! Per 27-tui-spec §19: Event-driven updates via SSE.
//! Replaces polling with push-based updates.

use crate::server::AppState;
use axum::Router;
use axum::routing::get;
use axum::{
    extract::State,
    response::{Sse, sse::Event},
};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

pub type EventSender = broadcast::Sender<String>;
#[allow(dead_code)]
pub type EventReceiver = broadcast::Receiver<String>;

/// SSE event broadcaster.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EventBroadcaster {
    sender: EventSender,
}

#[allow(dead_code)]
impl EventBroadcaster {
    pub fn new() -> Self {
        let (sender, _receiver) = broadcast::channel(100);
        Self { sender }
    }

    pub fn broadcast(&self, event: String) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> EventReceiver {
        self.sender.subscribe()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

/// SSE endpoint for real-time events.
///
/// Streams Focusa events as they occur, replacing polling.
/// Uses events_tx which receives events from the daemon event bus.
async fn sse_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let mut receiver = state.events_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match receiver.recv().await {
                Ok(json) => {
                    yield Ok(Event::default().event("focusa_event").data(json));
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // Client is lagging, continue with next event
                    continue;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

/// Health check endpoint for SSE.
async fn sse_health() -> &'static str {
    "SSE endpoint ready"
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/events/stream", get(sse_handler))
        .route("/v1/events/health", get(sse_health))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcaster() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        broadcaster.broadcast("test event".to_string());

        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap(), "test event");
    }

    #[test]
    fn test_multiple_subscribers() {
        let broadcaster = EventBroadcaster::new();
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();

        broadcaster.broadcast("broadcast".to_string());

        assert_eq!(rx1.try_recv().unwrap(), "broadcast");
        assert_eq!(rx2.try_recv().unwrap(), "broadcast");
    }
}
