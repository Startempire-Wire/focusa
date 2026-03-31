//! Multi-device synchronization — docs/43-multi-device-sync.md
//!
//! CRDT-based event log synchronization with conflict resolution.

pub mod crdt;

pub use crdt::{ConflictResolver, CrdtEvent, CrdtLog, VectorClock};
