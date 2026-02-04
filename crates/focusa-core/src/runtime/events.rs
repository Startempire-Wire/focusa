//! Event system — append-only log.
//!
//! Every state mutation emits an event with:
//!   - id (monotonic UUIDv7)
//!   - timestamp
//!   - type + payload
//!   - correlation_id
//!   - origin
//!
//! Events are: immutable, replayable, inspectable.

use crate::types::{EventLogEntry, FocusaEvent, SignalOrigin};
use chrono::Utc;
use uuid::Uuid;

/// Create a new event log entry.
pub fn create_entry(
    event: FocusaEvent,
    origin: SignalOrigin,
    correlation_id: Option<String>,
) -> EventLogEntry {
    EventLogEntry {
        id: Uuid::now_v7(),
        timestamp: Utc::now(),
        event,
        correlation_id,
        origin,
    }
}
