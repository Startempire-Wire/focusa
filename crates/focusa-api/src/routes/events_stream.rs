//! Event streaming (SSE) via in-process broadcast.
//!
//! GET /v1/events/stream

use crate::server::AppState;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::{Router, routing::get};
use std::convert::Infallible;
use std::sync::Arc;

async fn stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.events_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(json) => {
                    yield Ok(Event::default().event("focusa_event").data(json));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    // Drop lagged events silently; client can refetch /v1/events/recent.
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(15)))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/events/stream", get(stream))
}
