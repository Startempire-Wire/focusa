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

// The runtime event surface keeps the durable semantic event contract alongside
// the older signal event log without changing existing callers.
pub use crate::semantic_replay::{
    ReplayError as SemanticReplayError, SemanticEventEnvelope, SemanticPairEvent,
    replay as replay_semantic_pair,
};
/// Create a new event log entry.
pub fn create_entry(
    event: FocusaEvent,
    origin: SignalOrigin,
    correlation_id: Option<String>,
) -> EventLogEntry {
    EventLogEntry::captured(event, origin, correlation_id)
}
