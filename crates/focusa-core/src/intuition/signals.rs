//! Signal creation and classification.

use crate::types::*;
use chrono::Utc;
use uuid::Uuid;

/// Create a new signal.
pub fn create_signal(
    origin: SignalOrigin,
    kind: SignalKind,
    frame_context: Option<FrameId>,
    summary: String,
    payload_ref: Option<HandleRef>,
    tags: Vec<String>,
) -> Signal {
    Signal {
        id: Uuid::now_v7(),
        ts: Utc::now(),
        origin,
        kind,
        frame_context,
        summary,
        payload_ref,
        tags,
    }
}
